---
id: 25
uid: 289295e9-e0c7-4dbf-8efa-b60e335a50d9
title: Evaluate an optional TUI or desktop interface
type: feature
status: backlog
milestone: later
depends_on:
- 17
created: 2026-10-02
updated: 2026-10-02
priority: p3
effort: m
area: interface
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Only if useful after CLI feedback, prototype a thin interface over foldr-core. Consider a Rust TUI first; compare Iced, native macOS UI, or C++/Qt only if a graphical workflow becomes necessary. Do not introduce GUI dependencies into the core or CLI.

## Acceptance criteria

- [ ] The prototype solves a demonstrated usability problem and has a recorded go/no-go decision.
- [ ] It calls the same inspection, change planning and recovery operations as the CLI.
- [ ] The CLI can still build and run without graphical or TUI dependencies.
