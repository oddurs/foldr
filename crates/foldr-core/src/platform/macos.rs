//! Apple descriptor APIs. Extended ACL entries are summarized, never edited.
use super::{capability, property, xattrs};
use crate::model::*;
use std::{fs::File, io, mem::MaybeUninit, os::fd::AsRawFd, os::macos::fs::MetadataExt};
pub const IMMUTABLE_MASK: u64 = libc::UF_IMMUTABLE as u64;
pub const HIDDEN_MASK: Option<u64> = Some(libc::UF_HIDDEN as u64);

type Acl = *mut libc::c_void;
// These signatures and constants are from Apple's SDK sys/acl.h.
unsafe extern "C" {
    fn acl_get_fd_np(fd: libc::c_int, acl_type: libc::c_int) -> Acl;
    fn acl_get_entry(acl: Acl, entry_id: libc::c_int, entry: *mut Acl) -> libc::c_int;
    fn acl_get_tag_type(entry: Acl, tag: *mut libc::c_int) -> libc::c_int;
    fn acl_get_flagset_np(entry: Acl, flags: *mut Acl) -> libc::c_int;
    fn acl_get_flag_np(flags: Acl, flag: libc::c_int) -> libc::c_int;
    fn acl_free(acl: Acl) -> libc::c_int;
}
struct OwnedAcl(Acl);
impl Drop for OwnedAcl {
    fn drop(&mut self) {
        // SAFETY: OwnedAcl owns a successful allocation returned by acl_get_fd_np.
        unsafe {
            acl_free(self.0);
        }
    }
}
fn acl(file: &File) -> io::Result<AclInfo> {
    // SAFETY: descriptor is live; 0x100 is ACL_TYPE_EXTENDED from sys/acl.h.
    let pointer = unsafe { acl_get_fd_np(file.as_raw_fd(), 0x100) };
    if pointer.is_null() {
        let error = io::Error::last_os_error();
        // Darwin reports ENOENT when a valid descriptor has no extended ACL.
        if error.raw_os_error() == Some(libc::ENOENT) {
            return Ok(AclInfo {
                entries: vec![],
                has_extended_entries: false,
            });
        }
        return Err(error);
    }
    let acl = OwnedAcl(pointer);
    let mut entries = Vec::new();
    let mut entry = std::ptr::null_mut();
    let mut which = 0; // ACL_FIRST_ENTRY; subsequent ACL_NEXT_ENTRY = -1.
    loop {
        // SAFETY: acl is live and entry is writable storage.
        let result = unsafe { acl_get_entry(acl.0, which, &mut entry) };
        // Apple returns zero for success; EINVAL denotes exhausted entries.
        if result != 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINVAL) {
                break;
            }
            return Err(error);
        }
        which = -1;
        let mut tag = 0;
        let mut flags = std::ptr::null_mut();
        // SAFETY: entry belongs to live acl and output storage is valid.
        if unsafe { acl_get_tag_type(entry, &mut tag) } != 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: entry is valid, flags is writable storage.
        if unsafe { acl_get_flagset_np(entry, &mut flags) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let mut labels = vec![match tag {
            1 => "allow",
            2 => "deny",
            _ => "unknown disposition",
        }];
        for (flag, label) in [
            (1 << 4, "inherited"),
            (1 << 5, "file inherit"),
            (1 << 6, "directory inherit"),
            (1 << 7, "limit inherit"),
            (1 << 8, "inherit only"),
        ] {
            // SAFETY: flags belongs to live entry; flag values are from Apple SDK.
            if unsafe { acl_get_flag_np(flags, flag) } > 0 {
                labels.push(label);
            }
        }
        entries.push(format!(
            "entry {}: {}",
            entries.len() + 1,
            labels.join(", ")
        ));
        if entries.len() > 4096 {
            return Err(io::Error::other("unreasonable ACL entry count"));
        }
    }
    Ok(AclInfo {
        has_extended_entries: !entries.is_empty(),
        entries,
    })
}
fn filesystem(file: &File) -> io::Result<FilesystemInfo> {
    let mut value = MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: fd is live and value points to statfs-sized writable storage.
    if unsafe { libc::fstatfs(file.as_raw_fd(), value.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful fstatfs initializes the entire struct.
    let value = unsafe { value.assume_init() };
    let name: Vec<u8> = value
        .f_fstypename
        .iter()
        .take_while(|v| **v != 0)
        .map(|v| *v as u8)
        .collect();
    Ok(FilesystemInfo {
        name: String::from_utf8_lossy(&name).into_owned(),
        type_id: Some(value.f_type as u64),
        read_only: value.f_flags & libc::MNT_RDONLY as u32 != 0,
    })
}
pub(super) fn read_flags(file: &File) -> io::Result<u64> {
    Ok(file.metadata()?.st_flags() as u64)
}
pub(super) fn write_flags(file: &File, flags: u64) -> io::Result<()> {
    let flags = u32::try_from(flags)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "macOS flags exceed 32 bits"))?;
    // SAFETY: live descriptor, native flag integer; only this held directory is changed.
    if unsafe { libc::fchflags(file.as_raw_fd(), flags) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
pub(super) fn inspect(file: &File) -> NativeInspection {
    let filesystem = property(filesystem(file));
    let acl = property(acl(file));
    let flags = property(read_flags(file).map(|raw| {
        NativeFlags {
            raw,
            names: [
                (libc::UF_HIDDEN as u64, "hidden"),
                (libc::UF_IMMUTABLE as u64, "immutable"),
                (libc::UF_APPEND as u64, "append"),
                (libc::UF_NODUMP as u64, "nodump"),
                (libc::SF_IMMUTABLE as u64, "system immutable"),
                (libc::SF_APPEND as u64, "system append"),
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
            "Binary folder metadata; copies, archives, sync tools and Git may not preserve it.",
        ),
        capability(
            "hidden",
            &flags,
            &filesystem,
            "Finder hiding changes visibility without renaming the folder or blocking known-path access.",
        ),
        capability(
            "immutable",
            &flags,
            &filesystem,
            "Directory locking protects entries; it does not freeze the contents of existing child files.",
        ),
        capability(
            "acl",
            &acl,
            &filesystem,
            "Native extended ACL entries can grant or deny access and specify inheritance; this summary does not resolve effective permissions.",
        ),
    ];
    capabilities[3].write = CapabilityState::Unsupported {
        reason: "foldr currently inspects ACLs; native ACL editing is outside this release".into(),
    };
    for (name, key, description) in [
        (
            "finder_tags",
            b"com.apple.metadata:_kMDItemUserTags".as_slice(),
            "Finder tag metadata is retained as its original binary plist in xattrs.",
        ),
        (
            "custom_icon",
            b"com.apple.FinderInfo".as_slice(),
            "FinderInfo carries the custom icon marker; the icon artwork may live separately in an Icon resource file.",
        ),
    ] {
        let presence = property(raw_presence(file, key, name == "custom_icon"));
        capabilities.push(Capability {
            name: name.into(),
            read: super::state(&presence),
            write: CapabilityState::Unsupported {
                reason: "appearance editing is outside this CLI release".into(),
            },
            scope: Scope::Folder,
            description: format!(
                "{description} {}",
                match presence.value() {
                    Some(true) => "Metadata present.",
                    Some(false) => "Metadata absent.",
                    None => "Metadata could not be inspected.",
                }
            ),
        });
    }
    NativeInspection {
        platform: "macos".into(),
        filesystem,
        acl,
        flags,
        xattrs,
        capabilities,
    }
}

fn raw_presence(file: &File, key: &[u8], icon: bool) -> io::Result<bool> {
    Ok(super::raw::read_xattr(file, key)?.is_some_and(|bytes| {
        if icon {
            bytes
                .get(8..10)
                .is_some_and(|flags| u16::from_be_bytes([flags[0], flags[1]]) & 0x0400 != 0)
        } else {
            true
        }
    }))
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
    fn immutable_note_plan_and_undo_preserve_existing_child_contents() {
        let parent = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(parent.path()).unwrap();
        let path = root.join("folder");
        std::fs::create_dir(&path).unwrap();
        let file = File::open(&path).unwrap();
        let before = read_flags(&file).unwrap();
        let _guard = RestoreFlags(&file, before);
        let child = path.join("existing");
        std::fs::write(&child, b"before").unwrap();
        let state = root.join("state");
        let plan = changes::plan(
            &path,
            &ChangeRequest {
                note: Some(Some("locked".into())),
                immutable: Some(true),
                ..Default::default()
            },
            false,
        )
        .unwrap();
        assert!(!state.exists());
        assert_eq!(read_flags(&file).unwrap(), before);
        assert_eq!(changes::read_note(&path).unwrap(), None);
        assert!(matches!(
            plan.changes.last().unwrap().field,
            changes::Field::Flags
        ));
        let record = changes::apply(&plan, &state).unwrap();
        assert!(record.succeeded());
        assert_eq!(changes::read_note(&path).unwrap(), Some(b"locked".to_vec()));
        assert!(std::fs::write(path.join("new-entry"), b"blocked").is_err());
        std::fs::write(&child, b"still writable").unwrap();
        let undo_preview = changes::undo_plan(&record).unwrap();
        assert!(matches!(
            undo_preview.changes.first().unwrap().field,
            changes::Field::Flags
        ));
        assert!(changes::undo(&record, &state, false).unwrap().succeeded());
        assert_eq!(read_flags(&file).unwrap(), before);
        assert_eq!(changes::read_note(&path).unwrap(), None);
        assert_eq!(std::fs::read(&child).unwrap(), b"still writable");
    }
    #[test]
    fn unlock_then_note_edit_can_be_undone_in_reverse_order() {
        let parent = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(parent.path()).unwrap();
        let path = root.join("folder");
        std::fs::create_dir(&path).unwrap();
        let file = File::open(&path).unwrap();
        let before = read_flags(&file).unwrap();
        let _guard = RestoreFlags(&file, before);
        super::super::write_xattr(&file, b"com.foldr.note", Some(b"original")).unwrap();
        write_flags(&file, before | IMMUTABLE_MASK).unwrap();
        let state = root.join("state");
        let plan = changes::plan(
            &path,
            &ChangeRequest {
                note: Some(Some("edited".into())),
                immutable: Some(false),
                ..Default::default()
            },
            false,
        )
        .unwrap();
        assert!(matches!(
            plan.changes.first().unwrap().field,
            changes::Field::Flags
        ));
        let record = changes::apply(&plan, &state).unwrap();
        assert!(record.succeeded());
        assert_eq!(changes::read_note(&path).unwrap(), Some(b"edited".to_vec()));
        assert!(changes::undo(&record, &state, false).unwrap().succeeded());
        assert_eq!(read_flags(&file).unwrap(), before | IMMUTABLE_MASK);
        assert_eq!(
            changes::read_note(&path).unwrap(),
            Some(b"original".to_vec())
        );
    }
    #[test]
    fn hidden_plan_apply_undo_preserves_unrelated_metadata() {
        let parent = tempfile::tempdir().unwrap();
        let path = std::fs::canonicalize(parent.path()).unwrap();
        let file = File::open(&path).unwrap();
        let before = read_flags(&file).unwrap();
        let _guard = RestoreFlags(&file, before);
        super::super::write_xattr(&file, b"com.example.unrelated", Some(&[0, 255, 1])).unwrap();
        let state = tempfile::tempdir().unwrap();
        let state_path = std::fs::canonicalize(state.path()).unwrap().join("journal");
        let plan = changes::plan(
            &path,
            &ChangeRequest {
                hidden: Some(true),
                ..Default::default()
            },
            false,
        )
        .unwrap();
        assert_eq!(read_flags(&file).unwrap(), before);
        let record = changes::apply(&plan, &state_path).unwrap();
        assert!(record.succeeded());
        assert!(
            changes::undo(&record, &state_path, false)
                .unwrap()
                .succeeded()
        );
        assert_eq!(read_flags(&file).unwrap(), before);
        assert_eq!(
            super::super::read_xattr(&file, b"com.example.unrelated").unwrap(),
            Some(vec![0, 255, 1])
        );
    }
    #[test]
    fn hidden_flag_round_trip_preserves_other_bits() {
        let folder = tempfile::tempdir().unwrap();
        let file = File::open(folder.path()).unwrap();
        let before = read_flags(&file).unwrap();
        write_flags(&file, before | libc::UF_HIDDEN as u64).unwrap();
        assert_eq!(read_flags(&file).unwrap(), before | libc::UF_HIDDEN as u64);
        write_flags(&file, before).unwrap();
        assert_eq!(read_flags(&file).unwrap(), before);
    }
    #[test]
    fn empty_acl_is_reported_as_supported() {
        let folder = tempfile::tempdir().unwrap();
        let file = File::open(folder.path()).unwrap();
        let result = acl(&file).unwrap();
        assert!(!result.has_extended_entries);
    }
}
