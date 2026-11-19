# Tasks

## 1. Views and sections

- [x] 1.1 Verify view filters (Inbox/Today/All/Projects), sidebar counts, top-line totals, and empty states against `specs/task-views/spec.md`, extending the `smoke` headless tests; fix any gaps in `src/main.rs`
- [x] 1.2 Verify section UDA defaults, `A`/`R`/`X` management, `m` move, and blank-line section separation against `specs/task-sections/spec.md` via headless render tests; fix any gaps in `src/main.rs`

## 2. List

- [x] 2.1 Verify primary-only rows with no urgency numbers and no subtask counts, `›` cursor without expansion, section-then-urgency sort, and indented children against `specs/task-list/spec.md` via headless render tests; fix any gaps in `src/main.rs`
- [x] 2.2 Verify active full-row tint with marker, dimmed `COMPLETED` tail, quick-add single-create with paste joining, and live `/` find against `specs/task-list/spec.md` via headless tests; fix any gaps in `src/main.rs`

## 3. Editor and subtasks

- [x] 3.1 Verify popup `VIEW` read-only default (typing ignored), `e`/`i` entering `EDIT`, field navigation skipping read-only rows, and `[`/`]` pickers against `specs/task-editor/spec.md` via headless tests; fix any gaps in `src/main.rs`
- [x] 3.2 Verify save validation (empty description rejected), `esc` discard-to-`VIEW`, plain close from `VIEW`, `n` subtask inheritance, and `E` editor toast against `specs/task-editor/spec.md` and `specs/task-subtasks/spec.md` via headless tests; fix any gaps in `src/main.rs`
- [x] 3.3 Verify `depends` blocking with `blocked` marker, no `n/m` counts, and delete-cascades-children against `specs/task-subtasks/spec.md` via headless tests; fix any gaps in `src/main.rs`

## 4. Operations

- [x] 4.1 Verify `s` start/stop with tint, `x` complete to the tail, `d` with `y/n` confirm, and the list `e` hint against `specs/task-operations/spec.md` via headless tests; fix any gaps in `src/main.rs`
- [x] 4.2 Verify `space` toggle, `v` VISUAL range with `j/k` extension, bulk `d/m/t/x` with selection clearing, `esc` cancel, and `?` overlay matching the keymap against `specs/task-operations/spec.md` via headless tests; fix any gaps in `src/main.rs`

## 5. Theme and integration

- [x] 5.1 Verify palette roles, blank-space grouping, bright-vs-dim hierarchy, and glyph markers against `specs/ui-theme/spec.md` via headless render assertions; fix any gaps in `src/main.rs`
- [x] 5.2 Update `docs/main.md` to match the verified UI and verify every documented command runs as written
- [x] 5.3 Run `cargo fmt`, `cargo clippy -- -D warnings`, and the full `cargo test` suite and verify all stay clean

## Workflow follow-up

- Archive `siap-v1` without syncing once `siap-v2` is accepted, since v2 supersedes it.
- Archive `siap-v2` after review, syncing its specs to main specs.
