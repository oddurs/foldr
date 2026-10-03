---
id: 29
uid: 691e8c26-f569-47c1-9177-0938533aee39
title: Prepare CLI grammar and output contract
type: docs
status: done
milestone: v0.1
assignee: cli-engineer
depends_on:
- 6
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p0
area: cli
part_of:
- 10
---

## Context

CLI design can happen while core/platform work proceeds. Item 0010 remains dependent on the completed native adapters.

## Proposal

Agree CLI grammar and core API calls for inspection, diagnostics, metadata, flags, permissions, presets, recovery, comparisons, batches, completions and man output. Document JSON envelopes, human escaping and exit statuses in the item body.

## Acceptance criteria

- [x] A coherent CLI grammar is recorded.
- [x] Machine output, errors, and pipe behavior are specified.
- [x] The CLI integration contract is agreed with the core architect.

## 2026-10-02

CLI grammar: global --json and --state-dir; inspect PATH; doctor PATH; note get/set/remove PATH; attr get/set/remove PATH KEY (foldr-owned keys, --hex for binary writes); flags set PATH --hidden BOOL/--immutable BOOL; permissions set PATH OCTAL; preset save PATH --output FILE --fields note,attrs,mode,flags, show FILE, apply FILE PATH..., import/export FILE --output FILE; undo history/show/apply ID; diff LEFT RIGHT (--preset or --snapshot); completions SHELL; man. All folder mutation commands accept --dry-run and --follow-symlink; explicit path batches have no recursion.

## 2026-10-02

Output contract: successful JSON is {schema_version:1,command,data}; errors are {schema_version:1,error:{code,message}} on stderr only. Encoded path bytes and binary attribute bytes are preserved. Human output escapes control characters and does not emit color. Exit0 success, including a clean broken-pipe exit; exit1 ordinary operational failure; exit2 invalid CLI/input; exit3 unsupported operation; exit4 conflict; exit5 partial batch failure. Broken stdout pipes terminate cleanly without a diagnostic. Inspection performs no recursive scans.

## 2026-10-02

Core agreement: inspect(path)->FolderSnapshot; plan(path,&ChangeRequest,follow_symlinks)->ChangePlan; apply(&ChangePlan,state_dir)->ChangeRecord; history(state_dir)->Vec<ChangeRecord>; undo(&record,state_dir,dry_run)->ChangeRecord. ChangeRequest uses optional note, owned metadata keys/bytes, mode, hidden, immutable. Dry-run renders a plan without calling apply. Core owns native calls, lossless representations, precondition checks, journal persistence and conflict-aware undo; CLI owns argument parsing, envelopes and human escaping.

## Result

Agreed inspection, mutation and recovery APIs with core architect; recorded complete CLI grammar, JSON/error envelopes, escaping, stable exit codes and broken-pipe semantics. CLI scaffolding is active preparation; blocked release items remain open pending adapters.
