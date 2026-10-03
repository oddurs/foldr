---
id: 9
uid: 38f678ba-04a8-4568-be50-999504de191d
title: Inspect Linux folder metadata and filesystem capabilities
type: feature
status: doing
milestone: v0.1
assignee: platform-engineer
claimed: 2026-10-02
depends_on:
- 7
created: 2026-10-02
updated: 2026-10-02
priority: p1
effort: m
area: linux
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Read ownership, permissions, ACL summaries, xattrs, inode flags and filesystem identity using native APIs. Do not assume a filesystem name alone proves a particular write capability.

## Acceptance criteria

- [ ] Inspection works on the initial Linux test filesystem.
- [x] Missing ACL/xattr/flag support and permission failures are reported separately.
- [x] Capability queries remain read-only and uncertain support is shown as unknown.

## 2026-10-02

Linux adapter implemented with fstatfs/statvfs, descriptor xattrs and FS_IOC_GETFLAGS/SETFLAGS, kernel POSIX ACL xattr v2 decoding. Unknown filesystem types retain numeric identity. Linux hidden unsupported without rename; immutable privilege CAP_LINUX_IMMUTABLE described. Read-only capabilities are reported without mutation. Native Linux runtime verification requested from integration owner and acceptance1 remains unticked pending CI.
