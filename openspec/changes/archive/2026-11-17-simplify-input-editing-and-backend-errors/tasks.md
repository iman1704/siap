# Tasks

## 1. Backend error-handling simplification

- [x] 1.1 Remove `task_output_is_error` and `clean_task_output` and simplify `run_write_with_env` to exit-status-only, and verify with `git diff --cached -- src/backend.rs` showing only trimmed stderr/stdout error paths
- [x] 1.2 Update backend docs in `docs/backend.md` to describe exit-status-only writes, and verify the write-runner paragraph no longer mentions exit-0 detection

## 2. Append-only input editing

- [x] 2.1 Remove `insert_at`, `backspace_at`, `place_native_cursor`, `clamp_cursors`, `move_focused_cursor`, and all `*_cursor` fields (`desc_cursor`, `filter_cursor`, `search_cursor`, section-prompt `cursor`), and verify with `grep -rn "cursor.*SetCursorStyle\|insert_at\|backspace_at\|place_native_cursor" src/` returning no hits except popup `edit_cursor`
- [x] 2.2 Simplify `search_key`, `section_prompt_key`, and `add_key` to `push`/`pop` append-only with `Left`/`Right` consumed, and verify by typing, Backspace, and arrow keys in Search, section prompt, and each add-box picker
- [x] 2.3 Render inline `▌` on focused add-box, Search query, picker examples, and popup EDIT field and remove pending-cursor placement, and verify screenshots/show output contain `▌` and no native cursor positioning calls remain

## 3. Popup save and launch alignment

- [x] 3.1 Simplify `save_popup` to a single `build_modify_args` modify batch without a separate `status:` write, and verify a popup save refreshes from the backend and toasts `save failed: ...` only on non-zero exit
- [x] 3.2 Set default `sidebar_idx` to `0` so launch highlight matches the initial view, and verify launch shows the first permanent row highlighted with the matching list
- [x] 3.3 Update `docs/main.md` and `spec-update.md` revision notes to match append-only editing and backend behavior, and verify `cargo fmt --check` and `cargo clippy -- -D warnings` stay clean

## 4. Integration verification

- [x] 4.1 Run the smoke coverage for Search cursor display, add-box filter Backspace, and popup save paths, and verify the updated assertions (`nadia▌`, no fake cursor in list mode) pass
