# Proposal

## Why

Todoist-style single-parent subtasks (parent inversion, indented nesting, cascade delete, `n` to create) fight native Taskwarrior `depends` semantics and hide real blocking graphs. Adopting flat native `depends` blocking keeps every task top-level, visible, and deletable singly while still dimming blocked rows.

## What Changes

- Replace parent/child inversion with flat `Task.depends: Vec<String>` of blocking UUIDs; remove `parent`, `depends_label`, `children()`, `descendants_of()`, `link_child_backend()`, `add_subtask()`.
- **BREAKING** List rows stay flat and sorted (urgency/section); no indentation, no `SUBTASKS` nesting, no `n/m` counts. Blocked tasks stay visible dimmed with magenta `blocked` trailer.
- **BREAKING** Delete is single-only (Taskwarrior strips the UUID from others' `depends`); no cascade to descendants.
- **BREAKING** Remove `n` subtask creation from list and popup VIEW; `n` becomes a no-op (help/footer updated).
- Popup `DEPENDS` edits short IDs (`12, 14`), resolved to UUIDs with project-scoped dropdown (`ID  desc`, up to 8 visible rows), `Enter` adds one and stays in EDIT, validation toasts `unknown id: <tok>` / `can't depend on itself`; save diffs add-if-missing / remove-one.
- Popup gains read-only `BLOCKED BY` / `BLOCKS` relation sections (open tasks only, list row format without trailer) and drops the `SUBTASKS` section.
- `is_blocked(uuid)` means any `depends` UUID still open; `blocked_by()` / `blocking()` are the forward/reverse open indexes.
- Backend `build_depends_add_args` / `build_depends_remove_args` renamed to `task`/`dep` args; export maps `depends` directly with no inversion.

## Capabilities

### New Capabilities

- `task-dependencies`: native flat `depends` blocking model — multi-blocker DAG, open-only indexes, single delete, flat visibility rule. Central home for blocking semantics replacing the hierarchy.

### Modified Capabilities

- `task-subtasks`: retire single-parent hierarchy, inheritance, nesting, cascade, and `n` creation; requirements now point at `task-dependencies` flat behavior.
- `task-list`: flat rows only (no indent/nesting/counts); blocked trailer + dim retained.
- `task-editor`: `DEPENDS` ID-draft editing with project-scoped suggestions plus read-only `BLOCKED BY`/`BLOCKS`; remove popup `n` subtask creation.
- `task-operations`: remove `n` from keymap/help; delete confirms single-only.
- `taskwarrior-backend`: flat `depends` export mapping, per-UUID add/remove writes, single delete, no cascade.
- `task-views`: counts and filters include every open task (no top-level-only exclusion).

## Impact

- Code: `src/main.rs` (Task struct, export mapping, `open_rows`, render, popup, key handling, delete path), `src/backend.rs` (depends arg builders); smoke tests updated to flat expectations.
- UX breaking: no nested subtasks, no cascade delete, no `n` shortcut; `blocked` now means "waits on open deps".
- No new dependencies; Taskwarrior `depends` / `depends:-` are the only store mechanism.
