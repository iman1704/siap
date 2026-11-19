# Design

## Context

See `proposal.md` for the motivation. The app is a single ratatui binary: `App` owns `tasks`, the `projects` and `sections` vectors, the `AddDraft`, and the optional `Popup`; `row_line` builds one row; `render_list` inserts section headers while walking `open_rows`; `add_key` / `apply_add_suggestion` / `quick_add_commit` own the add box; `refresh_from_backend` currently appends projects; `project_color` matches project names by hand; `backend::build_add_args` always stamps a section.

Constraints:

- No new crates; the binary stays single-file with `backend.rs`.
- `open_rows` is the single cursor-order source; section headers must not desync the cursor.
- The section UDA and the backend calls remain the only persistence.
- Mock tasks remain the offline fallback, but hardcoded project and section vectors do not.

## Goals / Non-Goals

**Goals:**

- Make rows, project entries, and section headers reflect live task data.
- Make add-box attributes reversible and the `+ more` menu functional.
- Keep cursor, sidebar, and section indices consistent after a refresh.
- Keep section ordering deterministic and the section registry stable for a given task set.

**Non-Goals:**

- No popup editor changes.
- No per-project section registries; sections stay global.
- No persistence for empty user-added sections; they live for the process unless a task carries the value.
- No new backend sync, daemon, or file watcher.
- No table headers or boxed task cards; the row stays one airy line.

## Decisions

### Row layout uses two leading columns plus aligned metadata

`row_line` becomes a small layout function with a marker column (`›` cursor, `▌` selected, blank), a start column (`◉` only for active), the description, and aligned project/tags/due/priority columns. The project is colored through the project-color lookup and the `●` dot is removed. `App` computes the metadata widths once per frame and passes them into `row_line`.

Alternatives:

- Table headers (rejected: the user chose one-line aligned rows; headers add chrome).
- Keep the checkbox and pending circle (rejected: visual noise and the source of this change).

### Selection is a marker plus tint, not a checkbox

`space` / VISUAL selection still use `app.selected`. Rendering changes from `[×]`/`[ ]` to `▌` plus tint. When a row is both selected and under the cursor, `›` wins in the marker column; the selection count bar still shows the selection. This keeps one marker column and avoids two competing glyphs.

Alternative: keep `[×]` for selection (rejected: the request removes `[ ]` from the card).

### Add-box focus model stays; `Backspace` gets a clear branch

`AddFocus` does not change. `Shift+Tab` is handled both as `KeyCode::BackTab` and as `KeyCode::Tab` with `SHIFT`, decrementing the picker index with the same clamp as `Tab`. The `Enter` path for a non-description picker applies the suggestion and lets `apply_add_suggestion` decide the next focus; the outer reset back to `Desc` is removed for `+ more`. `Backspace` checks the filter first: a non-empty filter is edited, an empty filter clears the focused attribute (tags pop the last tag, project/due clear, priority unsets, More is a no-op). A small helper computes whether the footer should show `⌫ remove`.

Alternatives:

- Chip-focus mode (rejected by the user).
- A `[clear]` row in every picker (rejected: extra list noise and a second interaction model).

### Sections become a derived registry plus manual additions

`App` keeps `sections` as the active registry but stops seeding it with Now/Next/Later. A `sync_sections` step runs after every task load: it collects distinct task section values, merges the in-memory `manual_sections`, and sorts the result as unsectioned (rendered separately), then Now, Next, Later when present, then custom names alphabetically. `UNSECTIONED` is not stored in the registry; `render_list` emits it when an unsectioned group has rows. `render_list` iterates the registry (including empty manual sections) instead of discovering headers from rows; `open_rows` remains the flat cursor order and a helper partitions it into groups.

`A`/`R`/`X` open a new footer prompt mode instead of the hardcoded Someday actions. `A` adds the typed name to `manual_sections`; `R` renames the cursor task's current section across its tasks; `X` removes the named section only when empty. The prompt blocks list/add/popup input like the find and delete prompts do.

Alternatives:

- Task-derived only with no `A/R/X` (rejected: no UI path to create a section).
- Persist empty sections in taskwarrior config (rejected: adds a second configuration surface and is outside the request).

### Sidebar projects are a derived cache

Remove the hardcoded project vector from `App::new`. Keep `projects` as a cache rebuilt after every task load: unique non-empty task projects, sorted alphabetically. `apply_sidebar` and `switch_view` continue to index the cache. If the active `View::Project` is no longer in the cache after a refresh, fall back to All. The mock fallback tasks still produce projects, so the app remains usable without a backend.

### Project colors are deterministic and collision-resolved

Replace the name-matched `project_color` with a map rebuilt alongside the project cache. Each project gets a preferred slot from a stable hash over the six palette accents (blue, mint, yellow, magenta, cyan, orange); when the slot is taken, walk to the next free slot in sorted project order. This keeps the current project set unique and stable across refreshes, at the cost of colors shifting when the set changes. The map is used by the sidebar and by `row_line`.

Alternatives:

- Pure hash without collision handling (rejected: collisions violate uniqueness).
- Alphabetical index (rejected: adding an early project would shift every later color).

### Backend add path drops the section argument

`build_add_args` loses its section parameter and never appends `siapsection`. `ensure_section_uda` runs `task config` with `rc.confirmation=off` and the bootstrap treats a failed registration as a toast. Section writes from `m` or the `A/R/X` prompt still use the existing per-task section args; the registration failure path must not let `siapsection:` text reach the description.

## Risks / Trade-offs

- [Empty manual sections disappear on restart] -> They are in-memory by design; once a task carries the value it seeds the next launch. Documented in `docs/main.md` and the manual test cases.
- [Sections are global, so a project view can show sections used by other projects] -> Existing behavior; keep it and note it in the docs rather than introducing per-project state.
- [Project colors shift when the project set changes] -> Acceptable: the spec only guarantees uniqueness for the current set and stability across refreshes with the same set.
- [`Shift+Tab` terminal encoding varies] -> Handle both `BackTab` and shifted `Tab`; cover both in tests.
- [`Backspace` could delete text instead of clearing a chip] -> Only clear when the filter is empty; footer hint and tests lock the branch order.
- [`A/R/X` prompts add a new modal] -> Reuse the existing prompt/blocking pattern; handle the prompt before list keys and add tests for cancel and blocked input.
- [`render_list` rewrite can desync the cursor] -> Keep `open_rows` as the only flat order and partition it for rendering; assert cursor/row alignment in smoke tests.
- [Existing smoke tests assume hardcoded projects and sections] -> Update fixtures and assertions to derive from mock tasks and explicit seeded values.

## Migration Plan

No data migration: existing `siapsection` values seed the registry on launch, and tasks without a value render under `UNSECTIONED` without a write. Rollback is reverting the change; `cargo fmt` and `cargo clippy -- -D warnings` stay clean, and the smoke suite is updated to lock the new rows, add-box keys, sections, and sidebar.

## Open Questions

None. The A/R/X prompt model and the in-memory empty-section behavior were settled during exploration.
