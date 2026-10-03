---
id: 16
uid: 990237fd-a08c-4130-8340-a73cd9997495
title: Save and apply versioned partial folder presets
type: feature
status: done
milestone: v0.2
assignee: Oddur Sigurdsson
depends_on:
- 12
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p1
effort: m
area: presets
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Define TOML presets as patches with platform-qualified fields. Include note/flags/basic access and native appearance only when supported. Preflight every requested setting; default to refusal on unsupported settings, with explicit skip policy if provided.

## Acceptance criteria

- [x] Saving and applying a preset leaves omitted properties unchanged.
- [x] Unsupported requested settings are explained before writes begin.
- [x] Project/archive/private examples document exactly what they change and round-trip through export/import.

## 2026-10-02

The preset engine depends on the completed mutation/change contract (0012). Final editing-release verification in0017 explicitly depends on native metadata, flags and permissions (0013-0015), presets (0016), and the inspector verification (0011). This allows independent core work while preserving release gates.

## 2026-10-02

Claimed assigned core-architect work after0012 completed. Preparatory0031 supplied shared API; final item evidence now being verified including presets examples and CLI round-trip behavior.

## 2026-10-02

Published Preset and strict schema1 TOML settings patches, explicit remove_note/remove_metadata, platform qualification, exclusive export, snapshot capture and native-backed preset preflight. Tests prove omitted mode/native flags remain unchanged, arbitrary binary metadata including binary notes roundtrips, unknown schema/fields reject, project/archive/private example presets parse and roundtrip. Samples document exact note-only or folder0700 changes. CLI engineer verified import/export roundtrip and note-only preset preservation of unrelated metadata; unsupported hidden setting refused before native reads/writes.

## Result

Versioned strict TOML partial presets preserve omitted settings and binary metadata/deletions, preflight every requested setting, and refuse unsupported platforms/settings before mutation. Added documented project/archive note-only and private0700 examples with roundtrip tests; CLI import/export/save/apply uses shared engine.
