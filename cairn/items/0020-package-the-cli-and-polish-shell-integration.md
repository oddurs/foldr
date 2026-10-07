---
id: 20
uid: f533cfe2-624a-4718-b439-e1a6091e53e0
title: Package the CLI and polish shell integration
type: chore
status: done
milestone: v0.3
assignee: integration-lead
depends_on:
- 17
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p1
effort: m
area: release
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Provide reproducible macOS and Linux release artifacts, installation instructions, completions and man/help documentation. Select concrete architectures, minimum versions and distribution coverage during implementation.

## Acceptance criteria

- [x] A fresh supported macOS and Linux environment can install and run foldr.
- [x] Completions and examples match the implemented command grammar.
- [x] Release checks verify artifact identity, permissions and supported-target behavior.

## 2026-10-02

Final source a4f147c passed every native package job in https://github.com/oddurs/foldr/actions/runs/37093137858: ARM64 macOS, Intel macOS and Linux x86_64. Each fresh runner built the locked release executable, generated bash/zsh/fish completions and man page, included README/MIT license, verified SHA256, extracted the archive, executed foldr0.1.0 and passed scripts/verify-cli.py. install -m755 establishes executable permissions and extracted execution validates them. Native stable/MSRV CI37093137863 also passed every job. Package baselines are macOS13+ and Linux glibc2.35+; builds remain unsigned development artifacts, available as workflow downloads, with no public release automatically published.

## Result

Delivered tested native Linux x86_64 and Intel/ARM64 Mac archives with executable, MIT license, README, generated man/completions and SHA256. All extracted binaries pass real-filesystem workflows on fresh runners; supported baselines and source installation are documented. Release publication/signing remains a separate action.
