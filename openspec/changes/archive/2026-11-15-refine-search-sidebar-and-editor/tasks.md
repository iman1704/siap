# Tasks

The implementation already exists in the staged `src/main.rs` diff, so these tasks record the behavior to verify and the coverage gaps still open. Each task's verification is a test or a concrete observable.

## 1. Search view

- [x] 1.1 Add `View::Search`, `search_query`, `search_input`, description/project/tag matching, the Search empty hint, and the query-aware sidebar count; remove `find_mode`/`find_query`/`find_active` and the `/` key, and verify with `cargo test smoke::search_view_matches_and_clears_on_exit` plus `cargo test smoke::views_sort_search_and_bulk`
- [x] 1.2 Route Search input mode before the list keymap (printable keys type, `Tab`/`Shift+Tab`/`↑`/`↓` move the cursor, `i` focuses, `esc` exits or clears) and let list mode fall through to the shared keys, and verify by adding a smoke test that sends `q`/`l`/`?` in input mode and asserts the query changed while view, focus, and help did not
- [x] 1.3 Render the Search filter line with the block cursor and `(i input)` hint, hide the add box, label the bar `SEARCH`, clear the query when leaving the view, and verify with the render assertions in `cargo test smoke::views_sort_search_and_bulk` and `cargo test smoke::today_all_and_search_render_flat`
- [x] 1.4 Update `docs/main.md` for Search modes, filters, clear-on-exit, and the removed `/` find, and confirm the documented keys match the `?` overlay in `cargo test smoke::visual_range_bulk_and_help_match_keymap`

## 2. Sidebar live preview

- [x] 2.1 Replace the index-to-view match with `view_for_sidebar` (Search `0`, Inbox `1`, Today `2`, All `3`, projects `4+`), apply previews from `move_up`/`move_down` while keeping sidebar focus, confirm on `Enter`/`l`, and verify with `cargo test smoke::sidebar_projects_are_derived_sorted_and_fall_back` and `cargo test smoke::view_filters_counts_totals_and_empty_states`
- [x] 2.2 Add the Search row above Inbox with its live count and update every sidebar index assumption, and verify with `cargo test smoke::sidebar_projects_are_derived_sorted_and_fall_back` (the label list includes `Search` first) and `cargo test smoke::backend_section_move_persists`
- [x] 2.3 Drop the view title from the top bar while keeping the wordmark and open/due/active totals, add the `5` view shortcut for the first project, and verify that the top bar renders no view name in `cargo test smoke::view_filters_counts_totals_and_empty_states` and that help lists `1-5` in `cargo test smoke::visual_range_bulk_and_help_match_keymap`
- [x] 2.4 Update `docs/main.md` for live sidebar preview, Search in the permanent rows, and the title-free top bar

## 3. Flat views and section scope

- [x] 3.1 Make `open_rows` sort Search/Inbox/Today/All flat by urgency, keep project views sectioned with unsectioned tasks first, remove the `UNSECTIONED` group from `section_groups`/`navigable_rows`, and verify with `cargo test smoke::today_all_and_search_render_flat` and `cargo test smoke::sections_render_unsectioned_and_empty_manual_groups`
- [x] 3.2 Refuse `m`/`A`/`R`/`X` outside project views with the `sections live in project views` hint while preserving the hidden value, and verify with `cargo test smoke::section_moves_refused_in_flat_views`, `cargo test smoke::inbox_disables_section_keys`, and `cargo test smoke::section_prompts_add_rename_remove_and_block_keys`
- [x] 3.3 Insert the blank line before the first project-view section header when a flat unsectioned list precedes it, remove the redundant `overdue` row trailer while keeping `blocked`, and verify with `cargo test smoke::list_is_primary_only_with_air_and_no_subtask_counts` and `cargo test smoke::sections_render_unsectioned_and_empty_manual_groups`
- [x] 3.4 Update `docs/main.md` for flat Search/Inbox/Today/All, the unsectioned flat top list in project views, and the project-view-only section actions

## 4. Task editor cursor and feedback

- [x] 4.1 Add `edit_cursor`, `field_len`/`clamp_cursor`/`move_cursor`, and char-safe `popup_insert`/`popup_backspace`, and verify with the cursor assertions in `cargo test smoke::popup_is_read_only_until_edit_and_validates`
- [x] 4.2 Make `Tab`/`Shift+Tab`/`↑`/`↓` move only the suggestion highlight, `Enter` apply-then-save, `esc` revert only the focused field, and extend the popup smoke test to assert that a second field's unsaved edit survives `esc`
- [x] 4.3 Filter tag suggestions by `tag_last_token` and apply them with `apply_tag_pick` so `work, leet` + `leetcode` becomes `work, leetcode`, and add a unit test for the replace/append cases and the empty-filter case
- [x] 4.4 Open overdue tasks with `editable_due_input` (raw `YYYY-MM-DD`, blank for label-only tasks), add the read-only `id` row above urgency, render grey examples for empty typed fields while focused in `EDIT`, and verify with `cargo test smoke::overdue_editor_opens_raw_date_not_label` and `cargo test smoke::edit_mode_shows_grey_examples_for_empty_typed_fields`
- [x] 4.5 Toggle done/undone on `d` in popup `VIEW` without closing, refresh the status/dirty state, and add a smoke test that asserts the popup stays open and the task status flips both ways
- [x] 4.6 Update `docs/main.md` for the single-field `EDIT` flow, text cursor, per-field discard, token completion, raw-date open, `id` row, examples, and popup `d`

## 5. Add box, subtasks, and transient feedback

- [x] 5.1 Apply a typed non-empty project or due filter as-is on `Enter` (canonicalising known values, creating new projects, passing custom dates, `no date` clearing the due), show grey `e.g.` examples for empty Tags/Due filters, and verify with smoke tests for a new project, a custom date, `no date`, and the vanishing example
- [x] 5.2 Bind `n` to create a subtask from the highlighted list row with the parent's project and section, keep the popup path working, and verify with a list-row smoke test plus `cargo test smoke::popup_is_read_only_until_edit_and_validates`
- [x] 5.3 Timestamp toasts in `set_toast`/`clear_toast`, expire them after ~2.5s from the event loop, and add a smoke test that stales `toast_at` and asserts `expire_toast` clears the bar
- [x] 5.4 Align `?` help, the footer hints, and `print_help` with the final keys (no `/` find, Search modes, `n`, `d` in popup, `1-5`), update `docs/main.md`, and verify with `cargo test smoke::visual_range_bulk_and_help_match_keymap`
- [x] 5.5 Run `cargo fmt` and `cargo clippy -- -D warnings`, then the targeted smoke tests named above, and confirm both stay clean
