---
id: 33
uid: a3687012-aa41-4386-96e5-e70bbca1c593
title: Implement CLI command surfaces against agreed core contract
type: feature
status: done
milestone: v0.2
assignee: cli-engineer
depends_on:
- 29
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
area: cli
part_of:
- 10
---

## Problem

## Proposal

## Acceptance criteria

- [ ]

## Context

CLI dispatch can be developed while core and native adapters are implemented. Release criteria in 0010 and 0013–0019 remain gated by their dependencies and platform evidence. This item tracks executable preparation and its own focused integration checks.

## Acceptance criteria

- [x] The agreed command grammar dispatches to core APIs without native logic duplication or success placeholders.
- [x] JSON envelopes, encoded values, input errors, and broken pipes follow the recorded output contract.
- [x] Focused CLI integration checks cover non-UTF-8 paths, metadata dry-run and undo conflicts, presets and explicit batches.

## 2026-10-02

Implemented CLI parsing and inspection dispatch, completion/man generation buffered before stdout, versioned JSON envelopes and escaped human output. Mutation dispatch now calls core plan/apply/undo, with global state directory overrides and no state access on folder dry-run. Native adapter compilation is still in progress.

## 2026-10-02

All intended command surfaces dispatch to core APIs: inspect/doctor, note/owned attrs, flags, permissions, versioned TOML presets/import/export, conflict-aware undo, folder/preset/snapshot diff and explicit batches. Success placeholders removed. Human inspection uses octal modes and escaped text/hex values; dryrun previews show native fields and scopes. Ten CLI integration tests plus escaping unit test pass locally on macOS; CLI all-target clippy passes with warnings denied. Linux tests include byte0xff names, while APFS permits only valid UTF8. Full cross-platform release criteria remain tracked separately.

## Result

Implemented all agreed CLI dispatch and shell documents against shared native-backed core, with clean JSON/error channels, encoded data, no-write dryrun, isolated test state, explicit batch outcomes and stable error exits. Ten integration checks plus one escape test and CLI clippy pass on local macOS. Native Linux release validation remains gated by0010/0013 and team CI.
