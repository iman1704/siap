# Proposal

## Why

Search is hidden behind a `/` find mode that collides with typing, the sidebar requires `Enter` before it shows a view, section headers clutter views that never use them, and the task editor still makes users step in and out of a whole-form edit mode. This change gives search its own view, makes the sidebar preview views on cursor movement, flattens views that do not organize by section, and turns editing into a single-field, cursor-aware interaction.

## What Changes

- **BREAKING** Add a `Search` sidebar entry above Inbox. Search owns a filter line and two modes: input mode captures every printable key (so `q`/`l`/`?` type instead of firing hotkeys) and list mode navigates results. `i` focuses input, `esc` exits input then clears the query, and leaving Search clears it. Search filters open tasks by description, project, or tags; it has no add box. The `/` find hotkey and its `FIND` bar mode are removed.
- **BREAKING** The sidebar live-follows the cursor: `j/k`, arrows, and Tab while the sidebar is focused switch the view immediately, keeping sidebar focus; `Enter`/`l` confirms and focuses the list. Entering Search through the sidebar lands in input mode.
- **BREAKING** Section headers render only in project views. Search, Inbox, Today, and All are flat urgency-descending lists; project views render unsectioned tasks as a flat list above the section headers. `UNSECTIONED` no longer renders, and `m`/`A`/`R`/`X` are refused outside project views with a hint.
- The top bar drops the current view name and keeps the wordmark plus open/due/active totals.
- **BREAKING** The editor drops whole-form edit mode: `e` edits only the focused field at once, typing lands at a text cursor moved with `←`/`→`, `Tab`/`Shift+Tab`/`↑`/`↓` move only the suggestion highlight, `Enter` applies the highlight then saves, and `esc` discards only the focused field (other unsaved edits survive). `d` toggles done/undone without closing the popup.
- Tag suggestions complete the in-progress last token instead of appending a duplicate (`leet` + Enter → `leetcode`, not `leet, leetcode`).
- An overdue task opens its raw calendar date (`YYYY-MM-DD`) in the editor instead of the `Xd overdue` label; a read-only `id` row (short numeric id else UUID prefix) sits above urgency.
- Empty typed fields show a grey example while focused in `EDIT` (`e.g. work`, `e.g. 2026-11-20`, `depends` → `enter task ID`); the dropdown's extra hint line is removed.
- The add box applies a typed custom project or due date as-is on `Enter`, creating the project/date instead of requiring a preset, and shows grey `e.g.` examples for empty Tags/Due filters.
- `n` creates a subtask from the list as well as the popup; the redundant `overdue` row trailer is removed while `blocked` stays.
- Progress toasts auto-clear after ~2.5 seconds instead of staying indefinitely.

## Capabilities

### New Capabilities

(none — all changes modify existing spec behavior)

### Modified Capabilities

- `task-views`: Search view with input/list modes, sidebar live preview, Search sidebar entry and count, top bar without the view name.
- `task-list`: flat Search/Inbox/Today/All sorting, project-view unsectioned flat list, no overdue trailer, custom project/due in quick-add, grey add-box examples, no add box in Search.
- `task-editor`: single-field cursor-aware editing, highlight-only `Tab`/arrows, per-field discard, token-aware tag completion, overdue raw-date open, read-only `id` row, grey edit examples, `d` done without closing.
- `task-operations`: keymap changes (no `/`, `n` subtask from the list, `5` first project, Search modes), toast expiry.
- `task-sections`: sections only render in project views, no `UNSECTIONED` group, section actions refused in flat views.
- `task-subtasks`: `n` creates a subtask from the highlighted list row as well as from the popup.

## Impact

- `src/main.rs`: replaces `find_mode`/`find_query`/`find_active` with `search_query`/`search_input`; adds `View::Search`, `preview_sidebar`, `matches_search`, `set_toast`/`clear_toast`/`expire_toast`, popup `edit_cursor` and cursor insert/backspace helpers, `apply_tag_pick`, `editable_due_input`, `popup_is_dirty`/`revert_popup_field`, `toggle_popup_done`, custom project/due add paths, and updated sidebar indexing, rendering, help/footer/`--help` text, and smoke tests.
- `docs/main.md` updated to the new views, sidebar, sections, list, editor, add-box, and feedback behavior.
- No new dependencies, no backend arg changes, no data migration.
- Keymap break for existing users: `/` find is gone (Search replaces it), Today/All lose section headers, and sidebar rows shift by one (view hotkeys gain `5`).
