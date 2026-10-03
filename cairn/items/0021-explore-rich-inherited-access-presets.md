---
id: 21
uid: b9a2db56-f096-45f5-8f59-b99af1d00a37
title: Explore rich inherited-access presets
type: feature
status: backlog
milestone: later
depends_on:
- 17
created: 2026-10-02
updated: 2026-10-02
priority: p2
effort: m
area: permissions
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Investigate separate macOS ACL and Linux default-ACL adapters. Explain ACL masks, inheritance and existing/new/moved child behavior without presenting the platforms as identical.

## Acceptance criteria

- [ ] A supported semantics matrix and representative fixtures are recorded.
- [ ] Proposed changes show effective access and scope before writes.
- [ ] Implementation only proceeds through the shared change/recovery engine.
