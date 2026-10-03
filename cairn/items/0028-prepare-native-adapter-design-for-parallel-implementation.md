---
id: 28
uid: 45a5a545-7d09-441e-86c6-186d81be19e5
title: Prepare native adapter design for parallel implementation
type: docs
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 6
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
area: platform
part_of:
- 8
- 9
---

## Context

Native adapter preparation can happen while the core model is finalized. The implementation tasks 0008 and 0009 still depend on completed item 0007.

## Proposal

Research the installed Unix APIs and official documentation; define read-only inspection, ACL summary, filesystem detection, native flag operations, binary xattr handling, descriptor/symlink behavior and platform limitations. Coordinate an exact API with the core architect.

## Acceptance criteria

- [x] Native API choices and limitations are recorded.
- [x] The adapter interface is agreed with the core architect.
- [x] A verified implementation path exists for macOS and Linux without parsing shell commands.

## 2026-10-02

Agreed final descriptor bridge with core architect: core holds File and owns identity/mode/symlink checks. Platform exposes infallible inspect_native(File) with per-property statuses plus field-level read_xattr/write_xattr/read_flags/write_flags Result APIs. Raw attribute names and values remain bytes. macOS native fstatfs, fstat st_flags/fchflags, descriptor xattr syscalls, acl_get_fd_np/acl_get_entry; Linux fstatfs, descriptor xattrs, FS_IOC_GETFLAGS/SETFLAGS ioctl, POSIX ACL kernel xattr version-2 decoding. No shell parsing, no inspection writes; successful reads do not prove write authorization. ENOTSUP/ENOTTY -> unsupported, EACCES/EPERM -> permission denied, absent xattrs -> None. Linux hidden unsupported; immutable requires CAP_LINUX_IMMUTABLE. Directory lock protects entries, not existing child contents. Checked installed Apple SDK sys/acl.h, sys/xattr.h, sys/stat.h, sys/mount.h and official Linux UAPI posix_acl_xattr.h/fs.h; references https://github.com/torvalds/linux/blob/master/include/uapi/linux/posix_acl_xattr.h and https://man7.org/linux/man-pages/man2/ioctl_iflags.2.html .

## Result

Descriptor-based native APIs agreed with core architect and verified against installed Apple SDK and official Linux kernel UAPI. Implementation can proceed once core models complete0007.
