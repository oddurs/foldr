---
id: 10
uid: 3c3d26ed-b03d-4b88-98bf-eeef5e51b8b4
title: Deliver inspect and doctor commands
type: feature
status: done
milestone: v0.1
assignee: cli-engineer
depends_on:
- 8
- 9
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p1
effort: m
area: cli
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Finalize a consistent CLI grammar. Human output prioritizes useful properties; JSON is versioned and machine-readable. Doctor explains limitations and recovery steps. Avoid recursive size/count scans by default.

## Acceptance criteria

- [x] Inspect and doctor work using either OS adapter.
- [x] JSON output does not mix diagnostics into stdout and preserves encoded data.
- [x] Exit statuses, escaping, color handling and pipe behavior are documented.

## 2026-10-02

Implemented inspect and doctor through the shared macOS/Linux native adapters. Human inspection shows octal modes, identity, ACLs, flags, escaped path names and text/hex xattrs without recursive scans. Doctor describes per-property read/write capability states, folder locking scope and metadata portability. Global JSON uses schema_version/command/data; arbitrary path/xattr bytes remain lossless; diagnostics use stderr only; closed stdout pipes exit cleanly. Help/man document no color, explicit scopes, escaping and exits0–5. Ten integration checks and the escaping unit test passed on macOS, and native Ubuntu+macOS stable CI jobs succeeded at ef2a449: https://github.com/oddurs/foldr/actions/runs/37092594760 . Native unpacked-binary end-to-end checks passed for all three package targets: https://github.com/oddurs/foldr/actions/runs/37092594808 .

## 2026-10-02

Validation boundary: initial aggregate CI was not wholly green because Rust1.85 Clippy flagged format_collect in core comparison code. The core role repaired that lint and added descriptor traversal fixes; integration lead is running the next checkpoint. Those complete-workspace gates remain0011/0017, while this CLI item closes on successful native stable runtime and command/output evidence.

## Result

Delivered native-backed inspect and doctor on macOS and Linux, readable octal/text/hex output, versioned byte-preserving JSON and separate diagnostic channels. Commands document scopes, escaping, no-color behavior, exits and clean broken pipes. Native stable CI and all packaged executable end-to-end checks passed; full MSRV/release gates remain integration items.
