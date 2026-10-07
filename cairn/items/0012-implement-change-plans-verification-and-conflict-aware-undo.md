---
id: 12
uid: d31d9c15-f831-46a9-8506-bc6f69a51dc0
title: Implement change plans, verification, and conflict-aware undo
type: feature
status: done
milestone: v0.2
assignee: core-architect
depends_on:
- 7
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
effort: l
area: core
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Build field-level change plans and a persistent local recovery journal. Capture identity and old/new values before writes; revalidate before applying. Report partial success and refuse undo when observed values conflict. Plan restrictive operations last.

## Acceptance criteria

- [x] Dry-run performs no writes and shows exact affected fields and scope.
- [x] Failure mid-operation leaves recoverable per-field results instead of claiming atomicity.
- [x] Undo checks folder identity and intervening changes and preserves unrelated metadata.

## 2026-10-02

Team integration decision: implementation depends on the completed typed core contract (0007), allowing engine development alongside adapter/CLI work. Full inspection and editing release verification remains gated by 0011 and 0017; this item cannot close without actual backend and recovery evidence.

## 2026-10-02

Implemented native-backed field plans, dry-run read-only planning, pre-write identity/value revalidation, private descriptor-relative write-ahead journals synced before writes, Applying/Applied/Failed/Pending outcomes, native reread verification, partial failure recovery, and conflict-safe field-only undo. Core 23 tests cover injected second-write failure, persisted partial outcomes, crash-window Applying recovery, identity replacement, mode conflicts, unrelated binary xattrs, no state writes on dry-run, and insecure/symlink journal refusal. Owner read/search must remain set; unreadable/extended ACL chmod is refused. CI release gating remains0017.

## Result

Implemented explicit change plans, exact scopes and no-write dry runs, descriptor-stable native edits, durable per-field journals and verified partial outcomes. Undo checks device/inode and expected after values, restores only recorded fields and refuses conflicting changes. Restrictive mode/immutable edits are ordered after metadata; clearing immutable precedes other writes. Native APIs cannot eliminate external writer TOCTOU or inode-reuse limitations, documented in core.
