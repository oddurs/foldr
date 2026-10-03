---
id: 8
uid: 080053c7-f242-49e6-9d38-bb3521a15dfa
title: Inspect macOS folder metadata and native flags
type: feature
status: planned
milestone: v0.1
depends_on:
- 7
created: 2026-10-02
updated: 2026-10-02
priority: p1
effort: m
area: macos
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Use native Unix/Foundation APIs as needed to read ownership, permissions, ACL summaries, xattrs, flags, Finder tags/icon presence, and filesystem information. No read operation may mutate metadata.

## Acceptance criteria

- [ ] Inspection reports user-owned APFS folders accurately without altering them.
- [ ] Restricted or unsupported properties have actionable explanations.
- [ ] Folder locking and Finder hiding are described independently of child content access.
