---
id: 12
uid: d31d9c15-f831-46a9-8506-bc6f69a51dc0
title: Implement change plans, verification, and conflict-aware undo
type: feature
status: planned
milestone: v0.2
depends_on:
- 11
created: 2026-10-02
updated: 2026-10-02
priority: p0
effort: l
area: core
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Build field-level change plans and a persistent local recovery journal. Capture identity and old/new values before writes; revalidate before applying. Report partial success and refuse undo when observed values conflict. Plan restrictive operations last.

## Acceptance criteria

- [ ] Dry-run performs no writes and shows exact affected fields and scope.
- [ ] Failure mid-operation leaves recoverable per-field results instead of claiming atomicity.
- [ ] Undo checks folder identity and intervening changes and preserves unrelated metadata.
