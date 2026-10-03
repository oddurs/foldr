---
id: 34
uid: dc8df41f-a8fe-4c7c-b05f-e43be47eaf7a
title: Validate recovery schemas and malformed plans before writes and persist journal directory creation
type: bug
status: done
milestone: v0.2
assignee: Oddur Sigurdsson
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
part_of:
- 12
area: core
effort: s
---

## What happens

Reliability review found that recovery schemas and handcrafted field plans were not fully validated, and fresh recovery-directory entries lacked parent fsync before folder writes.

## What should happen

Refuse unknown record schemas and malformed or duplicate fields before opening/writing targets; use that validation for undo previews too. Create nested recovery directories by descriptor and sync each parent entry before permitting a target write.

## Reproduction

1. Pass a schema2 recovery record to load/undo, a plan containing a valid first field and malformed second field to apply, or a previously absent nested recovery directory to apply.

## Acceptance criteria

- [x] Future-schema records are refused by load and undo without folder/history writes.
- [x] Every field/value pairing, duplicate field and scope is checked before applying; malformed later fields cannot partially apply earlier fields.
- [x] Newly created nested state directories sync their child metadata and parent entries before any target write, with focused evidence.

## 2026-10-02

Added shared record schema/plan validation at load, history, undo_plan (including dry-run), and apply. Every before/after value variant, metadata suffix, mode bounds, folder scope and duplicate field is rejected before any writes. State directory creation now uses private mkdirat/openat descriptor walks and synchronizes each child inode then parent entry before descending; any sync failure aborts before target writes. Four focused tests prove future record load/undo no writes, all six malformed type pairings plus duplicate/scope refusal without note/history writes, nested child-parent fsync order and injected parent-sync failure. cargo test -p foldr-core35 passed; clippy -Dwarnings passed; strict Cairn prompt validation clean.

## Result

Refuse future-schema or malformed recovery records and validate every field/value/scope and duplicate before mutation or undo previews. Persist each newly created private journal directory inode and parent entry by descriptor before target writes. Four native-backed focused tests verify rejection/no-write guarantees, nested fsync order and creation-sync failure;35 core tests and clippy pass.
