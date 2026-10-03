---
id: 6
uid: 07a13519-5fdc-44f2-a189-5a29d6b25ccf
title: Set up the Rust core and CLI workspace
type: chore
status: done
milestone: v0.1
assignee: codex
depends_on:
- 5
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
effort: s
area: core
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Create foldr-core and foldr-cli. Keep platform adapters behind cfg modules and the core independent of terminal rendering. Declare a toolchain/MSRV and supported OS targets.

## Acceptance criteria

- [x] The workspace builds on macOS and Linux.
- [x] Formatting, linting, and meaningful checks run in CI on both systems.
- [x] The CLI provides help/version and architecture boundaries are documented.

## 2026-10-02

Added the Cargo workspace, Rust 1.85 minimum, stable toolchain, isolated macOS/Linux adapter modules, clap help/version entry points, and CI matrix for both operating systems and toolchains. Local formatting, clippy, build, test harness and CLI smoke checks passed on macOS. Cross-platform criteria await the first GitHub CI run.

## 2026-10-02

Verification complete: GitHub CI run https://github.com/oddurs/foldr/actions/runs/37090815971 passed all four jobs (macOS/Linux, stable/1.85.0). Each job checked formatting, clippy with denied warnings, workspace build, test harness, CLI help/version/invalid-option behavior, and rustdoc with denied warnings. The scaffold contains no filesystem operations yet.

## Result

Created foldr-core and foldr-cli with a shared Rust 2024 workspace, Rust 1.85 minimum, stable development toolchain, native adapter module boundaries, and clap help/version entry points. GitHub CI passed on macOS and Linux with stable Rust and Rust 1.85.0. Core models and filesystem behavior remain scheduled in items 0007 onward.
