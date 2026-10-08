//! Finder tags use a descriptor-held xattr, never Foundation's path-based setter.
//! CoreFoundation only decodes/encodes property lists; all I/O stays descriptor based.
use crate::model::FoldrError;
use serde::{Deserialize, Serialize};
use std::fs::File;

pub const TAG_XATTR: &[u8] = b"com.apple.metadata:_kMDItemUserTags";
pub const FINDER_INFO: &[u8] = b"com.apple.FinderInfo";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinderTag {
    pub name: String,
    /// Native suffix: 0 is explicitly uncolored, None is an unsuffixed native name.
    pub color: Option<u8>,
}
fn unsupported() -> FoldrError {
    FoldrError::Unsupported("Finder tags are a native macOS feature".into())
}
fn invalid(message: &str) -> FoldrError {
    FoldrError::InvalidInput(format!("Finder tags: {message}"))
}
fn validate_name(name: &str) -> Result<(), FoldrError> {
    if name.is_empty() || name.chars().any(char::is_control) || name.len() > 4096 {
        return Err(invalid(
            "names must be nonempty, at most 4096 UTF-8 bytes, and contain no control characters",
        ));
    }
    Ok(())
}
pub fn read_finder_tags_raw(file: &File) -> Result<Option<Vec<u8>>, FoldrError> {
    if !cfg!(target_os = "macos") {
        return Err(unsupported());
    }
    super::read_xattr(file, TAG_XATTR)
}
pub fn decode_finder_tags(raw: Option<&[u8]>) -> Result<Vec<FinderTag>, FoldrError> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = raw;
        Err(unsupported())
    }
    #[cfg(target_os = "macos")]
    {
        let Some(raw) = raw else {
            return Ok(vec![]);
        };
        let names = codec::decode(raw)?;
        let mut tags = Vec::new();
        for native in names {
            let (name, color) = match native.rsplit_once('\n') {
                Some((name, suffix))
                    if suffix.len() == 1 && matches!(suffix.as_bytes()[0], b'0'..=b'7') =>
                {
                    (name.to_string(), Some(suffix.as_bytes()[0] - b'0'))
                }
                Some(_) => {
                    return Err(invalid(
                        "unknown color suffix or malformed native name; refusing to guess",
                    ));
                }
                None => (native, None),
            };
            validate_name(&name)?;
            if tags.iter().any(|tag: &FinderTag| tag.name == name) {
                return Err(invalid("duplicate native names are ambiguous"));
            }
            tags.push(FinderTag { name, color });
        }
        Ok(tags)
    }
}
pub fn encode_finder_tags(tags: &[FinderTag]) -> Result<Vec<u8>, FoldrError> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = tags;
        Err(unsupported())
    }
    #[cfg(target_os = "macos")]
    {
        if tags.len() > 1024 {
            return Err(invalid("too many tags"));
        }
        let mut names = Vec::new();
        for (index, tag) in tags.iter().enumerate() {
            validate_name(&tag.name)?;
            if tags[..index]
                .iter()
                .any(|previous| previous.name == tag.name)
            {
                return Err(invalid("duplicate names"));
            }
            names.push(match tag.color {
                Some(color @ 0..=7) => format!("{}\n{color}", tag.name),
                Some(_) => return Err(invalid("unknown native color")),
                None => tag.name.clone(),
            });
        }
        codec::encode(&names)
    }
}
pub fn validate_finder_tags_raw(raw: Option<&[u8]>) -> Result<(), FoldrError> {
    decode_finder_tags(raw).map(|_| ())
}
/// Legacy FinderInfo labels become visible when tags are absent/empty. We never
/// rewrite foreign FinderInfo; ambiguous legacy-only state is an explicit refusal.
pub fn validate_finder_tags_write(file: &File, raw: Option<&[u8]>) -> Result<(), FoldrError> {
    let tags = decode_finder_tags(raw)?;
    if tags.is_empty() {
        if let Some(info) = super::read_xattr(file, FINDER_INFO)? {
            if info.len() != 32 {
                return Err(invalid("malformed FinderInfo prevents an empty tag edit"));
            }
            if u16::from_be_bytes([info[8], info[9]]) & 0x000e != 0 {
                return Err(invalid(
                    "legacy FinderInfo label would remain visible; use Finder to migrate the label before clearing tags",
                ));
            }
        }
    }
    Ok(())
}
pub fn read_finder_tags(file: &File) -> Result<Vec<FinderTag>, FoldrError> {
    let raw = read_finder_tags_raw(file)?;
    validate_finder_tags_write(file, raw.as_deref())?;
    decode_finder_tags(raw.as_deref())
}
pub fn update_finder_tags(file: &File, names: &[String]) -> Result<Option<Vec<u8>>, FoldrError> {
    let raw = read_finder_tags_raw(file)?;
    validate_finder_tags_write(file, raw.as_deref())?;
    let existing = decode_finder_tags(raw.as_deref())?;
    for (index, name) in names.iter().enumerate() {
        validate_name(name)?;
        if names[..index].contains(name) {
            return Err(invalid("duplicate requested names"));
        }
    }
    // Names represent membership. Preserve native order and suffixes of retained tags.
    let mut desired: Vec<_> = existing
        .iter()
        .filter(|tag| names.contains(&tag.name))
        .cloned()
        .collect();
    for name in names {
        if !desired.iter().any(|tag| tag.name == *name) {
            desired.push(FinderTag {
                name: name.clone(),
                color: Some(0),
            });
        }
    }
    if desired == existing {
        return Ok(raw);
    }
    let updated = if desired.is_empty() {
        None
    } else {
        Some(encode_finder_tags(&desired)?)
    };
    validate_finder_tags_write(file, updated.as_deref())?;
    Ok(updated)
}
pub fn write_finder_tags_raw(file: &File, raw: Option<&[u8]>) -> Result<(), FoldrError> {
    validate_finder_tags_write(file, raw)?;
    super::write_xattr(file, TAG_XATTR, raw)
}

#[cfg(target_os = "macos")]
mod codec {
    use super::{FoldrError, invalid};
    use std::{ffi::c_void, ptr};
    type Cf = *const c_void;
    #[repr(C)]
    struct Range {
        location: isize,
        length: isize,
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFRelease(value: Cf);
        fn CFGetTypeID(value: Cf) -> usize;
        fn CFDataCreate(allocator: Cf, bytes: *const u8, length: isize) -> Cf;
        fn CFDataGetLength(value: Cf) -> isize;
        fn CFDataGetBytePtr(value: Cf) -> *const u8;
        fn CFPropertyListCreateWithData(
            allocator: Cf,
            data: Cf,
            options: usize,
            format: *mut isize,
            error: *mut Cf,
        ) -> Cf;
        fn CFPropertyListCreateData(
            allocator: Cf,
            plist: Cf,
            format: isize,
            options: usize,
            error: *mut Cf,
        ) -> Cf;
        fn CFArrayGetTypeID() -> usize;
        fn CFArrayCreate(allocator: Cf, values: *const Cf, length: isize, callbacks: Cf) -> Cf;
        fn CFArrayGetCount(value: Cf) -> isize;
        fn CFArrayGetValueAtIndex(value: Cf, index: isize) -> Cf;
        fn CFStringGetTypeID() -> usize;
        fn CFStringCreateWithBytes(
            allocator: Cf,
            bytes: *const u8,
            length: isize,
            encoding: u32,
            external: u8,
        ) -> Cf;
        fn CFStringGetLength(value: Cf) -> isize;
        fn CFStringGetCharacters(value: Cf, range: Range, buffer: *mut u16);
    }
    struct Owned(Cf);
    impl Owned {
        fn new(value: Cf) -> Result<Self, FoldrError> {
            if value.is_null() {
                Err(invalid("malformed plist or failed native allocation"))
            } else {
                Ok(Self(value))
            }
        }
    }
    impl Drop for Owned {
        fn drop(&mut self) {
            // SAFETY: Owned holds a non-null object from a CF Create function.
            unsafe { CFRelease(self.0) }
        }
    }
    pub fn decode(raw: &[u8]) -> Result<Vec<String>, FoldrError> {
        if raw.len() > 65536 {
            return Err(invalid("native tag plist exceeds 64 KiB"));
        }
        // SAFETY: input slices stay live; CF Create copies bytes. Null allocator
        // selects the default; null format/error suppress optional outputs.
        let pointer = unsafe { CFDataCreate(ptr::null(), raw.as_ptr(), raw.len() as isize) };
        let data = Owned::new(pointer)?;
        // SAFETY: data is a live CFData, immutable options=0.
        let plist = Owned::new(unsafe {
            CFPropertyListCreateWithData(ptr::null(), data.0, 0, ptr::null_mut(), ptr::null_mut())
        })?;
        // SAFETY: plist is live; CF type checks precede array/string operations.
        if unsafe { CFGetTypeID(plist.0) != CFArrayGetTypeID() } {
            return Err(invalid("native plist must contain an array of strings"));
        }
        // SAFETY: plist was verified as a CFArray.
        let count = unsafe { CFArrayGetCount(plist.0) };
        if !(0..=1024).contains(&count) {
            return Err(invalid("unreasonable native tag count"));
        }
        let mut names = Vec::new();
        for index in 0..count {
            // SAFETY: index is within the live array; returned value is borrowed.
            let value = unsafe { CFArrayGetValueAtIndex(plist.0, index) };
            // SAFETY: CFArray property-list members are non-null live CF objects.
            if unsafe { CFGetTypeID(value) != CFStringGetTypeID() } {
                return Err(invalid("native tag entry is not a string"));
            }
            // SAFETY: value has been verified to be a CFString.
            let length = unsafe { CFStringGetLength(value) };
            if !(0..=4098).contains(&length) {
                return Err(invalid("unreasonable native tag name length"));
            }
            let mut units = vec![0u16; length as usize];
            // SAFETY: range spans the string and units contains length writable u16s.
            unsafe {
                CFStringGetCharacters(
                    value,
                    Range {
                        location: 0,
                        length,
                    },
                    units.as_mut_ptr(),
                )
            };
            names.push(
                String::from_utf16(&units).map_err(|_| invalid("invalid UTF-16 native name"))?,
            );
        }
        Ok(names)
    }
    pub fn encode(names: &[String]) -> Result<Vec<u8>, FoldrError> {
        let mut strings = Vec::new();
        for name in names {
            // SAFETY: name is live valid UTF-8; native Create copies bytes.
            strings.push(Owned::new(unsafe {
                CFStringCreateWithBytes(
                    ptr::null(),
                    name.as_ptr(),
                    name.len() as isize,
                    0x08000100,
                    0,
                )
            })?);
        }
        let pointers: Vec<_> = strings.iter().map(|value| value.0).collect();
        // SAFETY: null callbacks means no retains/releases; strings outlive array.
        let array = Owned::new(unsafe {
            CFArrayCreate(
                ptr::null(),
                pointers.as_ptr(),
                pointers.len() as isize,
                ptr::null(),
            )
        })?;
        // SAFETY: array contains live CFStrings; format 200 = binary plist v1.
        let data = Owned::new(unsafe {
            CFPropertyListCreateData(ptr::null(), array.0, 200, 0, ptr::null_mut())
        })?;
        // SAFETY: data is a live CFData; returned bytes remain valid until copied.
        let bytes = unsafe {
            std::slice::from_raw_parts(CFDataGetBytePtr(data.0), CFDataGetLength(data.0) as usize)
        }
        .to_vec();
        if bytes.len() > 65536 {
            return Err(invalid("native tag plist exceeds 64 KiB"));
        }
        Ok(bytes)
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use crate::inspect::open_directory;
    use std::{
        os::unix::fs::{MetadataExt, symlink},
        process::Command,
    };
    fn sample() -> Vec<FinderTag> {
        vec![
            FinderTag {
                name: "研究 🟣".into(),
                color: Some(6),
            },
            FinderTag {
                name: "Blue".into(),
                color: Some(4),
            },
            FinderTag {
                name: "unsuffixed".into(),
                color: None,
            },
        ]
    }
    #[test]
    fn native_codec_refuses_malformed_and_preserves_unicode_colors() {
        let tags = sample();
        let bytes = encode_finder_tags(&tags).unwrap();
        assert!(bytes.starts_with(b"bplist00"));
        assert_eq!(decode_finder_tags(Some(&bytes)).unwrap(), tags);
        assert_eq!(decode_finder_tags(None).unwrap(), vec![]);
        assert!(decode_finder_tags(Some(b"")).is_err());
        assert!(decode_finder_tags(Some(b"garbage")).is_err());
        for bad in [
            "<plist><array><integer>1</integer></array></plist>",
            "<plist><array><string>name\n9</string></array></plist>",
            "<plist><dict/></plist>",
        ] {
            assert!(decode_finder_tags(Some(bad.as_bytes())).is_err());
        }
    }
    #[test]
    fn descriptor_fixture_preserves_foreign_metadata_identity_and_exact_recovery() {
        let parent = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(parent.path()).unwrap();
        let path = root.join("folder");
        std::fs::create_dir(&path).unwrap();
        let link = root.join("link");
        symlink(&path, &link).unwrap();
        assert!(open_directory(&link, false).is_err());
        let file = open_directory(&link, true).unwrap();
        let mut info: Vec<u8> = (0..32).collect();
        info[9] &= !0x0e;
        super::super::write_xattr(&file, FINDER_INFO, Some(&info)).unwrap();
        super::super::write_xattr(&file, b"com.example.foreign", Some(&[0, 255, 7])).unwrap();
        let original = encode_finder_tags(&sample()).unwrap();
        write_finder_tags_raw(&file, Some(&original)).unwrap();
        let before = file.metadata().unwrap();
        let names = vec!["研究 🟣".into(), "Blue".into(), "added".into()];
        let updated = update_finder_tags(&file, &names).unwrap().unwrap();
        let after_preview = file.metadata().unwrap();
        assert_eq!(
            (
                before.ctime(),
                before.ctime_nsec(),
                before.mtime(),
                before.mtime_nsec()
            ),
            (
                after_preview.ctime(),
                after_preview.ctime_nsec(),
                after_preview.mtime(),
                after_preview.mtime_nsec()
            )
        );
        std::fs::rename(&path, root.join("held")).unwrap();
        std::fs::create_dir(&path).unwrap();
        write_finder_tags_raw(&file, Some(&updated)).unwrap();
        assert_eq!(
            read_finder_tags_raw(&File::open(&path).unwrap()).unwrap(),
            None
        );
        assert_eq!(
            read_finder_tags(&file).unwrap(),
            vec![
                sample()[0].clone(),
                sample()[1].clone(),
                FinderTag {
                    name: "added".into(),
                    color: Some(0)
                }
            ]
        );
        // Foundation is an independent native visibility oracle, never a writer.
        let script = root.join("oracle.swift");
        std::fs::write(&script,"import Foundation\nlet u=URL(fileURLWithPath:CommandLine.arguments[1]); let tags=try u.resourceValues(forKeys:[.tagNamesKey]).tagNames ?? []; for tag in tags { print(tag) }\n").unwrap();
        let oracle = Command::new("/usr/bin/swift")
            .arg(&script)
            .arg(root.join("held"))
            .output()
            .unwrap();
        assert!(
            oracle.status.success(),
            "{}",
            String::from_utf8_lossy(&oracle.stderr)
        );
        assert_eq!(
            String::from_utf8(oracle.stdout).unwrap(),
            "研究 🟣\nBlue\nadded\n"
        );
        write_finder_tags_raw(&file, Some(&original)).unwrap();
        assert_eq!(read_finder_tags_raw(&file).unwrap(), Some(original));
        assert_eq!(
            super::super::read_xattr(&file, FINDER_INFO).unwrap(),
            Some(info)
        );
        assert_eq!(
            super::super::read_xattr(&file, b"com.example.foreign").unwrap(),
            Some(vec![0, 255, 7])
        );
        write_finder_tags_raw(&file, None).unwrap();
        assert_eq!(read_finder_tags_raw(&file).unwrap(), None);
    }
    #[test]
    fn legacy_label_fallback_is_refused_without_writes() {
        let folder = tempfile::tempdir().unwrap();
        let file = File::open(folder.path()).unwrap();
        let mut info = vec![0; 32];
        info[9] = 8;
        super::super::write_xattr(&file, FINDER_INFO, Some(&info)).unwrap();
        assert!(read_finder_tags(&file).is_err());
        assert!(update_finder_tags(&file, &["added".into()]).is_err());
        let bytes = encode_finder_tags(&sample()).unwrap();
        write_finder_tags_raw(&file, Some(&bytes)).unwrap();
        assert_eq!(read_finder_tags(&file).unwrap(), sample());
        assert!(update_finder_tags(&file, &[]).is_err());
        assert_eq!(read_finder_tags_raw(&file).unwrap(), Some(bytes));
        assert_eq!(
            super::super::read_xattr(&file, FINDER_INFO).unwrap(),
            Some(info)
        );
    }
}
