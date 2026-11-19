# Tasks

## 1. Backend data and native writes

- [x] 1.1 Add `Annotation { entry, text }`, parse `annotations[]` from `task export` into ordered notes, and keep `annotations_text` for compact checks; verify unit tests load entry plus text and turn an absent array into no notes.
- [x] 1.2 Add `is_recur_cleared`, extend `build_add_args` with `depends` and `recur`, and extend `build_modify_args` with recur set/clear; verify backend tests assert `depends:u1,u2`, `recur:weekly` on add, and a bare `recur:` on clear.
- [x] 1.3 Add `build_annotate_args` / `build_denotate_args` and re-export round-trip coverage; verify the tests run `annotate` / `denotate` with the exact note text.
- [x] 1.4 Update `docs/backend.md` with the new arg builders, `is_recur_cleared`, and the removed deferred-fields limitation; verify the signatures match the code.

## 2. Popup editor

- [x] 2.1 Add `F_RECUR` as the seventh editable field with fixed presets, `[`/`]` cycling, the `now: *` reference, normalized dirty checks, and the save/clear path; verify a popup test sets `weekly`, drops the badge on clear, and reports `saved`.
- [x] 2.2 Add the inline annotation VIEW sub-mode (`a` draft, `Enter` annotate, `esc` cancel, `[`/`]` select, `D`/`x` denotate, timestamped rows and count, `annotated` / `annotation removed` toasts); verify the add/select/delete smoke test.
- [x] 2.3 Convert the `depends` editor to a fixed list with `j`/`k` highlight movement, ignored typing, `Enter` append-and-save, and `Backspace` pop-last-link-and-save; verify the append, Backspace, and cycle tests.
- [x] 2.4 Move `EDIT` typing onto the field row with `place_cursor`, add `Delete`, keep the shared examples, and remove the duplicate input line and inline `▌`; verify the cursor-position test covers empty and non-empty fields.
- [x] 2.5 Validate recurrence-without-due and dependency cycles before any write; verify the toast wording and that no task or link changes on refusal.
- [x] 2.6 Update `docs/main.md` for the seven fields, note keys, and relation-row IDs; verify the documented key list matches `render_help` and the popup footer.

## 3. Quick-add and list rendering

- [x] 3.1 Add the `& depends` and `* recur` triggers, their fixed pickers, the `& <ids>` and `* recur` chips, the `+ more` entries, and the trimmed due presets; verify add-box tests append links, stay in the picker, and show grey/blue recurrence chips.
- [x] 3.2 Validate `description required`, `recurrence needs a due date`, and unknown depends IDs before commit, and carry `depends`/`recur` into the create; verify the created task stores the link and recurrence and the draft clears.
- [x] 3.3 Add the far-left ID column to open, completed, and relation rows through `RowCols.id` and add the blue `↻ <recur>` badge before the `blocked` trailer; verify ID ordering, truncation at narrow widths, and badge rendering.
- [x] 3.4 Make the add-box description and text filters editable in place, keep the description caret across picker switches, and keep trigger characters reserved; verify the caret movement, Backspace/Delete, and trigger tests.
- [x] 3.5 Update `docs/main.md` for the new triggers, chips, ID column, and caret behavior; verify the quick-add and list sections describe the rendered affordances.

## 4. Search, section prompt, and hotkeys

- [x] 4.1 Move Search query editing to the caret (`←`/`→`, `Backspace`/`Delete`, native cursor) and add caret editing to the section prompt; verify the Search and section-prompt caret tests.
- [x] 4.2 Add `/` as the Search shortcut, trim the list footer to `j/k move · space select · a new · d done · s start · D delete · u undo · / search · ? help · q quit`, and regroup the `?` overlay plus `print_help` under List / Edit / Search; verify the hint and help tests.
- [x] 4.3 Update `docs/main.md` for the Search caret, `/` shortcut, and grouped help; verify the key tables and hints match the rendered footer.

## 5. Integration verification

- [x] 5.1 Run `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`; verify all green and search the rendered strings for stale `▌` echoes or `Enter open/toggle` / `r refresh` hints.
- [x] 5.2 Cross-check every capability delta against the staged diff with `openspec validate "recurrence-annotations-and-input-polish" --strict`; verify it reports the change valid.

## Workflow follow-up

- Sync the delta specs and archive the change after review accepts the staged behavior.
- Verify the archived main specs render the new requirements without leftover deferred-field text.
