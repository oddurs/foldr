---
id: 42
uid: 4ec02ca4-92ec-456c-80a1-9ad58f6c545d
title: Define v0.2.0 compatibility, command grammar and schema contracts
type: docs
status: planned
milestone: release-0.2.0
assignee: core-architect
created: 2026-10-07
updated: 2026-10-07
priority: p0
effort: s
area: contracts
---

## Problem

New named sources, structured ACL summaries and native tag edits touch contracts already published in v0.1.0. Decide these before parallel implementation rather than letting each command invent a format.

## Proposal

Record command argument groups, read-only compliance statuses, library paths and named-source precedence. Keep existing file-based commands working. Proposed preset check returns0 for compliant,6 for definite drift, and existing error categories for unsupported/indeterminate/operational failures; error precedence and mixed-target JSON are decided explicitly. Existing diff remains informational unless a check option is explicitly requested.

Define reader/writer compatibility for preset, snapshot, JSON-envelope and recovery versions separately. Load real v0.1 fixtures; a new field/type requiring a schema bump gets a migration or clear refusal, never silent loss. Typed Finder metadata is a narrowly authorized native field; do not weaken generic foldr-owned-attribute validation. No new promise of filesystem CAS or atomic multi-field changes.

## Acceptance criteria

- [ ] A command/schema compatibility table includes old commands, file-vs-name conflicts, drift exit6, unknown-version handling and per-target error precedence.
- [ ] Actual v0.1.0 preset/snapshot/recovery fixtures are captured with binary values and omission/removal semantics; required readers preserve them or explicitly document a reviewed migration.
- [ ] Core/platform/CLI reviewers agree on native field authorization, JSON capability states and symlink/identity rules before dependent features start.
