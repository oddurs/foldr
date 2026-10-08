---
id: 49
uid: 5c20539c-eaf3-4d73-87d7-fe8a1e6b96e7
title: Verify compatibility and publish the v0.2.0 release
type: chore
status: planned
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
priority: p0
effort: m
area: release
---

## Problem

New workflows must be a tested release that existing users can install and recover with, not just a collection of merged features.

## Proposal

Run the existing macOS/Linux stable and Rust1.85 matrix, native archive checks and independent real-filesystem workflows extended to named profiles, drift checks, permission explanations and filtered history. Include Finder tags only after the native go decision and implementation gates pass. Keep the tested three targets and platform baselines unless a separately recorded decision changes them. Update version to0.2.0 only at the release gate, test installed binaries and publish notes/checksums/assets from the matching source.

## Acceptance criteria

- [ ] Mandatory children pass native regression and v0.1 compatibility fixtures; read-only commands and dry-run preserve metadata/state, and invalid schemas or unsupported settings refuse clearly.
- [ ] Finder decision is reflected truthfully in milestone, dependency graph, CLI docs and presets; a no-go removes0048 from the release dependency set and leaves it deferred rather than done.
- [ ] README walkthrough and generated shell/manual docs cover implemented commands; all three extracted native archives pass executable/version/checksum/real-workflow checks.
- [ ] Release commit, version0.2.0, tag and uploaded artifact digests agree; notes state supported baselines, unsigned build status and recovery/concurrency limitations, and Cairn is validated before the milestone closes.
