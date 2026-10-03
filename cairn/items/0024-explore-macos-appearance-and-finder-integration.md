---
id: 24
uid: 09b8fcaf-0e6f-4203-aa6a-1b264202db6a
title: Explore macOS appearance and Finder integration
type: feature
status: backlog
milestone: later
depends_on:
- 17
created: 2026-10-02
updated: 2026-10-02
priority: p2
effort: m
area: macos
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Use native APIs to edit tags/custom icons while preserving unrelated Finder metadata. Consider an Open in foldr action that launches the CLI or an optional interface. A custom document package type remains a separate experiment.

## Acceptance criteria

- [ ] Appearance changes have verified preservation and recovery behavior.
- [ ] Terminal-only use remains fully supported.
- [ ] Finder/package experiments clearly document their platform-specific effects.
