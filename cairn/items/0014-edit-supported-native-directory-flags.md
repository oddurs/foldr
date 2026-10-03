---
id: 14
uid: a8d7eb14-2b0d-4c51-b535-ea06ae40f650
title: Edit supported native directory flags
type: feature
status: doing
milestone: v0.2
assignee: platform-engineer
depends_on:
- 12
created: 2026-10-02
updated: 2026-10-02
priority: p1
effort: m
area: platform
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Expose macOS hidden/uchg and Linux inode flags only with verified support and sufficient rights. Ordinary permission failures remain errors; no automatic privilege escalation. Linux hiding by rename is a separate explicit operation if implemented.

## Acceptance criteria

- [ ] Supported toggles support dry-run, verification and undo.
- [x] Directory immutability is explained as entry protection rather than recursive content freezing.
- [ ] Unavailable writes name the capability or privilege requirement and cause no unrelated changes.

## 2026-10-02

macOS integration verified: simultaneous note+uchg applies metadata before lock and undo clears lock before restoring note; prelocked folder unlock+note edit applies unlock first and undo restores note before lock. Native immutable test confirms new-entry creation fails while existing child contents remain writable. Hidden apply/undo preserves unrelated binary metadata. All29 native macOS core tests and core clippy -Dwarnings pass. Linux native test verifies immutable toggle/undo when authorized or actionable CAP_LINUX_IMMUTABLE denial with unchanged flags and foreign metadata; pending native CI before closing cross-platform criterion1/3.
