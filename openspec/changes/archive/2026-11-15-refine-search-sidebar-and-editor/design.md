# Design

## Context

See proposal.md for motivation. `src/main.rs` is one ratatui TUI over a `task` subprocess backend. Before this change the list had a modal `/` find mode (`find_mode` / `find_query` / `find_active`), the sidebar required `Enter`/`l` to apply a highlight, `open_rows()` grouped every non-Inbox view by section, and the popup entered a whole-form `EDIT` mode with no text cursor. The diff under review is already staged, so this design records the approach and the boundaries of what landed.

## Goals / Non-Goals

**Goals:**

- Search as a first-class view with two explicit modes, replacing the `/` find mode without key collisions.
- A sidebar that previews while focused and confirms on `Enter`/`l`.
- Sections and `UNSECTIONED` only where they carry meaning: project views.
- A single-field popup edit with a real text cursor, per-field discard, and token-aware tag completion.
- Feedback that clears itself, and quick-add that accepts custom projects and dates.

**Non-Goals:**

- Fuzzy, debounced, or cross-field search, and search history.
- Persisting the Search query or preview without confirmation.
- Editing more than one field at a time in `EDIT`, multi-field undo, or a second undo level in the popup.
- Configurable section or project rules; `UNSECTIONED` is removed rather than made optional.
- New backend commands or taskwarrior arguments.

## Decisions

- **Search is a `View::Search` variant plus a mode flag.** Reusing the view enum keeps filters, counts, empty hints, and completed-tail rendering on one code path; `search_input: bool` separates typing from navigation. Alternative (keep a find overlay) was rejected because it needed a separate filter path and collided with list hotkeys.
- **Search routing runs before the normal keymap.** In input mode every printable key is consumed into the query, so `q`/`l`/`?` cannot fire; in list mode the handler falls through to the shared list keymap, so `j/k`, `Enter`, `space`, and `n` still work. `Tab`/`Shift+Tab`/`↑`/`↓` move the cursor in both modes. Alternative (auto-switch modes on key type) was rejected as unpredictable.
- **The query lives only inside Search.** `preview_sidebar` clears it when the preview leaves Search, so results never leak into another view. The sidebar's Search count uses an explicit-view matcher (`matches_search_for`) because `count_for` is called with a foreign view while `self.view` may still be the old one.
- **Sidebar mapping is split into a pure resolver and an applier.** `view_for_sidebar` maps index `0..3` to Search/Inbox/Today/All and `4+i` to projects; `preview_sidebar` applies the view, resets cursor/selection, and enters Search in input mode; `apply_sidebar` then drops sidebar focus. Alternative (mutating state inside the index resolver) was rejected because it made `move_up`/`move_down` re-entrant and hard to test.
- **Flatness is decided from the view, not stored per row.** `open_rows` sorts flat for Search/Inbox/Today/All and sectioned for projects; `navigable_rows` emits project-view unsectioned tasks first, then registry groups. `section_groups` no longer synthesises an `UNSECTIONED` group, and `render_list` inserts the section cushion when flat tasks precede the first header (`seen_task`). Alternative (a per-view `sectioned` flag on tasks) was rejected as redundant state.
- **Section actions gate on `View::Project`.** The existing Inbox refusal is generalised in one helper used by both `m` and the `A`/`R`/`X` prompt, so all four keys share one hint and one rule.
- **The popup cursor is a char offset per draft.** `edit_cursor` clamps to `field_len` (computed from `popup_field_text`), `←`/`→` move it, and `popup_insert`/`popup_backspace` convert char offsets to byte offsets before `String::insert`/`Vec<char>` removal. This keeps multibyte text safe. Alternative (byte indices) was rejected for UTF-8 correctness.
- **`Tab` no longer applies suggestions.** `Tab`/`Shift+Tab`/`↑`/`↓` share `step_popup_suggest`; `Enter` applies then calls `save_popup`. This removes the old split where `Tab` completed but did not save. Alternative (keep Tab-to-apply) was rejected because it duplicated the partial-tag bug path.
- **Tag completion replaces the in-progress token.** `tag_last_token` filters the dropdown and `apply_tag_pick` replaces the last comma/space-separated token, normalising the separator. Alternative (append as before) was rejected because `leet` + `leetcode` produced two tags.
- **Per-field discard recomputes dirty from the saved task.** `revert_popup_field` restores only the focused field, then `popup_is_dirty` compares every draft field so unrelated unsaved edits survive. `editable_due_input` is the shared baseline: overdue tasks open their raw date and legacy label-only mocks open blank, so an untouched overdue field is never dirty.
- **Read-only `id` row is derived at render.** `task_id_display` prefers the short numeric id and falls back to the first eight UUID chars, matching what `depends` wants without storing a second identity. The row sits above urgency in the grey block.
- **Toasts carry an `Instant`.** `set_toast` stamps, `clear_toast` resets, and `expire_toast` runs each event-loop iteration before drawing. A direct `toast` assignment (tests/fixtures) is stamped on the next expiry check, so no call site can leave a permanent message. Alternative (a timer thread or `crossterm` tick) was rejected as extra machinery for a 2.5s fade.
- **Quick-add custom values branch before the highlight path.** A non-empty project/due filter is applied as-is on `Enter`, with a case-insensitive canonical match when the text names an existing suggestion; `no date` clears the due. Alternative (require a preset) was rejected as the reported bug.

## Risks / Trade-offs

- [Live sidebar preview switches views on cursor movement] Arrowing past Search clears the query even when the user intended to keep it → Accepted: the query belongs to the view, and `Enter` confirms deliberately; the rule is documented in help.
- [Search consumes all printable keys in input mode] `?` and `q` cannot open help or quit until `esc` exits input → Mitigated by the `SEARCH input` footer hint and `esc` exiting to list mode.
- [Removing `UNSECTIONED` and Today/All headers] Tasks that relied on header grouping look flatter → Accepted per Revision 7; projections still group by section, and section values are preserved.
- [Per-field `esc` only] Users expecting `esc` to discard the whole draft can leave earlier edits unsaved → Mitigated by `● unsaved`, the footer text, and save-on-`Enter`; the alternative discarded work silently.
- [Toast expiry uses wall-clock `Instant`] A frozen or suspended process may expire several toasts at once → Accepted: the next iteration clears once, and `set_toast` restarts the timer.
- [Search completed tail is unfiltered by project] Search's COMPLETED tail spans every project → Accepted: Search is global by definition; the open rows still match the query.

## Migration Plan

- The staged diff lands in one step: view/state fields, sidebar resolver, row and section rendering, popup cursor/rendering, add-box branches, event routing, help/footer/`--help`, and smoke tests. There is no data migration; no stored value changes shape.
- Rollback: revert the commit. Sidebar indices, `/` find, and section headers in Today/All return together, so partial rollback would mix index assumptions and is not supported.
- User-visible break: `/` no longer finds, sidebar rows shift by one (`5` selects the first project), and Today/All lose section headers.
