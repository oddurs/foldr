---
id: 35
uid: 3abc979c-d015-48f3-b605-006d9cfc0098
title: Traverse searchable parent directories without requiring list permission
type: bug
status: done
milestone: v0.2
assignee: Oddur Sigurdsson
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p1
part_of:
- 7
- 12
area: core
effort: s
---

## What happens

The no-follow directory descriptor walk opens every ancestor read-only, requiring read/list permission even when ordinary path lookup only needs search permission. A readable target under an owner0111 parent is wrongly rejected.

## What should happen

Use macOS O_SEARCH or Linux O_PATH for descriptor-relative traversal, retain O_DIRECTORY/O_NOFOLLOW, and reopen the held final directory read-only for native inspection/mutation.

## Reproduction

1. Create a directory with an owner-readable target child, set the parent mode to0111, and attempt a no-follow note edit.

## Acceptance criteria

- [x] A readable target under a searchable but unlistable parent can be opened and edited without following symlinks.
- [x] A parent lacking search permission rejects nonroot access and intermediate symlinks remain refused.
- [x] Focused native tests and core checks pass while descriptor ownership stays explicit.

## 2026-10-02

Platform reviewer reproduced on native macOS: parent mode0111 permits direct readonly target open but fails intermediate O_RDONLY parent open(EACCES); O_SEARCH traversal succeeds. Verified flags O_SEARCH=0x40100000(O_EXEC0x40000000|O_DIRECTORY0x00100000), O_NOFOLLOW0x100, O_CLOEXEC0x01000000 against installed SDK/libc. O_SEARCH opens root and cwd; O_SEARCH|DIR|NOFOLLOW rejects symlink with ENOTDIR. parent0111 search descriptor -> child search descriptor -> openat held child dot with O_RDONLY|DIR|CLOEXEC|NOFOLLOW yields usable final descriptor. Suggested regression tests search-only ancestor success, nonsearchable ancestor rejection, and nofollow symlink ancestor rejection. Linux equivalent uses O_PATH, preserving arch libc flags rather than literal constants.

## 2026-10-02

Nofollow opening now uses native Mac O_SEARCH or Linux O_PATH for initial slash/dot and all walked components, preserving DIRECTORY/CLOEXEC/NOFOLLOW. The final already-held directory is reopened read-only via openat dot so native operations receive a usable fd without resolving ancestors again. Native Mac test proves note plan/apply under parent0111, refuses nonroot parent0000 and symlink intermediate parents; permissions restore via held fd for cleanup. Stable and Rust1.85 core36 tests and Clippy -Dwarnings pass.

## Result

Use search-only descriptor traversal for nofollow ancestors and initial slash/dot, then reopen the held target directory read-only. Native integration proves edits through owner0111 parents while missing-search and symlink ancestors still refuse. Stable and Rust1.85 checks pass.
