---
id: 16
uid: 990237fd-a08c-4130-8340-a73cd9997495
title: Save and apply versioned partial folder presets
type: feature
status: planned
milestone: v0.2
depends_on:
- 13
- 14
- 15
created: 2026-10-02
updated: 2026-10-02
priority: p1
effort: m
area: presets
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Define TOML presets as patches with platform-qualified fields. Include note/flags/basic access and native appearance only when supported. Preflight every requested setting; default to refusal on unsupported settings, with explicit skip policy if provided.

## Acceptance criteria

- [ ] Saving and applying a preset leaves omitted properties unchanged.
- [ ] Unsupported requested settings are explained before writes begin.
- [ ] Project/archive/private examples document exactly what they change and round-trip through export/import.
