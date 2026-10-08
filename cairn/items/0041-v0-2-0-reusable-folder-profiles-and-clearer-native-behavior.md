---
id: 41
uid: 4f7102df-75ed-47fa-85d8-0bfda9bf6f90
key: release-0.2.0
title: v0.2.0 — Reusable folder profiles and clearer native behavior
type: milestone
status: done
assignee: integration-lead
created: 2026-10-07
updated: 2026-10-07
closed_at: 2026-10-07
priority: p0
effort: l
area: release
---

Make foldr useful across repeated folder workflows: install a named preset once, apply it explicitly, check for drift in scripts, understand permission inheritance, and find the relevant recovery record. Continue using Rust and the existing CLI/core/native split.

## Baseline and versioning

The public baseline is v0.1.0, released from25a6bf7. The original Cairn keys v0.1/v0.2/v0.3 describe bootstrap phases, all delivered in that first release. This actual next release uses key release-0.2.0 to preserve historical identifiers without confusing shipped and planned versions. Package version stays0.1.0 until the release gate explicitly changes it; planning alone changes no source code.

## Product workflow

Proposed grammar, finalized in0042:

```sh
foldr preset install inbox.toml --name inbox
foldr preset list
foldr preset apply --name inbox ./incoming --dry-run
foldr preset apply --name inbox ./incoming
foldr preset check --name inbox ./incoming
foldr permissions explain ./incoming
foldr undo history --path ./incoming --limit 10
# macOS only, if the native preservation gate passes:
foldr tags add ./incoming Inbox --dry-run
```

Named presets are an explicit user library; folders never activate a preset merely by being inspected or entered. Checking reports drift independently of errors and writes neither metadata nor recovery state. Directory operations retain explicit targets and the existing nonrecursive behavior.

## Delivery order and ownership

| Wave | Item | Owner | Purpose |
| --- | --- | --- | --- |
| 1 | 0042 | core-architect with CLI review | Compatibility, grammar, schemas and drift exit contract |
| 1 | 0047 | platform-engineer | Native Finder tag feasibility and preservation decision |
| 2 | 0043 | cli-engineer | Named preset library |
| 2 | 0045 | core-architect | Filtered recovery history |
| 2 | 0046 | platform-engineer | Read-only permission and inheritance explanations |
| 3 | 0044 | cli-engineer | Preset compliance checks using named/file sources |
| 3 | 0048 | platform-engineer with core/CLI integration | Finder tags, conditional on0047 |
| 4 | 0049 | integration-lead | Native regression, docs and release artifacts |

These are work owners, not newly launched agents. Integration controls shared schema/grammar decisions; implementations can proceed concurrently once their actual dependencies are done. Effort fields are relative sizes, not calendar promises.

## Finder decision and bounded scope

Finder tags are the single conditional native addition.0047 must prove identity/symlink behavior, related metadata preservation and exact recovery using temporary folders before0048 starts. If it fails, record the specific reason, move0048 back to later/backlog, remove that dependency from0049 and record the narrower release scope here. Do not mark an unimplemented feature done. Linux continues to report this native feature unsupported.

Rich ACL writes, custom icons, arbitrary native-attribute edits, ownership changes, encryption, quotas, compression/casefold setup, recursion, watchers and GUI/TUI remain later backlog. Homebrew/crates.io publication and signed/notarized builds are separate distribution work; this release keeps tested GitHub archives and source installation.

## Acceptance criteria

- [x] Named presets, read-only compliance checks, permission explanations and filtered recovery history meet their child acceptance criteria on supported platforms.
- [x] Finder tags have a recorded native go/no-go decision; they either pass preservation/recovery gates or are explicitly deferred with milestone/dependencies updated.
- [x] Existing v0.1.0 CLI/data behavior remains covered and all mandatory children plus the native release gate are complete.

## 2026-10-07

All 12 release-0.2.0 children are complete. Finder decision 0047 was GO and 0048 passed native preservation/recovery gates. Named presets, stdout-only read-only drift checks, permission explanations and identity-filtered recovery history are shipped. Public v0.2.0 at https://github.com/oddurs/foldr/releases/tag/v0.2.0 resolves to verified merge 92955690e05edb721b33d414909097a85fcc2908 with Linux/Intel Mac/ARM Mac archives and verified uploaded SHA256 digests; CI 37715685128 and Packages 37715685115 passed all seven native jobs. Compatibility fixtures came from actual v0.1.0.

## Result

Shipped v0.2.0: reusable named presets, read-only compliance checks with drift exit 6, native permission explanations, filtered recovery history, and macOS Finder tag editing with exact recovery. All 12 release children completed. Native Ubuntu/macOS stable/MSRV and Linux/Intel Mac/ARM Mac package gates passed; 80 tests per local toolchain, genuine v0.1 compatibility fixtures and independent filesystem workflows passed. The public tag resolves to merge 92955690e05edb721b33d414909097a85fcc2908; all six release assets match verified digests. Optional interfaces and advanced filesystem features remain later backlog.
