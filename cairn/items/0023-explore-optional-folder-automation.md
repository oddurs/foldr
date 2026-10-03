---
id: 23
uid: 915d243d-e4fa-4609-9e3c-53485638c098
title: Explore optional folder automation
type: feature
status: backlog
milestone: later
depends_on:
- 17
created: 2026-10-02
updated: 2026-10-02
priority: p3
effort: l
area: automation
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Investigate watcher-backed rules or macOS Folder Actions. Keep the CLI useful without a daemon and expose lifecycle, readiness, retries and partial-file handling if a service is introduced.

## Acceptance criteria

- [ ] The proposal explains service ownership and how users disable or remove rules.
- [ ] Event loss, duplicate events and files still being written have defined handling.
- [ ] No automation starts merely because folder metadata is inspected.
