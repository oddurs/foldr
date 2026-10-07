//! Explicit field patches with durable write-ahead recovery and checked undo.
//!
//! Planning only reads. Each field has a durable `Applying` record before its
//! write, followed by native reread and a durable `Applied` result. An incomplete
//! journal therefore supports recovery after a process crash. Undo refuses a
//! changed device/inode or field value, and restores only recorded fields.
//! Native filesystems do not provide value-based compare-and-swap for these
//! fields: independent writers can still race between a check and a native call.
//! Device/inode checks also cannot prove identity after eventual inode reuse.
use crate::{
    inspect::{identity, open_directory},
    model::*,
    platform,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::CString,
    fs::{self, File, OpenOptions},
    io::Write,
    os::fd::{AsRawFd, FromRawFd},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeRequest {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_note_patch"
    )]
    pub note: Option<Option<String>>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Option<Vec<u8>>>,
    pub mode: Option<u32>,
    pub hidden: Option<bool>,
    pub immutable: Option<bool>,
}
fn deserialize_note_patch<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(deserializer).map(Some)
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Field {
    Xattr { name: Vec<u8> },
    Mode,
    Flags,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FieldValue {
    Bytes { value: Option<Vec<u8>> },
    Mode { value: u32 },
    Flags { value: u64 },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldChange {
    pub field: Field,
    pub before: FieldValue,
    pub after: FieldValue,
    pub scope: Scope,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangePlan {
    pub schema_version: u32,
    pub path: EncodedPath,
    pub identity: FolderIdentity,
    pub changes: Vec<FieldChange>,
    pub warnings: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeStatus {
    Pending,
    Applying,
    Applied,
    Failed,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldResult {
    pub change: FieldChange,
    pub status: ChangeStatus,
    pub error: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeRecord {
    pub schema_version: u32,
    pub id: String,
    pub path: EncodedPath,
    pub identity: FolderIdentity,
    pub fields: Vec<FieldResult>,
    pub undo_of: Option<String>,
    pub completed: bool,
}
impl ChangeRecord {
    pub fn succeeded(&self) -> bool {
        self.completed
            && self
                .fields
                .iter()
                .all(|f| f.status == ChangeStatus::Applied)
    }
}
pub fn metadata_name(key: &str) -> Result<Vec<u8>, FoldrError> {
    if key.is_empty()
        || !key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.')
    {
        return Err(FoldrError::InvalidInput(
            "metadata keys must contain only letters, digits, '.', '_' or '-'".into(),
        ));
    }
    Ok(format!(
        "{}{key}",
        if cfg!(target_os = "macos") {
            "com.foldr."
        } else {
            "user.foldr."
        }
    )
    .into_bytes())
}
pub fn read_note(path: impl AsRef<Path>) -> Result<Option<Vec<u8>>, FoldrError> {
    let file = open_directory(path.as_ref(), true)?;
    platform::read_xattr(&file, &metadata_name("note")?)
}
pub fn read_metadata(path: impl AsRef<Path>, key: &str) -> Result<Option<Vec<u8>>, FoldrError> {
    let file = open_directory(path.as_ref(), true)?;
    platform::read_xattr(&file, &metadata_name(key)?)
}
pub fn plan(
    path: impl AsRef<Path>,
    request: &ChangeRequest,
    follow_symlinks: bool,
) -> Result<ChangePlan, FoldrError> {
    let file = open_directory(path.as_ref(), follow_symlinks)?;
    let canonical = fs::canonicalize(path.as_ref())?;
    // The canonical name must still identify the opened object.
    let id = identity(&file.metadata()?);
    if identity(&fs::metadata(&canonical)?) != id {
        return Err(FoldrError::Conflict("folder changed while planning".into()));
    }
    let mut changes = Vec::new();
    let mut warnings = Vec::new();
    let mut metadata = request.metadata.clone();
    if let Some(note) = &request.note {
        if metadata.contains_key("note") {
            return Err(FoldrError::InvalidInput("note specified twice".into()));
        }
        metadata.insert("note".into(), note.as_ref().map(|n| n.as_bytes().to_vec()));
    }
    for (key, value) in metadata {
        let field = Field::Xattr {
            name: metadata_name(&key)?,
        };
        push_change(&file, &mut changes, field, FieldValue::Bytes { value })?;
    }
    if let Some(mode) = request.mode {
        if mode > 0o7777 {
            return Err(FoldrError::InvalidInput(
                "mode must be between 0000 and 7777".into(),
            ));
        }
        if mode & 0o500 != 0o500 {
            return Err(FoldrError::Unsupported(
                "owner read and search must remain enabled so recorded changes can be recovered"
                    .into(),
            ));
        }
        let acl = platform::inspect_native(&file).acl;
        if acl.value().is_none() || acl.value().is_some_and(|a| a.has_extended_entries) {
            return Err(FoldrError::Unsupported(
                "permission changes require a readable ACL without extended entries; inspect and review the ACL first"
                    .into(),
            ));
        }
        warnings.push("Directory read lists names; execute traverses names; write changes entries. Existing child modes remain unchanged.".into());
        push_change(
            &file,
            &mut changes,
            Field::Mode,
            FieldValue::Mode { value: mode },
        )?;
    }
    if request.hidden.is_some() || request.immutable.is_some() {
        if request.hidden.is_some() && platform::HIDDEN_MASK.is_none() {
            return Err(FoldrError::Unsupported(
                "hidden flags are available only on macOS; Linux dot names require renaming".into(),
            ));
        }
        let raw = platform::read_flags(&file)?;
        let mut value = raw;
        if let Some(hidden) = request.hidden {
            if !cfg!(target_os = "macos") {
                return Err(FoldrError::Unsupported(
                    "hidden flags are available only on macOS; Linux dot names require renaming"
                        .into(),
                ));
            }
            set_bit(
                &mut value,
                platform::HIDDEN_MASK
                    .ok_or_else(|| FoldrError::Unsupported("hidden flags unavailable".into()))?,
                hidden,
            );
        }
        if let Some(immutable) = request.immutable {
            set_bit(&mut value, platform::IMMUTABLE_MASK, immutable);
            warnings.push("Immutable protects this directory's entries; it does not recursively freeze existing files.".into());
        }
        // Clearing a lock must precede metadata/mode edits; setting a lock goes last.
        let lock = platform::IMMUTABLE_MASK;
        if raw & lock != 0 && value & lock == 0 && raw != value {
            changes.insert(
                0,
                FieldChange {
                    field: Field::Flags,
                    before: FieldValue::Flags { value: raw },
                    after: FieldValue::Flags { value },
                    scope: Scope::Folder,
                },
            );
        } else {
            push_change(
                &file,
                &mut changes,
                Field::Flags,
                FieldValue::Flags { value },
            )?;
        }
    }
    Ok(ChangePlan {
        schema_version: 1,
        path: EncodedPath::new(canonical),
        identity: id,
        changes,
        warnings,
    })
}
fn set_bit(value: &mut u64, mask: u64, enabled: bool) {
    if enabled {
        *value |= mask
    } else {
        *value &= !mask
    }
}
fn push_change(
    file: &File,
    changes: &mut Vec<FieldChange>,
    field: Field,
    after: FieldValue,
) -> Result<(), FoldrError> {
    let before = read_field(file, &field)?;
    if before != after {
        changes.push(FieldChange {
            field,
            before,
            after,
            scope: Scope::Folder,
        });
    }
    Ok(())
}
pub(crate) fn read_field(file: &File, field: &Field) -> Result<FieldValue, FoldrError> {
    match field {
        Field::Xattr { name } => Ok(FieldValue::Bytes {
            value: platform::read_xattr(file, name)?,
        }),
        Field::Mode => Ok(FieldValue::Mode {
            value: file.metadata()?.mode() & 0o7777,
        }),
        Field::Flags => Ok(FieldValue::Flags {
            value: platform::read_flags(file)?,
        }),
    }
}
fn write_field(file: &File, field: &Field, value: &FieldValue) -> Result<(), FoldrError> {
    match (field, value) {
        (Field::Xattr { name }, FieldValue::Bytes { value }) => {
            platform::write_xattr(file, name, value.as_deref())
        }
        (Field::Mode, FieldValue::Mode { value }) => {
            file.set_permissions(fs::Permissions::from_mode(*value))?;
            Ok(())
        }
        (Field::Flags, FieldValue::Flags { value }) => platform::write_flags(file, *value),
        _ => Err(FoldrError::InvalidInput("field/value type mismatch".into())),
    }
}
pub fn apply(plan: &ChangePlan, state_dir: &Path) -> Result<ChangeRecord, FoldrError> {
    apply_inner(plan, state_dir, None, None)
}
fn apply_inner(
    plan: &ChangePlan,
    state_dir: &Path,
    undo_of: Option<String>,
    fail_after: Option<usize>,
) -> Result<ChangeRecord, FoldrError> {
    validate_plan(plan)?;
    let file = open_directory(&plan.path.to_path_buf(), false)?;
    if identity(&file.metadata()?) != plan.identity {
        return Err(FoldrError::Conflict(
            "folder identity changed since planning".into(),
        ));
    }
    for change in &plan.changes {
        if read_field(&file, &change.field)? != change.before {
            return Err(FoldrError::Conflict(
                "folder properties changed since planning".into(),
            ));
        }
    }
    if prospective_state_path(state_dir)?.starts_with(plan.path.to_path_buf()) {
        return Err(FoldrError::Journal(
            "recovery state must live outside the target folder".into(),
        ));
    }
    let state = prepare_state(state_dir)?;
    let id = format!(
        "{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| FoldrError::Journal(e.to_string()))?
            .as_nanos(),
        std::process::id()
    );
    let mut record = ChangeRecord {
        schema_version: 1,
        id,
        path: plan.path.clone(),
        identity: plan.identity.clone(),
        fields: plan
            .changes
            .iter()
            .cloned()
            .map(|change| FieldResult {
                change,
                status: ChangeStatus::Pending,
                error: None,
            })
            .collect(),
        undo_of,
        completed: false,
    };
    persist(&state, &record)?;
    for index in 0..record.fields.len() {
        let change = &record.fields[index].change;
        let result = if fail_after == Some(index) {
            Err(FoldrError::Io {
                operation: "injected write failure".into(),
                source: std::io::Error::other("test failure"),
            })
        } else {
            match read_field(&file, &change.field) {
                Err(error) => Err(error),
                Ok(observed) if observed != change.before => {
                    Err(FoldrError::Conflict("field changed while applying".into()))
                }
                Ok(_) => {
                    record.fields[index].status = ChangeStatus::Applying;
                    persist(&state, &record)?;
                    let change = &record.fields[index].change;
                    write_field(&file, &change.field, &change.after).and_then(|()| {
                        file.sync_all()?;
                        if read_field(&file, &change.field)? == change.after {
                            Ok(())
                        } else {
                            Err(FoldrError::Conflict("write verification failed".into()))
                        }
                    })
                }
            }
        };
        match result {
            Ok(()) => {
                record.fields[index].status = ChangeStatus::Applied;
                persist(&state, &record)?;
            }
            Err(error) => {
                record.fields[index].status = ChangeStatus::Failed;
                record.fields[index].error = Some(error.to_string());
                persist(&state, &record)?;
                return Ok(record);
            }
        }
    }
    record.completed = true;
    persist(&state, &record)?;
    Ok(record)
}
fn validate_plan(plan: &ChangePlan) -> Result<(), FoldrError> {
    if plan.schema_version != 1 {
        return Err(FoldrError::InvalidInput("unsupported change schema".into()));
    }
    let mut seen = BTreeSet::new();
    for change in &plan.changes {
        if !seen.insert(&change.field) {
            return Err(FoldrError::InvalidInput(
                "duplicate field in change plan".into(),
            ));
        }
        if change.scope != Scope::Folder {
            return Err(FoldrError::InvalidInput(
                "native folder fields require folder scope".into(),
            ));
        }
        let paired = matches!(
            (&change.field, &change.before, &change.after),
            (
                Field::Xattr { .. },
                FieldValue::Bytes { .. },
                FieldValue::Bytes { .. }
            ) | (
                Field::Mode,
                FieldValue::Mode { .. },
                FieldValue::Mode { .. }
            ) | (
                Field::Flags,
                FieldValue::Flags { .. },
                FieldValue::Flags { .. }
            )
        );
        if !paired {
            return Err(FoldrError::InvalidInput(
                "field/value type mismatch in change plan".into(),
            ));
        }
        match &change.field {
            Field::Xattr { name } => {
                let prefix = if cfg!(target_os = "macos") {
                    b"com.foldr.".as_slice()
                } else {
                    b"user.foldr.".as_slice()
                };
                if !name.starts_with(prefix) || name.contains(&0) {
                    return Err(FoldrError::InvalidInput(
                        "only foldr-owned xattrs may be edited".into(),
                    ));
                }
                let key = std::str::from_utf8(&name[prefix.len()..]).map_err(|_| {
                    FoldrError::InvalidInput("foldr metadata keys must be UTF8".into())
                })?;
                metadata_name(key)?;
            }
            Field::Mode => {
                if !matches!(change.before,FieldValue::Mode{value} if value<=0o7777) {
                    return Err(FoldrError::InvalidInput("invalid original mode".into()));
                }
                if !matches!(change.after,FieldValue::Mode{value} if value<=0o7777 && value&0o500==0o500)
                {
                    return Err(FoldrError::InvalidInput("invalid mode".into()));
                }
            }
            Field::Flags => {
                let (FieldValue::Flags { value: before }, FieldValue::Flags { value: after }) =
                    (&change.before, &change.after)
                else {
                    return Err(FoldrError::InvalidInput("invalid flag value".into()));
                };
                let allowed = platform::IMMUTABLE_MASK | platform::HIDDEN_MASK.unwrap_or(0);
                if (before ^ after) & !allowed != 0 {
                    return Err(FoldrError::Unsupported(
                        "changing unknown native flags is forbidden".into(),
                    ));
                }
            }
        }
    }
    Ok(())
}
pub fn undo(
    record: &ChangeRecord,
    state_dir: &Path,
    dry_run: bool,
) -> Result<ChangeRecord, FoldrError> {
    let plan = undo_plan(record)?;
    if dry_run {
        return Ok(ChangeRecord {
            schema_version: 1,
            id: format!("preview-undo-{}", record.id),
            path: plan.path,
            identity: plan.identity,
            fields: plan
                .changes
                .into_iter()
                .map(|change| FieldResult {
                    change,
                    status: ChangeStatus::Pending,
                    error: None,
                })
                .collect(),
            undo_of: Some(record.id.clone()),
            completed: false,
        });
    }
    apply_inner(&plan, state_dir, Some(record.id.clone()), None)
}
pub fn undo_plan(record: &ChangeRecord) -> Result<ChangePlan, FoldrError> {
    validate_record(record)?;
    let file = open_directory(&record.path.to_path_buf(), false)?;
    if identity(&file.metadata()?) != record.identity {
        return Err(FoldrError::Conflict(
            "folder identity differs from recovery record".into(),
        ));
    }
    let mut changes = Vec::new();
    for result in record.fields.iter().rev() {
        if result.status == ChangeStatus::Pending {
            continue;
        }
        let observed = read_field(&file, &result.change.field)?;
        if result.status != ChangeStatus::Applied && observed == result.change.before {
            continue;
        }
        if observed != result.change.after {
            return Err(FoldrError::Conflict(
                "undo refused: a recorded field has an intervening change".into(),
            ));
        }
        changes.push(FieldChange {
            field: result.change.field.clone(),
            before: result.change.after.clone(),
            after: result.change.before.clone(),
            scope: result.change.scope.clone(),
        });
    }
    let plan = ChangePlan {
        schema_version: 1,
        path: record.path.clone(),
        identity: record.identity.clone(),
        changes,
        warnings: vec![
            "Restore only recorded fields; unrelated metadata remains untouched.".into(),
        ],
    };
    validate_plan(&plan)?;
    Ok(plan)
}
fn validate_record(record: &ChangeRecord) -> Result<(), FoldrError> {
    if record.schema_version != 1 {
        return Err(FoldrError::Unsupported(format!(
            "unsupported recovery schema {}",
            record.schema_version
        )));
    }
    validate_plan(&ChangePlan {
        schema_version: 1,
        path: record.path.clone(),
        identity: record.identity.clone(),
        changes: record
            .fields
            .iter()
            .map(|field| field.change.clone())
            .collect(),
        warnings: Vec::new(),
    })
}
fn prospective_state_path(state_dir: &Path) -> Result<std::path::PathBuf, FoldrError> {
    if state_dir.exists() {
        return Ok(fs::canonicalize(state_dir)?);
    }
    if state_dir
        .components()
        .any(|c| c == std::path::Component::ParentDir)
    {
        return Err(FoldrError::InvalidInput(
            "new state directory paths cannot contain '..'".into(),
        ));
    }
    let absolute = if state_dir.is_absolute() {
        state_dir.to_path_buf()
    } else {
        std::env::current_dir()?.join(state_dir)
    };
    let mut ancestor = absolute.clone();
    let mut missing = Vec::new();
    while !ancestor.exists() {
        let name = ancestor
            .file_name()
            .ok_or_else(|| FoldrError::InvalidInput("invalid state directory".into()))?
            .to_owned();
        missing.push(name);
        if !ancestor.pop() {
            return Err(FoldrError::InvalidInput("invalid state directory".into()));
        }
    }
    let mut result = fs::canonicalize(ancestor)?;
    for name in missing.into_iter().rev() {
        result.push(name);
    }
    Ok(result)
}
fn prepare_state(state_dir: &Path) -> Result<File, FoldrError> {
    prepare_state_with_sync(state_dir, |directory| {
        directory
            .sync_all()
            .map_err(|e| FoldrError::io("sync recovery directory creation", e))
    })
}
fn prepare_state_with_sync(
    state_dir: &Path,
    mut sync: impl FnMut(&File) -> Result<(), FoldrError>,
) -> Result<File, FoldrError> {
    if state_dir.exists() {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .open(state_dir)?;
        validate_state_directory(&file)?;
        return Ok(file);
    }
    let mut ancestor = prospective_state_path(state_dir)?;
    let mut missing = Vec::new();
    while !ancestor.exists() {
        missing.push(
            ancestor
                .file_name()
                .ok_or_else(|| FoldrError::InvalidInput("invalid state directory".into()))?
                .to_owned(),
        );
        ancestor.pop();
    }
    let mut directory = open_directory(&ancestor, false)?;
    for name in missing.into_iter().rev() {
        use std::os::unix::ffi::OsStrExt;
        let name = CString::new(name.as_bytes())
            .map_err(|_| FoldrError::InvalidInput("NUL byte in state directory".into()))?;
        // SAFETY: descriptor and NUL-terminated component are valid for mkdirat.
        if unsafe { libc::mkdirat(directory.as_raw_fd(), name.as_ptr(), 0o700) } < 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::EEXIST) {
                return Err(FoldrError::io("create private recovery directory", error));
            }
        }
        // SAFETY: descriptor and name remain valid; no-follow prevents a competing
        // directory creator from substituting a symlink during this walk.
        let descriptor = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if descriptor < 0 {
            return Err(FoldrError::io(
                "open new recovery directory",
                std::io::Error::last_os_error(),
            ));
        }
        // SAFETY: successful openat returned a new owned descriptor.
        let child = unsafe { File::from_raw_fd(descriptor) };
        validate_state_directory(&child)?;
        // Sync the child inode, then its entry in the parent, before descending.
        // Failure aborts before any target-field write or journal publication.
        sync(&child)?;
        sync(&directory)?;
        directory = child;
    }
    validate_state_directory(&directory)?;
    Ok(directory)
}
fn validate_state_directory(file: &File) -> Result<(), FoldrError> {
    let metadata = file.metadata()?;
    if metadata.uid() != unsafe_uid() {
        return Err(FoldrError::Journal(
            "state directory belongs to another user".into(),
        ));
    }
    if metadata.mode() & 0o077 != 0 {
        return Err(FoldrError::Journal("existing state directory must be private (mode 0700); refusing to change its permissions".into()));
    }
    Ok(())
}
fn unsafe_uid() -> u32 {
    // SAFETY: geteuid takes no pointers and cannot violate memory safety.
    unsafe { libc::geteuid() }
}
fn persist(state_dir: &File, record: &ChangeRecord) -> Result<(), FoldrError> {
    let path = CString::new(format!("{}.json", record.id))
        .map_err(|_| FoldrError::Journal("invalid journal ID".into()))?;
    let temporary = CString::new(format!(".{}.tmp", record.id))
        .map_err(|_| FoldrError::Journal("invalid journal ID".into()))?;
    let data = serde_json::to_vec_pretty(record).map_err(|e| FoldrError::Journal(e.to_string()))?;
    // SAFETY: state descriptor and NUL-terminated name remain valid; mode is
    // supplied for O_CREAT, and O_EXCL prevents overwriting an existing file.
    let descriptor = unsafe {
        libc::openat(
            state_dir.as_raw_fd(),
            temporary.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600,
        )
    };
    if descriptor < 0 {
        return Err(FoldrError::io(
            "create recovery journal",
            std::io::Error::last_os_error(),
        ));
    }
    // SAFETY: successful openat returned a new descriptor owned by this File.
    let mut file = unsafe { File::from_raw_fd(descriptor) };
    let result = (|| -> Result<(), FoldrError> {
        file.write_all(&data)?;
        file.sync_all()?;
        // SAFETY: both names and the directory descriptor remain valid. renameat
        // atomically replaces a directory entry, without following destination links.
        if unsafe {
            libc::renameat(
                state_dir.as_raw_fd(),
                temporary.as_ptr(),
                state_dir.as_raw_fd(),
                path.as_ptr(),
            )
        } < 0
        {
            return Err(FoldrError::io(
                "publish recovery journal",
                std::io::Error::last_os_error(),
            ));
        }
        state_dir.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        // SAFETY: name and descriptor remain valid; only this temporary entry is removed.
        unsafe { libc::unlinkat(state_dir.as_raw_fd(), temporary.as_ptr(), 0) };
    }
    result
}
pub fn history(state_dir: &Path) -> Result<Vec<ChangeRecord>, FoldrError> {
    if !state_dir.exists() {
        return Ok(Vec::new());
    }
    let mut records = Vec::new();
    for entry in fs::read_dir(state_dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "json") {
            records.push(load_record_path(&path)?);
        }
    }
    records.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(records)
}
pub fn load_record(state_dir: &Path, id: &str) -> Result<ChangeRecord, FoldrError> {
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit() || b == b'-') {
        return Err(FoldrError::InvalidInput("invalid change ID".into()));
    }
    load_record_path(&state_dir.join(format!("{id}.json")))
}
fn load_record_path(path: &Path) -> Result<ChangeRecord, FoldrError> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    let record = serde_json::from_reader(file).map_err(|e| FoldrError::Journal(e.to_string()))?;
    validate_record(&record)?;
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    fn folder() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temp.path()).unwrap();
        let folder = root.join("folder");
        fs::create_dir(&folder).unwrap();
        (temp, folder, root.join("state"))
    }
    #[test]
    fn json_note_patch_distinguishes_omission_from_removal() {
        for note in [None, Some(None), Some(Some("value".into()))] {
            let request = ChangeRequest {
                note,
                ..Default::default()
            };
            let encoded = serde_json::to_vec(&request).unwrap();
            assert_eq!(
                serde_json::from_slice::<ChangeRequest>(&encoded).unwrap(),
                request
            );
        }
    }
    #[test]
    fn dry_plan_does_not_create_state_or_mutate() {
        let (_temp, folder, state) = folder();
        let before = fs::metadata(&folder).unwrap().mode();
        let plan = plan(
            &folder,
            &ChangeRequest {
                mode: Some(0o700),
                ..Default::default()
            },
            false,
        )
        .unwrap();
        assert_eq!(plan.changes.len(), 1);
        assert_eq!(fs::metadata(folder).unwrap().mode(), before);
        assert!(!state.exists());
    }
    #[test]
    fn applied_permissions_can_be_undone_but_conflicts_refuse() {
        let (_temp, folder, state) = folder();
        let initial = fs::metadata(&folder).unwrap().mode() & 0o7777;
        let plan = plan(
            &folder,
            &ChangeRequest {
                mode: Some(0o700),
                ..Default::default()
            },
            false,
        )
        .unwrap();
        let record = apply(&plan, &state).unwrap();
        assert!(record.succeeded());
        fs::set_permissions(&folder, fs::Permissions::from_mode(0o711)).unwrap();
        assert!(undo(&record, &state, false).is_err());
        fs::set_permissions(&folder, fs::Permissions::from_mode(0o700)).unwrap();
        assert!(undo(&record, &state, false).unwrap().succeeded());
        assert_eq!(fs::metadata(folder).unwrap().mode() & 0o7777, initial);
    }
    #[test]
    fn partial_failure_has_durable_field_results() {
        let (_temp, folder, state) = folder();
        let request = ChangeRequest {
            note: Some(Some("binary-safe".into())),
            mode: Some(0o700),
            ..Default::default()
        };
        let plan = plan(&folder, &request, false).unwrap();
        let record = apply_inner(&plan, &state, None, Some(1)).unwrap();
        assert!(!record.completed);
        assert_eq!(record.fields[0].status, ChangeStatus::Applied);
        assert_eq!(record.fields[1].status, ChangeStatus::Failed);
        assert_eq!(load_record(&state, &record.id).unwrap(), record);
        assert!(undo(&record, &state, false).unwrap().succeeded());
        assert_eq!(read_note(folder).unwrap(), None);
    }
    #[test]
    fn replacing_folder_refuses_apply() {
        let (_temp, folder, state) = folder();
        let plan = plan(
            &folder,
            &ChangeRequest {
                mode: Some(0o700),
                ..Default::default()
            },
            false,
        )
        .unwrap();
        fs::rename(&folder, folder.with_extension("old")).unwrap();
        fs::create_dir(&folder).unwrap();
        assert!(apply(&plan, &state).is_err());
        assert!(!state.exists());
    }
    #[test]
    fn undo_preserves_unrelated_binary_metadata() {
        let (_temp, folder, state) = folder();
        let request = ChangeRequest {
            note: Some(Some("owned".into())),
            ..Default::default()
        };
        let record = apply(&plan(&folder, &request, false).unwrap(), &state).unwrap();
        let file = open_directory(&folder, false).unwrap();
        platform::write_xattr(
            &file,
            &metadata_name("unrelated").unwrap(),
            Some(&[0, 255, 10]),
        )
        .unwrap();
        assert!(undo(&record, &state, false).unwrap().succeeded());
        assert_eq!(
            read_metadata(&folder, "unrelated").unwrap(),
            Some(vec![0, 255, 10])
        );
        assert_eq!(read_note(folder).unwrap(), None);
    }
    #[test]
    fn recovery_handles_crash_after_native_write_before_applied_record() {
        let (_temp, folder, state) = folder();
        let request = ChangeRequest {
            note: Some(Some("written".into())),
            ..Default::default()
        };
        let mut record = apply(&plan(&folder, &request, false).unwrap(), &state).unwrap();
        // This is exactly the durable state left between the native write and
        // the next journal update. Recovery resolves it by rereading the field.
        record.fields[0].status = ChangeStatus::Applying;
        record.completed = false;
        assert!(undo(&record, &state, false).unwrap().succeeded());
        assert_eq!(read_note(folder).unwrap(), None);
    }
    #[test]
    fn insecure_or_symlink_state_refuses_before_target_write() {
        let (_temp, folder, state) = folder();
        let request = ChangeRequest {
            note: Some(Some("not written".into())),
            ..Default::default()
        };
        let plan = plan(&folder, &request, false).unwrap();
        fs::create_dir(&state).unwrap();
        fs::set_permissions(&state, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(apply(&plan, &state).is_err());
        assert_eq!(fs::metadata(&state).unwrap().mode() & 0o7777, 0o755);
        let link = state.with_extension("link");
        std::os::unix::fs::symlink(&state, &link).unwrap();
        assert!(apply(&plan, &link).is_err());
        assert_eq!(read_note(folder).unwrap(), None);
    }
    #[test]
    fn owner_read_and_search_must_remain_recoverable() {
        let (_temp, folder, state) = folder();
        assert!(
            plan(
                &folder,
                &ChangeRequest {
                    mode: Some(0),
                    ..Default::default()
                },
                false
            )
            .is_err()
        );
        assert!(!state.exists());
    }
    #[test]
    fn state_inside_target_is_refused_without_creating_any_entries() {
        let (_temp, folder, _state) = folder();
        let state = folder.join("recovery");
        let request = ChangeRequest {
            note: Some(Some("not written".into())),
            ..Default::default()
        };
        let plan = plan(&folder, &request, false).unwrap();
        assert!(apply(&plan, &state).is_err());
        assert!(!state.exists());
        assert_eq!(read_note(&folder).unwrap(), None);
        assert_eq!(fs::read_dir(folder).unwrap().count(), 0);
    }
    #[test]
    fn malformed_later_fields_are_refused_before_earlier_writes_or_journals() {
        let (_temp, folder, state) = folder();
        let valid = plan(
            &folder,
            &ChangeRequest {
                note: Some(Some("must not write".into())),
                ..Default::default()
            },
            false,
        )
        .unwrap();
        let bytes = FieldValue::Bytes { value: None };
        let mode = FieldValue::Mode { value: 0o700 };
        let flags = FieldValue::Flags { value: 0 };
        let other = Field::Xattr {
            name: metadata_name("other").unwrap(),
        };
        for (field, before, after) in [
            (other.clone(), mode.clone(), bytes.clone()),
            (other, bytes.clone(), flags.clone()),
            (Field::Mode, bytes.clone(), mode.clone()),
            (Field::Mode, mode.clone(), flags.clone()),
            (Field::Flags, bytes.clone(), flags.clone()),
            (Field::Flags, flags.clone(), mode.clone()),
        ] {
            let mut malformed = valid.clone();
            malformed.changes.push(FieldChange {
                field,
                before,
                after,
                scope: Scope::Folder,
            });
            let error = apply(&malformed, &state).unwrap_err();
            assert!(error.to_string().contains("type mismatch"));
            assert_eq!(read_note(&folder).unwrap(), None);
            assert!(!state.exists());
        }
        let mut duplicate = valid.clone();
        duplicate.changes.push(duplicate.changes[0].clone());
        assert!(
            apply(&duplicate, &state)
                .unwrap_err()
                .to_string()
                .contains("duplicate")
        );
        let mut wrong_scope = valid.clone();
        wrong_scope.changes[0].scope = Scope::FutureChildren;
        assert!(apply(&wrong_scope, &state).is_err());
        assert_eq!(read_note(folder).unwrap(), None);
        assert!(!state.exists());
    }
    #[test]
    fn future_records_are_refused_on_load_and_undo_without_writes() {
        let (_temp, folder, state) = folder();
        let plan = plan(
            &folder,
            &ChangeRequest {
                note: Some(Some("must not write".into())),
                ..Default::default()
            },
            false,
        )
        .unwrap();
        let record = ChangeRecord {
            schema_version: 2,
            id: "123".into(),
            path: plan.path,
            identity: plan.identity,
            fields: plan
                .changes
                .into_iter()
                .map(|change| FieldResult {
                    change,
                    status: ChangeStatus::Applied,
                    error: None,
                })
                .collect(),
            undo_of: None,
            completed: true,
        };
        assert!(matches!(
            undo(&record, &state, true),
            Err(FoldrError::Unsupported(_))
        ));
        assert!(matches!(
            undo(&record, &state, false),
            Err(FoldrError::Unsupported(_))
        ));
        assert!(!state.exists());
        assert_eq!(read_note(&folder).unwrap(), None);
        let source = folder.parent().unwrap().join("source");
        fs::create_dir(&source).unwrap();
        let encoded = serde_json::to_vec(&record).unwrap();
        fs::write(source.join("123.json"), &encoded).unwrap();
        assert!(matches!(
            load_record(&source, "123"),
            Err(FoldrError::Unsupported(_))
        ));
        assert!(history(&source).is_err());
        assert_eq!(fs::read(source.join("123.json")).unwrap(), encoded);
        assert!(!state.exists());
        assert_eq!(read_note(folder).unwrap(), None);
    }
    #[test]
    fn nested_state_creation_syncs_each_child_then_parent_before_returning() {
        let (_temp, folder, _state) = folder();
        let root = folder.parent().unwrap();
        let state = root.join("new-parent").join("history");
        let mut synced = Vec::new();
        prepare_state_with_sync(&state, |file| {
            synced.push(identity(&file.metadata()?));
            file.sync_all()?;
            Ok(())
        })
        .unwrap();
        let parent_id = identity(&fs::metadata(root.join("new-parent")).unwrap());
        let root_id = identity(&fs::metadata(root).unwrap());
        let state_id = identity(&fs::metadata(&state).unwrap());
        assert_eq!(
            synced,
            vec![parent_id.clone(), root_id, state_id, parent_id]
        );
        assert_eq!(fs::metadata(&state).unwrap().mode() & 0o777, 0o700);
        assert_eq!(fs::read_dir(state).unwrap().count(), 0);
    }
    #[test]
    fn creation_sync_failure_returns_before_any_folder_or_history_write() {
        let (_temp, folder, state) = folder();
        let mut calls = 0;
        let result = prepare_state_with_sync(&state, |_| {
            calls += 1;
            if calls == 2 {
                Err(FoldrError::Journal("injected parent sync failure".into()))
            } else {
                Ok(())
            }
        });
        assert!(result.is_err());
        assert_eq!(calls, 2);
        assert_eq!(fs::read_dir(state).unwrap().count(), 0);
        assert_eq!(read_note(folder).unwrap(), None);
    }
}
