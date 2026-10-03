---
id: 13
uid: f411bf50-a445-4698-8f0c-180e46025cfc
title: Edit foldr-owned folder notes and custom metadata
type: feature
status: planned
milestone: v0.2
depends_on:
- 12
created: 2026-10-02
updated: 2026-10-02
priority: p1
effort: m
area: metadata
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Implement get/set/remove for com.foldr.* on macOS and user.foldr.* on Linux through the change engine. Use small versioned values and explicit export/import for portability.

## Acceptance criteria

- [ ] Notes round-trip on both supported platforms.
- [ ] Removing a foldr attribute never clears other attributes.
- [ ] Size/support failures and copy/Git/sync limitations are explained.
