---
id: 45
uid: 645e5009-9eb6-460a-80c0-e3a8e8b92219
title: Filter recovery history by folder and explain record status
type: feature
status: planned
milestone: release-0.2.0
assignee: core-architect
depends_on:
- 42
created: 2026-10-07
updated: 2026-10-07
priority: p1
effort: m
area: recovery
---

## Problem

Once several folders have been edited, a global history list makes it hard to find the correct record and distinguish complete, partial or interrupted work.

## Proposal

Add undo history --path PATH --limit N with deterministic newest-first ordering. Match the inspected directory identity, expose the recorded path and distinguish renamed/replaced folders; never select another inode just because the path string matches. Summarize complete/partial/interrupted records and show the explicit undo show/apply commands. Keep actions tied to a selected record ID; no automatic latest undo or history deletion in this release.

## Acceptance criteria

- [ ] v0.1 records remain readable and history without new flags remains compatible; statuses agree with per-field durable journal states.
- [ ] Folder filtering handles symlinks, renamed/replaced targets and byte-encoded paths according to0042; equal timestamps have a deterministic tie-break and invalid limits fail clearly.
- [ ] Human summaries omit note/attribute contents by default; JSON remains lossless, explicit show retains full record detail and corrupt records receive specific diagnostics.
- [ ] Temporary-state tests prove filtering/status rendering creates no state and does not rewrite, prune or apply recovery records.
