---
id: 9
uid: 38f678ba-04a8-4568-be50-999504de191d
title: Inspect Linux folder metadata and filesystem capabilities
type: feature
status: planned
milestone: v0.1
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
- [ ] Missing ACL/xattr/flag support and permission failures are reported separately.
- [ ] Capability queries remain read-only and uncertain support is shown as unknown.
