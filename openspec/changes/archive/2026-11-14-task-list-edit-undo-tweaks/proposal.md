# Proposal

## Why

Navigating sections and the completed tail costs extra keystrokes, done/undone lives on different keys (`x` vs nothing), and the popup's two-step edit mode plus dashed read-only values slow down quick triage. This change tightens the list/edit loop so headers collapse, one key toggles done, and editing is a single step.

## What Changes

- Section headers and the `COMPLETED` header become cursor stops showing `▾`/`▸`; `Enter` on a header collapses/expands it, hiding its rows.
- The cursor reaches completed rows; completing follows the task into the tail, uncompleting follows it back.
- **BREAKING** `d` toggles done/undone (`task <uuid> done` for open tasks, `task <uuid> modify status:pending` for completed ones), selection-aware. Delete moves to `D`.
- `u` (and `Ctrl+z`) performs single-level undo: `task undo` on the backend, in-memory snapshot in mock mode.
- List `e` does nothing (the "select a task with Enter first" hint is removed).
- Popup edit collapses to one step: `VIEW` moves with `j/k`, `e` types into the selected field at once, `Enter` saves back to `VIEW`, `esc` discards. No navigate/type split, no `i` mode.
- Editable empty values render blank (no `—`/`-`); a blank line separates the grey read-only block (urgency orange, `start` relative age, `recur none`/empty annotation grey). No `read-only` tags.
- The `start/end` label becomes `start`, showing relative age from the export `start` timestamp (`just now`, `15 min ago`, `1 hour ago`, `3 days ago`), blank when never started.
- Add-box `#` picker: a non-empty filter plus `Enter` creates/toggles custom tags (split on comma/space, strips `+`/`#`) instead of only offering existing suggestions.

## Capabilities

### New Capabilities

(none — all changes modify existing spec behavior)

### Modified Capabilities

- `task-list`: headers become collapsible cursor stops, the `COMPLETED` tail becomes cursor-navigable and collapsible, quick-add creates custom tags.
- `task-editor`: single-step `e`-to-edit workflow, blank editable empties, grey read-only block with `start` relative age instead of `start/end`.
- `task-operations`: `d` done/undone toggle, `D` delete, `u` undo, list `e` no-op, updated minimal keymap.
- `task-sections`: `Enter` on a section header collapses/expands it.
- `taskwarrior-backend`: undone via `modify status:pending`, `undo` command, `start` relative-age reads.

## Impact

- `src/main.rs`: navigable-row model, collapse state, toggle/undo flows, simplified popup keyboard and rendering, start-age helpers, custom-tag creation, updated footer/help/`--help` text and smoke tests.
- `src/backend.rs`: new `build_undone_args` and `build_undo_args` arg builders plus unit tests.
- `docs/main.md`, `docs/backend.md`: behavior docs updated.
- Keymap is breaking for muscle memory (`x` done removed, `d` delete moved to `D`).
