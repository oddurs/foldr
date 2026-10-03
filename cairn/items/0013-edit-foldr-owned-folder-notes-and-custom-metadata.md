---
id: 13
uid: f411bf50-a445-4698-8f0c-180e46025cfc
title: Edit foldr-owned folder notes and custom metadata
type: feature
status: doing
milestone: v0.2
assignee: cli-engineer
claimed: 2026-10-02
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
- [x] Removing a foldr attribute never clears other attributes.
- [x] Size/support failures and copy/Git/sync limitations are explained.

## 2026-10-02

CLI note get/set/remove and owned-attribute get/set/remove are executable through the shared engine; UTF8 note values are stored as native raw bytes, binary metadata uses --hex, with lossless JSON output. macOS ten focused CLI integration checks pass: no-write note previews, note conflicts, binary roundtrip/removal preserving another key, partial preset and import/export. Note/attr --help and doctor explain filesystem size/support constraints and copy/archive/sync/Git loss. Linux runtime evidence is still pending and criterion1 remains unticked until it arrives.
