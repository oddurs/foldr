---
id: 19
uid: 4715b6a4-bdfb-446f-8c46-d6a3b2c2f490
title: Apply explicit batch operations with per-folder recovery
type: feature
status: planned
milestone: v0.3
depends_on:
- 17
created: 2026-10-02
updated: 2026-10-02
priority: p1
effort: l
area: cli
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Accept explicit folder lists and optional bounded recursive selection. Preflight the selection, show scope/count, preserve symlink and mount boundaries, and report each result. Do not advertise all-or-nothing transactions.

## Acceptance criteria

- [ ] Selection rules and dry-run make every target visible.
- [ ] Failure behavior and aggregate exit statuses are specified and tested.
- [ ] Each applied target has recovery information and successful edits survive unrelated failures.
