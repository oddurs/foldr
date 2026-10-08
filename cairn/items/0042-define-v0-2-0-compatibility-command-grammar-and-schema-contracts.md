---
id: 42
uid: 4ec02ca4-92ec-456c-80a1-9ad58f6c545d
title: Define v0.2.0 compatibility, command grammar and schema contracts
type: docs
status: done
milestone: release-0.2.0
assignee: core-architect
created: 2026-10-07
updated: 2026-10-07
closed_at: 2026-10-07
priority: p0
effort: s
area: contracts
---

## Problem

New named sources, structured ACL summaries and native tag edits touch contracts already published in v0.1.0. Decide these before parallel implementation rather than letting each command invent a format.

## Proposal

Record command argument groups, read-only compliance statuses, library paths and named-source precedence. Keep existing file-based commands working. Proposed preset check returns0 for compliant,6 for definite drift, and existing error categories for unsupported/indeterminate/operational failures; error precedence and mixed-target JSON are decided explicitly. Existing diff remains informational unless a check option is explicitly requested.

Define reader/writer compatibility for preset, snapshot, JSON-envelope and recovery versions separately. Load real v0.1 fixtures; a new field/type requiring a schema bump gets a migration or clear refusal, never silent loss. Typed Finder metadata is a narrowly authorized native field; do not weaken generic foldr-owned-attribute validation. No new promise of filesystem CAS or atomic multi-field changes.

## Acceptance criteria

- [x] A command/schema compatibility table includes old commands, file-vs-name conflicts, drift exit6, unknown-version handling and per-target error precedence.
- [x] Actual v0.1.0 preset/snapshot/recovery fixtures are captured with binary values and omission/removal semantics; required readers preserve them or explicitly document a reviewed migration.
- [x] Core/platform/CLI reviewers agree on native field authorization, JSON capability states and symlink/identity rules before dependent features start.

## 2026-10-07

Reviewed contract (core-architect, cli-specialist, native-platform-specialist and orchestrator):

| Surface | v0.2.0 contract |
| --- | --- |
| Existing inspect/doctor/note/attr/flags/permissions/save/show/import/export/undo/diff | Existing grammar retained. diff informational exit0 on successful comparison. |
| Named apply/check | Without --name first positional is FILE and remaining positional paths are explicit targets; with --name all positionals are targets. Exclusive source modes, no extension heuristics or file/name fallback. install FILE --name NAME, list. |
| Library | --preset-dir wins; otherwise macOS HOME/Library/Application Support/foldr/presets; Linux absolute nonempty XDG_CONFIG_HOME/foldr/presets else HOME/.config/foldr/presets. Library reads never create storage. |
| Check results | JSON envelope1 data explicit_folders/recursive=false plus ordered targets, lossless EncodedPath, status/differences/error. All selected targets evaluated; definite drift emits stdout exit6 without stderr; compliant exit0. Indeterminate selected values are errors, never compliance. |
| Error precedence | Global source load/parse failure stops before targets with existing category. Across targets operational/indeterminate1 > conflict4 > unsupported3 > invalid2 > drift6 > compliant0. Both human and machine modes agree. Existing batch mutation partial5 unchanged. |
| Preset | Readers accept1, preserve binary arrays, omission and explicit removal. Typed native tag settings require2 and platform=macos;1 must never contain tags. Writer1 for existing-only fields,2 for tags. Unknown version or settings fail clearly. |
| Snapshot/JSON | Snapshot1 and envelope1 retained; existing ACL fields unchanged. Permission explanations use separate result structures. Unknown snapshot/envelope versions rejected. Property/Capability five states remain supported/unsupported/unknown/permission_denied/unavailable. |
| Plans/recovery | Existing-only plans/records write1 and readers preserve1. Typed FinderTags raw exact bytes require2; reader accepts1/2 but rejects unknown versions and typed tag field under1. Generic Xattr authorization remains only com.foldr.* or user.foldr.* and never grants native tag access. |
| Identity/history | Read-only inspect/check/history follow symlinks. Writes keep explicit follow-symlink requirement at planning, descriptor identity/native rereads and nofollow apply/undo. History path filters current descriptor device/inode, excludes replacement inode, matches renamed objects and keeps recorded byte path. New query newest-first by numeric timestamp then deterministic ID tie-break; legacy history remains full lossless records. No filesystem CAS/multi-field atomicity promise. |

Evidence: unmodified v0.1.0 git tag exported and built in /private/tmp/foldr-v010-fixtures-source; six real CLI output fixtures in crates/foldr-core/tests/fixtures/v0.1.0 capture note, binary00ff0a, snapshot inspect envelope, preset save/import omissions/removals and native recovery journals. cargo test --locked -p foldr-core --test compatibility passed3 tests including actual load_record on macOS; historical paths unchanged. Explicit same-version lossless serialization tests protect reader compatibility. Native/CLI agreement obtained before dependent implementation.

## Result

Agreed command/source/check/schema/native identity contracts with core/CLI/platform; actual released-tag binary fixtures and reader roundtrip/load tests pass.
