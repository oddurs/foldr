//! Descriptor-based native adapters. Inspection never probes support by writing.
use crate::model::*;
use std::{fs::File, io};
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
mod raw;
#[cfg(target_os = "linux")]
use linux as native;
#[cfg(target_os = "macos")]
use macos as native;

pub const IMMUTABLE_MASK: u64 = native::IMMUTABLE_MASK;
pub const HIDDEN_MASK: Option<u64> = native::HIDDEN_MASK;
pub fn inspect_native(file: &File) -> NativeInspection {
    native::inspect(file)
}
pub fn read_xattr(file: &File, name: &[u8]) -> Result<Option<Vec<u8>>, FoldrError> {
    raw::read_xattr(file, name).map_err(|e| FoldrError::io("read extended attribute", e))
}
pub fn write_xattr(file: &File, name: &[u8], value: Option<&[u8]>) -> Result<(), FoldrError> {
    raw::write_xattr(file, name, value).map_err(|e| FoldrError::io("write extended attribute", e))
}
pub fn read_flags(file: &File) -> Result<u64, FoldrError> {
    native::read_flags(file).map_err(|e| FoldrError::io("read native flags", e))
}
pub fn write_flags(file: &File, flags: u64) -> Result<(), FoldrError> {
    native::write_flags(file, flags).map_err(|e| FoldrError::io("write native flags", e))
}

fn property<T>(result: io::Result<T>) -> Property<T> {
    match result {
        Ok(value) => Property::Supported { value },
        Err(error) => match error.raw_os_error() {
            Some(code)
                if code == libc::ENOTSUP || code == libc::EOPNOTSUPP || code == libc::ENOTTY =>
            {
                Property::Unsupported {
                    reason: format!(
                        "native operation is not supported on this filesystem: {error}"
                    ),
                }
            }
            Some(code) if code == libc::EPERM || code == libc::EACCES => {
                Property::PermissionDenied {
                    reason: format!("current user cannot read this property: {error}"),
                }
            }
            _ => Property::Unavailable {
                reason: error.to_string(),
            },
        },
    }
}
fn state<T>(value: &Property<T>) -> CapabilityState {
    match value {
        Property::Supported { .. } => CapabilityState::Supported,
        Property::Unsupported { reason } => CapabilityState::Unsupported {
            reason: reason.clone(),
        },
        Property::Unknown { reason } => CapabilityState::Unknown {
            reason: reason.clone(),
        },
        Property::PermissionDenied { reason } => CapabilityState::PermissionDenied {
            reason: reason.clone(),
        },
        Property::Unavailable { reason } => CapabilityState::Unavailable {
            reason: reason.clone(),
        },
    }
}
fn capability<T>(
    name: &str,
    property: &Property<T>,
    filesystem: &Property<FilesystemInfo>,
    description: &str,
) -> Capability {
    let read = state(property);
    let write=match property {
        Property::Unsupported{reason}=>CapabilityState::Unsupported{reason:reason.clone()},
        _ if filesystem.value().is_some_and(|fs|fs.read_only)=>CapabilityState::Unavailable{reason:"filesystem is mounted read-only".into()},
        _=>CapabilityState::Unknown{reason:"inspection is read-only; write permission and filesystem policy are checked when applying a change".into()},
    };
    Capability {
        name: name.into(),
        read,
        write,
        scope: Scope::Folder,
        description: description.into(),
    }
}
fn xattrs(file: &File) -> io::Result<Vec<ExtendedAttribute>> {
    let mut attributes = Vec::new();
    for name in raw::list_xattrs(file)? {
        if let Some(value) = raw::read_xattr(file, &name)? {
            attributes.push(ExtendedAttribute {
                name: EncodedPath::from_bytes(name),
                value,
            });
        }
    }
    attributes.sort_by(|a, b| a.name.bytes.cmp(&b.name.bytes));
    Ok(attributes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsupported_permission_and_unavailable_remain_distinct() {
        assert!(matches!(
            property::<()>(Err(io::Error::from_raw_os_error(libc::ENOTTY))),
            Property::Unsupported { .. }
        ));
        assert!(matches!(
            property::<()>(Err(io::Error::from_raw_os_error(libc::EACCES))),
            Property::PermissionDenied { .. }
        ));
        assert!(matches!(
            property::<()>(Err(io::Error::from_raw_os_error(libc::EIO))),
            Property::Unavailable { .. }
        ));
        let filesystem = Property::Supported {
            value: FilesystemInfo {
                name: "test".into(),
                type_id: None,
                read_only: false,
            },
        };
        let cap = capability(
            "flags",
            &Property::Supported { value: 0u64 },
            &filesystem,
            "test",
        );
        assert!(matches!(cap.write, CapabilityState::Unknown { .. }));
    }
    #[test]
    fn binary_xattrs_survive_path_replacement_and_preserve_unrelated_values() {
        let parent = tempfile::tempdir().unwrap();
        let path = parent.path().join("directory");
        std::fs::create_dir(&path).unwrap();
        let file = File::open(&path).unwrap();
        #[cfg(target_os = "macos")]
        let prefix = b"com.foldr.".as_slice();
        #[cfg(target_os = "linux")]
        let prefix = b"user.foldr.".as_slice();
        let mut key = prefix.to_vec();
        key.extend_from_slice(b"binary");
        // Linux accepts arbitrary non-NUL names. Darwin validates UTF-8 names.
        #[cfg(target_os = "linux")]
        key.push(255);
        let mut other = prefix.to_vec();
        other.extend_from_slice(b"other");
        let bytes = [0, 255, 1, 0, 10];
        write_xattr(&file, &key, Some(&bytes)).unwrap();
        write_xattr(&file, &other, Some(b"preserved")).unwrap();
        std::fs::rename(&path, parent.path().join("renamed")).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert_eq!(read_xattr(&file, &key).unwrap(), Some(bytes.to_vec()));
        assert_eq!(read_xattr(&File::open(&path).unwrap(), &key).unwrap(), None);
        write_xattr(&file, &key, Some(b"")).unwrap();
        assert_eq!(read_xattr(&file, &key).unwrap(), Some(vec![]));
        write_xattr(&file, &key, None).unwrap();
        assert_eq!(read_xattr(&file, &key).unwrap(), None);
        assert_eq!(
            read_xattr(&file, &other).unwrap(),
            Some(b"preserved".to_vec())
        );
    }
    #[test]
    fn invalid_xattr_name_is_rejected_without_native_call() {
        let folder = tempfile::tempdir().unwrap();
        let file = File::open(folder.path()).unwrap();
        assert!(write_xattr(&file, b"com.foldr.bad\0key", Some(b"v")).is_err());
    }
    #[test]
    fn inspection_does_not_change_directory_metadata() {
        use std::os::unix::fs::MetadataExt;
        let folder = tempfile::tempdir().unwrap();
        let file = File::open(folder.path()).unwrap();
        let before = file.metadata().unwrap();
        let inspected = inspect_native(&file);
        assert!(inspected.filesystem.value().is_some());
        #[cfg(target_os = "macos")]
        assert!(inspected.flags.value().is_some());
        #[cfg(target_os = "linux")]
        assert!(matches!(
            inspected.flags,
            Property::Supported { .. } | Property::Unsupported { .. }
        ));
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
    }
}
