# Tasks

## 1. Inbox filter and flat list

- [x] 1.1 Unify Inbox predicate to open + project-empty (due ignored) in `src/main.rs` and verify `cargo test smoke` passes the updated Inbox filter assertions
- [x] 1.2 Render Inbox flat (no section headers/separators, urgency-only sort, keep subtask indent) and verify a headed `cargo test smoke` render shows Inbox rows with no uppercase section labels
- [x] 1.3 Guard `m`/`A`/`R`/`X` in Inbox with a hint, preserving hidden section values, and verify pressing `m` in Inbox leaves the task section unchanged with the hint shown

## 2. Row signals (project dot, active marker)

- [x] 2.1 Hide the `● project` dot in list rows only when the view is `Project(_)`, keeping Today/All dots and the Enter popup field, and verify `cargo test smoke` covers project-view-no-dot plus popup-still-shows-project
- [x] 2.2 Reduce active to the mint `◉` checkbox only (remove tint-for-active and `● active` text, keep cursor/selection tint) and verify `cargo test smoke` shows `◉` with no tint and no active text on a non-cursor active row

## 3. Docs and validation

- [x] 3.1 Update `docs/main.md` views/sections/list/active wording to match the deltas and verify the doc no longer mentions Inbox due filtering, Inbox sections, project dots in project views, or tint/`● active` text
- [x] 3.2 Run `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test smoke`, and `openspec validate --change task-list-ui-update` and verify all four stay clean
