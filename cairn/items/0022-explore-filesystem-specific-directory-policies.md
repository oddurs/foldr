---
id: 22
uid: 5b5be32a-1386-4fdb-9183-ae3b2bc18b5a
title: Explore filesystem-specific directory policies
type: feature
status: backlog
milestone: later
depends_on:
- 17
created: 2026-10-02
updated: 2026-10-02
priority: p3
effort: l
area: filesystem
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Investigate Btrfs compression policies, ext4/F2FS casefold support, project quotas and fscrypt integrations separately. Treat filesystem configuration, empty-directory constraints and privileges as prerequisites; do not offer unsupported toggles.

## Acceptance criteria

- [ ] Each proposed policy lists filesystem/kernel prerequisites and inheritance behavior.
- [ ] Inspection and dry-run distinguish existing data from newly created children.
- [ ] Potentially irreversible setup has documented limits and a reviewed implementation plan.
