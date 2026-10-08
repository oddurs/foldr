---
id: 44
uid: 2484c12c-d8f0-48dd-802a-030c7105ef2a
title: Check preset compliance with scriptable drift results
type: feature
status: planned
milestone: release-0.2.0
assignee: cli-engineer
depends_on:
- 42
- 43
created: 2026-10-07
updated: 2026-10-07
priority: p1
effort: m
area: compare
---

## Problem

Diffs show changes but always succeed as an informational command. Scripts need a read-only answer to whether explicit folders satisfy a file or named partial preset.

## Proposal

Add preset check with the same exclusive file/name source selection as apply. Use shared semantic comparison for requested fields only. Report compliant, drift or indeterminate per target, with field reasons and desired/observed values; unsupported or denied properties cannot count as compliant. Follow the agreed exit0/6/error precedence from0042. Existing diff behavior stays compatible.

## Acceptance criteria

- [ ] A partial preset checks only selected fields, including binary metadata and explicit removals; unrelated fields and volatile identity do not create drift.
- [ ] Single/multiple targets produce deterministic human and versioned JSON results; drift is distinct from inaccessible/unsupported/malformed inputs with documented aggregate precedence.
- [ ] Read-only checks leave modes, flags, attributes, directory timestamps and recovery state unchanged; file and named presets produce equivalent results.
- [ ] Native macOS/Linux checks cover match, drift, mixed targets and unsupported optional native fields without adding implicit recursion or writes.
