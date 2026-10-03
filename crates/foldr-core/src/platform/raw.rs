//! Descriptor-only Unix xattr calls. Keep names and values as uninterpreted bytes.
use std::{ffi::CString, fs::File, io, os::fd::AsRawFd};

const MAX_ATTRIBUTE_BYTES: usize = 16 * 1024 * 1024;

fn bounded_buffer(
    mut call: impl FnMut(*mut libc::c_void, usize) -> libc::ssize_t,
) -> io::Result<Vec<u8>> {
    // Attributes may grow between the length and data queries. Retry ERANGE,
    // but bound both allocations and retries for hostile or changing folders.
    for _ in 0..4 {
        let size = call(std::ptr::null_mut(), 0);
        if size < 0 {
            return Err(io::Error::last_os_error());
        }
        let size = size as usize;
        if size > MAX_ATTRIBUTE_BYTES {
            return Err(io::Error::other(
                "extended attribute exceeds 16 MiB inspection limit",
            ));
        }
        let mut bytes = vec![0u8; size];
        // For zero-size attributes pass a non-null pointer with zero capacity;
        // the APIs return zero without accessing it.
        let read = call(bytes.as_mut_ptr().cast(), size);
        if read >= 0 {
            if read as usize > size {
                continue;
            }
            bytes.truncate(read as usize);
            return Ok(bytes);
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ERANGE) {
            return Err(error);
        }
    }
    Err(io::Error::other(
        "extended attributes changed repeatedly during inspection",
    ))
}

fn c_name(name: &[u8]) -> io::Result<CString> {
    if name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "empty extended attribute name",
        ));
    }
    CString::new(name).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "extended attribute name contains NUL",
        )
    })
}

pub(super) fn missing(error: &io::Error) -> bool {
    #[cfg(target_os = "macos")]
    {
        error.raw_os_error() == Some(libc::ENOATTR)
    }
    #[cfg(target_os = "linux")]
    {
        error.raw_os_error() == Some(libc::ENODATA)
    }
}

pub(super) fn list_xattrs(file: &File) -> io::Result<Vec<Vec<u8>>> {
    let fd = file.as_raw_fd();
    let bytes = bounded_buffer(|pointer, size| {
        // SAFETY: fd is held by File; pointer is null or a writable allocation of size bytes.
        unsafe {
            #[cfg(target_os = "macos")]
            {
                libc::flistxattr(fd, pointer.cast(), size, 0)
            }
            #[cfg(target_os = "linux")]
            {
                libc::flistxattr(fd, pointer.cast(), size)
            }
        }
    })?;
    if !bytes.is_empty() && bytes.last() != Some(&0) {
        return Err(io::Error::other("malformed extended attribute name list"));
    }
    Ok(bytes
        .split(|b| *b == 0)
        .filter(|name| !name.is_empty())
        .map(Vec::from)
        .collect())
}

pub(super) fn read_xattr(file: &File, name: &[u8]) -> io::Result<Option<Vec<u8>>> {
    let name = c_name(name)?;
    let fd = file.as_raw_fd();
    let result = bounded_buffer(|pointer, size| {
        // SAFETY: name is NUL terminated, fd is live, and pointer has the requested capacity.
        unsafe {
            #[cfg(target_os = "macos")]
            {
                libc::fgetxattr(fd, name.as_ptr(), pointer, size, 0, 0)
            }
            #[cfg(target_os = "linux")]
            {
                libc::fgetxattr(fd, name.as_ptr(), pointer, size)
            }
        }
    });
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if missing(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

pub(super) fn write_xattr(file: &File, name: &[u8], value: Option<&[u8]>) -> io::Result<()> {
    let name = c_name(name)?;
    let fd = file.as_raw_fd();
    // SAFETY: fd is held open, name is terminated, and value is a valid slice for the call duration.
    let result = unsafe {
        match value {
            Some(value) => {
                #[cfg(target_os = "macos")]
                {
                    libc::fsetxattr(fd, name.as_ptr(), value.as_ptr().cast(), value.len(), 0, 0)
                }
                #[cfg(target_os = "linux")]
                {
                    libc::fsetxattr(fd, name.as_ptr(), value.as_ptr().cast(), value.len(), 0)
                }
            }
            None => {
                #[cfg(target_os = "macos")]
                {
                    libc::fremovexattr(fd, name.as_ptr(), 0)
                }
                #[cfg(target_os = "linux")]
                {
                    libc::fremovexattr(fd, name.as_ptr())
                }
            }
        }
    };
    if result == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if value.is_none() && missing(&error) {
        Ok(())
    } else {
        Err(error)
    }
}
