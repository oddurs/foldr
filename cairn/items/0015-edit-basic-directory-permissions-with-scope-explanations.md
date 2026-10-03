---
id: 15
uid: 454d3252-9259-49e8-8b41-8ea69c40f01c
title: Edit basic directory permissions with scope explanations
type: feature
status: planned
milestone: v0.2
assignee: platform-engineer
depends_on:
- 12
created: 2026-10-02
updated: 2026-10-02
priority: p1
effort: m
area: permissions
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Edit basic mode bits using the shared engine and explain directory read/list, execute/traverse and write semantics. Include sticky and setgid descriptions where applicable. Detect interactions with existing ACLs; do not silently normalize them.

## Acceptance criteria

- [ ] Mode changes are previewed, verified and recoverable.
- [ ] Existing ACL interactions are shown or unsupported edits are rejected with a reason.
- [ ] Changes affect only the selected directory unless recursive work is explicitly requested.
