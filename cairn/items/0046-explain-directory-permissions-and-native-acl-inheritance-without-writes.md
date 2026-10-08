---
id: 46
uid: 15374927-76b3-43d6-a458-b92f1be4b483
title: Explain directory permissions and native ACL inheritance without writes
type: feature
status: done
milestone: release-0.2.0
assignee: platform-engineer
depends_on:
- 42
created: 2026-10-07
updated: 2026-10-07
closed_at: 2026-10-07
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

- [x] Examples distinguish list/search/entry modification, sticky/setgid and newly created versus existing or moved-in children, with platform-specific inheritance explanations.
- [x] Structured native ACL data retains principals, rights, ordering and scope; unknown flags, malformed entries and denied/unavailable properties remain explicit rather than being guessed.
- [x] Human and versioned JSON explanations escape untrusted names and clearly qualify effective-access uncertainty; existing ACL-sensitive mode-edit refusal remains intact.
- [x] Native macOS/Linux ACL fixtures verify explanations and preservation of metadata, contents, timestamps and recovery state; unsupported fixture cases are recorded.

## 2026-10-07

Implemented platform::permissions::explain separate schema1 result preserving published snapshot1 ACL fields. Owner/group/other list/search/write+search interpretation, sticky/setgid observed bits, read-only mount and immutable observations, uncertainty and existing/moved/new child distinctions are explicit. Linux ACL external ABI retains numeric user/group principals, access/default scopes, native order and per-scope mask intersections; malformed headers/rights/principals/base entries/duplicate principals/missing masks are unavailable. macOS uses installed SDK sys/acl.h acl_copy_ext_native and documented sys/kauth.h layout; ordered UUID principals, allow/deny, directory rights, full raw flags/rights and unknown masks survive. Unknown dispositions are explicit. No production ACL setters were added.

## 2026-10-07

Evidence: cargo test platform17 passed native macOS ordered allow/deny + actual child inheritance, unknown flags/rights/malformed external-layout parser, Linux mask parser fixture, read-only metadata/mode/mtime/ctime/xattrs/child contents preservation and existing ACL-sensitive mode-edit refusal. CLI history_permissions2 passed schema1 JSON and human control-safe path presentation with no recovery-state creation. Exact Rust1.85 core all-target Clippy passes; default-toolchain Linux all-target cross-check passes. Requested setgid on macOS fixture is silently cleared by this runtime even after assigning own group; test accurately reports observed bits, Linux fixture requires requested setgid. Native Linux ACL/default/mask fixture is present but awaits real Linux CI runtime; do not tick criterion4 until that executes.

## 2026-10-07

Final candidate f66d6c0 passed all four real native macOS/Linux stable/Rust1.85 jobs in https://github.com/oddurs/foldr/actions/runs/37715685128. Unit/CLI native ACL fixtures, permission explanations, read-only timestamps/attrs/state and old/new independent workflows passed. Linux default ACL/mask and observed setgid cases execute in native test matrix, with unsupported filesystem fixture handling explicit; Mac ordered ACL principal/rights/inheritance behavior and extended-ACL mode refusal verified. No complete effective-access oracle is claimed.

## Result

Implemented read-only mode/native ACL explanations with principal/rights/order/masks/inheritance and explicit uncertainty. Real Linux/macOS stable/MSRV fixtures and independent workflows pass without metadata/state changes; unknown and unsupported details remain explicit.
