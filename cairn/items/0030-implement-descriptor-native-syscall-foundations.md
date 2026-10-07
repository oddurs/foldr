---
id: 30
uid: 26a82129-447e-4b92-b6d8-3aa7ac135e9e
title: Implement descriptor-native syscall foundations
type: chore
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
area: platform
part_of:
- 8
- 9
---

## 2026-10-02

Implement raw io::Result descriptor syscall wrappers and binary decoding while core models are finalized. Acceptance: native wrappers preserve byte xattrs; descriptor-only operations use native APIs; no write probes in reads. Higher-level model adapters remain0008/0009.

## Result

Descriptor-only native foundations implemented for macOS and Linux. macOS tests pass; Linux runtime validation remains0009 and will run in native CI.

## 2026-10-02

Reopened to finish Linux native syscall and ACL decoding foundation while0007 checks finalize;0008/0009 verification remains separate.
