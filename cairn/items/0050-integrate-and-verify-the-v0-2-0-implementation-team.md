---
id: 50
uid: 32d00a4b-1448-4806-b89b-aa39edf76f9f
title: Integrate and verify the v0.2.0 implementation team
type: chore
status: doing
milestone: release-0.2.0
assignee: integration-lead
claimed: 2026-10-07
created: 2026-10-07
updated: 2026-10-07
priority: p0
area: integration
part_of:
- 49
---

## Proposal

Coordinate implementation of planned v0.2.0 using the user-authorized project team roles. Root owns independent black-box verification, root documentation, workflows and final release; core, CLI and native adapters have disjoint owners. Publish reviewable implementation, finish native CI/packages, merge and release only matching verified source/assets.

## Acceptance criteria

- [x] Shared contracts and file ownership are followed and implementation meets the planned release scope.
- [ ] Independent v0.1/v0.2 real-filesystem checks and native stable/MSRV/package jobs pass.
- [ ] Verified implementation is merged, release artifacts are published and Cairn records the actual outcomes.

## 2026-10-07

User authorized complete implementation, merge and public release. Merged the all-green planning PR2 as75d7c6a. Writable temporary checkout is /private/tmp/foldr-v020.TKxybZ with branch feat/v0.2.0; original .git is protected. Team ownership: core_v020 core/contracts/recovery except native modules; platform_v020 native ACL/tags; cli_v020 all command/render/test surfaces; integration lead root docs/workflows/independent checks/version/package/release. Shared core model/lib interfaces coordinated by core; no agent commits/pushes. Original workspace hashes captured so later source synchronization preserves user edits.

## 2026-10-07

Independent v0.2 verifier is prepared: named preset install/list security, file-vs-name checks, stdout-only drift6, error-over-drift precedence, read-only timestamps/attrs/state, explanation/history reads, and tagged native binary plist/FinderInfo/color preservation plus exact undo and external-edit conflict. CI and native package workflows both execute the new verifier in addition to the original v0.1 walkthrough. Native tag research has a descriptor-based go design after proving the Foundation setter resets colors; implementation awaits completed engine/native gates.

## 2026-10-07

Local integration is complete and frozen: named library, drift checks, identity-filtered history, native permission explanations and GO Finder tags all pass independent real-filesystem workflow on macOS. Original v0.1 workflow and real old-version data fixtures pass. Native setters preserve raw colors/FinderInfo and use typed schema2 journaling, while unchanged schemas remain1. Preparing reviewable candidate with80 passed tests per local toolchain; Linux native/runtime package proof is pending.
