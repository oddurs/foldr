---
id: 47
uid: 3636b079-bc24-47f7-bcd3-577dcaa840d4
title: Prove Finder tag API preservation and recovery feasibility
type: chore
status: done
milestone: release-0.2.0
assignee: platform-engineer
created: 2026-10-07
updated: 2026-10-07
closed_at: 2026-10-07
priority: p1
effort: s
area: macos
---

## Problem

v0.1 retains Finder tag bytes but offers no semantic tag editing. Adding this native feature needs evidence that the selected API preserves unrelated metadata and respects foldr directory identity and symlink guarantees.

## Proposal

Research Apple's supported tagNamesKey API alongside the existing descriptor-held metadata path. The supported Foundation API is documented at https://developer.apple.com/documentation/foundation/urlresourcekey/tagnameskey; its existence alone does not establish descriptor identity safety or byte-exact recovery. Use disposable macOS folders to compare native/Finder-visible state before and after edits, including names, colors, absent tags, malformed data and foreign metadata. Record exactly which fields an operation changes and how the change engine can journal and restore them.

## Acceptance criteria

- [x] A recorded API/encoding/side-effect matrix includes Unicode tag names, colors, empty/absent/malformed values and relevant FinderInfo/tag attributes.
- [x] A native fixture proves no-follow/opt-in symlink policy, held-target identity, unrelated metadata preservation and byte-exact before/after recovery; concurrent external edits have a defined refusal case.
- [x] A concrete go/no-go decision lists supported semantics and required typed adapter fields; a no-go explicitly defers0048 and updates0041/0049 rather than weakening existing mutation guarantees.

## 2026-10-07

Native macOS APFS feasibility matrix: Foundation URLResourceValues.tagNames reads Unicode names and strips recognized color suffixes; setter reserializes every retained name with newline0, destroys colors, and modifies FinderInfo label bits in absent-tag case. Empty bytes/malformed bytes are treated as nil by Foundation, so foldr refuses them rather than overwriting. Valid binary/XML array-of-strings is parsed by CoreFoundation only, descriptor fget/fsetxattr performs I/O. Names retain color0..7 or absent suffix; unknown suffix, duplicate/control/empty names, nonstring root/entries are refused. Existing native tag order/colors persist and added names use color0. FinderInfo is never changed. Absent/empty tags with nonzero FinderInfo label are ambiguous legacy-only metadata and refused; removing final tag with legacy label is refused. Nonempty colored tag arrays with legacy label remain supported. Foreign attrs/FinderInfo are preserved.

## 2026-10-07

GO: platform/tags.rs uses minimal installed-SDK CoreFoundation FFI without new dependency, exports FinderTag{name,color}, TAG_XATTR, decode/encode, descriptor read/update/validate/write. Typed native Field::FinderTags holds optional raw bytes and plan/recovery schema2; generic Xattr remains foldr-owned. Existing core descriptor identity and byte reread gates must refuse external tag bytes that differ from planned/recorded values before writes; no CAS or atomic multifield claim. Native tests platform::tags (3 passed) prove binary Unicode/colors roundtrip, malformed refusal, nofollow/opt-in symlink, held descriptor after rename/path replacement, Foundation independent visibility oracle, byte-exact restore including absent state, foreign metadata preservation and no preview timestamps changes. Typed engine concurrent-edit/partial/undo tests follow under0048.

## Result

GO for narrow descriptor-held native plist tag editing with exact typed raw-byte recovery; Foundation setter rejected due to color/FinderInfo side effects; ambiguous legacy-only and malformed states explicitly refused.
