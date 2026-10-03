---
id: 11
uid: 64b0f27a-415c-40bd-925e-1c723d4e7e17
title: Verify the read-only inspector release
type: chore
status: done
milestone: v0.1
assignee: integration-lead
depends_on:
- 10
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
effort: m
area: verification
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Add integration fixtures in temporary folders and a documented OS/filesystem check matrix. Test observable behavior, including no changes to flags/xattrs/content during inspection.

## Acceptance criteria

- [x] Checks cover unusual filenames, binary xattrs, symlinks and unreadable properties.
- [x] macOS and Linux checks pass and unsupported fixture cases are explicitly skipped.
- [x] A release checklist documents supported platforms, limitations and read-only guarantees.

## 2026-10-02

Release verification checklist: native macOS and Linux stable/MSRV jobs run formatting, denied-warning lint, builds, core/CLI tests, rustdoc and scripts/verify-cli.py. Inspection and doctor preserve mode/mtime/ctime/xattrs and create no state; test fixtures cover control-character paths, binary metadata, symlinks and denied searchable ancestors. Non-UTF-8 creation is skipped explicitly on filesystems that reject it; permission-denial assertions skip for root. Property-state tests cover permission_denied and unsupported mapping. Native packages must pass archive checksum, executable/docs presence and the same real-filesystem walkthrough after extraction. Supported targets: ARM64/Intel macOS 13+ and x86_64 Linux glibc 2.35+. No privileged Linux immutable-success claim; per-filesystem capability limits, extended-ACL mode refusal, symlink policy, recovery concurrency and unsigned development artifacts are documented in README. Final integration rerun is pending the parent-search and Rust1.85 lint repairs.

## 2026-10-02

Final source revision a4f147c passed all four native stable/Rust1.85.0 jobs on Linux and macOS: https://github.com/oddurs/foldr/actions/runs/37093137863. Each job ran the release checklist including read-only guarantees, filename/binary fixtures, denied-access/state mapping tests, symlink policy, CLI integration and the independent real-filesystem walkthrough. Local equivalent passed47 macOS tests. Supported/unsupported fixture limits are explicit in tests and README; checklist is recorded above.

## Result

Verified read-only inspector on native macOS and Linux with stable and Rust1.85.0. Metadata preservation, byte-safe output, unusual paths, symlink policy, capability states and searchable/denied ancestors are covered; supported targets and fixture limits are documented.
