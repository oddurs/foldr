---
id: 45
uid: 645e5009-9eb6-460a-80c0-e3a8e8b92219
title: Filter recovery history by folder and explain record status
type: feature
status: done
milestone: release-0.2.0
assignee: core-architect
depends_on:
- 42
created: 2026-10-07
updated: 2026-10-07
closed_at: 2026-10-07
priority: p1
effort: m
area: recovery
---

## Problem

Once several folders have been edited, a global history list makes it hard to find the correct record and distinguish complete, partial or interrupted work.

## Proposal

Add undo history --path PATH --limit N with deterministic newest-first ordering. Match the inspected directory identity, expose the recorded path and distinguish renamed/replaced folders; never select another inode just because the path string matches. Summarize complete/partial/interrupted records and show the explicit undo show/apply commands. Keep actions tied to a selected record ID; no automatic latest undo or history deletion in this release.

## Acceptance criteria

- [x] v0.1 records remain readable and history without new flags remains compatible; statuses agree with per-field durable journal states.
- [x] Folder filtering handles symlinks, renamed/replaced targets and byte-encoded paths according to0042; equal timestamps have a deterministic tie-break and invalid limits fail clearly.
- [x] Human summaries omit note/attribute contents by default; JSON remains lossless, explicit show retains full record detail and corrupt records receive specific diagnostics.
- [x] Temporary-state tests prove filtering/status rendering creates no state and does not rewrite, prune or apply recovery records.

## 2026-10-07

Implemented history_query and HistoryQuery/HistoryEntry/RecordStatus alongside unchanged legacy history JSON and v0.1 record readers. Query follows read-only inspection symlinks and pins canonical-directory identity, filters device/inode only, exposes original byte path plus matched current path/renamed indication, orders numeric nanosecond ID prefix descending with deterministic ID tie-break, rejects zero limit. Complete/partial/failed/interrupted derive only durable completed/per-field states. Load errors identify corrupt filename and retain future-schema refusal. Missing state returns empty without creation.

Evidence: core tests history_query_matches_identity_and_never_rewrites_state and history_status_uses_durable_fields_and_encoded_record_paths pass on native macOS Rust1.85.0. They prove rename/symlink matches, replacement exclusion, numeric9/10 and equal timestamp tie-break, byte-encoded recorded path, exact journal bytes unchanged, no note write or missing-state creation. Actual released v0.1 compatibility fixture3 tests pass. CLI reviewer confirms filtered_history_preserves_legacy_records_handles_renames_and_hides_contents_in_human_output passes exactRust1.85.0 native macOS: lossless legacy/full-show JSON; new summary JSON; sensitive-secret absent from default/filtered human output; explicit undo show command shown; no journal changes; zero limit2; corrupt broken.json diagnostic1. Whole native Linux execution remains0049 release gate; no false claim of Linux verification at this stage.

## Result

Read-only identity-filtered recovery history with deterministic newest-first queries, durable status summaries, private human output and lossless legacy JSON; core/CLI native macOS Rust1.85 tests pass.
