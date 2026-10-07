---
id: 9
uid: 38f678ba-04a8-4568-be50-999504de191d
title: Inspect Linux folder metadata and filesystem capabilities
type: feature
status: done
milestone: v0.1
assignee: platform-engineer
depends_on:
- 7
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p1
effort: m
area: linux
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Read ownership, permissions, ACL summaries, xattrs, inode flags and filesystem identity using native APIs. Do not assume a filesystem name alone proves a particular write capability.

## Acceptance criteria

- [x] Inspection works on the initial Linux test filesystem.
- [x] Missing ACL/xattr/flag support and permission failures are reported separately.
- [x] Capability queries remain read-only and uncertain support is shown as unknown.

## 2026-10-02

Linux adapter implemented with fstatfs/statvfs, descriptor xattrs and FS_IOC_GETFLAGS/SETFLAGS, kernel POSIX ACL xattr v2 decoding. Unknown filesystem types retain numeric identity. Linux hidden unsupported without rename; immutable privilege CAP_LINUX_IMMUTABLE described. Read-only capabilities are reported without mutation. Native Linux runtime verification requested from integration owner and acceptance1 remains unticked pending CI.

## 2026-10-02

Native Linux runtime verified at checkpoint ef2a449 by Ubuntu stable CI run37092594760 job111115894608: full unit, CLI integration and executable end-to-end checks passed. Ubuntu package run37092594808 job111115894444 passed unpacked binary, checksums and end-to-end checks. Linux native fixtures include arbitrary nonUTF8 xattr names/filenames, raw binary values, POSIX default ACL decoding and hidden unsupported behavior. CI overall failure is unrelated Rust1.85 format_collect lint in compare.rs; native Linux stable runtime passed. Evidence https://github.com/oddurs/foldr/actions/runs/37092594760 and https://github.com/oddurs/foldr/actions/runs/37092594808 .

## Result

Linux native inspector verified on Ubuntu stable CI and packaged executable. Binary xattrs, numeric ACL summaries, inode flags, filesystem identity and read-only capability states pass native tests; missing support and permission failures are distinct.
