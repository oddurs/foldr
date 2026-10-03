---
id: 36
uid: f0fe4318-486f-4a49-ad2e-4eb81785613f
title: Keep hexadecimal comparison keys compatible with Rust 1.85 Clippy
type: bug
status: done
milestone: v0.3
assignee: Oddur Sigurdsson
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
part_of:
- 18
- 20
area: core
effort: s
---

## What happens

Native stable CI passes, but Rust1.85 Clippy denies format_collect in hexadecimal xattr comparison keys.

## What should happen

Build lowercase hexadecimal keys with one preallocated string and direct hex digits, keeping structured field names identical and eliminating per-byte formatting allocations.

## Reproduction

1. Run cargo +1.85.0 clippy --workspace --all-targets -- -D warnings against the original formatter.

## Acceptance criteria

- [x] Binary attribute comparison field names remain unchanged.
- [x] Stable and Rust1.85 core Clippy and tests pass.

## 2026-10-02

Replaced per-byte format collection at both comparison key sites with one preallocated String and a shared lowercase hexadecimal digit helper. Existing binary comparison test now explicitly asserts unchanged xattr:ff field name. cargo test -p foldr-core and cargo +1.85.0 test -p foldr-core each pass36 tests; stable and185 Clippy -Dwarnings pass locally on Mac. Native Linux rerun remains root CI evidence.

## Result

Shared allocation-efficient hexadecimal attribute field encoding preserves exact structured keys and eliminates Rust1.85 format_collect warnings. Both stable and Rust1.85 core tests and strict Clippy pass.
