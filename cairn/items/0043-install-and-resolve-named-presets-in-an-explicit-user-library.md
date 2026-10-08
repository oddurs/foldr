---
id: 43
uid: f47f11fd-1e56-4dfc-a75c-e2a944a5cba7
title: Install and resolve named presets in an explicit user library
type: feature
status: planned
milestone: release-0.2.0
assignee: cli-engineer
depends_on:
- 42
created: 2026-10-07
updated: 2026-10-07
priority: p1
effort: m
area: presets
---

## Problem

Current presets require users to remember individual TOML file paths. A named library should reuse the existing versioned files without turning folders into automatically executed configuration.

## Proposal

Add preset list and preset install FILE --name NAME, and named-source resolution for apply/check. Keep file-path commands intact. Use an explicit --preset-dir override, macOS ~/Library/Application Support/foldr/presets and Linux $XDG_CONFIG_HOME/foldr/presets or ~/.config/foldr/presets. The library stores existing TOML presets, independent of recovery state. Install by exclusive creation; a duplicate name refuses until an explicit replacement policy is designed.

## Acceptance criteria

- [ ] Install/list and file-vs-name source selection match0042; shipped examples can be installed and reused, and existing file-based apply still works.
- [ ] Names reject traversal/separators/NUL and unsafe paths; corrupt presets, duplicates and symlink substitutions return actionable errors without overwriting user files.
- [ ] Binary values, omitted settings and explicit removals round-trip; listing/resolution create no folder metadata or recovery records and missing libraries read as empty.
- [ ] macOS/Linux temporary-library integration checks cover deterministic precedence, exclusive installation and unsupported platform settings being refused by existing preflight.
