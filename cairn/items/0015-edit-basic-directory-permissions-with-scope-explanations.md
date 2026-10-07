---
id: 15
uid: 454d3252-9259-49e8-8b41-8ea69c40f01c
title: Edit basic directory permissions with scope explanations
type: feature
status: done
milestone: v0.2
assignee: platform-engineer
depends_on:
- 12
created: 2026-10-02
updated: 2026-10-02
closed_at: 2026-10-02
priority: p1
effort: m
area: permissions
---

## Context

Foldr is a CLI-first folder inspector and configuration tool for macOS and Linux. Follow the recorded concept and architecture; these criteria describe future implementation work.

## Proposal

Edit basic mode bits using the shared engine and explain directory read/list, execute/traverse and write semantics. Include sticky and setgid descriptions where applicable. Detect interactions with existing ACLs; do not silently normalize them.

## Acceptance criteria

- [x] Mode changes are previewed, verified and recoverable.
- [x] Existing ACL interactions are shown or unsupported edits are rejected with a reason.
- [x] Changes affect only the selected directory unless recursive work is explicitly requested.

## 2026-10-02

Verified preview/apply/verification/undo mode changes through existing core applied_permissions_can_be_undone_but_conflicts_refuse and CLI symlink_mutation_requires_explicit_follow_and_mode_changes_do_not_recurse; both pass natively on macOS and checkpoint ef2a449 native Ubuntu CI full suites passed. Child mode remains0755 when selected folder changes0700. Constructed actual macOS extended allow ACL via acl_init/acl_create_entry/mbr_uid_to_uuid/acl_set_qualifier/acl_add_perm/acl_set_fd_np (no shell utilities): inspect reports supported extended ACL; both ordinary permission apply and dry-run reject with ACL review reason, leave folder/child modes unchanged, and create no recovery state. Linux native default ACL binary fixture/summary verified in Ubuntu CI; shared planner refuses readable extended ACLs and unreadable ACL states. CLI owner requested sticky/setgid scope explanations to cover accepted special bits.

## 2026-10-02

README now explains Linux setgid group inheritance for newly created entries, sticky deletion/rename restrictions for unprivileged users, and that existing or moved-in entries are not rewritten. Basic directory read/list, search/traverse and write/entry semantics, owner recovery access and ACL refusal are documented. Scope explanations complete alongside verified native mode and ACL behavior.

## Result

Basic mode edits support exact preview, native verification, durable recovery and conflict-aware undo without recursion. Existing or unreadable extended ACLs are refused with a review reason; actual native macOS extended ACL fixture verifies no writes or state creation. Native macOS/Linux integration suites pass; README explains directory permission, setgid inheritance and sticky entry-deletion scopes.
