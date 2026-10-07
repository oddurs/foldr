//! Linux descriptor APIs. ACL summaries decode the documented kernel xattr ABI.
use super::{capability, property, raw, xattrs};
use crate::model::*;
use std::{fs::File, io, mem::MaybeUninit, os::fd::AsRawFd};
pub const IMMUTABLE_MASK: u64 = 0x10; // FS_IMMUTABLE_FL
pub const HIDDEN_MASK: Option<u64> = None;
fn filesystem(file: &File) -> io::Result<FilesystemInfo> {
    let mut value = MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: descriptor is live; value provides writable statfs storage.
    if unsafe { libc::fstatfs(file.as_raw_fd(), value.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful fstatfs initializes the struct.
    let value = unsafe { value.assume_init() };
    let type_id = value.f_type as u64;
    let name = match type_id {
        0xef53 => "ext",
        0x9123683e => "btrfs",
        0x58465342 => "xfs",
        0x01021994 => "tmpfs",
        0x794c7630 => "overlay",
        0x6969 => "nfs",
        0xff534d42 => "cifs",
        0x65735546 => "fuse",
        0x2fc12fc1 => "zfs",
        0x4d44 => "fat",
        0x2011bab0 => "exfat",
        0x5346544e => "ntfs",
        0x73717368 => "squashfs",
        _ => "unknown",
    };
    // fstatvfs carries portable mount flags; unlike filesystem names these do
    // not guess behavior from a filesystem family.
    let mut mount = MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: descriptor live and mount provides writable statvfs storage.
    if unsafe { libc::fstatvfs(file.as_raw_fd(), mount.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful fstatvfs initializes the struct.
    let mount = unsafe { mount.assume_init() };
    Ok(FilesystemInfo {
        name: name.into(),
        type_id: Some(type_id),
        read_only: mount.f_flag & libc::ST_RDONLY != 0,
    })
}
pub(super) fn read_flags(file: &File) -> io::Result<u64> {
    let mut flags: libc::c_int = 0;
    // SAFETY: fd live; ioctl requires a writable int for FS_IOC_GETFLAGS.
    if unsafe { libc::ioctl(file.as_raw_fd(), libc::FS_IOC_GETFLAGS, &mut flags) } == 0 {
        Ok(flags as u32 as u64)
    } else {
        Err(io::Error::last_os_error())
    }
}
pub(super) fn write_flags(file: &File, flags: u64) -> io::Result<()> {
    let flags = u32::try_from(flags).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Linux inode flags exceed 32 bits",
        )
    })? as libc::c_int;
    // SAFETY: fd live; ioctl reads an int from flags; held descriptor prevents path replacement races.
    if unsafe { libc::ioctl(file.as_raw_fd(), libc::FS_IOC_SETFLAGS, &flags) } == 0 {
        Ok(())
    } else {
        let error = io::Error::last_os_error();
        if matches!(error.raw_os_error(), Some(libc::EPERM) | Some(libc::EACCES)) {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "native flag change denied: changing immutable requires CAP_LINUX_IMMUTABLE; ownership, filesystem and mount policies also apply ({error})"
                ),
            ))
        } else {
            Err(error)
        }
    }
}
fn decode_acl(bytes: &[u8], scope: &str) -> io::Result<Vec<String>> {
    if bytes.len() < 4
        || (bytes.len() - 4) % 8 != 0
        || u32::from_le_bytes(bytes[..4].try_into().unwrap()) != 2
    {
        return Err(io::Error::other(
            "unsupported or malformed POSIX ACL xattr encoding",
        ));
    }
    bytes[4..]
        .chunks_exact(8)
        .map(|entry| {
            let tag = u16::from_le_bytes(entry[..2].try_into().unwrap());
            let permissions = u16::from_le_bytes(entry[2..4].try_into().unwrap());
            let id = u32::from_le_bytes(entry[4..8].try_into().unwrap());
            let who = match tag {
                0x01 => "owner".into(),
                0x02 => format!("user {id}"),
                0x04 => "owning group".into(),
                0x08 => format!("group {id}"),
                0x10 => "mask".into(),
                0x20 => "other".into(),
                _ => return Err(io::Error::other("unknown POSIX ACL entry tag")),
            };
            if permissions & !7 != 0 {
                return Err(io::Error::other("invalid POSIX ACL permission bits"));
            }
            let rights: String = [(4, 'r'), (2, 'w'), (1, 'x')]
                .into_iter()
                .map(|(mask, ch)| if permissions & mask != 0 { ch } else { '-' })
                .collect();
            Ok(format!("{scope}: {who}: {rights}"))
        })
        .collect()
}
fn acl(file: &File) -> io::Result<AclInfo> {
    let mut entries = Vec::new();
    for (key, scope) in [
        (b"system.posix_acl_access".as_slice(), "access"),
        (
            b"system.posix_acl_default".as_slice(),
            "default (future children)",
        ),
    ] {
        if let Some(bytes) = raw::read_xattr(file, key)? {
            entries.extend(decode_acl(&bytes, scope)?);
        }
    }
    Ok(AclInfo {
        has_extended_entries: !entries.is_empty(),
        entries,
    })
}
pub(super) fn inspect(file: &File) -> NativeInspection {
    let filesystem = property(filesystem(file));
    let acl = property(acl(file));
    let flags = property(read_flags(file).map(|raw| {
        NativeFlags {
            raw,
            names: [
                (0x10, "immutable"),
                (0x20, "append"),
                (0x40, "nodump"),
                (0x80, "noatime"),
                (0x08, "sync"),
                (0x04, "compressed"),
                (0x00800000, "nocow"),
                (0x10000000, "casefold"),
                (0x800, "encrypted"),
                (0x80000, "verity"),
            ]
            .into_iter()
            .filter(|(mask, _)| raw & mask != 0)
            .map(|(_, name)| name.into())
            .collect(),
        }
    }));
    let xattrs = property(xattrs(file));
    let mut capabilities = vec![
        capability(
            "xattrs",
            &xattrs,
            &filesystem,
            "Binary metadata in filesystem namespaces; foldr writes only user.foldr.* by default.",
        ),
        capability(
            "immutable",
            &flags,
            &filesystem,
            "Directory locking protects entries, not existing child file contents. Setting or clearing immutable on Linux requires CAP_LINUX_IMMUTABLE.",
        ),
        capability(
            "acl",
            &acl,
            &filesystem,
            "Access ACLs constrain this directory; default ACLs seed permissions of new children. Effective permissions also depend on mode, identity and mask.",
        ),
        Capability {
            name: "hidden".into(),
            read: CapabilityState::Unsupported {
                reason: "Linux has no equivalent of Finder's native hidden flag".into(),
            },
            write: CapabilityState::Unsupported {
                reason: "native hiding is unsupported; foldr never silently renames a directory"
                    .into(),
            },
            scope: Scope::Folder,
            description:
                "Leading-dot visibility is a filename convention, not a native directory flag."
                    .into(),
        },
    ];
    capabilities[2].write = CapabilityState::Unsupported {
        reason: "foldr currently inspects ACLs; native ACL editing is outside this release".into(),
    };
    for (name, description, scope) in [
        (
            "compression",
            "Filesystem-specific compression policy; an inode flag alone does not establish effective compression support.",
            Scope::FutureChildren,
        ),
        (
            "casefold",
            "Filesystem-specific casefold policy; support and required directory state cannot be inferred from a filesystem name.",
            Scope::Folder,
        ),
        (
            "encryption",
            "Filesystem encryption policy and key management require a dedicated adapter.",
            Scope::FutureChildren,
        ),
    ] {
        capabilities.push(Capability{name:name.into(),read:CapabilityState::Unknown{reason:"native inode flags may expose hints; dedicated policy inspection is not implemented".into()},write:CapabilityState::Unsupported{reason:"filesystem-specific configuration is outside this release".into()},scope,description:description.into()});
    }
    NativeInspection {
        platform: "linux".into(),
        filesystem,
        acl,
        flags,
        xattrs,
        capabilities,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::changes::{self, ChangeRequest};
    struct RestoreFlags<'a>(&'a File, u64);
    impl Drop for RestoreFlags<'_> {
        fn drop(&mut self) {
            let _ = write_flags(self.0, self.1);
        }
    }
    #[test]
    fn immutable_plan_is_verified_or_reports_privilege_without_metadata_changes() {
        let parent = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(parent.path()).unwrap();
        let path = root.join("folder");
        std::fs::create_dir(&path).unwrap();
        let file = File::open(&path).unwrap();
        let before = match read_flags(&file) {
            Ok(flags) => flags,
            Err(error)
                if matches!(
                    error.raw_os_error(),
                    Some(libc::ENOTTY) | Some(libc::EOPNOTSUPP)
                ) =>
            {
                return;
            }
            Err(error) => panic!("native flag read failed: {error}"),
        };
        let _guard = RestoreFlags(&file, before);
        raw::write_xattr(&file, b"user.example.unrelated", Some(&[0, 255, 1])).unwrap();
        let child = path.join("existing");
        std::fs::write(&child, b"before").unwrap();
        let state = root.join("state");
        let plan = changes::plan(
            &path,
            &ChangeRequest {
                immutable: Some(true),
                ..Default::default()
            },
            false,
        )
        .unwrap();
        assert_eq!(read_flags(&file).unwrap(), before);
        assert!(!state.exists());
        let record = changes::apply(&plan, &state).unwrap();
        if record.succeeded() {
            assert_eq!(read_flags(&file).unwrap(), before | IMMUTABLE_MASK);
            assert!(std::fs::write(path.join("new-entry"), b"blocked").is_err());
            std::fs::write(&child, b"still writable").unwrap();
            assert!(changes::undo(&record, &state, false).unwrap().succeeded());
            assert_eq!(std::fs::read(&child).unwrap(), b"still writable");
        } else {
            assert!(
                record.fields[0]
                    .error
                    .as_ref()
                    .unwrap()
                    .contains("CAP_LINUX_IMMUTABLE")
            );
        }
        assert_eq!(read_flags(&file).unwrap(), before);
        assert_eq!(
            raw::read_xattr(&file, b"user.example.unrelated").unwrap(),
            Some(vec![0, 255, 1])
        );
    }
    #[test]
    fn native_default_acl_is_summarized_without_modifying_it() {
        let folder = tempfile::tempdir().unwrap();
        let file = File::open(folder.path()).unwrap();
        let mut bytes = 2u32.to_le_bytes().to_vec();
        for (tag, rights, id) in [
            (1u16, 7u16, u32::MAX),
            (2, 5, 1234),
            (4, 5, u32::MAX),
            (16, 5, u32::MAX),
            (32, 0, u32::MAX),
        ] {
            bytes.extend_from_slice(&tag.to_le_bytes());
            bytes.extend_from_slice(&rights.to_le_bytes());
            bytes.extend_from_slice(&id.to_le_bytes());
        }
        let key = b"system.posix_acl_default";
        if let Err(error) = raw::write_xattr(&file, key, Some(&bytes)) {
            if error.raw_os_error() == Some(libc::EOPNOTSUPP) {
                assert!(matches!(property(acl(&file)), Property::Unsupported { .. }));
                return;
            }
            panic!("default ACL setup failed: {error}");
        }
        let summary = acl(&file).unwrap();
        assert!(summary.has_extended_entries);
        assert!(
            summary
                .entries
                .iter()
                .any(|entry| entry == "default (future children): user 1234: r-x")
        );
        assert_eq!(raw::read_xattr(&file, key).unwrap(), Some(bytes));
    }
    #[test]
    fn acl_decoding_preserves_numeric_identity_and_scope() {
        let mut bytes = 2u32.to_le_bytes().to_vec();
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&5u16.to_le_bytes());
        bytes.extend_from_slice(&1234u32.to_le_bytes());
        assert_eq!(
            decode_acl(&bytes, "access").unwrap(),
            vec!["access: user 1234: r-x"]
        );
        bytes.push(0);
        assert!(decode_acl(&bytes, "access").is_err());
    }
    #[test]
    fn native_hidden_is_explicitly_unsupported() {
        let folder = tempfile::tempdir().unwrap();
        let file = File::open(folder.path()).unwrap();
        let result = inspect(&file);
        let hidden = result
            .capabilities
            .iter()
            .find(|cap| cap.name == "hidden")
            .unwrap();
        assert!(matches!(hidden.read, CapabilityState::Unsupported { .. }));
        assert!(matches!(hidden.write, CapabilityState::Unsupported { .. }));
    }
}
