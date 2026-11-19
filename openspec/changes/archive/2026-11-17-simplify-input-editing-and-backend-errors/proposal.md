# Proposal

## Why

The staged implementation already simplifies text input and backend error handling to reduce cursor-management complexity and terminal dependencies: mid-text editing, native terminal cursor placement, and exit-0 validation sniffing proved fragile and hard to maintain. This proposal retroactively records that simplification so specs match the shipped behavior.

## What Changes

- Replace native terminal cursor (`SetCursorStyle`, `place_native_cursor`) with an inline `▌` block glyph rendered at the end of focused inputs (add box, Search query, popup EDIT field, picker filter examples).
- Simplify all text inputs to append-only editing: typing appends (`push`), `Backspace` removes the last char (`pop`); remove `insert_at`, `backspace_at`, `clamp_cursors`, `move_focused_cursor`, and all per-input char cursors (`desc_cursor`, `filter_cursor`, `search_cursor`, section-prompt `cursor`).
- **BREAKING**: Remove `←`/`→` text-cursor movement in Search input, section prompt, add box, and popup editor. `←`/`→` are consumed / no-ops (Search explicitly consumes them so sidebar focus never steals typing) instead of moving an insertion point.
- Simplify `run_write_with_env` to exit-status-only error handling: remove `task_output_is_error` and `clean_task_output`; success returns `stdout`, failure returns trimmed `stderr`/`stdout`. Taskwarrior validation text on exit 0 is no longer treated as failure.
- Simplify popup `save_popup` to a single `task <uuid> modify` batch via `build_modify_args`; remove the follow-up `status:` write, old-status diff, and explicit depends-label guard comments (depends was and remains unwritten).
- Fix launch selection: default `sidebar_idx` from `1` to `0` so the highlighted sidebar entry matches the initial view.
- Remove cursor-position plumbing from render (`render_add_box` no longer returns a cursor position, `render_list` no longer places a pending cursor, popup no longer computes EDIT cursor offsets).

## Capabilities

### New Capabilities

- None — this change only simplifies existing behaviors.

### Modified Capabilities

- `task-list`: quick-add and add-box rendering/editing become append-only with inline `▌` cursor; no mid-text insert or arrow movement.
- `task-editor`: popup EDIT field renders `▌` at the text cursor (end of text or example) and edits append-only; `←`/`→` no longer move the insertion point; save is a single modify without a separate status write.
- `task-views`: Search input becomes append-only with `▌`; `←`/`→` no longer move the query cursor; launch sidebar selection aligns with the initial view.
- `taskwarrior-backend`: write failures are defined by process exit status only; exit-0 validation text is treated as success.

## Impact

- Affected code: `src/main.rs` (input state, key handlers, render paths, sidebar default, popup save), `src/backend.rs` (error detection helpers removed).
- Affected docs: `docs/backend.md`, `docs/main.md` (already updated unstaged to match).
- No new dependencies; removes `crossterm::cursor::SetCursorStyle` usage.
- User-visible: simpler but less flexible editing (no mid-text fixes, arrows do nothing in inputs); error toasts now only reflect non-zero exits; status edits via popup save path change.
