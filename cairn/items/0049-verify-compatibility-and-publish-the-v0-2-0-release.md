---
id: 49
uid: 5c20539c-eaf3-4d73-87d7-fe8a1e6b96e7
title: Verify compatibility and publish the v0.2.0 release
type: chore
status: done
milestone: release-0.2.0
assignee: integration-lead
depends_on:
- 43
- 44
- 45
- 46
- 47
- 48
created: 2026-10-07
updated: 2026-10-07
closed_at: 2026-10-07
priority: p0
effort: m
area: release
---

## Problem

New workflows must be a tested release that existing users can install and recover with, not just a collection of merged features.

## Proposal

Run the existing macOS/Linux stable and Rust1.85 matrix, native archive checks and independent real-filesystem workflows extended to named profiles, drift checks, permission explanations and filtered history. Include Finder tags only after the native go decision and implementation gates pass. Keep the tested three targets and platform baselines unless a separately recorded decision changes them. Update version to0.2.0 only at the release gate, test installed binaries and publish notes/checksums/assets from the matching source.

## Acceptance criteria

- [x] Mandatory children pass native regression and v0.1 compatibility fixtures; read-only commands and dry-run preserve metadata/state, and invalid schemas or unsupported settings refuse clearly.
- [x] Finder decision is reflected truthfully in milestone, dependency graph, CLI docs and presets; a no-go removes0048 from the release dependency set and leaves it deferred rather than done.
- [x] README walkthrough and generated shell/manual docs cover implemented commands; all three extracted native archives pass executable/version/checksum/real-workflow checks.
- [x] Release commit, version0.2.0, tag and uploaded artifact digests agree; notes state supported baselines, unsigned build status and recovery/concurrency limitations, and Cairn is validated before the milestone closes.

## 2026-10-07

Release candidate f66d6c0fb3e1d1384c01db144020472922e8cbd0 passed all four native stable/Rust 1.85 CI jobs (run 37715685128) and all three extracted Linux/Intel Mac/ARM Mac package workflow jobs (run 37715685115). Both independent filesystem verifiers passed; 80 tests per local toolchain and actual v0.1.0 fixtures passed. Finder research is GO with descriptor-held raw tag bytes and a native reader oracle; docs record its supported and refused cases.

## 2026-10-07

Published https://github.com/oddurs/foldr/releases/tag/v0.2.0 from merged PR #3, commit 92955690e05edb721b33d414909097a85fcc2908. Its tree exactly equals tested f66d6c0. All three archives independently passed path/executable/LICENSE/manual/completion/checksum checks; all six GitHub-uploaded asset sizes and SHA256 digests matched local artifacts before and after publication. Tag resolves to the release commit. Notes state macOS 13+, glibc 2.35+, unsigned Mac builds, compatibility and recovery/concurrency limits.

## Result

v0.2.0 is publicly released at https://github.com/oddurs/foldr/releases/tag/v0.2.0, tagged at verified merge 92955690e05edb721b33d414909097a85fcc2908. Four stable/MSRV native CI jobs and three native package jobs passed. The three native archives and all six published SHA256 digests are independently verified; 80 tests per local toolchain, actual v0.1 fixtures and both independent real-filesystem workflows passed. Release compatibility and platform/concurrency limits are documented.
