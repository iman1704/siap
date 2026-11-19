# Design

## Context

See `proposal.md` for motivation. Current state: `src/main.rs` is a single-file mock (~2.3k lines) with one `App` state machine, a `Popup` with `VIEW`/`EDIT` modes, whitespace render functions, and `TestBackend` smoke tests. The `siap-v1` specs and the old `--mainvN`/`--taskvN` flag dispatch are gone. Constraints: Rust + ratatui + crossterm stay; mock data stays (no taskwarrior backend in this change); `cargo fmt` and `cargo clippy -- -D warnings` stay clean; `docs/main.md` mirrors `src/main.rs` per `AGENTS.md`.

## Goals / Non-Goals

**Goals:**
- Specs that describe the current mock exactly, so a later backend swap changes data sources without changing UI behavior.
- A pinned theme contract (palette roles + whitespace rules) that future screens reuse instead of re-deriving.
- A testable popup mode machine (VIEW ignores typing, EDIT drafts, discard restores) covered headlessly.

**Non-Goals:**
- Real taskwarrior integration (`task export`/`modify`, UDA auto-register against a live taskrc) — deferred; the mock's in-memory moves stand in.
- Kanban layout, gum alerts, note sync — remain excluded as in v1.
- Splitting `main.rs` into modules — explicitly deferred to keep this change spec-only in behavior.

## Decisions

- **Delta specs mirror the `siap-v1` capability paths plus one new `ui-theme` capability** over reusing the `siap-v1` files. Rationale: `siap-v1` stays untouched as history; the v2 deltas supersede it path-for-path, and `ui-theme` owns the new visual contract. Alternative (editing `siap-v1` in place) rejected: it would rewrite an in-progress change's history.
- **VIEW/EDIT as an explicit `PopupMode` enum, not a boolean.** Rationale: keyboard handling splits cleanly (VIEW: navigation only; EDIT: text + pickers), discard is a reload of the stored task, and tests can assert the mode. Alternative (single `editing: bool`) rejected: equivalent code, less readable at match sites.
- **Borrow-safe popup handling via short reborrows** (`as_ref` for ids/fields, `as_mut` per arm). Rationale: the first implementation held `app.popup.as_mut()` across `app` method calls and failed borrow-checking; the pattern is now the house style for popup code. Alternative (cloning the popup per keystroke) rejected: needless churn.
- **Theme as `Color::Rgb` constants named for roles (`BG`, `TINT`, `INK`, …), not raw hex at call sites.** Rationale: the palette file maps 1:1 to constants, so a future palette swap touches one block; render code reads intent (`MINT` = active) not values. Alternative (helper returning styles) rejected: over-abstraction for a mock.
- **Whitespace via empty `ListItem`s and a blank gutter column, not padding widgets.** Rationale: blank lines are data in the item list (easy to assert in `TestBackend` content), and the 4-wide gutter column guarantees the sidebar/main separation at any width. Alternative (ratatui padding/margin) rejected: harder to test and uneven across widgets.

## Risks / Trade-offs

- [Tint collision: cursor and active rows share `#1e2742`] → Mitigation: cursor adds the `›` marker plus bold-bright text; spec requires glyphs to carry state without color.
- [`siap-v1` and `siap-v2` both in-progress with overlapping paths] → Mitigation: proposal declares v2 supersedes v1; archive v1 without syncing when v2 is accepted.
- [Mock-only section/picker moves drift from real `task modify` semantics] → Mitigation: specs describe observable behavior (section value updates, project cycles), not CLI strings; backend swap re-validates scenarios.
- [Dim `#3a4a6d` on `#0a0f1e` is low contrast on poor displays] → Mitigation: faint color is restricted to non-essential chrome (hints, gutters); all state has a glyph backup.

## Migration Plan

No migration: this change only adds planning artifacts. When applied (a later change), the mock already implements the specs — apply work is verification (headless render tests per scenario) plus archiving `siap-v1` without sync. Rollback = delete the `siap-v2` change directory.

## Open Questions

- Exact default for unsectioned legacy tasks (`Now` vs preserve empty) — answerable during backend implementation without changing these specs.
- `$EDITOR` round-trip for annotations (tempfile vs direct `task annotate`) — same, implementation detail.
