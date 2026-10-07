---
id: 19
uid: 4715b6a4-bdfb-446f-8c46-d6a3b2c2f490
title: Apply explicit batch operations with per-folder recovery
type: feature
status: done
milestone: v0.3
assignee: Oddur Sigurdsson
depends_on:
- 16
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p1
effort: l
area: cli
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Accept explicit folder lists and optional bounded recursive selection. Preflight the selection, show scope/count, preserve symlink and mount boundaries, and report each result. Do not advertise all-or-nothing transactions.

## Acceptance criteria

- [x] Selection rules and dry-run make every target visible.
- [x] Failure behavior and aggregate exit statuses are specified and tested.
- [x] Each applied target has recovery information and successful edits survive unrelated failures.

## 2026-10-02

Explicit batch application builds on the completed preset/change engine (0016). Publication remains gated by editing validation0017 and packaging0020. No recursive traversal is included without a separate reviewed selection policy.

## 2026-10-02

Shared apply_batch preflights every explicitly listed path before writes; dry-run returns all per-target plans/errors and creates no journal. Successful targets receive durable independent records even when other targets fail. There is no recursion, so symlink and mount boundaries are never traversed implicitly. Core batch test verifies one valid and one missing target, no-write preview, successful native note plus one recovery record; CLI presets_preserve_omitted_fields_and_report_independent_batch_failures verifies explicit paths, failures and aggregate exit5 without losing success. Optional recursive selection is intentionally not exposed.

## Result

Implemented explicit multi-folder preset application with complete preflight, per-target plans/results, no-write dry runs, independent durable recovery journals and aggregate failure exit5. Successful edits remain recoverable when other folders fail; selection is explicitly nonrecursive.
