---
id: 40
uid: b47684f3-f2f9-464a-bbc2-0b447f1f6fea
title: Plan the v0.2.0 release after the first public CLI release
type: docs
status: done
milestone: release-0.2.0
assignee: integration-lead
created: 2026-10-07
updated: 2026-10-07
closed_at: 2026-10-07
priority: p0
area: planning
---

## Proposal

Define a bounded v0.2.0 release from published v0.1.0, preserving the Rust CLI-first architecture. Store the concept, prioritized deliverables, dependency graph, role ownership, acceptance criteria and release gates in Cairn. Historical bootstrap milestones must be distinguished from actual release versions.

## Acceptance criteria

- [x] The release outcome, scope and tradeoffs are recorded against the current released implementation.
- [x] Deliverables have concrete acceptance criteria, owners, effort, priorities and dependency ordering.
- [x] Cairn validates and a reviewable planning branch is published without implementing the features.

## 2026-10-07

Reviewed actual released CLI grammar, native adapters and current main d0b3900. Created actual release milestone0041 and eight planned implementation/research/release items0042–0049 with roles, relative effort, priorities and dependency graph. Historical bootstrap phases are labeled as shipped in v0.1.0. Source code and published package versions are untouched.

## 2026-10-07

Published reviewable planning PR https://github.com/oddurs/foldr/pull/2 on plan/v0.2.0. Strict/render/prompts validation passes49 items with0 warnings. Eight planned children remain unstarted; phase labels and existing exploratory boundaries are explicit. Only Cairn, generated roadmap and milestone guidance changed; no runtime features, package versions or release tags changed.

## Result

Planned actual release v0.2.0 in milestone0041 with eight prioritized, dependency-linked items0042–0049 and named owner roles. Scope covers named presets, read-only drift checks, history filtering and permission explanations; Finder tags require a native recovery/preservation go decision. Published planning PR2 and validated Cairn; implementation remains planned.
