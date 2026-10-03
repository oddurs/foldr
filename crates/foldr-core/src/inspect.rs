//! Inspection follows directory symlinks and reports their resolved target.
//! Mutations reject every symlink component unless explicitly enabled.
use crate::{model::*, platform};
use std::{
    ffi::CString,
    fs::{self, File, OpenOptions},
    os::fd::{AsRawFd, FromRawFd},
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Component, Path},
};

pub fn inspect(path: impl AsRef<Path>) -> Result<FolderSnapshot, FoldrError> {
    let path = path.as_ref();
    let requested = fs::symlink_metadata(path).map_err(|e| FoldrError::io("inspect path", e))?;
    let symlink_target = if requested.file_type().is_symlink() {
        Some(EncodedPath::new(fs::canonicalize(path)?))
    } else {
        None
    };
    let directory = open_directory(path, true)?;
    let metadata = directory.metadata()?;
    let native = platform::inspect_native(&directory);
    Ok(FolderSnapshot {
        schema_version: 1,
        path: EncodedPath::new(path),
        symlink_target,
        identity: identity(&metadata),
        owner: metadata.uid(),
        group: metadata.gid(),
        mode: metadata.mode() & 0o7777,
        platform: native.platform,
        filesystem: native.filesystem,
        acl: native.acl,
        flags: native.flags,
        xattrs: native.xattrs,
        capabilities: native.capabilities,
    })
}
pub(crate) fn identity(metadata: &fs::Metadata) -> FolderIdentity {
    FolderIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    }
}
pub(crate) fn open_directory(path: &Path, follow: bool) -> Result<File, FoldrError> {
    if !follow {
        return open_without_symlinks(path);
    }
    let flags = libc::O_DIRECTORY | libc::O_CLOEXEC | if follow { 0 } else { libc::O_NOFOLLOW };
    OpenOptions::new()
        .read(true)
        .custom_flags(flags)
        .open(path)
        .map_err(|e| FoldrError::io("open directory", e))
}
fn open_without_symlinks(path: &Path) -> Result<File, FoldrError> {
    #[cfg(target_os = "macos")]
    let access = libc::O_SEARCH;
    #[cfg(target_os = "linux")]
    let access = libc::O_PATH;
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let access = libc::O_RDONLY;
    let traversal_flags = access | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW;
    let base = if path.is_absolute() { c"/" } else { c"." };
    let mut directory = open_at(libc::AT_FDCWD, base, traversal_flags)?;
    for component in path.components() {
        let name = match component {
            Component::RootDir | Component::CurDir => continue,
            Component::Normal(name) => name.as_bytes(),
            Component::ParentDir => b"..",
            Component::Prefix(_) => {
                return Err(FoldrError::InvalidInput("unsupported path prefix".into()));
            }
        };
        let name =
            CString::new(name).map_err(|_| FoldrError::InvalidInput("NUL byte in path".into()))?;
        directory = open_at(directory.as_raw_fd(), &name, traversal_flags)?;
    }
    // Search-only descriptors avoid requiring list permission on ancestors.
    // Reopen this already-held object for native reads and writes; no pathname
    // components are resolved again, so ancestor replacement cannot redirect it.
    open_at(
        directory.as_raw_fd(),
        c".",
        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
    )
}
fn open_at(
    parent: libc::c_int,
    name: &std::ffi::CStr,
    flags: libc::c_int,
) -> Result<File, FoldrError> {
    // SAFETY: the parent descriptor and NUL-terminated name remain valid during
    // openat. AT_FDCWD is a valid selector for the first directory in the walk.
    let descriptor = unsafe { libc::openat(parent, name.as_ptr(), flags) };
    if descriptor < 0 {
        let error = std::io::Error::last_os_error();
        if matches!(
            error.raw_os_error(),
            Some(libc::ELOOP) | Some(libc::ENOTDIR)
        ) {
            return Err(FoldrError::InvalidInput("path contains a non-directory or symlink component; symlinks require --follow-symlink".into()));
        }
        return Err(FoldrError::io(
            "open directory component (symlinks require --follow-symlink)",
            error,
        ));
    }
    // SAFETY: successful openat returned a new descriptor owned by this File.
    Ok(unsafe { File::from_raw_fd(descriptor) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::{ffi::OsStringExt, fs::symlink};
    #[test]
    fn encoding_is_lossless_and_terminal_safe() {
        let raw = b"folder\xff\n\x1b".to_vec();
        let value = EncodedPath::from_bytes(raw.clone());
        assert_eq!(value.to_path_buf().as_os_str().as_bytes(), raw);
        assert!(!value.display.contains('\n'));
        assert!(!value.display.contains('\x1b'));
        let decoded: EncodedPath =
            serde_json::from_str(&serde_json::to_string(&value).unwrap()).unwrap();
        assert_eq!(decoded, value);
    }
    #[test]
    fn distinguishes_property_states() {
        assert_ne!(
            Property::<u8>::Unknown { reason: "x".into() },
            Property::Unsupported { reason: "x".into() }
        );
        assert!(
            Property::<u8>::from_io(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
                .reason()
                .is_some()
        );
    }
    #[test]
    fn symlink_mutation_requires_explicit_choice() {
        let temp = tempfile::tempdir().unwrap();
        let folder = temp.path().join("folder");
        fs::create_dir(&folder).unwrap();
        let link = temp.path().join("link");
        symlink(&folder, &link).unwrap();
        assert!(open_directory(&link, false).is_err());
        assert!(open_directory(&link, true).is_ok());
    }
    #[test]
    fn non_utf8_folder_opens() {
        let temp = tempfile::tempdir().unwrap();
        let folder = fs::canonicalize(temp.path())
            .unwrap()
            .join(std::ffi::OsString::from_vec(b"folder\xff".to_vec()));
        match fs::create_dir(&folder) {
            Ok(()) => assert!(open_directory(&folder, false).is_ok()),
            Err(error)
                if cfg!(target_os = "macos") && error.raw_os_error() == Some(libc::EILSEQ) => {}
            Err(error) => panic!("create non-UTF8 folder: {error}"),
        }
    }
    #[test]
    fn searchable_unlistable_parents_allow_edits_without_following_symlinks() {
        use std::os::unix::fs::PermissionsExt;
        struct RestorePermissions(File, u32);
        impl Drop for RestorePermissions {
            fn drop(&mut self) {
                let _ = self.0.set_permissions(fs::Permissions::from_mode(self.1));
            }
        }
        let temp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temp.path()).unwrap();
        let parent = root.join("parent");
        let target = parent.join("target");
        fs::create_dir(&parent).unwrap();
        fs::create_dir(&target).unwrap();
        let parent_file = File::open(&parent).unwrap();
        let mode = parent_file.metadata().unwrap().mode();
        let restore = RestorePermissions(parent_file, mode);
        let alias = root.join("parent-link");
        symlink(&parent, &alias).unwrap();
        restore
            .0
            .set_permissions(fs::Permissions::from_mode(0o111))
            .unwrap();
        assert!(open_directory(&target, false).is_ok());
        let request = crate::ChangeRequest {
            note: Some(Some("search-only parent".into())),
            ..Default::default()
        };
        let record = crate::apply(
            &crate::plan(&target, &request, false).unwrap(),
            &root.join("state"),
        )
        .unwrap();
        assert!(record.succeeded());
        assert_eq!(
            crate::read_note(&target).unwrap(),
            Some(b"search-only parent".to_vec())
        );
        assert!(matches!(
            open_directory(&alias.join("target"), false),
            Err(FoldrError::InvalidInput(_))
        ));
        restore
            .0
            .set_permissions(fs::Permissions::from_mode(0o0))
            .unwrap();
        // SAFETY: geteuid takes no pointers and cannot violate memory safety.
        if unsafe { libc::geteuid() } != 0 {
            assert!(open_directory(&target, false).is_err());
        }
    }
}
