---
id: 48
uid: a25d910a-0ac8-4b74-8717-3d8466db02f7
title: Edit Finder tag names through the verified recovery engine
type: feature
status: doing
milestone: release-0.2.0
assignee: platform-engineer
claimed: 2026-10-07
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

- [x] Native tag list/add/remove semantics match the verified0047 contract, preserve unrelated tags/colors and Finder metadata, and handle unsupported/malformed states without guessing.
- [x] Dry-run performs no writes; apply, partial failure and undo use typed validated records, refuse identity/external-field conflicts and restore original bytes for every affected field.
- [x] Explicitly selected tag presets/diffs preserve omission semantics, and Linux preflight refuses unsupported tag edits before any requested field is changed.
- [ ] Native macOS temporary fixtures and terminal-only walkthrough pass; Linux CLI reports unsupported consistently, with generated help/completions/man matching the new grammar.

## 2026-10-07

Verified0047 native adapter integrated by core typed Field::FinderTags optional raw byte field, request/preset finder_tags explicit membership with schema2 only for native tags; existing-only preset/recovery writers remain schema1. Core native tests passed injected partial tag-then-mode failure with byte-exact noncanonical XML restoration, retained colors and foreign FinderInfo/xattrs, schema/namespace/malformed refusal before any note write or state creation, external apply/undo byte conflicts. Preset tests cover schema2/platformmacOS, omission, explicit clear+undo, native order-independent membership comparison and legacy-label indeterminate state. Linux core preflight refusal fixture exists; real Linux execution pending CI.

## 2026-10-07

Native terminal-only walkthrough passed independently against built CLI: seeded Python plistlib binary [Unicode研究 newline6, Keep newline2], FinderInfo bytes0..31 including nonzero legacy label, foreign binary attr and native provenance. list reports both names/colors; add --dry-run preserves entire xattr map and creates no state; actual add retains native order/colors and byte-identical FinderInfo; preset save --fields finder_tags followed by check succeeds; undo restores byte-exact original entire attr map; remove Keep followed by undo does likewise. Remaining criterion4 gate is actual Linux runtime unsupported/preflight fixture plus generated help/completions/man consistency.

## 2026-10-07

CLI tags list/add/remove are wired only to shared typed same-descriptor finder_tag_plan for incremental edits. Native macOS CLI fixture finder_tags_cli_preserves_native_colors_foreign_metadata_and_byte_exact_undo passed exactRust1.85.0 and stable: noncanonical colored XML seed, retained native ordering/colors, untouched FinderInfo/foreign xattr, dry-run mode/mtime/ctime/no-state preservation, schema2 typed plan/recovery, explicit selected tag save/check, remove and byte-exact successive undo, default symlink refusal/explicit follow, and remove-all preserving foreign metadata. malformed_and_legacy_native_tags_are_not_reported_compliant_or_changed verifies invalid native plist and legacy label states remain indeterminate/refused before state/write. generated_documents_include_new_grammar_and_library_option checks root/subcommand help, bash completion and generated roff man; Linux native CLI unsupported fixture is authored and execution remains0049 gate. Full CLI2unit+24integration and denied-warning Clippy passed exactRust1.85.0 and stable on macOS.
