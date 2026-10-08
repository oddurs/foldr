---
id: 47
uid: 3636b079-bc24-47f7-bcd3-577dcaa840d4
title: Prove Finder tag API preservation and recovery feasibility
type: chore
status: planned
milestone: release-0.2.0
assignee: platform-engineer
created: 2026-10-07
updated: 2026-10-07
priority: p1
effort: s
area: macos
---

## Problem

v0.1 retains Finder tag bytes but offers no semantic tag editing. Adding this native feature needs evidence that the selected API preserves unrelated metadata and respects foldr directory identity and symlink guarantees.

## Proposal

Research Apple's supported tagNamesKey API alongside the existing descriptor-held metadata path. The supported Foundation API is documented at https://developer.apple.com/documentation/foundation/urlresourcekey/tagnameskey; its existence alone does not establish descriptor identity safety or byte-exact recovery. Use disposable macOS folders to compare native/Finder-visible state before and after edits, including names, colors, absent tags, malformed data and foreign metadata. Record exactly which fields an operation changes and how the change engine can journal and restore them.

## Acceptance criteria

- [ ] A recorded API/encoding/side-effect matrix includes Unicode tag names, colors, empty/absent/malformed values and relevant FinderInfo/tag attributes.
- [ ] A native fixture proves no-follow/opt-in symlink policy, held-target identity, unrelated metadata preservation and byte-exact before/after recovery; concurrent external edits have a defined refusal case.
- [ ] A concrete go/no-go decision lists supported semantics and required typed adapter fields; a no-go explicitly defers0048 and updates0041/0049 rather than weakening existing mutation guarantees.
