---
id: 8
uid: 080053c7-f242-49e6-9d38-bb3521a15dfa
title: Inspect macOS folder metadata and native flags
type: feature
status: done
milestone: v0.1
assignee: platform-engineer
depends_on:
- 7
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p1
effort: m
area: macos
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Use native Unix/Foundation APIs as needed to read ownership, permissions, ACL summaries, xattrs, flags, Finder tags/icon presence, and filesystem information. No read operation may mutate metadata.

## Acceptance criteria

- [x] Inspection reports user-owned APFS folders accurately without altering them.
- [x] Restricted or unsupported properties have actionable explanations.
- [x] Folder locking and Finder hiding are described independently of child content access.

## 2026-10-02

Implemented descriptor fstatfs with actual APFS name/read-only status, st_flags/fchflags, binary xattrs, native ACL allow/deny/inheritance summary and Finder tag/custom icon metadata markers. Six platform tests pass on native macOS, including hidden toggle preservation, non-mutating inspection, binary/empty xattrs, held-descriptor path replacement and status classification. macOS rejects invalidUTF8 xattr names and APFS rejects invalidUTF8 filenames; raw encoding remains lossless. Write capabilities remain unknown without write probes, read-only mounts unavailable. Locking describes entry protection and Finder visibility independently.

## Result

Native macOS APFS inspector and descriptor field reads/writes implemented; six focused native tests pass. Property failures have separate statuses and native hiding/locking scopes are explained.
