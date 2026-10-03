---
id: 17
uid: 5eb7fa03-8fd3-43ac-9fcf-cf7a721fcb8b
title: Verify editing, recovery, and first-use documentation
type: chore
status: planned
milestone: v0.2
depends_on:
- 16
created: 2026-10-02
updated: 2026-10-02
priority: p0
effort: m
area: verification
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Validate real filesystem behavior in temporary folders. Document shell examples, metadata preservation, restrictive flag ordering and recovery from partial failure. Exercise ordinary-user paths on both OSes.

## Acceptance criteria

- [ ] Integration checks cover partial failure, permission loss, external modification and undo conflicts.
- [ ] No test modifies user folders or silently requires administrator rights.
- [ ] The walkthrough demonstrates inspect, note edit, dry-run preset application and undo on both OSes.
