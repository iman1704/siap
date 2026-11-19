# Design

## Context

See proposal.md for motivation. The app is a single-file ratatui TUI (`src/main.rs`, ~7000 lines) over a `task` subprocess backend (`src/backend.rs`). The cursor currently indexes `open_rows()` only, so headers and completed rows are unreachable; the popup has a two-mode (`VIEW` / `EDIT`-navigate / `EDIT`-type) modal with an `insert` flag; tasks carry a display-only `start_end` string with no raw `start` timestamp.

## Goals / Non-Goals

**Goals:**

- One navigable-row model that makes headers and completed rows first-class cursor stops while reusing the existing open/completed row builders.
- One-key done/undone toggle and single-level undo that work in both mock (offline tests) and backend modes.
- A popup with exactly two states (`VIEW`/`EDIT`) and a grey read-only block fed by real export data.

**Non-Goals:**

- Multi-level undo history, redo, or undo of section-registry-only edits beyond the single snapshot.
- Scrolling/virtualized lists; collapsing only hides rows, it does not change sort or fetch logic.
- Recur/annotation editing (still read-only by spec).

## Decisions

- **Navigable rows as a derived view, not a stored list.** A `NavRow` enum (`SectionHeader` / `Task` / `CompletedHeader` / `CompletedTask`) is rebuilt from `open_rows()` + `section_groups()` + `completed_rows()` on each use, filtered by a `collapsed: HashSet<String>` of uppercase labels. Alternative (storing a persistent row list) was rejected: it would need invalidation on every refresh, filter, and view switch, while the derived view is always consistent.
- **Collapse keys are uppercase labels plus `COMPLETED`.** Case-insensitive section names collapse under one key, matching the existing case-insensitive registry. Survives refresh because it lives outside `tasks`.
- **Refresh holds by uuid across navigable rows.** The pre-refresh `cursor_id()` (which now resolves completed rows too) is looked up in the post-refresh navigable rows. This makes done follow into the tail and undone follow back with one code path, instead of special-casing each direction.
- **Undo is hybrid.** Backend mode runs `task rc.confirmation=no undo` then re-exports (real persistence, one call); mock mode restores a single `UndoState` snapshot (tasks + sections + manual sections) taken before each mutation. Alternative (pure snapshot in backend mode) was rejected because it would diverge from the store on the next refresh.
- **Popup drops the `insert` flag.** `EDIT` types into the focused field immediately; prio/status fields ignore free text and keep `[`/`]` + `Tab`. This removes the navigate-vs-type branch from keyboard handling, rendering, and hints while keeping all existing suggestion paths.
- **Start age is computed at render from a new `start_raw` field.** `task_from_value` reads the export `start`; mocks use generated `mock_start_iso` ages. Buckets (`just now` / `min` / `hour(s)` / `day(s)`) keep the label short and timezone-safe by diffing `Local` datetimes.
- **Custom tags toggle by parsed words, never by highlight alone.** A non-empty tag filter splits on comma/space, strips `+`/`#`, and toggles each word, so new tags need no pre-existing suggestion. Empty-filter `Enter` keeps the old highlight-toggle path.

## Risks / Trade-offs

- [Cursor follows done into the tail] May surprise users expecting the cursor to stay on the next open row → Accepted: it demonstrates the now-navigable tail and is symmetric with undone; `j/k` returns immediately.
- [Delete on `D`, section-remove stays on `X`] `Shift+d` vs `Shift+x` confusion → Mitigated by help/footer/`--help` all showing `D` delete and by accepting only `D` (no `X` alias) to avoid ambiguity.
- [Undo snapshot is single-level] Rapid double-undo reports "nothing to undo" → Accepted per spec; documented in help-adjacent toast.
- [Empty-project view hides headers] The empty-hint early return suppresses even count-0 manual headers when a view has zero tasks → Accepted: matches prior empty-view behavior; headers reappear as soon as any task matches.

## Migration Plan

- Land `backend.rs` arg builders first (pure, unit-tested), then the `main.rs` model/render/keymap changes; all existing smoke tests were updated in the same change and new tests cover collapse, toggle, undo, start age, and custom tags.
- Rollback: revert the change; the keymap break (`x`/`d`) is the only user-visible migration, announced in help text.

## Open Questions

None — the confirmed interaction choices (Enter-on-header, `X`-for-delete rejected in favor of `D`, in-memory single undo, `e`-to-edit, Enter-creates-tag) are recorded in the specs above.
