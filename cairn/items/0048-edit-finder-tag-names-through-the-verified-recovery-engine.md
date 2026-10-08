---
id: 48
uid: a25d910a-0ac8-4b74-8717-3d8466db02f7
title: Edit Finder tag names through the verified recovery engine
type: feature
status: planned
milestone: release-0.2.0
assignee: platform-engineer
depends_on:
- 42
- 47
created: 2026-10-07
updated: 2026-10-07
priority: p2
effort: l
area: macos
---

## Problem

Users should be able to label folders from the terminal and recover those edits with the same workflow as notes and flags.

## Proposal

Conditional on0047 go: add tags list/add/remove PATH TAG with explicit dry-run and existing follow-symlink options. Do not expose color mutation until its exact preservation/encoding semantics are verified. Use a narrow typed native field and journal every changed field, preserving existing color data when manipulating names. Linux returns an explicit native-feature limitation; it must not receive pretend Finder metadata.

Connect inspect/diff and an explicitly selected finder_tags preset field only if the same field model supports lossless round-trip and refusal-before-write on Linux. Missing tags in a partial preset never clear them. Broad appearance exploration0024 remains backlog for icons and other Finder behavior.

## Acceptance criteria

- [ ] Native tag list/add/remove semantics match the verified0047 contract, preserve unrelated tags/colors and Finder metadata, and handle unsupported/malformed states without guessing.
- [ ] Dry-run performs no writes; apply, partial failure and undo use typed validated records, refuse identity/external-field conflicts and restore original bytes for every affected field.
- [ ] Explicitly selected tag presets/diffs preserve omission semantics, and Linux preflight refuses unsupported tag edits before any requested field is changed.
- [ ] Native macOS temporary fixtures and terminal-only walkthrough pass; Linux CLI reports unsupported consistently, with generated help/completions/man matching the new grammar.
