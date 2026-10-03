---
id: 37
uid: 0f8391b8-db5e-4507-b10b-df4f1fcf75ed
title: Fix CLI hex formatting lint on Rust 1.85
type: bug
status: done
milestone: v0.3
assignee: cli-engineer
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
area: cli
---

## What happens

## What should happen

## Reproduction

1.

## Context

Rust 1.85 Clippy flags per-byte format-and-collect hex rendering in the CLI. Stable Clippy accepts the existing expressions, but the minimum supported compiler must also pass warning-denied checks.

## Acceptance criteria

- [x] CLI binary-value rendering uses one preallocated string without per-byte string allocation.
- [x] Exact Rust 1.85.0 and stable CLI all-target Clippy and CLI tests pass.

## 2026-10-02

Replaced both per-byte format!-then-collect expressions in output.rs with one shared hex_bytes helper that preallocates exactly two characters per byte and writes each byte directly into the string. No output or command-grammar changes. Exact cargo +1.85.0 clippy -p foldr-cli --all-targets -- -D warnings and stable equivalent passed; exact1.85.0 andstable cargo test -p foldr-cli both passed all ten integration checks and escaping unit test.

## Result

CLI hexadecimal rendering now shares a preallocated string writer compatible with minimum Rust1.85 Clippy. Exact1.85.0 andstable all-target CLI Clippy with denied warnings, and all11 CLI checks on both toolchains, pass.
