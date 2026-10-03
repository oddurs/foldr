//! Lossless, platform-neutral folder inspection and mutation types.
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fmt, io,
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncodedPath {
    pub display: String,
    pub bytes: Vec<u8>,
}
impl EncodedPath {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self::from_bytes(path.as_ref().as_os_str().as_bytes().to_vec())
    }
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            display: escape_bytes(&bytes),
            bytes,
        }
    }
    pub fn to_path_buf(&self) -> PathBuf {
        std::ffi::OsString::from_vec(self.bytes.clone()).into()
    }
}
pub fn escape_bytes(bytes: &[u8]) -> String {
    let mut result = String::new();
    let mut remaining = bytes;
    while !remaining.is_empty() {
        match std::str::from_utf8(remaining) {
            Ok(text) => {
                for ch in text.chars() {
                    result.extend(ch.escape_debug());
                }
                break;
            }
            Err(error) => {
                let valid = &remaining[..error.valid_up_to()];
                for ch in std::str::from_utf8(valid)
                    .expect("valid UTF-8 prefix")
                    .chars()
                {
                    result.extend(ch.escape_debug());
                }
                let size = error.error_len().unwrap_or(remaining.len() - valid.len());
                for byte in &remaining[valid.len()..valid.len() + size] {
                    result.push_str(&format!("\\x{byte:02x}"));
                }
                remaining = &remaining[valid.len() + size..];
            }
        }
    }
    result
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Property<T> {
    Supported { value: T },
    Unsupported { reason: String },
    Unknown { reason: String },
    PermissionDenied { reason: String },
    Unavailable { reason: String },
}
impl<T> Property<T> {
    pub fn value(&self) -> Option<&T> {
        if let Self::Supported { value } = self {
            Some(value)
        } else {
            None
        }
    }
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Supported { .. } => None,
            Self::Unsupported { reason }
            | Self::Unknown { reason }
            | Self::PermissionDenied { reason }
            | Self::Unavailable { reason } => Some(reason),
        }
    }
    pub fn from_io(error: io::Error) -> Self {
        let reason = error.to_string();
        match error.kind() {
            io::ErrorKind::PermissionDenied => Self::PermissionDenied { reason },
            io::ErrorKind::Unsupported => Self::Unsupported { reason },
            _ => Self::Unavailable { reason },
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum CapabilityState {
    Supported,
    Unsupported { reason: String },
    Unknown { reason: String },
    PermissionDenied { reason: String },
    Unavailable { reason: String },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Folder,
    FutureChildren,
    ExistingDescendants,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub read: CapabilityState,
    pub write: CapabilityState,
    pub scope: Scope,
    pub description: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderIdentity {
    pub device: u64,
    pub inode: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilesystemInfo {
    pub name: String,
    pub type_id: Option<u64>,
    pub read_only: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AclInfo {
    pub entries: Vec<String>,
    pub has_extended_entries: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeFlags {
    pub raw: u64,
    pub names: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtendedAttribute {
    pub name: EncodedPath,
    pub value: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeInspection {
    pub platform: String,
    pub filesystem: Property<FilesystemInfo>,
    pub acl: Property<AclInfo>,
    pub flags: Property<NativeFlags>,
    pub xattrs: Property<Vec<ExtendedAttribute>>,
    pub capabilities: Vec<Capability>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderSnapshot {
    pub schema_version: u32,
    pub path: EncodedPath,
    pub symlink_target: Option<EncodedPath>,
    pub identity: FolderIdentity,
    pub owner: u32,
    pub group: u32,
    pub mode: u32,
    pub platform: String,
    pub filesystem: Property<FilesystemInfo>,
    pub acl: Property<AclInfo>,
    pub flags: Property<NativeFlags>,
    pub xattrs: Property<Vec<ExtendedAttribute>>,
    pub capabilities: Vec<Capability>,
}
#[derive(Debug)]
pub enum FoldrError {
    Io {
        operation: String,
        source: io::Error,
    },
    InvalidInput(String),
    Unsupported(String),
    Conflict(String),
    Journal(String),
}
impl FoldrError {
    pub fn io(operation: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            operation: operation.into(),
            source,
        }
    }
}
impl fmt::Display for FoldrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
            Self::InvalidInput(s) | Self::Unsupported(s) | Self::Conflict(s) | Self::Journal(s) => {
                f.write_str(s)
            }
        }
    }
}
impl std::error::Error for FoldrError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Self::Io { source, .. } = self {
            Some(source)
        } else {
            None
        }
    }
}
impl From<io::Error> for FoldrError {
    fn from(source: io::Error) -> Self {
        Self::io("filesystem operation", source)
    }
}
pub fn os_bytes(value: &OsStr) -> &[u8] {
    value.as_bytes()
}
