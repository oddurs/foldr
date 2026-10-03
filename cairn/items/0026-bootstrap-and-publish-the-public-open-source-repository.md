---
id: 26
uid: 773438fd-d2e5-432c-b435-87e78124581c
title: Bootstrap and publish the public open-source repository
type: chore
status: doing
milestone: v0.1
created: 2026-10-02
updated: 2026-10-02
priority: p0
area: repository
---

## Context

The user requested Git initialization and a public oddurs/foldr repository, and selected the MIT license.

## Proposal

Initialize the main branch, add the MIT license and contributor documentation, publish the Cairn plan and Rust starter workspace, and verify the public repository and CI.

## Acceptance criteria

- [ ] Git is initialized on main with a clean, committed starter project.
- [x] MIT licensing, README, contributor guidance, and build/CI configuration are present.
- [ ] The public oddurs/foldr repository exists and its main branch matches the local commit.
- [ ] Local checks and the initial macOS/Linux CI complete successfully.
