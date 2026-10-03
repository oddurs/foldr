---
id: 14
uid: a8d7eb14-2b0d-4c51-b535-ea06ae40f650
title: Edit supported native directory flags
type: feature
status: planned
milestone: v0.2
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
- [ ] Directory immutability is explained as entry protection rather than recursive content freezing.
- [ ] Unavailable writes name the capability or privilege requirement and cause no unrelated changes.
