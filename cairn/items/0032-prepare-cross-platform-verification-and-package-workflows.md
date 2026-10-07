---
id: 32
uid: 4f88157f-8d66-49fc-a7ff-db984f42b0fd
title: Prepare cross-platform verification and package workflows
type: chore
status: done
milestone: v0.1
depends_on:
- 6
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
area: verification
part_of:
- 11
- 17
- 20
---

## Context

The integration lead can prepare independent release checks while implementation proceeds. Full inspector/editing/package verification remains in0011,0017,0020.

## Proposal

Define a real-filesystem end-to-end matrix for native descriptors, odd filenames, binary metadata, dry-run, partial writes, restricted access and undo conflicts. Prepare CLI packaging workflows and documentation interfaces without marking future runtime behavior verified.

## Acceptance criteria

- [x] The verification matrix and ownership are recorded.
- [x] Cross-platform CI and artifact packaging are configured for selected targets.
- [x] Verification interfaces are agreed with core and CLI engineers.

## 2026-10-02

Verification matrix: inspect/doctor must preserve folder metadata and report unsupported fields; preserve non-UTF-8 names, control characters and binary attributes; mutation dry-run changes neither folder nor history; notes/custom attributes preserve unrelated metadata; identity checks reject a replaced target; symlink mutations require explicit follow; restrictive mode/flag behavior has truthful recovery limits; partial writes persist recoverable records; undo refuses external modification; presets are partial and preflight unsupported fields; batch reports per-target failures and aggregate status; stdout JSON/completion/man must be pipe-safe. Root owns independent scripts/verify-cli.py plus CI/packaging; CLI engineer owns Rust CLI integration tests and command contract.

## 2026-10-02

Configured native package jobs for x86_64 Linux (Ubuntu22.04/glibc2.35), ARM64 macOS and Intel macOS. The package script creates tar.gz archives with binary, MIT license, README, bash/zsh/fish completions, a man page and SHA256 checksum. Workflows upload CI artifacts only, with contents:read; they do not publish a GitHub release. Root-owned end-to-end verifier will exercise unpacked native binaries. Official runner image labels verified against actions/runner-images.

## 2026-10-02

CLI engineer agreed generated-document interfaces: foldr completions SHELL and foldr man write to stdout. inspect/doctor use versioned JSON envelopes; note/attribute, flags/permissions, presets, undo, diff and batches share the core engine. Verification uses temporary folders and an explicit temporary state directory.

## Result

Prepared the real-filesystem end-to-end verification matrix and coordinated CLI contracts. Added native CI artifact packaging for ARM64/Intel macOS and x86_64 Linux, including man/completions/LICENSE/README/checksums. Runtime package and workflow verification remains gated by0011,0017,0020.
