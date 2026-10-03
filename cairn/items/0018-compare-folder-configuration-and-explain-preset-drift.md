---
id: 18
uid: 39e7d331-4e4b-4d9f-b7a7-91c0e796dad7
title: Compare folder configuration and explain preset drift
type: feature
status: planned
milestone: v0.3
depends_on:
- 17
created: 2026-10-02
updated: 2026-10-02
priority: p1
effort: m
area: cli
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Add comparisons between folders, recorded snapshots and presets using semantic property differences. Exclude volatile fields by default and explain unsupported comparisons.

## Acceptance criteria

- [ ] Diff distinguishes missing properties from unsupported reads.
- [ ] Human and structured output identify each changed field and scope.
- [ ] Comparisons perform no writes and support lossless path/value encodings.
