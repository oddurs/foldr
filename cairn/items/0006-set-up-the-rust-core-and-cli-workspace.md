---
id: 6
uid: 07a13519-5fdc-44f2-a189-5a29d6b25ccf
title: Set up the Rust core and CLI workspace
type: chore
status: doing
milestone: v0.1
assignee: codex
claimed: 2026-10-02
depends_on:
- 5
created: 2026-10-02
updated: 2026-10-02
priority: p0
effort: s
area: core
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Create foldr-core and foldr-cli. Keep platform adapters behind cfg modules and the core independent of terminal rendering. Declare a toolchain/MSRV and supported OS targets.

## Acceptance criteria

- [ ] The workspace builds on macOS and Linux.
- [ ] Formatting, linting, and meaningful checks run in CI on both systems.
- [x] The CLI provides help/version and architecture boundaries are documented.

## 2026-10-02

Added the Cargo workspace, Rust 1.85 minimum, stable toolchain, isolated macOS/Linux adapter modules, clap help/version entry points, and CI matrix for both operating systems and toolchains. Local formatting, clippy, build, test harness and CLI smoke checks passed on macOS. Cross-platform criteria await the first GitHub CI run.
