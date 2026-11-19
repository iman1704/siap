# Tasks

## 1. Backend arg builders

- [ ] 1.1 Add `build_undone_args` (`modify status:pending`) and `build_undo_args` (`rc.confirmation=no undo`) with unit tests, and verify with `cargo test backend`
- [ ] 1.2 Document the new builders in `docs/backend.md` and verify `cargo fmt` shows no diff

## 2. Navigable rows and collapse

- [ ] 2.1 Add `NavRow`, `collapsed` set, `navigable_rows()`, collapse helpers, and cursor helpers (`cursor_nav`, `cursor_to_first_task`, `hold_cursor_on` over navigable rows), and verify the collapse smoke test passes
- [ ] 2.2 Rewrite list rendering to walk navigable rows with `▾`/`▸` header markers, cursor tint on headers and completed rows, and the empty-view early return, and verify with `cargo test smoke::sections_render` and `cargo test smoke::view_filters`
- [ ] 2.3 Rewire `Enter` (header toggles, task opens), `move_up`/`move_down`, `extend_visual` (headers skipped), and sidebar/view cursor placement, and verify with `cargo test smoke::list_is_primary_only` and `cargo test smoke::visual_range_bulk`

## 3. Done/undone toggle and undo

- [ ] 3.1 Implement `toggle_ids` (open→done, completed→pending, selection-aware) with `push_undo` snapshots and refresh-by-uuid cursor hold, rebind list keys (`d` toggle, `D` delete, `u`/`Ctrl+z` undo), and verify with `cargo test done_key_toggles` and `cargo test undo_restores`
- [ ] 3.2 Implement hybrid `undo()` (backend `task undo` + refresh, mock snapshot restore) and add `start_raw` tracking to start/stop and popup save paths, and verify with `cargo test backend_start_stop_done` and `cargo test undo_restores`

## 4. Popup simplification and read-only block

- [ ] 4.1 Remove the popup `insert` flag and collapse keyboard handling to `VIEW`/`EDIT` (`e` edits at once, `Enter` saves to `VIEW`, `esc` discards; list `e` no-op), update footer/help/`--help` hints, and verify with `cargo test popup_is_read_only`
- [ ] 4.2 Rework popup rendering (blank editable empties, blank separator, grey read-only block, `start` label with `start_display` relative age from new `start_raw`, no `read-only` tags) plus `format_start_ago` buckets and export mapping, and verify with `cargo test start_label` and `cargo test popup_shows_deferred`

## 5. Add-box custom tags and docs

- [ ] 5.1 Make a non-empty tag filter create/toggle custom tags on `Enter` (split comma/space, strip `+`/`#`, stay in picker), and verify with `cargo test add_box_creates_custom_tags` and the existing `cargo test add_box_triggers`
- [ ] 5.2 Update `docs/main.md` keymap/list/editor/operations/backend sections and fix all smoke tests for the new cursor model (header-tolerant assertions, `goto` over navigable rows), and verify with full `cargo test` (56 passed), `cargo fmt`, and `cargo clippy -- -D warnings`
