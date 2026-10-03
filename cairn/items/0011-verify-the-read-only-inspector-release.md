---
id: 11
uid: 64b0f27a-415c-40bd-925e-1c723d4e7e17
title: Verify the read-only inspector release
type: chore
status: planned
milestone: v0.1
depends_on:
- 10
created: 2026-10-02
updated: 2026-10-02
priority: p0
effort: m
area: verification
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Add integration fixtures in temporary folders and a documented OS/filesystem check matrix. Test observable behavior, including no changes to flags/xattrs/content during inspection.

## Acceptance criteria

- [ ] Checks cover unusual filenames, binary xattrs, symlinks and unreadable properties.
- [ ] macOS and Linux checks pass and unsupported fixture cases are explicitly skipped.
- [ ] A release checklist documents supported platforms, limitations and read-only guarantees.
