---
id: 52
uid: 08d9fb94-cb26-4d26-a94c-3d17cf6945ce
title: Prepare version 0.2.0 for native release verification
type: chore
status: done
milestone: release-0.2.0
assignee: integration-lead
depends_on:
- 42
created: 2026-10-07
updated: 2026-10-07
closed_at: 2026-10-07
priority: p0
part_of:
- 49
area: release
---

## Proposal

After local feature integration is complete, align the workspace/lockfile/CI version with0.2.0 so native verification and archives execute the actual release candidate. This preparatory item lets native evidence satisfy0046/0048 before finalization0049 is claimed. Public tagging and publication remain0049.

## Acceptance criteria

- [x] Workspace/lockfile/CI version agree and the built executable reports0.2.0.
- [x] Local stable/MSRV integration and independent old/new filesystem workflows pass before native CI submission.

## 2026-10-07

Release-candidate source is now0.2.0 in workspace, both lockfile packages and CLI CI entrypoint. Root aggregate passed stable and exactRust1.85.0 formatting, workspace denied-warningClippy and80 tests per toolchain (51core,3actualv0.1 compatibility,2CLIunit,24CLIintegration). Stable build, original v0.1 walkthrough and independent v0.2 workflow passed; rustdoc deniedwarnings and actionlint/shell/Python script checks pass. Core/CLI/native ownership is frozen; actual Linux execution remains next native CI gate.

## Result

Prepared and locally verified actual v0.2.0 source/lock/CI version and binary. Stable/MSRV80-test workspace checks and independent old/new native workflows pass; candidate can proceed to native release validation.
