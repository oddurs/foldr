---
id: 27
uid: efd59519-3cca-4f99-9a3f-2b471e740453
title: Coordinate the implementation team and integrate CLI milestones
type: chore
status: done
milestone: v0.3
assignee: integration-lead
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
area: integration
---

## Context

The user explicitly requested a team organized by roles to implement foldr. Four concurrent roles are available: project lead/integration (root), core architect, platform engineer, and CLI engineer.

## Proposal

Implement the planned v0.1-v0.3 CLI milestones with shared API contracts and disjoint file ownership. Core architect owns model/engine/presets; platform engineer owns native adapters; CLI engineer owns command/rendering code; root owns integration checks, CI, packaging and contributor documentation. Agents must claim Cairn items, record evidence, preserve other agents changes, and avoid committing or pushing shared work. Publish a reviewable branch and pull request after checks. Optional GUI/TUI, watcher services, and advanced filesystem policies remain exploratory backlog work.

## Acceptance criteria

- [x] Role ownership and a shared core/platform/CLI API contract are recorded.
- [x] Planned CLI functionality is integrated and verified on macOS and Linux.
- [x] Cairn reflects verified outcomes and any concrete limitations.
- [x] A reviewable implementation branch and pull request are published with passing checks.

## 2026-10-02

Org/API agreement: root owns integration/verification, CI, packaging and root docs; core-architect owns typed model/inspect/change/preset/diff/batch engine and core manifest; platform-engineer owns platform/** native adapters; cli-engineer owns CLI grammar/rendering/commands and CLI integration tests. The core holds an open directory File and validates symlink/identity; native adapter exposes inspect_native(&File)->NativeInspection, read/write_xattr(&File, raw name, optional bytes), read/write_flags(&File,u64). JSON preserves arbitrary paths/values as byte arrays with escaped display. Native writes never parse shell utility output. Preparatory items permit independent API work; release validation remains separate.

## 2026-10-02

First complete local integration run: formatting/clippy,19 core tests,1 output unit test,9 CLI integration tests and scripts/verify-cli.py passed. The independent workflow covered read-only inspect/doctor, terminal-safe path bytes, notes/binary attributes, exact no-write dry-run, recovery and undo conflict refusal, explicit symlink follow, permissions, native hidden flags, partial presets/batches, generated shell/manual docs and clean broken pipes. Additional native tests are being added; Linux runtime remains a required CI gate.

## 2026-10-02

Native Apple Silicon package built, checksum verified, extracted contents reviewed, and packaged binary passed the independent end-to-end verifier. Packaging review removed ambient macOS AppleDouble metadata from archives and actionlint caught/fixed checksum glob quoting. Both workflows now pass actionlint; shell/Python verification scripts parse cleanly. Core audit is fixing unknown recovery schemas, malformed/duplicate plan preflight, and syncing newly created journal parents before publication.

## 2026-10-02

Checkpoint ef2a449 published as draft PR https://github.com/oddurs/foldr/pull/1. Native stable CI passed macOS and Linux (run37092594760), and all3 native package jobs passed (run37092594808), including checksum, extractedbinary, man/completions, and independent end-to-end workflows. Rust1.85 Clippy found only two format_collect sites in corecompare; these are being fixed together with reviewer-reproduced execute-only parent directory traversal. No platform runtime failure occurred.

## 2026-10-02

Final source fixes are integrated: descriptor traversal now uses search-only ancestors while final folder descriptors remain readable; symlink ancestors still refuse. Core and CLI hexadecimal formatting now pass exact Rust1.85.0 denied-warning Clippy. Root aggregate stable/MSRV formatting, full workspace lint and47 macOS tests pass; stable build, standalone real-filesystem walkthrough and denied-warning rustdoc pass. Both workflows pass actionlint. Remote native CI/package rerun will verify this source revision before release gates close.

## 2026-10-02

Team delivery is complete. Native source validation at a4f147c is green across all four macOS/Linux stable/MSRV jobs (CI37093137863), and all three native archives passed extracted-binary workflows/checksums (Packages37093137858). Inspector/edit/recovery/package gates0011/0017/0020 are closed with evidence. Public https://github.com/oddurs/foldr/pull/1 is ready for review. All planned v0.1-v0.3 CLI deliverables are implemented; optional UI/watchers/rich ACL and filesystem experiments remain explicitly later backlog. Native support, privilege limits, recovery race/identity caveats and unsigned artifacts are documented. No merge or public release was performed.

## Result

Four-role team delivered the complete planned Rust CLI with native macOS/Linux adapters, verified mutation/recovery, presets/diff/batches and native packages. All stable/MSRV and native archive checks passed; PR1 is ready for review, Cairn records evidence, and exploratory work remains backlog.
