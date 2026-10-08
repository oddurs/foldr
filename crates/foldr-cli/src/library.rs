//! Explicit preset configuration, separate from recovery state and target folders.
use crate::output::CliError;
use foldr_core::{EncodedPath, Preset};
use serde::Serialize;
use std::{
    ffi::{CStr, CString, OsString},
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{
            ffi::{OsStrExt, OsStringExt},
            fs::OpenOptionsExt,
        },
    },
    path::{Component, Path, PathBuf},
};

fn validate_name(name: &str) -> Result<(), CliError> {
    if name.is_empty()
        || name.len() > 64
        || !name.as_bytes()[0].is_ascii_alphanumeric()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(CliError::usage(
            "Preset names must start with an ASCII letter or digit, contain only letters, digits, '-' or '_', and be at most 64 bytes.",
        ));
    }
    Ok(())
}

pub fn directory(explicit: Option<&Path>) -> Result<PathBuf, CliError> {
    if let Some(path) = explicit {
        return Ok(path.to_owned());
    }
    if cfg!(target_os = "linux") {
        if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
            let path = PathBuf::from(path);
            if path.is_absolute() {
                return Ok(path.join("foldr/presets"));
            }
        }
    }
    let home = std::env::var_os("HOME")
        .ok_or_else(|| CliError::usage("Set --preset-dir when HOME is unavailable."))?;
    if home.is_empty() {
        return Err(CliError::usage("Set --preset-dir when HOME is empty."));
    }
    Ok(PathBuf::from(home).join(if cfg!(target_os = "macos") {
        "Library/Application Support/foldr/presets"
    } else {
        ".config/foldr/presets"
    }))
}

fn cstring(bytes: &[u8]) -> Result<CString, CliError> {
    CString::new(bytes).map_err(|_| CliError::usage("Preset paths must not contain NUL bytes."))
}

fn search_flags() -> i32 {
    #[cfg(target_os = "macos")]
    {
        libc::O_SEARCH | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC
    }
    #[cfg(target_os = "linux")]
    {
        libc::O_PATH | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC
    }
}

fn openat(parent: i32, name: &CStr, flags: i32, mode: libc::c_uint) -> io::Result<File> {
    // SAFETY: name is NUL terminated; parent is held open by the caller; flags/mode are valid.
    let fd = unsafe { libc::openat(parent, name.as_ptr(), flags, mode) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful openat returns a new descriptor transferred exactly once to File.
    Ok(unsafe { File::from_raw_fd(fd) })
}

fn path_error(path: &Path, error: io::Error) -> CliError {
    CliError {
        code: if matches!(error.raw_os_error(), Some(libc::ELOOP | libc::ENOTDIR)) {
            2
        } else {
            1
        },
        message: format!(
            "Cannot access preset library {}: {error}. Library path components must be directories and must not be symbolic links.",
            EncodedPath::new(path).display
        ),
    }
}

/// Pin every path component. No canonicalization or pathname re-open can follow a substitution.
fn open_directory(path: &Path, create: bool) -> Result<Option<File>, CliError> {
    if path.as_os_str().is_empty() {
        return Err(CliError::usage("Preset directory must not be empty."));
    }
    cstring(path.as_os_str().as_bytes())?;
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    if absolute
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return Err(CliError::usage(
            "Preset directory must not contain '..' components.",
        ));
    }
    let mut parent =
        openat(libc::AT_FDCWD, c"/", search_flags(), 0).map_err(|e| path_error(path, e))?;
    for component in absolute.components() {
        let Component::Normal(component) = component else {
            continue;
        };
        let name = cstring(component.as_bytes())?;
        let opened = openat(parent.as_raw_fd(), &name, search_flags(), 0);
        parent = match opened {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if !create {
                    return Ok(None);
                }
                // SAFETY: parent remains open and name is NUL terminated; mode is private-directory mode.
                let result = unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) };
                if result < 0 {
                    let error = io::Error::last_os_error();
                    if error.kind() != io::ErrorKind::AlreadyExists {
                        return Err(path_error(path, error));
                    }
                }
                openat(parent.as_raw_fd(), &name, search_flags(), 0)
                    .map_err(|e| path_error(path, e))?
            }
            Err(error) => return Err(path_error(path, error)),
        };
    }
    Ok(Some(parent))
}

fn read_file(mut file: File, label: &Path) -> Result<Preset, CliError> {
    if !file.metadata()?.is_file() {
        return Err(CliError::usage(format!(
            "Preset {} must be a regular file.",
            EncodedPath::new(label).display
        )));
    }
    let mut text = String::new();
    file.read_to_string(&mut text).map_err(|e| CliError {
        code: 2,
        message: format!(
            "Cannot read preset {}: {e}",
            EncodedPath::new(label).display
        ),
    })?;
    Preset::from_toml(&text).map_err(|error| {
        let mut error = CliError::from(error);
        error.message = format!(
            "Preset {}: {}",
            EncodedPath::new(label).display,
            error.message
        );
        error
    })
}

fn read_named(parent: &File, path: &Path, name: &str) -> Result<Preset, CliError> {
    let filename = format!("{name}.toml");
    let file = openat(parent.as_raw_fd(), &cstring(filename.as_bytes())?, libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK, 0)
        .map_err(|error| CliError { code: if error.raw_os_error() == Some(libc::ELOOP) { 2 } else { 1 }, message: format!("Cannot read installed preset '{name}' in {}: {error}; install a regular TOML preset explicitly.", EncodedPath::new(path).display) })?;
    read_file(file, &path.join(filename))
}

pub fn resolve(explicit: Option<&Path>, name: &str) -> Result<Preset, CliError> {
    validate_name(name)?;
    let path = directory(explicit)?;
    let parent = open_directory(&path, false)?.ok_or_else(|| {
        CliError::usage(format!(
            "Preset '{name}' is not installed: library {} does not exist.",
            EncodedPath::new(&path).display
        ))
    })?;
    read_named(&parent, &path, name)
}

pub fn install(explicit: Option<&Path>, source: &Path, name: &str) -> Result<PathBuf, CliError> {
    validate_name(name)?;
    // Validate input before creating a library. A source symlink never becomes an installed preset.
    let input = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(source)
        .map_err(|error| CliError {
            code: if error.raw_os_error() == Some(libc::ELOOP) {
                2
            } else {
                1
            },
            message: format!(
                "Cannot install source {}: {error}; use a regular TOML file.",
                EncodedPath::new(source).display
            ),
        })?;
    let text = read_file(input, source)?.to_toml()?;
    let path = directory(explicit)?;
    let parent = open_directory(&path, true)?.expect("create returns a directory");
    let filename = cstring(format!("{name}.toml").as_bytes())?;
    let mut file = openat(parent.as_raw_fd(), &filename, libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC, 0o600)
        .map_err(|error| CliError { code: if error.kind() == io::ErrorKind::AlreadyExists { 4 } else { 1 }, message: format!("Cannot install preset '{name}': {error}. Existing names are never replaced; choose a different name.") })?;
    if let Err(error) = file
        .write_all(text.as_bytes())
        .and_then(|()| file.sync_all())
    {
        return Err(error.into());
    }
    Ok(path.join(format!("{name}.toml")))
}

#[derive(Serialize)]
pub struct InstalledPreset {
    name: String,
    file: EncodedPath,
    preset: Preset,
}

struct DirectoryStream(*mut libc::DIR);
impl Drop for DirectoryStream {
    fn drop(&mut self) {
        // SAFETY: this object exclusively owns a successful fdopendir result.
        unsafe {
            libc::closedir(self.0);
        }
    }
}

pub fn list(explicit: Option<&Path>) -> Result<Vec<InstalledPreset>, CliError> {
    let path = directory(explicit)?;
    let Some(parent) = open_directory(&path, false)? else {
        return Ok(vec![]);
    };
    let readable = openat(
        parent.as_raw_fd(),
        c".",
        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
        0,
    )?;
    use std::os::fd::IntoRawFd;
    let fd = readable.into_raw_fd();
    // SAFETY: fd is an owned readable directory descriptor; fdopendir takes ownership on success.
    let pointer = unsafe { libc::fdopendir(fd) };
    if pointer.is_null() {
        let error = io::Error::last_os_error();
        // SAFETY: fdopendir failed, so fd is still owned here.
        unsafe {
            libc::close(fd);
        }
        return Err(error.into());
    }
    let stream = DirectoryStream(pointer);
    let mut names = Vec::new();
    loop {
        // SAFETY: errno access returns a valid thread-local pointer; stream is a live DIR.
        let entry = unsafe {
            #[cfg(target_os = "macos")]
            {
                *libc::__error() = 0;
            }
            #[cfg(target_os = "linux")]
            {
                *libc::__errno_location() = 0;
            }
            libc::readdir(stream.0)
        };
        if entry.is_null() {
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(0) {
                return Err(error.into());
            }
            break;
        }
        // SAFETY: readdir returned a live dirent with NUL-terminated d_name until next iteration.
        let bytes = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) }.to_bytes();
        if bytes.ends_with(b".toml") {
            names.push(OsString::from_vec(bytes.to_vec()));
        }
    }
    names.sort();
    names
        .into_iter()
        .map(|filename| {
            let name = filename
                .to_str()
                .and_then(|s| s.strip_suffix(".toml"))
                .ok_or_else(|| {
                    CliError::usage(
                        "Installed preset filenames must be UTF-8 names ending in .toml.",
                    )
                })?;
            validate_name(name)?;
            let preset = read_named(&parent, &path, name)?;
            Ok(InstalledPreset {
                name: name.into(),
                file: EncodedPath::new(path.join(&filename)),
                preset,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_reject_traversal_controls_and_nul() {
        for name in [
            "", ".", "..", "../a", "a/b", "a\\b", "a\0b", "a\nb", "a.toml",
        ] {
            assert!(validate_name(name).is_err(), "{name:?}");
        }
        assert!(validate_name("Project_2-archive").is_ok());
    }
}
