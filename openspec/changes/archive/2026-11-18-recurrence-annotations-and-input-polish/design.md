# Design

## Context

See `proposal.md` — Why. The staged `src/main.rs` and `src/backend.rs` diff already implements this change; this document records the approach so review, follow-ups, and archive stay consistent. Before the change, `Task.annotations` was a joined `String`, `recur` and annotations rendered read-only in the popup, quick-add was append-only with an inline `▌` glyph, `depends` was typed by hand, and rows had no ID column. The implementation lives against live `task export`/`task add`/`task modify` data with `TASKDATA`/`TASKRC` isolation.

## Goals / Non-Goals

**Goals:**
- Make recurrence and annotations first-class editable task data on the existing popup and add-box surfaces.
- Keep one contextual-input language between the add box and the popup editor (shared presets, clear rows, cap, and caret behavior).
- Put the real terminal caret on every typing surface so no surface renders an inline cursor glyph or a duplicate echo line.
- Surface task IDs consistently in list, completed, and relation rows.
- Keep the minimal keymap while advertising the newly common actions (`/`, `D`).

**Non-Goals:**
- No custom recurrence strings or durations beyond the five fixed rows; taskwarrior duration parsing stays unreachable from the UI.
- No recurrence-template management, `until`/`wait`/`mask` editing, or annotation editing of existing text.
- No background sync, new views, mouse support, or changes to selection/undo semantics.
- No data migration: annotations change shape only inside the app, and stored values remain native taskwarrior attributes.

## Decisions

- **Fixed recurrence presets plus an explicit `no recurrence` row over free-text recurrence.** Rationale: taskwarrior requires a due date for recurrence, and fixed rows keep both surfaces identical and validatable. Alternative (typed durations) rejected: no validation story and more UI surface.
- **`recur:` is omitted on add when cleared and always written on modify (`recur:<value>` to set, bare `recur:` to clear).** Rationale: a create has nothing to clear, while a modify must overwrite an existing value; a single `modify` write keeps the save atomic. Alternative (separate clear command) rejected: extra write and worse failure handling.
- **Annotations become `Vec<Annotation>` carrying the raw `entry` timestamp and `text`.** Rationale: render needs the local date and denotate needs the exact description; a joined string cannot recover either. Alternative (keep a display string, re-parse on save) rejected: lossy and fragile with `; ` in note text.
- **Annotation editing is a VIEW sub-mode (`annotating`), not an eighth editable field.** Rationale: annotations are a list with selection, add, and remove, which a single draft string cannot represent. Alternative (one popup field per note) rejected: unbounded field count and tab order. The sub-mode reuses the same caret helpers and escapes back to VIEW.
- **`depends` becomes a fixed list of open same-project tasks in both surfaces, with no typed IDs.** Rationale: typing IDs was error-prone and duplicated the popup picker; fixed lists guarantee resolvable links and let `Backspace` remove the last link. Alternative (keep typed IDs in one surface) rejected: parity drift and unknown-ID errors.
- **Cycle rejection runs as a DFS over the in-memory `depends` graph before any write (`creates_cycle`).** Rationale: taskwarrior accepts cyclic links, so the app must refuse them; the graph is already loaded and small. Alternative (rely on taskwarrior) rejected: silent cycles. Trade-off: a stale in-memory graph can miss links added externally until the next refresh.
- **One `place_cursor` helper plus char-index edit helpers replaces every inline `▌` glyph and the popup's duplicate input line.** Rationale: one caret implementation across add box, Search, section prompt, popup fields, and annotation draft; `text_width` handles wide glyphs and the caret clamps inside its pane. Alternative (keep per-surface cursor strings) rejected: the duplicate-line bug and drift the change set out to fix.
- **A far-left ID column is added to the shared row renderer (`RowCols.id`), so list, completed tail, and relation rows inherit it.** Rationale: one format and one width budget; short IDs clamp to 2–8 cells and fall back to the UUID prefix. Alternative (list-only ID) rejected: relation rows would lose the IDs users match against `depends`.
- **Hotkey chrome is trimmed, not the keymap: `/` calls `switch_view(0)`, `r` still refreshes, and only the hints and help groups change.** Rationale: keep muscle memory for refresh while making Search and Delete discoverable. Alternative (bind `/` to a new find mode) rejected: duplicates Search.

## Risks / Trade-offs

- [Risk] Terminal caret lands on a wrong cell when labels or wide glyphs change → Mitigation: smoke tests assert exact cursor positions for each surface, and `place_cursor` clamps to the pane.
- [Risk] Cycle detection misses links created outside the app until refresh → Mitigation: every write re-exports, so the graph catches up after the next successful mutation.
- [Risk] `denotate` uses exact text, so duplicate notes with identical text are ambiguous → Mitigation: the popup selects one note and taskwarrior removes by description; duplicate-text behavior is accepted for now.
- [Risk] A fixed list hides tasks filtered out by project, so cross-project links cannot be created → Mitigation: accepted scope, matching the existing popup picker; cross-project stored links still render and block.
- [Risk] The ID column narrows descriptions and can truncate them at small widths → Mitigation: `RowCols` reserves the ID width and tests check the ID stays visible at the narrowest widths.
- [Risk] Recurrence without a due date is refused, which can surprise users editing due first → Mitigation: the toast names the missing due date in both surfaces.
- [Risk] Taskwarrior recurrence templates leaking into views → Mitigation: exports stay limited to pending and completed tasks, so templates with other statuses never load.

## Migration Plan

- Land the staged `src/main.rs` and `src/backend.rs` as-is; no data migration. Existing recurrence and annotation values already load from the export.
- Update `docs/main.md` and `docs/backend.md` in the same worktree (already staged as worktree edits per `AGENTS.md`).
- Rollback: revert the change directory deltas plus the staged files; the current `openspec/specs/` contract remains the fallback until archive.
- Follow-ups (out of scope): custom recurrence durations, editing existing annotation text, cross-project depends pickers, and any `until`/`wait`/`mask` work.

## Open Questions

- None that change the specs, approach, or task breakdown. Duplicate-annotation-text behavior and cross-project linking can be revisited after usage.
