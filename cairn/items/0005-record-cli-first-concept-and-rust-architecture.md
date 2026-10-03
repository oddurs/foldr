---
id: 5
uid: 61297733-fcc3-421a-b15b-d006941b3012
title: Record CLI-first concept and Rust architecture
type: docs
status: done
milestone: v0.1
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
area: architecture
---

## Product concept

Foldr is a settings tool for folders: inspect properties people rarely see, explain what they mean, make deliberate changes, and reuse configurations as presets. The primary interface is a command-line application. No GUI, TUI, daemon, indexing service, or cloud account is required for the first release.

Start with macOS and Linux inspection; ship ordinary user-owned folder editing on both platforms in v0.2. Platform-specific features should remain visibly platform-specific. Windows is outside the initial scope, although the core data model should not unnecessarily prevent a future adapter.

## Language recommendation

Use Rust for the core and CLI. It offers native performance, compile-time memory and thread safety in safe code, expressive error types, and direct access to Unix APIs. Foreign-function and unsafe code must be isolated and reviewed; Rust does not itself guarantee correct filesystem operations.

Go is a credible alternative when development simplicity is the overriding preference; its garbage collector is unlikely to matter for ordinary folder inspection. It still needs OS-specific adapters. C++ is reasonable if a Qt desktop application becomes central. Zig is attractive for explicit control and C integration but requires manual memory management; that is additional correctness work for this project. The recommendation is Rust based on the user's stated priorities of cross-platform support, speed, and reliability. No graphical toolkit is selected.

Sources: https://rust-lang.org/ ; https://ziglang.org/learn/overview/ ; https://go.dev/doc/gc-guide ; https://doc.qt.io/qt-6/qtwidgets-index.html

## Architecture

A small Cargo workspace with foldr-core (model, capability descriptions, platform adapters, change planning, presets) and foldr-cli (argument handling and output). Keep macOS and Linux adapters as modules initially. Introduce more crates only when necessary. Use Rust's standard library and focused dependencies for CLI parsing, serialization, Unix calls, and xattrs; pick current compatible versions during implementation.

Represent properties separately from capabilities. A capability can be supported, unsupported, unknown, or unavailable to the current user, with a reason. Distinguish read support from write support. Treat folders on network/removable filesystems as potentially different from local defaults. A read-only inspector must not probe support by writing to the folder.

Preserve arbitrary Unix filenames and binary xattr values. Structured output needs an explicit lossless representation for non-UTF-8 paths; human output must escape control characters. Define symlink behavior explicitly: inspection reports the link and target, while mutating through a link requires an explicit target choice.

## Command direction

- foldr inspect PATH [--json]: identity, owner/group, permissions, ACL summary, flags, xattrs, filesystem and capabilities.
- foldr doctor PATH: explain why operations are unsupported or fail.
- foldr note get/set PATH: folder notes in foldr-owned xattrs.
- foldr flags set PATH ...: native flags, with platform-specific semantics.
- foldr permissions set PATH ...: basic access changes.
- foldr preset save/apply PATH ... [--dry-run]: versioned partial configurations.
- foldr undo CHANGE_ID: restore only recorded fields when the folder identity and expected post-change values still match.

Command spelling is provisional; finalize a consistent grammar in the CLI item. Mutating commands are explicit actions, support dry-run, and never become recursive implicitly. Preview describes scope: folder itself, future children, or existing descendants. No generic confirmation is needed for ordinary explicitly requested edits; destructive or large recursive workflows require a clear scope and recovery plan.

## Metadata and presets

Native properties continue to work without foldr. Foldr-owned notes/custom fields live in com.foldr.* xattrs on macOS and user.foldr.* on Linux. Own and edit only those keys by default. Never clear all xattrs or replace unrelated Finder metadata.

Use a versioned, human-editable preset format, initially TOML. Presets are patches: omitted fields remain untouched. Built-in examples are project (note/appearance where supported), archive (classification with optional structure lock), and private (explicit access settings). Export/import is explicit; xattrs are not guaranteed to survive Git, archives, cross-filesystem copies, or synchronization. Store local presets and undo records outside target folders.

## Reliability contract

Inspect, calculate changes, preview, apply, re-read, and record results. A batch of filesystem operations is not atomic. Persist recovery information before writes, expose partial failures, and never silently overwrite intervening changes. Track device/inode identity plus observed state, while recognizing identity limits across moves, mount changes, and inode reuse. Locking a directory protects entries, not every existing file's contents. Apply restrictive access and immutable flags last when possible.

Use native APIs rather than parsing shell utility output for core operations. Run as the ordinary user; report privileged operations with actionable explanations. Meaningful integration checks cover binary metadata, filename edge cases, unsupported features, permissions, symlinks, partial application, and undo conflicts.

## Release boundaries

v0.1: read-only inspector and capability explanations. v0.2: single-folder metadata, native flags, permissions, partial presets, verification and undo. v0.3: compare, batch, packaging and shell polish. Later: richer ACL editing, filesystem-specific settings, automation, Finder integration, and optional TUI/GUI.

## Acceptance criteria

- [x] The CLI-first product concept and release boundaries are recorded in this item.
- [x] The language recommendation, alternatives, and source references are recorded.
- [x] The architecture, metadata ownership, and mutation/recovery contract are recorded.

## 2026-10-02

Planning outcome: the complete concept, comparison of Rust/Go/C++/Zig, CLI architecture, and release boundaries are recorded in the body. Implementation has not started; graphical interfaces remain optional.

## Result

Recorded the CLI-first product concept and recommendation to use Rust in foldr-core and foldr-cli. The plan covers macOS and Linux, capability-aware inspection, native adapters, partial presets, verified edits and conflict-aware recovery. A TUI or GUI is optional future work.
