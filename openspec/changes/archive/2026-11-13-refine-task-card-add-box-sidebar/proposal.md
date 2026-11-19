# Proposal

## Why

The live task list still carries hardcoded chrome and navigation: every row shows a `[ ]` selection box and a `○` pending placeholder, the add box's `+ more` menu never opens the chosen picker, quick-add stamps `siapsection:Now` on every task, and both the sidebar projects and list sections are fixed in code. That adds visual noise, blocks attribute entry, writes unwanted UDA data, and hides real taskwarrior state behind stale entries.

## What Changes

- **Task card**: remove the `[ ]` selection box and the `○` pending placeholder; show the mint `◉` start indicator only for active tasks; mark selection with `▌` plus tint; drop the project dot in favor of the project's color; align project, tags, due, and priority in one line.
- **Add task box**: add `Shift+Tab` reverse picker navigation; allow removing an added attribute with `Backspace` on an empty filter; make `Enter` on a `+ more` item open the chosen picker; stop writing `siapsection` when a task is added.
- **Sections**: remove the hardcoded Now/Next/Later registry. Seed the registry from existing `siapsection` values plus user-added sections, render unsectioned tasks under a display-only `UNSECTIONED` group, render empty user-added sections, and make `A`/`R`/`X` generic inline name prompts instead of Someday-specific shortcuts. Only an explicit `m`/section action writes the UDA.
- **Sidebar**: rebuild the project list from current taskwarrior tasks on every refresh (alphabetical, stale projects removed), keep Inbox/Today/All permanent, fall back to All when the active project disappears, and assign a deterministic palette color per project.
- **Spec consistency**: replace the old `[×]`/`[ ]`/`○` glyph contract and resolve the active-tint contradiction between `task-list` and `task-operations`.
- **BREAKING** behavior: tasks without a `siapsection` value now render under `UNSECTIONED` instead of `NOW`; default section headers no longer appear until a task carries the value; `A`/`R`/`X` require a name prompt; rows lose the selection checkbox glyph.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `task-list`: row markers and alignment, active-only start indicator, selection cue, add-box Shift+Tab and Backspace removal, `+ more` Enter behavior, quick-add without a section stamp.
- `task-sections`: dynamic registry seeded from task values, `UNSECTIONED` group, generic `A`/`R`/`X` prompts, empty-section rendering, write-only-on-move rule.
- `task-views`: sidebar project list derived from taskwarrior, permanent Inbox/Today/All, stale-project removal, project-disappearance fallback.
- `ui-theme`: glyph set without selection boxes or pending circles, project name colors and deterministic project palette.
- `task-operations`: active state shows only the `◉` marker (no full-row tint), and selection uses `▌` plus tint.

## Impact

- `src/main.rs`: `row_line` rendering; `AddDraft`/`add_key`/`apply_add_suggestion`; `App` section registry, project list, refresh, sidebar rows, section management, and color assignment; smoke tests.
- `src/backend.rs`: `build_add_args` no longer takes or stamps a section; UDA bootstrap must run without an interactive confirmation prompt.
- `docs/main.md` and `docs/manual-testing.md` must mirror the new rows, add-box keys, section model, and sidebar behavior.
- No new dependencies. No data migration: existing `siapsection` values remain valid and seed the registry.
