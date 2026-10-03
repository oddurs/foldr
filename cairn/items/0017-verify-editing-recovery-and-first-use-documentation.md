---
id: 17
uid: 5eb7fa03-8fd3-43ac-9fcf-cf7a721fcb8b
title: Verify editing, recovery, and first-use documentation
type: chore
status: done
milestone: v0.2
assignee: integration-lead
depends_on:
- 11
- 13
- 14
- 15
- 16
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
effort: m
area: verification
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Validate real filesystem behavior in temporary folders. Document shell examples, metadata preservation, restrictive flag ordering and recovery from partial failure. Exercise ordinary-user paths on both OSes.

## Acceptance criteria

- [x] Integration checks cover partial failure, permission loss, external modification and undo conflicts.
- [x] No test modifies user folders or silently requires administrator rights.
- [x] The walkthrough demonstrates inspect, note edit, dry-run preset application and undo on both OSes.

## 2026-10-02

Native Linux/macOS stable and Rust1.85.0 validation all passed at a4f147c (CI37093137863). Temporary-folder core tests exercise injected partial writes, crash between native write and journal update, sync failure before writes, replaced target identity, external edits/undo conflicts and refusal of unrecoverable permission loss; descriptor traversal tests exercise denied search and searchable-only ancestors. Native macOS ACL refusal was separately verified by the platform role. No tests touch existing user folders; each uses temporary target/state, and Linux immutable ordinary-user denial is explicitly covered without elevation. scripts/verify-cli.py and README walkthrough demonstrate inspect, note edits, no-write previews, partial presets/batches and undo on both native OSes. README documents field-level recovery, concurrency limitations, restrictive flags and permission scope.

## Result

Verified editing and recovery on native macOS/Linux stable and MSRV jobs. Partial/crash recovery, permission-loss refusal, identity and field conflicts, binary preservation, no-write dry-run and explicit presets/batches pass in isolated fixtures. First-use documentation and actual CLI walkthrough match, with concurrency and platform limitations stated.
