---
id: 7
uid: 78b9a64a-c778-4346-bd27-7dcc4c2a04ed
title: Model folder identity, properties, and capabilities
type: feature
status: done
milestone: v0.1
assignee: core-architect
depends_on:
- 6
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
effort: m
area: core
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Define inspection snapshots, typed errors, property scopes, lossless path/value encodings, and read/write capability states with reasons. Specify symlink handling and inspect-only behavior.

## Acceptance criteria

- [x] Non-UTF-8 names, control characters, and binary xattrs have lossless structured representations.
- [x] Unsupported, unknown, and permission-denied properties remain distinguishable.
- [x] Scope and symlink semantics are documented and covered by focused checks.

## 2026-10-02

Published lossless byte-array path/xattr representations, escaped terminal display, explicit unsupported/unknown/permission-denied states, property scopes, and descriptor-based inspection. Native bridge agreed with platform engineer; core inspection follows and reports symlink targets while mutation requires explicit follow choice.

## 2026-10-02

Verified cargo test -p foldr-core: lossless control/non-UTF8 encoding JSON round trips, binary native xattrs and unrelated attributes, all property statuses, no-follow descriptor opening, explicit symlink inspection/target rules. APFS rejects invalid UTF8 filenames with EILSEQ; the model encoding still round-trips arbitrary Unix bytes and Linux integration exercises actual invalid byte names. Scope types explain folder/future children/descendants. Mutation opening now uses per-component openat O_NOFOLLOW and test fixtures canonicalize macOS aliases.

## Result

Implemented serializable lossless folder snapshots, typed errors, scoped capability/property states, terminal-safe escaping, and directory-descriptor inspection. Native adapters consume the published model; tests cover binary metadata, encoding and explicit symlink semantics.
