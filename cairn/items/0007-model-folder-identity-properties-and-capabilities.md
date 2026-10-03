---
id: 7
uid: 78b9a64a-c778-4346-bd27-7dcc4c2a04ed
title: Model folder identity, properties, and capabilities
type: feature
status: planned
milestone: v0.1
depends_on:
- 6
created: 2026-10-02
updated: 2026-10-02
priority: p0
effort: m
area: core
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Define inspection snapshots, typed errors, property scopes, lossless path/value encodings, and read/write capability states with reasons. Specify symlink handling and inspect-only behavior.

## Acceptance criteria

- [ ] Non-UTF-8 names, control characters, and binary xattrs have lossless structured representations.
- [ ] Unsupported, unknown, and permission-denied properties remain distinguishable.
- [ ] Scope and symlink semantics are documented and covered by focused checks.
