---
id: 20
uid: f533cfe2-624a-4718-b439-e1a6091e53e0
title: Package the CLI and polish shell integration
type: chore
status: planned
milestone: v0.3
assignee: integration-lead
depends_on:
- 17
created: 2026-10-02
updated: 2026-10-02
priority: p1
effort: m
area: release
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Provide reproducible macOS and Linux release artifacts, installation instructions, completions and man/help documentation. Select concrete architectures, minimum versions and distribution coverage during implementation.

## Acceptance criteria

- [ ] A fresh supported macOS and Linux environment can install and run foldr.
- [ ] Completions and examples match the implemented command grammar.
- [ ] Release checks verify artifact identity, permissions and supported-target behavior.
