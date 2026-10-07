---
id: 31
uid: a633861f-5c23-4ebc-ad9c-d9a8d08bbb29
title: Implement the shared mutation and preset engine ahead of release validation
type: feature
status: done
milestone: v0.2
assignee: Oddur Sigurdsson
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
part_of:
- 12
- 16
- 18
- 19
area: core
effort: xl
---

## Problem

Parallel CLI and native implementation needs the shared core engine before the release verification milestones are complete. Delivery items retain their release gates.

## Proposal

Publish and test native-backed change plans, durable field recovery, partial versioned TOML presets, and semantic comparisons. Transfer evidence to delivery items once their dependencies finish.

## Acceptance criteria

- [x] Shared mutation API compiles and validates plans before any write.
- [x] Partial failure and conflicting undo have native-backed test evidence.
- [x] Presets preserve omitted fields and reject unknown schemas/settings.
- [x] Semantic comparisons preserve binary values and distinguish missing from unavailable reads.

## 2026-10-02

Implement core planning, durable field journal, conflict-aware undo, TOML patches, comparisons and explicit batches. Original delivery items remain gated by release evidence and will be closed only after their dependencies and criteria pass.

## 2026-10-02

Preparatory shared APIs delivered and evidence transferred to completed0012/0016/0018/0019. Core30 native-backed macOS tests and clippy -Dwarnings pass, including crash-window recovery, binary notes, documented sample preset roundtrips, independent batches and unsupported comparisons. Release integration/native Linux execution remains root verification work.

## Result

Published and tested the shared descriptor-backed mutation, durable recovery, strict partial TOML preset, semantic comparison and explicit batch engine; all evidence transferred to delivery items.
