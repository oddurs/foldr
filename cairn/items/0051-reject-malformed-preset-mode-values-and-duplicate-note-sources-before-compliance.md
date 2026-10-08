---
id: 51
uid: fba49bde-aeca-4103-9c23-8d0842ee4d17
title: Reject malformed preset mode values and duplicate note sources before compliance
type: bug
status: done
milestone: release-0.2.0
assignee: core-architect
created: 2026-10-07
updated: 2026-10-07
closed_at: 2026-10-07
priority: p1
effort: s
area: core
part_of:
- 41
---

## What happens

A schema1 preset with mode4096 is syntactically valid TOML but outside Unix mode0000–7777; read-only comparison could report drift instead of invalid input. A note plus metadata.note also reaches comparison, where the note silently replaces the other requested value, although apply refuses this duplicate source.

## What should happen

Shared preset validation rejects malformed field requests during parsing, writing, planning and checking. Native platform restrictions and explicit omission/removal semantics remain unchanged. Existing releasedv0.1 fixtures still load losslessly.

## Reproduction

1. Check a preset with `schema_version=1`, `[settings]`, `mode=4096`; previously this compared an impossible mode as drift.
2. Check a preset with `[settings] note='one'` and `[settings.metadata] note=[116,119,111]`; previously comparison silently selected one value.

## Acceptance criteria

- [x] Out-of-range modes and duplicate note sources return invalid input consistently before comparison or target planning.
- [x] Meaningful regression tests reject both TOML and programmatic requests while actualv0.1 fixtures and existing valid partial presets still pass.

## 2026-10-07

Shared validate_preset now checks known schema before fields, rejects mode beyond07777, validates metadata suffix keys and rejects simultaneous note/metadata.note; invoked from TOML reader, TOML writer, preset_plan and compare_preset/check_preset. Meaningful malformed_preset_requests_do_not_count_as_drift test rejects programmatic invalid mode/double note and both malformed TOML forms. ExactRust1.85.0 native macOS final core suite passes51 unit+3 actual releasedv0.1 fixture compatibility tests; cargo +1.85.0 clippy --locked -p foldr-core --all-targets -- -D warnings passes. No new dependencies.

## Result

Malformed preset mode and duplicate note sources are consistently invalid before comparison/planning; regression and actualv0.1 compatibility tests pass on Rust1.85.0.
