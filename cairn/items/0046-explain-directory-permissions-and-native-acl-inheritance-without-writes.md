---
id: 46
uid: 15374927-76b3-43d6-a458-b92f1be4b483
title: Explain directory permissions and native ACL inheritance without writes
type: feature
status: planned
milestone: release-0.2.0
assignee: platform-engineer
depends_on:
- 42
created: 2026-10-07
updated: 2026-10-07
priority: p1
effort: m
area: permissions
---

## Problem

Mode bits and abbreviated ACL entries do not explain why listing, traversal, entry deletion and new-child permissions behave differently. macOS and Linux ACL semantics must remain distinct.

## Proposal

Add permissions explain PATH with owner/group/other mode interpretation, sticky/setgid scope, read-only mount/immutable observations and decoded native ACL details. Linux explanations cover access vs default ACLs, mask effects and interaction with requested creation mode. macOS summaries include native principal, allow/deny rights and inheritance flags. Explain observed rules and remaining uncertainty; do not claim a complete effective-access oracle for arbitrary users or security modules.

Primary reference for Linux inheritance/masks: https://man7.org/linux/man-pages/man5/acl.5.html. Verify Apple ACL declarations against the installed SDK and native temporary fixtures. This is read-only scope extracted from broader exploratory0021; native ACL edits remain later work.

## Acceptance criteria

- [ ] Examples distinguish list/search/entry modification, sticky/setgid and newly created versus existing or moved-in children, with platform-specific inheritance explanations.
- [ ] Structured native ACL data retains principals, rights, ordering and scope; unknown flags, malformed entries and denied/unavailable properties remain explicit rather than being guessed.
- [ ] Human and versioned JSON explanations escape untrusted names and clearly qualify effective-access uncertainty; existing ACL-sensitive mode-edit refusal remains intact.
- [ ] Native macOS/Linux ACL fixtures verify explanations and preservation of metadata, contents, timestamps and recovery state; unsupported fixture cases are recorded.
