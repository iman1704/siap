# Proposal

## Why

The task editor and the add-task box are the two places where users set task attributes, but both force memorized syntax or scattered hotkeys: the editor splits fields into PRIMARY/SECONDARY groups with mixed navigation keys, and the add box is a bare text line with no attribute affordance. A Todoist-style model — description first, attribute pickers one key away, contextual hints only — makes attribute entry discoverable without memorization.

## What Changes

- Task editor: one flat attribute list (no PRIMARY/SECONDARY split); project renders as its name only (no `●` dot).
- Task editor: `Enter` saves and keeps the editor open instead of closing it.
- Task editor: strict modal flow — `e` enters EDIT navigate, `j`/`k` moves there, `i` types, `Enter` saves, `e`/`esc` exits discarding to VIEW.
- Task editor: focused `project`/`tags`/`due`/`priority`/`status` fields show an inline suggestion dropdown filtered by typed text; `Tab` applies, `↑`/`↓` picks.
- Add-task box: description keeps focus; `#` tags, `~` due, `@` project, `!` priority, `+` more open pickers with only their own hints.
- Add-task box: `↑`/`↓`/`Tab` owns the picker list so `j`/`k` stay typable; attribute chips turn unique colors once set; `Enter` commits and stays open; minimal footer.

## Capabilities

### New Capabilities

- None. This change revises interaction behavior inside two existing capabilities.

### Modified Capabilities

- `task-editor`: flat single-list layout without the project dot; `Enter` save-stays-open in both VIEW and EDIT; modal `e` → navigate → `i` → type flow with `j`/`k` only in navigate; inline filtered suggestions with `Tab` apply.
- `task-list`: Todoist-style add box — trigger-char pickers (`#`/`~`/`@`/`!`/`+`), arrows/`Tab` list control with typable `j`/`k`, unique chip colors for set attributes, stay-open commit, minimal footer.

## Impact

- Affected code: `src/main.rs` (single-file binary: `App`/`AddDraft`/`Popup` state, `popup_key`/`add_key` input, `render_add_box`/`render_popup`/footer rendering, `smoke` tests).
- Docs: `docs/main.md` mirrors the interaction contract.
- No new dependencies, no taskwarrior backend changes; mock-data behavior only.
