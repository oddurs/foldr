//! Versioned TOML presets are partial patches, never whole-folder replacement.
use crate::{changes::*, model::*};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap, fs::OpenOptions, io::Write, os::unix::fs::OpenOptionsExt, path::Path,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preset {
    pub schema_version: u32,
    pub name: Option<String>,
    pub platform: Option<String>,
    pub request: ChangeRequest,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WirePreset {
    schema_version: u32,
    name: Option<String>,
    platform: Option<String>,
    settings: Settings,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    note: Option<String>,
    #[serde(default)]
    remove_note: bool,
    #[serde(default)]
    metadata: BTreeMap<String, Vec<u8>>,
    #[serde(default)]
    remove_metadata: Vec<String>,
    mode: Option<u32>,
    hidden: Option<bool>,
    immutable: Option<bool>,
    finder_tags: Option<Vec<String>>,
}
impl Preset {
    pub fn from_toml(text: &str) -> Result<Self, FoldrError> {
        let wire: WirePreset = toml::from_str(text)
            .map_err(|e| FoldrError::InvalidInput(format!("invalid preset: {e}")))?;
        if !matches!(wire.schema_version, 1 | 2) {
            return Err(FoldrError::Unsupported(format!(
                "unsupported preset schema {}",
                wire.schema_version
            )));
        }
        if let Some(names) = &wire.settings.finder_tags {
            if wire.schema_version != 2 || wire.platform.as_deref() != Some("macos") {
                return Err(FoldrError::InvalidInput(
                    "Finder tag presets require schema_version 2 and platform=macos".into(),
                ));
            }
            validate_tag_names(names)?;
        }
        if wire.settings.note.is_some() && wire.settings.remove_note {
            return Err(FoldrError::InvalidInput(
                "preset cannot set and remove note".into(),
            ));
        }
        let mut metadata = wire
            .settings
            .metadata
            .into_iter()
            .map(|(k, v)| (k, Some(v)))
            .collect::<BTreeMap<_, _>>();
        for key in wire.settings.remove_metadata {
            if metadata.insert(key, None).is_some() {
                return Err(FoldrError::InvalidInput(
                    "preset cannot set and remove the same metadata key".into(),
                ));
            }
        }
        for key in metadata.keys() {
            metadata_name(key)?;
        }
        let note = if wire.settings.remove_note {
            Some(None)
        } else {
            wire.settings.note.map(Some)
        };
        let preset = Self {
            schema_version: wire.schema_version,
            name: wire.name,
            platform: wire.platform,
            request: ChangeRequest {
                note,
                metadata,
                mode: wire.settings.mode,
                hidden: wire.settings.hidden,
                immutable: wire.settings.immutable,
                finder_tags: wire.settings.finder_tags,
            },
        };
        validate_preset(&preset)?;
        Ok(preset)
    }
    pub fn to_toml(&self) -> Result<String, FoldrError> {
        validate_preset(self)?;
        let mut settings = Settings {
            note: self.request.note.clone().flatten(),
            remove_note: self.request.note == Some(None),
            mode: self.request.mode,
            hidden: self.request.hidden,
            immutable: self.request.immutable,
            finder_tags: self.request.finder_tags.clone(),
            ..Default::default()
        };
        for (key, value) in &self.request.metadata {
            match value {
                Some(bytes) => {
                    settings.metadata.insert(key.clone(), bytes.clone());
                }
                None => settings.remove_metadata.push(key.clone()),
            }
        }
        toml::to_string_pretty(&WirePreset {
            schema_version: self.schema_version,
            name: self.name.clone(),
            platform: self.platform.clone(),
            settings,
        })
        .map_err(|e| FoldrError::InvalidInput(format!("encode preset: {e}")))
    }
}
pub(crate) fn validate_preset(preset: &Preset) -> Result<(), FoldrError> {
    if !matches!(preset.schema_version, 1 | 2) {
        return Err(FoldrError::Unsupported(format!(
            "unsupported preset schema {}",
            preset.schema_version
        )));
    }
    if preset.request.mode.is_some_and(|mode| mode > 0o7777) {
        return Err(FoldrError::InvalidInput(
            "mode must be between 0000 and 7777".into(),
        ));
    }
    if preset.request.note.is_some() && preset.request.metadata.contains_key("note") {
        return Err(FoldrError::InvalidInput("note specified twice".into()));
    }
    for key in preset.request.metadata.keys() {
        metadata_name(key)?;
    }
    if let Some(names) = &preset.request.finder_tags {
        if preset.schema_version != 2 || preset.platform.as_deref() != Some("macos") {
            return Err(FoldrError::InvalidInput(
                "Finder tag presets require schema 2 and platform=macos".into(),
            ));
        }
        validate_tag_names(names)?;
    }
    Ok(())
}
pub(crate) fn validate_tag_names(names: &[String]) -> Result<(), FoldrError> {
    if names.len() > 1024 {
        return Err(FoldrError::InvalidInput("too many Finder tags".into()));
    }
    for (index, name) in names.iter().enumerate() {
        if name.is_empty()
            || name.len() > 4096
            || name.chars().any(char::is_control)
            || names[..index].contains(name)
        {
            return Err(FoldrError::InvalidInput("Finder tag names must be unique, nonempty, at most 4096 UTF-8 bytes and contain no controls".into()));
        }
    }
    Ok(())
}
pub fn read_preset(path: impl AsRef<Path>) -> Result<Preset, FoldrError> {
    Preset::from_toml(&std::fs::read_to_string(path)?)
}
pub fn write_preset(path: impl AsRef<Path>, preset: &Preset) -> Result<(), FoldrError> {
    let text = preset.to_toml()?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    Ok(())
}
pub fn preset_plan(
    path: impl AsRef<Path>,
    preset: &Preset,
    follow_symlinks: bool,
) -> Result<ChangePlan, FoldrError> {
    validate_preset(preset)?;
    if preset
        .platform
        .as_deref()
        .is_some_and(|p| p != std::env::consts::OS)
    {
        return Err(FoldrError::Unsupported(format!(
            "preset targets {}, current platform is {}",
            preset.platform.as_deref().unwrap(),
            std::env::consts::OS
        )));
    }
    plan(path, &preset.request, follow_symlinks)
}
pub fn preset_from_snapshot(
    snapshot: &FolderSnapshot,
    fields: &[String],
) -> Result<Preset, FoldrError> {
    let fields = if fields.is_empty() {
        vec!["note".into(), "metadata".into(), "mode".into()]
    } else {
        fields.to_vec()
    };
    let mut request = ChangeRequest::default();
    let mut platform = None;
    let saves_note = fields.iter().any(|field| field == "note");
    for field in fields {
        match field.as_str() {
            "note" | "metadata" => {
                let xattrs = snapshot.xattrs.value().ok_or_else(|| {
                    FoldrError::Unsupported(format!(
                        "cannot save {field}: {}",
                        snapshot.xattrs.reason().unwrap_or("unavailable")
                    ))
                })?;
                let prefix = if snapshot.platform == "macos" {
                    b"com.foldr.".as_slice()
                } else {
                    b"user.foldr.".as_slice()
                };
                for attr in xattrs {
                    if let Some(key) = attr.name.bytes.strip_prefix(prefix) {
                        if field == "note" && key == b"note" {
                            request.note = Some(Some(
                                String::from_utf8(attr.value.clone()).map_err(|_| {
                                    FoldrError::InvalidInput(
                                        "folder note is binary; save metadata to preserve it"
                                            .into(),
                                    )
                                })?,
                            ));
                        } else if field == "metadata" && (key != b"note" || !saves_note) {
                            let key = String::from_utf8(key.to_vec()).map_err(|_| {
                                FoldrError::InvalidInput("foldr metadata key is not UTF8".into())
                            })?;
                            metadata_name(&key)?;
                            request.metadata.insert(key, Some(attr.value.clone()));
                        }
                    }
                }
            }
            "finder_tags" => {
                let tags = crate::finder_tags_from_snapshot(snapshot);
                let tags = tags.value().ok_or_else(|| {
                    FoldrError::Unsupported(format!(
                        "cannot save Finder tags: {}",
                        tags.reason().unwrap_or("unavailable")
                    ))
                })?;
                request.finder_tags = Some(tags.iter().map(|tag| tag.name.clone()).collect());
                platform = Some("macos".into());
            }
            "mode" | "permissions" => request.mode = Some(snapshot.mode),
            "hidden" => {
                if snapshot.platform != "macos" {
                    return Err(FoldrError::Unsupported(
                        "hidden flag unavailable on this platform".into(),
                    ));
                }
                request.hidden = Some(
                    snapshot
                        .flags
                        .value()
                        .ok_or_else(|| FoldrError::Unsupported("cannot read native flags".into()))?
                        .raw
                        & 0x8000
                        != 0,
                );
                platform = Some(snapshot.platform.clone());
            }
            "immutable" => {
                request.immutable = Some(
                    snapshot
                        .flags
                        .value()
                        .ok_or_else(|| FoldrError::Unsupported("cannot read native flags".into()))?
                        .raw
                        & if snapshot.platform == "macos" {
                            2
                        } else {
                            0x10
                        }
                        != 0,
                );
            }
            "flags" => {
                request.immutable = Some(
                    snapshot
                        .flags
                        .value()
                        .ok_or_else(|| FoldrError::Unsupported("cannot read native flags".into()))?
                        .raw
                        & if snapshot.platform == "macos" {
                            2
                        } else {
                            0x10
                        }
                        != 0,
                );
                if snapshot.platform == "macos" {
                    request.hidden = Some(snapshot.flags.value().unwrap().raw & 0x8000 != 0);
                    platform = Some(snapshot.platform.clone());
                }
            }
            other => {
                return Err(FoldrError::InvalidInput(format!(
                    "unknown preset field {other}; choose note, metadata, mode, hidden, immutable, flags or finder_tags"
                )));
            }
        }
    }
    Ok(Preset {
        schema_version: if request.finder_tags.is_some() { 2 } else { 1 },
        name: None,
        platform,
        request,
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchResult {
    pub path: EncodedPath,
    pub plan: Option<ChangePlan>,
    pub record: Option<ChangeRecord>,
    pub error: Option<String>,
}
pub fn apply_batch(
    paths: &[std::path::PathBuf],
    preset: &Preset,
    state_dir: &Path,
    follow_symlinks: bool,
    dry_run: bool,
) -> Vec<BatchResult> {
    // Preflight every target before starting any write. Targets remain independent.
    let mut results = paths
        .iter()
        .map(|path| match preset_plan(path, preset, follow_symlinks) {
            Ok(plan) => BatchResult {
                path: EncodedPath::new(path),
                plan: Some(plan),
                record: None,
                error: None,
            },
            Err(error) => BatchResult {
                path: EncodedPath::new(path),
                plan: None,
                record: None,
                error: Some(error.to_string()),
            },
        })
        .collect::<Vec<_>>();
    if dry_run {
        return results;
    }
    for result in &mut results {
        if let Some(plan) = &result.plan {
            match apply(plan, state_dir) {
                Ok(record) => {
                    if !record.succeeded() {
                        result.error =
                            Some("partially applied; consult per-field recovery record".into());
                    }
                    result.record = Some(record);
                }
                Err(error) => result.error = Some(error.to_string()),
            }
        }
    }
    results
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finder_tag_presets_require_explicit_new_schema_and_mac_platform() {
        let text = "schema_version=2\nplatform='macos'\n[settings]\nfinder_tags=['研究 🟣','kept']";
        let preset = Preset::from_toml(text).unwrap();
        assert_eq!(
            Preset::from_toml(&preset.to_toml().unwrap()).unwrap(),
            preset
        );
        assert_eq!(preset.request.note, None);
        let empty =
            Preset::from_toml("schema_version=2\nplatform='macos'\n[settings]\nfinder_tags=[]")
                .unwrap();
        assert_eq!(empty.request.finder_tags, Some(vec![]));
        assert_eq!(Preset::from_toml(&empty.to_toml().unwrap()).unwrap(), empty);
        for bad in [
            "schema_version=1\nplatform='macos'\n[settings]\nfinder_tags=[]",
            "schema_version=2\n[settings]\nfinder_tags=[]",
            "schema_version=2\nplatform='macos'\n[settings]\nfinder_tags=['same','same']",
        ] {
            assert!(Preset::from_toml(bad).is_err());
        }
        let mut wrong = preset;
        wrong.schema_version = 1;
        assert!(wrong.to_toml().is_err());
        let folder = tempfile::tempdir().unwrap();
        assert!(preset_plan(folder.path(), &wrong, false).is_err());
        assert_eq!(crate::read_note(folder.path()).unwrap(), None);
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn selected_tag_presets_compare_names_and_omission_preserves_colors() {
        use crate::platform::tags::*;
        let temp = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(temp.path()).unwrap();
        let folder = root.join("folder");
        std::fs::create_dir(&folder).unwrap();
        let file = std::fs::File::open(&folder).unwrap();
        let tags = vec![
            FinderTag {
                name: "研究 🟣".into(),
                color: None,
            },
            FinderTag {
                name: "kept".into(),
                color: Some(5),
            },
        ];
        let raw = encode_finder_tags(&tags).unwrap();
        write_finder_tags_raw(&file, Some(&raw)).unwrap();
        let snapshot = crate::inspect(&folder).unwrap();
        let captured = preset_from_snapshot(&snapshot, &["finder_tags".into()]).unwrap();
        assert_eq!(captured.schema_version, 2);
        assert_eq!(captured.platform.as_deref(), Some("macos"));
        assert_eq!(
            crate::check_preset(&snapshot, &captured).unwrap().status,
            crate::ComplianceStatus::Compliant
        );
        let reverse = Preset::from_toml(
            "schema_version=2\nplatform='macos'\n[settings]\nfinder_tags=['研究 🟣','kept']",
        )
        .unwrap();
        assert!(
            crate::compare_preset(&snapshot, &reverse)
                .unwrap()
                .is_empty()
        );
        assert!(
            preset_plan(&folder, &reverse, false)
                .unwrap()
                .changes
                .is_empty()
        );
        let add = crate::finder_tag_plan(&folder, &["added".into()], &[], false).unwrap();
        let added = crate::apply(&add, &root.join("state")).unwrap();
        let desired = Preset::from_toml("schema_version=2\nplatform='macos'\n[settings]\nfinder_tags=['added','kept','研究 🟣']").unwrap();
        assert_eq!(
            crate::check_preset(&crate::inspect(&folder).unwrap(), &desired)
                .unwrap()
                .status,
            crate::ComplianceStatus::Compliant
        );
        assert!(
            crate::undo(&added, &root.join("state"), false)
                .unwrap()
                .succeeded()
        );
        assert_eq!(read_finder_tags(&file).unwrap(), tags);
        let omitted = Preset::from_toml("schema_version=1\n[settings]\nnote='unrelated'").unwrap();
        assert!(
            crate::apply(
                &preset_plan(&folder, &omitted, false).unwrap(),
                &root.join("state")
            )
            .unwrap()
            .succeeded()
        );
        assert_eq!(read_finder_tags_raw(&file).unwrap(), Some(raw));
        let clear =
            Preset::from_toml("schema_version=2\nplatform='macos'\n[settings]\nfinder_tags=[]")
                .unwrap();
        assert_eq!(
            crate::check_preset(&crate::inspect(&folder).unwrap(), &clear)
                .unwrap()
                .status,
            crate::ComplianceStatus::Drift
        );
        let record = crate::apply(
            &preset_plan(&folder, &clear, false).unwrap(),
            &root.join("state"),
        )
        .unwrap();
        assert!(record.succeeded());
        assert_eq!(read_finder_tags_raw(&file).unwrap(), None);
        assert!(
            crate::undo(&record, &root.join("state"), false)
                .unwrap()
                .succeeded()
        );
        assert_eq!(read_finder_tags(&file).unwrap(), tags);
        let mut legacy = vec![0; 32];
        legacy[9] = 8;
        write_finder_tags_raw(&file, None).unwrap();
        crate::platform::write_xattr(&file, FINDER_INFO, Some(&legacy)).unwrap();
        assert_eq!(
            crate::check_preset(&crate::inspect(&folder).unwrap(), &clear)
                .unwrap()
                .status,
            crate::ComplianceStatus::Indeterminate
        );
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn native_tag_preset_refuses_linux_without_mutating_other_fields() {
        let temp = tempfile::tempdir().unwrap();
        let preset=Preset::from_toml("schema_version=2\nplatform='macos'\n[settings]\nnote='must not write'\nfinder_tags=['native']").unwrap();
        assert!(matches!(
            preset_plan(temp.path(), &preset, false),
            Err(FoldrError::Unsupported(_))
        ));
        assert_eq!(crate::read_note(temp.path()).unwrap(), None);
    }
    #[test]
    fn partial_preset_roundtrip_preserves_binary_and_deletions() {
        let mut request = ChangeRequest {
            note: Some(None),
            mode: Some(0o700),
            ..Default::default()
        };
        request
            .metadata
            .insert("binary".into(), Some(vec![0, 255, 10]));
        request.metadata.insert("old".into(), None);
        let preset = Preset {
            schema_version: 1,
            name: Some("private".into()),
            platform: None,
            request,
        };
        assert_eq!(
            Preset::from_toml(&preset.to_toml().unwrap()).unwrap(),
            preset
        );
    }
    #[test]
    fn unknown_schema_or_fields_are_refused() {
        assert!(Preset::from_toml("schema_version=3\n[settings]").is_err());
        assert!(Preset::from_toml("schema_version=1\n[settings]\nmagic=true").is_err());
    }
    #[test]
    fn applying_partial_preset_preserves_unmentioned_fields() {
        let temp = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(temp.path()).unwrap();
        let folder = root.join("folder");
        std::fs::create_dir(&folder).unwrap();
        let before = crate::inspect(&folder).unwrap();
        let preset = Preset::from_toml("schema_version=1\n[settings]\nnote='project'").unwrap();
        let record = apply(
            &preset_plan(&folder, &preset, false).unwrap(),
            &root.join("state"),
        )
        .unwrap();
        assert!(record.succeeded());
        let after = crate::inspect(folder).unwrap();
        assert_eq!(before.mode, after.mode);
        assert_eq!(before.flags, after.flags);
    }
    #[test]
    fn shipped_examples_roundtrip_and_have_exact_scopes() {
        for text in [
            include_str!("../../../examples/presets/project.toml"),
            include_str!("../../../examples/presets/archive.toml"),
            include_str!("../../../examples/presets/private.toml"),
        ] {
            let preset = Preset::from_toml(text).unwrap();
            assert_eq!(
                Preset::from_toml(&preset.to_toml().unwrap()).unwrap(),
                preset
            );
            assert!(preset.request.metadata.is_empty());
            assert_eq!(preset.request.immutable, None);
            assert_eq!(preset.request.hidden, None);
        }
    }
    #[test]
    fn binary_note_can_be_saved_as_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let mut snapshot = crate::inspect(temp.path()).unwrap();
        snapshot.xattrs = Property::Supported {
            value: vec![ExtendedAttribute {
                name: EncodedPath::from_bytes(metadata_name("note").unwrap()),
                value: vec![0, 255],
            }],
        };
        let preset = preset_from_snapshot(&snapshot, &["metadata".into()]).unwrap();
        assert_eq!(preset.request.metadata["note"], Some(vec![0, 255]));
        assert_eq!(
            Preset::from_toml(&preset.to_toml().unwrap()).unwrap(),
            preset
        );
    }
    #[test]
    fn batch_dry_run_and_failures_preserve_successful_targets() {
        let temp = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(temp.path()).unwrap();
        let good = root.join("good");
        std::fs::create_dir(&good).unwrap();
        let missing = root.join("missing");
        let preset = Preset::from_toml("schema_version=1\n[settings]\nnote='batch'").unwrap();
        let state = root.join("state");
        let paths = vec![good.clone(), missing];
        let preview = apply_batch(&paths, &preset, &state, false, true);
        assert_eq!(preview.len(), 2);
        assert!(preview[0].plan.is_some());
        assert!(preview[1].error.is_some());
        assert_eq!(read_note(&good).unwrap(), None);
        assert!(!state.exists());
        let results = apply_batch(&paths, &preset, &state, false, false);
        assert!(results[0].record.as_ref().unwrap().succeeded());
        assert!(results[1].error.is_some());
        assert_eq!(history(&state).unwrap().len(), 1);
        assert_eq!(read_note(good).unwrap(), Some(b"batch".to_vec()));
    }
}
