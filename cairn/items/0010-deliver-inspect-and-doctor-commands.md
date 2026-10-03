---
id: 10
uid: 3c3d26ed-b03d-4b88-98bf-eeef5e51b8b4
title: Deliver inspect and doctor commands
type: feature
status: planned
milestone: v0.1
assignee: cli-engineer
depends_on:
- 8
- 9
created: 2026-10-02
updated: 2026-10-02
priority: p1
effort: m
area: cli
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Finalize a consistent CLI grammar. Human output prioritizes useful properties; JSON is versioned and machine-readable. Doctor explains limitations and recovery steps. Avoid recursive size/count scans by default.

## Acceptance criteria

- [ ] Inspect and doctor work using either OS adapter.
- [ ] JSON output does not mix diagnostics into stdout and preserves encoded data.
- [ ] Exit statuses, escaping, color handling and pipe behavior are documented.
