---
id: 44
uid: 2484c12c-d8f0-48dd-802a-030c7105ef2a
title: Check preset compliance with scriptable drift results
type: feature
status: done
milestone: release-0.2.0
assignee: cli-engineer
depends_on:
- 42
- 43
created: 2026-10-07
updated: 2026-10-07
closed_at: 2026-10-07
priority: p1
effort: m
area: compare
---

## Problem

Diffs show changes but always succeed as an informational command. Scripts need a read-only answer to whether explicit folders satisfy a file or named partial preset.

## Proposal

Add preset check with the same exclusive file/name source selection as apply. Use shared semantic comparison for requested fields only. Report compliant, drift or indeterminate per target, with field reasons and desired/observed values; unsupported or denied properties cannot count as compliant. Follow the agreed exit0/6/error precedence from0042. Existing diff behavior stays compatible.

## Acceptance criteria

- [x] A partial preset checks only selected fields, including binary metadata and explicit removals; unrelated fields and volatile identity do not create drift.
- [x] Single/multiple targets produce deterministic human and versioned JSON results; drift is distinct from inaccessible/unsupported/malformed inputs with documented aggregate precedence.
- [x] Read-only checks leave modes, flags, attributes, directory timestamps and recovery state unchanged; file and named presets produce equivalent results.
- [x] Native macOS/Linux checks cover match, drift, mixed targets and unsupported optional native fields without adding implicit recursion or writes.

## 2026-10-07

Implemented preset check FILE PATH... or --name NAME PATH... via shared core semantic comparator. Partial selections compare only requested fields, preserve binary bytes and explicit removals, and produce ordered target status/differences/error JSON. Aggregate precedence is1>4>3>2>6>0; code6 emits stdout with no diagnostic. Four portable integration tests passed native macOS exactRust1.85.0: selected binary/removal and file/name equivalence, mode/atime/mtime/ctime/xattrs/child/state preservation, mixed compliant/drift/missing targets with operational errors outranking drift, malformed2/unknownschema3/platformunsupported3/no named fallback, and actual released v0.1 preset fixtures through CLI. Native malformed Finder tags/legacy labels additionally prove indeterminate cannot become compliant. All existing CLI behavior tests remain passing; portable native Linux fixtures exist but Linux execution remains mandatory0049 release gate, not yet verified.

## Result

Read-only explicit-folder compliance implemented with shared selected-field semantics, deterministic per-target results, scriptable0/6 and agreed failure precedence; portable native fixtures and macOS exact1.85 passed, with Linux native execution still required by0049.

## 2026-10-07

Final full CLI2unit+24integration suite and Clippy --all-targets -D warnings passed macOS exactRust1.85.0 and stable. CLI source frozen for root aggregate/nativeCI release validation; no Linux runtime pass claimed.
