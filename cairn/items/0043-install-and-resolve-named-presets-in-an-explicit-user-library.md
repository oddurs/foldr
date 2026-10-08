---
id: 43
uid: f47f11fd-1e56-4dfc-a75c-e2a944a5cba7
title: Install and resolve named presets in an explicit user library
type: feature
status: done
milestone: release-0.2.0
assignee: cli-engineer
depends_on:
- 42
created: 2026-10-07
updated: 2026-10-07
closed_at: 2026-10-07
priority: p1
effort: m
area: presets
---

## Problem

Current presets require users to remember individual TOML file paths. A named library should reuse the existing versioned files without turning folders into automatically executed configuration.

## Proposal

Add preset list and preset install FILE --name NAME, and named-source resolution for apply/check. Keep file-path commands intact. Use an explicit --preset-dir override, macOS ~/Library/Application Support/foldr/presets and Linux $XDG_CONFIG_HOME/foldr/presets or ~/.config/foldr/presets. The library stores existing TOML presets, independent of recovery state. Install by exclusive creation; a duplicate name refuses until an explicit replacement policy is designed.

## Acceptance criteria

- [x] Install/list and file-vs-name source selection match0042; shipped examples can be installed and reused, and existing file-based apply still works.
- [x] Names reject traversal/separators/NUL and unsafe paths; corrupt presets, duplicates and symlink substitutions return actionable errors without overwriting user files.
- [x] Binary values, omitted settings and explicit removals round-trip; listing/resolution create no folder metadata or recovery records and missing libraries read as empty.
- [x] macOS/Linux temporary-library integration checks cover deterministic precedence, exclusive installation and unsupported platform settings being refused by existing preflight.

## 2026-10-07

Implemented explicit library resolution with --preset-dir precedence; Linux uses absolute XDG_CONFIG_HOME else HOME/.config, macOS uses HOME/Library/Application Support. Names are ASCII alphanumeric/_/- max64. Every library component is opened relative to pinned descriptors with no-follow search-only flags; no canonicalization bypass. Installs validate regular non-symlink source before creating private directories and use O_EXCL mode0600. Missing library list is empty and creates no state. Named apply treats all positionals as targets; unnamed apply retains first FILE then targets, including directory names ending .toml. Five portable integration scenarios pass local native macOS with Rust1.85.0, including concurrent exclusive install, partial/binary/removal roundtrip, deterministic/default precedence, corrupt/duplicate/symlink/nonregular refusal, and foreign-platform preflight; Linux native execution remains release gate evidence.

## Result

Explicit named TOML library implemented with pinned no-follow components, exclusive installs and deterministic source resolution; five portable native integration fixtures pass on macOS Rust1.85.0. Native Linux execution remains mandatory0049 release gate, not yet verified.

## 2026-10-07

Final full CLI2unit+24integration suite and Clippy --all-targets -D warnings passed macOS exactRust1.85.0 and stable; no implicit env/current-folder lookup and installed-name overrides are documented by generated command help.

## 2026-10-07

Final native release gate passed on Ubuntu and macOS, stable and Rust 1.85, including library path and refusal checks (CI 37715685128 and Packages 37715685115).
