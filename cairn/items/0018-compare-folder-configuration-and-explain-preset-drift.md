---
id: 18
uid: 39e7d331-4e4b-4d9f-b7a7-91c0e796dad7
title: Compare folder configuration and explain preset drift
type: feature
status: done
milestone: v0.3
assignee: Oddur Sigurdsson
depends_on:
- 10
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p1
effort: m
area: cli
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Add comparisons between folders, recorded snapshots and presets using semantic property differences. Exclude volatile fields by default and explain unsupported comparisons.

## Acceptance criteria

- [x] Diff distinguishes missing properties from unsupported reads.
- [x] Human and structured output identify each changed field and scope.
- [x] Comparisons perform no writes and support lossless path/value encodings.

## 2026-10-02

Semantic comparison needs verified inspection and the CLI contract (0010). Publication/package readiness remains gated by0017 and0020. This work can proceed independently of mutation UX.

## 2026-10-02

Core comparisons distinguish missing xattrs from unsupported/unknown/permission-denied properties, normalize immutable/hidden semantics, exclude identity/path volatility, and preserve binary values via byte arrays. Core unit tests cover binary differences and unsupported versus missing states. CLI comparisons_are_read_only_and_work_with_snapshots_and_presets verifies identical live/snapshot diffs, requested-only preset drift, no journal creation and unchanged mode. Human renderer shows field,left/right,scope; structured output retains tagged states and lossless bytes.

## Result

Implemented read-only semantic folder/snapshot/preset diffs with scopes, missing-versus-unavailable states, platform-aware known flag meanings, and lossless byte values. CLI renders human and structured differences and ignores volatile identity fields.
