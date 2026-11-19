# Tasks

## 1. Row rendering

- [x] 1.1 Refactor `row_line` to render a marker column (`›`/`▌`/blank), a start column (`◉` only when active), the description, and aligned project/tag/due/priority columns; verify a `TestBackend` frame shows aligned columns and no project dot.
- [x] 1.2 Implement selection rendering with `▌` plus tint and make the cursor `›` win when a row is both selected and cursor-focused; verify selected, cursor, and both-at-once frames in row tests.
- [x] 1.3 Remove `[ ]`, `○`, and `●` from row rendering and update the affected row/theme tests; verify rendered output contains none of those glyphs and still contains `◉` for an active task.
- [x] 1.4 Update the row/marker section of `docs/main.md` to describe the aligned card, `▌` selection, and active-only `◉`; verify the doc matches the rendered example.

## 2. Add task box

- [x] 2.1 Handle `Shift+Tab` as both `KeyCode::BackTab` and `Tab` with `SHIFT`, moving the picker highlight backward with the same clamp as `Tab`; verify forward/backward reach the first and last suggestions in add-box tests.
- [x] 2.2 Fix `Enter` on a `+ more` item so the chosen picker opens and keeps focus instead of resetting to the description; verify the project/priority/tag/due choices in add-box tests.
- [x] 2.3 Implement `Backspace` clearing for project, due, and priority, and last-tag removal, only when the focused filter is empty; verify each branch plus the non-empty-filter edit path in tests.
- [x] 2.4 Show the `⌫ remove` footer hint only when the focused attribute is set; verify the footer text in the add-box render tests.
- [x] 2.5 Remove the section argument from `build_add_args` and the quick-add path; verify a committed task has no `siapsection` value and the add-box tests still pass.
- [x] 2.6 Update the add-box paragraph in `docs/main.md` and the add-box cases in `docs/manual-testing.md`; verify the keys and removal behavior are documented exactly as implemented.
- [x] 2.7 Update `docs/backend.md` for the section-free `build_add_args` signature and the non-interactive UDA bootstrap; verify the documented signatures match `src/backend.rs`.

## 3. Sections

- [x] 3.1 Replace the hardcoded section vector with a registry seeded from task `siapsection` values plus in-memory manual sections, sorted unsectioned-first / Now-Next-Later / custom A–Z; verify an empty store has an empty registry and seeded values appear.
- [x] 3.2 Render an `UNSECTIONED` group and render empty manual sections with count 0 by walking the registry instead of deriving headers from rows; verify `TestBackend` frames for both cases.
- [x] 3.3 Update `m` to cycle the registry and move an unsectioned task to the first section; verify section changes and the Inbox refusal in section tests.
- [x] 3.4 Add the `A`/`R`/`X` inline footer prompt mode (add, rename the cursor task's section, remove an empty section) and block other keys while it is open; verify add/rename/remove/cancel and blocked-input cases.
- [x] 3.5 Make UDA registration non-interactive and treat a failed registration as a toast; verify an isolated `TASKDATA` run leaves no `siapsection:` text in a task description.
- [x] 3.6 Update the section section of `docs/main.md` and the section cases in `docs/manual-testing.md`; verify the docs describe the seeded registry, `UNSECTIONED`, and the generic prompts.

## 4. Sidebar and project colors

- [x] 4.1 Remove the hardcoded project vector, rebuild the project cache from task data on every refresh, sort it alphabetically, and fall back to All when the active project disappears; verify the sidebar and fallback in view tests.
- [x] 4.2 Replace name-matched `project_color` with a deterministic palette map that resolves collisions in sorted project order and is shared by the sidebar and rows; verify stability, uniqueness, and sidebar/row agreement in tests.
- [x] 4.3 Update the sidebar section of `docs/main.md` and the sidebar cases in `docs/manual-testing.md`; verify the docs list the permanent views, dynamic projects, and color behavior.

## 5. Integration checks

- [x] 5.1 Run `cargo fmt` and `cargo clippy -- -D warnings`; verify both are clean.
- [x] 5.2 Run the targeted smoke tests for rows, add box, sections, views, and theme; verify they pass.
- [x] 5.3 Render representative frames at a narrow width and with long project/tag values; verify columns truncate without panic or overflow and the cursor stays aligned with its row.

## Workflow follow-up

- Archive the change after the code and docs pass review.
- Verify the archived main specs contain the new row, add-box, section, sidebar, and theme requirements.
