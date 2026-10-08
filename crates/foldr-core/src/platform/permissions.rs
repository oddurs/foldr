//! Read-only directory permission explanation. This is not an access oracle.
use crate::{
    inspect::{identity, open_directory},
    model::*,
};
use serde::{Deserialize, Serialize};
#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(any(target_os = "linux", test))]
use std::io;
use std::{os::unix::fs::MetadataExt, path::Path};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeClass {
    pub class: String,
    pub list: bool,
    pub search: bool,
    pub modify_entries: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AclEntry {
    pub index: usize,
    pub scope: String,
    pub principal: String,
    pub disposition: String,
    pub rights: Vec<String>,
    pub raw_rights: u64,
    pub raw_flags: u64,
    pub inheritance: Vec<String>,
    pub unknown_rights: u64,
    pub unknown_flags: u64,
    /// Linux mask intersection for named users/groups and owning group only.
    pub effective_rights: Option<Vec<String>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeAcl {
    pub entries: Vec<AclEntry>,
    pub raw_flags: u64,
    pub unknown_flags: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionExplanation {
    pub schema_version: u32,
    pub path: EncodedPath,
    pub identity: FolderIdentity,
    pub platform: String,
    pub owner: u32,
    pub group: u32,
    pub mode: u32,
    pub mode_classes: Vec<ModeClass>,
    pub sticky: bool,
    pub setgid: bool,
    pub filesystem: Property<FilesystemInfo>,
    pub flags: Property<NativeFlags>,
    pub acl: Property<NativeAcl>,
    pub observations: Vec<String>,
    pub limitations: Vec<String>,
}
pub fn explain(path: impl AsRef<Path>) -> Result<PermissionExplanation, FoldrError> {
    let path = path.as_ref();
    let file = open_directory(path, true)?;
    let metadata = file.metadata()?;
    let native = super::inspect_native(&file);
    let mode = metadata.mode() & 0o7777;
    #[cfg(target_os = "macos")]
    let acl = super::property(super::macos::acl_detail(&file));
    #[cfg(target_os = "linux")]
    let acl = super::property(linux_acl(&file));
    let mode_classes = [("owner", 6), ("group", 3), ("other", 0)]
        .into_iter()
        .map(|(class, shift)| {
            let rights = (mode >> shift) & 7;
            ModeClass {
                class: class.into(),
                list: rights & 4 != 0,
                search: rights & 1 != 0,
                modify_entries: rights & 3 == 3,
            }
        })
        .collect();
    let mut observations = vec![
        "Read permits listing entry names; search (execute) permits traversing known names. Listing does not imply traversal.".into(),
        "Write together with search permits creating, renaming and deleting directory entries; it does not grant write access to existing child file contents.".into(),
        format!("Sticky bit is {}: when set, entry deletion/rename is restricted to the entry owner, directory owner or a privileged process.", if mode&0o1000!=0 {"set"} else {"clear"}),
        format!("Setgid is {}: on Linux new children inherit the directory group and new subdirectories retain setgid; moved-in/existing children are not rewritten. macOS child group rules are native filesystem policy; setgid alone is not a portable guarantee.", if mode&0o2000!=0 {"set"} else {"clear"}),
    ];
    if cfg!(target_os = "linux") {
        observations.extend([
            "Linux access ACL applies to this directory. Its mask limits named users, owning group and named groups; owner and other are not masked. Mode group bits show the mask when one exists.".into(),
            "Linux default ACL seeds access ACLs of newly created children, limited by the application's requested creation mode. Without a default ACL, requested mode is reduced by umask. Existing or moved-in children keep their ACLs.".into(),
        ]);
    } else {
        observations.extend([
            "macOS ordered ACL entries identify UUID principals and allow/deny directory rights. File-inherit/directory-inherit select future child types; inherit-only does not apply to this directory; limit-inherit restricts further propagation.".into(),
            "Inherited marks an entry inherited from a parent. Ordinary creation inherits eligible entries; existing children are not recursively changed, and moved-in children retain their metadata unless native deferred-inheritance policy applies.".into(),
        ]);
    }
    match native.filesystem.value() {
        Some(fs) if fs.read_only => observations
            .push("Observed read-only mount prevents writes despite mode/ACL grants.".into()),
        Some(_) => observations.push(
            "Observed writable mount does not establish permission to perform a particular change."
                .into(),
        ),
        None => observations.push(
            "Mount write policy could not be determined; see filesystem property state.".into(),
        ),
    }
    match native.flags.value() {
        Some(flags) if flags.names.iter().any(|name| name.contains("immutable")) => observations.push("Observed immutable flag restricts directory-entry changes; existing child contents can still be writable.".into()),
        Some(_) => observations.push("No immutable flag was observed; other native policies can still restrict changes.".into()),
        None => observations.push("Immutable state could not be determined; see flags property state.".into()),
    }
    let limitations = vec![
        "Observed rules are not a complete effective-access decision for any user: credentials, supplementary groups, ancestor search, privileges, ACL ordering, mount policy and security modules can affect the result.".into(),
        "Denied, unsupported, unavailable and unknown properties remain explicit. Unknown ACL flags/rights are retained numerically and are not interpreted.".into(),
        "This command performs no writes, creates no recovery records and does not edit ACLs. Mode editing still refuses extended ACLs.".into(),
    ];
    Ok(PermissionExplanation {
        schema_version: 1,
        path: EncodedPath::new(path),
        identity: identity(&metadata),
        platform: native.platform,
        owner: metadata.uid(),
        group: metadata.gid(),
        mode,
        mode_classes,
        sticky: mode & 0o1000 != 0,
        setgid: mode & 0o2000 != 0,
        filesystem: native.filesystem,
        flags: native.flags,
        acl,
        observations,
        limitations,
    })
}
#[cfg(any(target_os = "linux", test))]
fn rights(bits: u64) -> Vec<String> {
    [(4, "list"), (2, "write entries"), (1, "search")]
        .into_iter()
        .filter(|(mask, _)| bits & mask != 0)
        .map(|(_, name)| name.into())
        .collect()
}
#[cfg(any(target_os = "linux", test))]
fn decode_linux_acl(bytes: &[u8], scope: &str) -> io::Result<Vec<AclEntry>> {
    if bytes.len() < 4
        || (bytes.len() - 4) % 8 != 0
        || u32::from_le_bytes(bytes[..4].try_into().unwrap()) != 2
    {
        return Err(io::Error::other(
            "unsupported or malformed POSIX ACL xattr encoding",
        ));
    }
    let mut entries = Vec::new();
    let mut mask = None;
    for (index, entry) in bytes[4..].chunks_exact(8).enumerate() {
        let tag = u16::from_le_bytes(entry[..2].try_into().unwrap());
        let bits = u16::from_le_bytes(entry[2..4].try_into().unwrap()) as u64;
        let id = u32::from_le_bytes(entry[4..8].try_into().unwrap());
        if bits & !7 != 0 {
            return Err(io::Error::other("malformed POSIX ACL rights"));
        }
        let principal = match tag {
            1 => "owner".into(),
            2 => format!("user {id}"),
            4 => "owning group".into(),
            8 => format!("group {id}"),
            16 => {
                mask = Some(bits);
                "mask".into()
            }
            32 => "other".into(),
            _ => return Err(io::Error::other("unknown POSIX ACL entry tag")),
        };
        if matches!(tag, 2 | 8) == (id == u32::MAX) {
            return Err(io::Error::other("malformed POSIX ACL qualifier"));
        }
        entries.push(AclEntry {
            index,
            scope: scope.into(),
            principal,
            disposition: "allow".into(),
            rights: rights(bits),
            raw_rights: bits,
            raw_flags: tag as u64,
            inheritance: vec![],
            unknown_rights: 0,
            unknown_flags: 0,
            effective_rights: None,
        });
    }
    if entries.is_empty() {
        if scope == "default" {
            return Ok(entries);
        }
        return Err(io::Error::other("empty access ACL is malformed"));
    }
    for mandatory in [1, 4, 32] {
        if entries.iter().filter(|e| e.raw_flags == mandatory).count() != 1 {
            return Err(io::Error::other("malformed POSIX ACL base entries"));
        }
    }
    if entries.iter().filter(|e| e.raw_flags == 16).count() > 1
        || (entries.iter().any(|e| matches!(e.raw_flags, 2 | 8)) && mask.is_none())
    {
        return Err(io::Error::other("malformed POSIX ACL mask"));
    }
    let mut principals = std::collections::BTreeSet::new();
    for entry in &mut entries {
        if !principals.insert(entry.principal.clone()) {
            return Err(io::Error::other("duplicate POSIX ACL principal"));
        }
        if matches!(entry.raw_flags, 2 | 4 | 8) {
            entry.effective_rights = mask.map(|mask| rights(entry.raw_rights & mask));
        }
    }
    Ok(entries)
}
#[cfg(target_os = "linux")]
fn linux_acl(file: &File) -> io::Result<NativeAcl> {
    let mut entries = Vec::new();
    for (key, scope) in [
        (b"system.posix_acl_access".as_slice(), "access"),
        (b"system.posix_acl_default".as_slice(), "default"),
    ] {
        if let Some(bytes) = super::raw::read_xattr(file, key)? {
            entries.extend(decode_linux_acl(&bytes, scope)?);
        }
    }
    Ok(NativeAcl {
        entries,
        raw_flags: 0,
        unknown_flags: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn linux_bytes(entries: &[(u16, u16, u32)]) -> Vec<u8> {
        let mut bytes = 2u32.to_le_bytes().to_vec();
        for (tag, bits, id) in entries {
            bytes.extend_from_slice(&tag.to_le_bytes());
            bytes.extend_from_slice(&bits.to_le_bytes());
            bytes.extend_from_slice(&id.to_le_bytes());
        }
        bytes
    }
    #[test]
    fn linux_mask_and_malformed_principals_are_explicit() {
        let entries = [
            (1, 7, u32::MAX),
            (2, 7, 1234),
            (4, 7, u32::MAX),
            (16, 5, u32::MAX),
            (32, 0, u32::MAX),
        ];
        let decoded = decode_linux_acl(&linux_bytes(&entries), "default").unwrap();
        assert_eq!(decoded[1].principal, "user 1234");
        assert_eq!(decoded[1].scope, "default");
        assert_eq!(decoded[1].rights, vec!["list", "write entries", "search"]);
        assert_eq!(
            decoded[1].effective_rights,
            Some(vec!["list".into(), "search".into()])
        );
        assert!(decoded[0].effective_rights.is_none());
        assert!(decode_linux_acl(b"bad", "access").is_err());
        assert!(decode_linux_acl(&linux_bytes(&[(2, 7, u32::MAX)]), "access").is_err());
        assert!(decode_linux_acl(&linux_bytes(&[(99, 7, 1)]), "access").is_err());
        assert!(decode_linux_acl(&linux_bytes(&[(1, 8, u32::MAX)]), "access").is_err());
        assert!(decode_linux_acl(&linux_bytes(&entries[..3]), "access").is_err());
    }
    #[test]
    fn explanation_is_read_only_and_escapes_path() {
        let parent = tempfile::tempdir().unwrap();
        let path = parent.path().join("folder\n\x1b");
        std::fs::create_dir(&path).unwrap();
        use std::os::fd::AsRawFd;
        use std::os::unix::fs::PermissionsExt;
        let directory = std::fs::File::open(&path).unwrap();
        // SAFETY: fixture descriptor live; own directory, unchanged uid and own gid.
        let status = unsafe { libc::fchown(directory.as_raw_fd(), u32::MAX, libc::getgid()) };
        assert_eq!(status, 0);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o3750)).unwrap();
        std::fs::write(path.join("existing"), b"contents").unwrap();
        let before = std::fs::metadata(&path).unwrap();
        let result = explain(&path).unwrap();
        assert!(result.sticky);
        assert_eq!(result.setgid, before.mode() & 0o2000 != 0);
        #[cfg(target_os = "linux")]
        assert!(result.setgid);
        assert!(!result.path.display.contains('\n'));
        assert!(!result.path.display.contains('\x1b'));
        assert!(result.mode_classes[0].modify_entries);
        assert!(result.mode_classes[1].list && result.mode_classes[1].search);
        assert!(!result.mode_classes[1].modify_entries);
        let after = std::fs::metadata(&path).unwrap();
        assert_eq!(
            (
                before.mode(),
                before.ctime(),
                before.ctime_nsec(),
                before.mtime(),
                before.mtime_nsec()
            ),
            (
                after.mode(),
                after.ctime(),
                after.ctime_nsec(),
                after.mtime(),
                after.mtime_nsec()
            )
        );
        assert_eq!(std::fs::read(path.join("existing")).unwrap(), b"contents");
        assert!(!parent.path().join("state").exists());
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn native_ordered_acl_inheritance_and_metadata_are_preserved() {
        use std::{fs::File, process::Command};
        let folder = tempfile::tempdir().unwrap();
        let path = std::fs::canonicalize(folder.path()).unwrap();
        // A disposable native fixture only; production never edits ACLs.
        let allow = "everyone allow list,search,add_file,file_inherit,directory_inherit";
        let deny = "everyone deny delete_child";
        for (index, entry) in [allow, deny].into_iter().enumerate() {
            let output = Command::new("/bin/chmod")
                .args(["+a#", &index.to_string(), entry])
                .arg(&path)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "ACL fixture setup: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let child = path.join("child");
        std::fs::create_dir(&child).unwrap();
        let file = File::open(&path).unwrap();
        super::super::write_xattr(&file, b"com.foldr.fixture", Some(&[0, 255, 1])).unwrap();
        let before = file.metadata().unwrap();
        let before_acl = super::super::macos::acl_detail(&file).unwrap();
        let result = explain(&path).unwrap();
        let acl = result.acl.value().unwrap();
        assert_eq!(acl, &before_acl);
        assert_eq!(acl.entries.len(), 2);
        assert_eq!(acl.entries[0].disposition, "allow");
        assert_eq!(acl.entries[1].disposition, "deny");
        assert!(acl.entries[0].principal.starts_with("uuid "));
        assert!(acl.entries[0].rights.contains(&"list".into()));
        assert!(acl.entries[0].rights.contains(&"search".into()));
        assert!(acl.entries[0].inheritance.contains(&"file inherit".into()));
        assert!(
            acl.entries[0]
                .inheritance
                .contains(&"directory inherit".into())
        );
        assert!(acl.entries[1].rights.contains(&"delete child".into()));
        let inherited = explain(&child).unwrap();
        assert!(
            inherited
                .acl
                .value()
                .unwrap()
                .entries
                .iter()
                .any(|entry| entry.inheritance.contains(&"inherited".into()))
        );
        let mode_change = crate::changes::plan(
            &path,
            &crate::changes::ChangeRequest {
                mode: Some(0o700),
                ..Default::default()
            },
            false,
        );
        assert!(matches!(mode_change, Err(FoldrError::Unsupported(_))));
        let after = file.metadata().unwrap();
        assert_eq!(
            (
                before.mode(),
                before.ctime(),
                before.ctime_nsec(),
                before.mtime(),
                before.mtime_nsec()
            ),
            (
                after.mode(),
                after.ctime(),
                after.ctime_nsec(),
                after.mtime(),
                after.mtime_nsec()
            )
        );
        assert_eq!(
            super::super::read_xattr(&file, b"com.foldr.fixture").unwrap(),
            Some(vec![0, 255, 1])
        );
        assert_eq!(super::super::macos::acl_detail(&file).unwrap(), before_acl);
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn native_linux_default_acl_mask_and_preservation() {
        use std::fs::File;
        let folder = tempfile::tempdir().unwrap();
        let file = File::open(folder.path()).unwrap();
        let bytes = linux_bytes(&[
            (1, 7, u32::MAX),
            (2, 7, 1234),
            (4, 5, u32::MAX),
            (16, 5, u32::MAX),
            (32, 0, u32::MAX),
        ]);
        let key = b"system.posix_acl_default";
        if let Err(error) = super::super::raw::write_xattr(&file, key, Some(&bytes)) {
            if error.raw_os_error() == Some(libc::EOPNOTSUPP) {
                assert!(matches!(
                    super::super::property(linux_acl(&file)),
                    Property::Unsupported { .. }
                ));
                return;
            }
            panic!("native ACL fixture: {error}");
        }
        std::fs::write(folder.path().join("existing"), b"contents").unwrap();
        let before = file.metadata().unwrap();
        let result = explain(folder.path()).unwrap();
        let acl = result.acl.value().unwrap();
        let named = acl
            .entries
            .iter()
            .find(|entry| entry.principal == "user 1234")
            .unwrap();
        assert_eq!(named.scope, "default");
        assert_eq!(
            named.effective_rights,
            Some(vec!["list".into(), "search".into()])
        );
        let after = file.metadata().unwrap();
        assert_eq!(
            (
                before.mode(),
                before.ctime(),
                before.ctime_nsec(),
                before.mtime(),
                before.mtime_nsec()
            ),
            (
                after.mode(),
                after.ctime(),
                after.ctime_nsec(),
                after.mtime(),
                after.mtime_nsec()
            )
        );
        assert_eq!(
            super::super::raw::read_xattr(&file, key).unwrap(),
            Some(bytes)
        );
        assert_eq!(
            std::fs::read(folder.path().join("existing")).unwrap(),
            b"contents"
        );
        assert!(matches!(
            crate::changes::plan(
                folder.path(),
                &crate::changes::ChangeRequest {
                    mode: Some(0o700),
                    ..Default::default()
                },
                false
            ),
            Err(FoldrError::Unsupported(_))
        ));
    }
}
