# Proposal

## Why

The `siap-v1` specs describe the superseded `--mainv2`/`--taskv2` mockups (boxed rows, inline urgency numbers, `n/m` subtask counts, an always-editable popup, four version flags). That mock UI was scrapped and rebuilt as a single whitespace-driven UI on the WarGames Tokyo Night palette with a read-only-first editor, incorporating the `spec-update.md` revisions. The specs must be rebased onto the current mock, otherwise implementation will build the wrong UI.

## What Changes

- Rebase the six `siap-v1` capability deltas (`task-views`, `task-sections`, `task-list`, `task-editor`, `task-subtasks`, `task-operations`) onto the new single UI. Filter, sorting, UDA, and operation semantics carry over; chrome, markers, and key handling are restated to match the mock.
- Add a `ui-theme` capability pinning the Tokyo Night (WOPR) palette mapping and the whitespace-layout principles, so future screens stay coherent.
- Fold in the `spec-update.md` revisions: no subtask `n/m` counts in rows, full-row tint for active tasks, dimmed `COMPLETED` list at the bottom, popup opens read-only (`VIEW`) with `e` entering `EDIT`.
- **BREAKING**: remove the versioned mockup flags (`--mainv1`/`--taskv1`/`--mainv2`/`--taskv2`). There is one UI; only `--help` remains.
- These deltas supersede the `siap-v1` deltas for the same capability paths (`siap-v1` remains as history and should not be synced to main specs).

## Capabilities

### New Capabilities

- `task-views`: view filter definitions (Inbox/Today/All/Projects), counts, header totals, empty states — restated against the new top-line chrome.
- `task-sections`: app-owned section UDA lifecycle, defaults, add/rename/remove, move — unchanged semantics, new surface (no boxed headers).
- `task-list`: whitespace list rows (primary only, no urgency number, no subtask counts), `›` cursor, active full-row tint, collapsed-spacing sections, quick-add single-create, `/` find, dimmed `COMPLETED` tail, NORMAL/VISUAL keymap and bars.
- `task-editor`: `Enter` popup with read-only `VIEW` default, `e`/`i` entering `EDIT`, `esc` stepping back discarding, urgency + start/end read-only, `n` new subtask, validation and save semantics.
- `task-subtasks`: `depends` blocking, project inheritance, indented display without progress counts, `blocked` marker, delete propagation.
- `task-operations`: single + bulk lifecycle (`s`/`x`/`d`/`m`/`t`, `space`, `v` range, delete confirm) against the new single-line mode/bulk bars.
- `ui-theme`: Tokyo Night (WOPR) palette mapping and whitespace principles (no row boxes, blank-line grouping, dim-vs-bright information hierarchy).

### Modified Capabilities

- None — main specs (`openspec/specs/`) are empty.

## Impact

- Affects: new delta specs under `openspec/changes/siap-v2/specs/`; reference implementation is the current `src/main.rs` mock plus `docs/main.md`. No backend, API, or dependency changes.
- Systems: local TUI behavior only; taskwarrior integration, kanban, gum alerts, and note sync remain non-goals as in v1.
