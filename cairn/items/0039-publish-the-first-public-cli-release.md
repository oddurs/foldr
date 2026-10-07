---
id: 39
uid: 56e07ce2-00f2-40bd-a6c0-afeea5235b33
title: Publish the first public CLI release
type: chore
status: done
milestone: v0.3
assignee: integration-lead
created: 2026-10-06
updated: 2026-10-06
closed_at: 2026-10-06
priority: p2
area: release
---

## Proposal

Publish GitHub release v0.1.0 from merged main with the native macOS/Linux binaries already verified by CI. Include checksums, platform baselines, installation instructions and concrete recovery limitations.

## Acceptance criteria

- [x] Release tag identifies the merged source and package contents match verified build outputs.
- [x] All three native archives and checksums are published with release notes.
- [x] Published release assets and Cairn records are verified.

## 2026-10-06

Prepared v0.1.0 notes and all six native archive/checksum assets from passing package run37093270299. Independently verified SHA256, archive paths, no symlink/AppleDouble entries, exact0755 executable, MIT/README/man/bash/zsh/fish contents. Tag target is merged commit25a6bf7 with tree exactly matching CI build source a2592e9. Draft publication is next.

## 2026-10-06

Published https://github.com/oddurs/foldr/releases/tag/v0.1.0 as the latest non-prerelease, with three archives and three checksum files. Authenticated tag-ref verification confirms v0.1.0 points directly to25a6bf74ec688bf48de08986e0d03288c772656b. Every uploaded asset name, size, uploaded state and server-side SHA256 digest exactly matches the downloaded passing CI artifact. Draft lookup by tag returned404; the following publish command still ran, so full server-side digest/tag verification was completed immediately after publication. Release notes include platform baselines, archive/source installation and native recovery limitations.

## Result

Released foldr v0.1.0 publicly at https://github.com/oddurs/foldr/releases/tag/v0.1.0 from merged main. Linux x86_64 and Intel/ARM64 Mac archives plus SHA256 files are uploaded; all six server digests and the exact tag target are verified against passing native CI outputs.
