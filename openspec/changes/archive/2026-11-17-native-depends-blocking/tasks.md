# Tasks

## 1. Flat depends model and backend builders

- [x] 1.1 Replace `Task.parent`/`depends_label` with `depends: Vec<String>` plus `blocked_by()`/`blocking()`/`is_blocked()` and verify `cargo clippy -- -D warnings` passes on the model
- [x] 1.2 Map `task export` `depends` arrays directly with no inversion and verify deleted rows drop while a blocked/blocker pair both render top-level
- [x] 1.3 Rename backend builders to task/dep args and verify `build_depends_add_args`/`build_depends_remove_args` emit `task <id> modify depends:<dep>` / `depends:-<dep>`

## 2. Flat list rendering

- [x] 2.1 Render every open task as its own top-level row with dim + `blocked` trailer and verify no indentation, nesting, or `n/m` counts appear
- [x] 2.2 Update section counts, project filters, and sidebar counts to include all open tasks and verify project view isolates by exact `project` match

## 3. Popup depends editing and relations

- [x] 3.1 Edit `DEPENDS` as sorted short IDs with project-scoped `ID  desc` suggestions and verify `Enter` adds one and stays in EDIT while already-added IDs hide
- [x] 3.2 Validate drafts (`unknown id`, self-link) and save via add-missing/remove-one writes with display-ID refresh and verify dirty-check compares UUID sets
- [x] 3.3 Render read-only `BLOCKED BY`/`BLOCKS` with list row format sans trailer and verify empty sections hide and example reads `e.g. 12, 14`

## 4. Keymap and delete cleanup

- [x] 4.1 Remove `n` subtask creation from list, popup VIEW, help, and footer and verify `n` is a no-op with no task created
- [x] 4.2 Switch delete confirm to single-only batches and verify blocker/blocked peers survive each other's deletion

## 5. Integration verification

- [x] 5.1 Run `cargo fmt` plus `cargo clippy -- -D warnings` and the smoke suite and verify flat blocking, dropdown, save, and single-delete scenarios pass
