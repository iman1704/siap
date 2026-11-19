# Design

## Context

See proposal.md (Why) for motivation. The app is a single-file ratatui binary (`src/main.rs`): `App` owns list state, an optional `Popup` draft owns editor state, and `render_*` functions draw one frame per event. The current editor already separates `VIEW`/`EDIT` modes and the list already hosts a one-line quick-add; this design reshapes both interactions without touching views, sections, subtasks, or any backend (mock data only).

## Goals / Non-Goals

**Goals:**

- One strict modal contract shared by both boxes: navigation keys never type, typing keys never navigate.
- Every attribute reachable within one keystroke from the description, with hints that reflect only the focused input.

**Non-Goals:**

- No taskwarrior integration, persistence, or sync behavior.
- No new crates, modules, or multi-file split; the binary stays single-file.
- No mouse support and no changes to list rows, sections, or bulk operations.

## Decisions

- **Editor modal as `mode + insert` flag, not a third mode.** `PopupMode::{View, Edit}` stays; a boolean `insert` splits EDIT into navigate (`j`/`k`/`Tab`/arrows move fields, `i` types, `Enter` saves, `e`/`esc` discards to VIEW) and type (keystrokes edit, `↑`/`↓` moves the suggestion cursor, `Tab` applies, `Enter` saves back into navigate, `esc` discards to VIEW). Alternative (a three-variant enum) was rejected: the flag keeps every existing match site and test scaffold working with minimal churn.
- **`e` enters navigate, `i` enters type.** Alternative (both enter typing) blurred the modal line and reintroduced the `j`/`k`-types-or-moves ambiguity this change removes. `e` is the safe default; `i` is the shortcut for users who already know their field.
- **Suggestions as pure filtered views.** `all_projects` / `all_tags` / `due_presets` feed `popup_suggestions` and `add_suggestions`, which filter on the current text and truncate to five rows. No picker state is stored besides a cursor index, so the dropdown can never desync from the draft.
- **Arrows/`Tab` own picker lists; `j`/`k` stays typable.** The prototype originally bound `j`/`k` to picker movement, which made tags like `judo` untypable and hid project/priority entry. Moving list control to `↑`/`↓`/`Tab` (apply on `Enter`, `1`/`2`/`3` as direct priority shortcuts) resolves both, at the cost of `Tab` no longer advancing editor fields in type mode — field movement lives in navigate mode instead.
- **Chips carry set-state color.** Unset chips stay grey; set chips take attribute colors (`#` cyan, `~` yellow, project color, priority color) with project/priority chips rendered only when set, plus a `prio_set` flag so the default `P2` does not masquerade as a choice. Alternative (always showing all chips) was rejected as visual noise against the minimal-footer goal.

## Risks / Trade-offs

- [Trigger chars hijack description text] → A description cannot begin with `# ~ @ ! +` because the keystroke opens a picker. Mitigation: accepted and documented; `esc` returns to the description and the character can be typed after other text.
- [`Enter` stay-open breaks close-on-save muscle memory] → Mitigation: every save toasts `saved · esc closes` / `created 1 task · esc done`, and `esc` always exits.
- [`Tab` no longer advances editor fields in type mode] → Mitigation: footers and `?` help state the navigate/type split explicitly; `Tab` advancing still works in navigate mode.

## Migration Plan

No migration: mock-data TUI only. Rollback is reverting the change; `cargo fmt` and `cargo clippy -- -D warnings` stay clean per repo rules, and the `smoke` suite is updated to lock the new flows.

## Open Questions

None. The interaction contract was prototyped and reviewed in-session before this proposal.
