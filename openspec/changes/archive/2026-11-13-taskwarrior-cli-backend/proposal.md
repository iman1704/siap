# Proposal

## Why

siap's UI is fully mocked (`mock_tasks()` in `src/main.rs`, "no real taskwarrior integration" in `docs/main.md`). Users cannot see or change their real tasks, so every spec describing taskwarrior semantics (UDA sections, urgency sort, `depends` subtasks, start/done/delete) is currently unexercised. Wiring the CLI backend now unblocks all real usage.

## What Changes

- Read live tasks via `task export` subprocess (`+PENDING` for open rows, `+COMPLETED` capped at 10 recent by `end` for the tail; `deleted` never fetched).
- Replace mock `Task` identity `u32 id` with stable `uuid` key (numeric `id` kept display-only); re-key selection, cursor, children, and popup.
- Map real fields: ISO `due` to friendly display (`today HH:MM`, `tomorrow`, weekday, `Xd overdue`, `—`) with local-day overdue/today computed via `chrono`; absent `priority` renders P3; computed `urgency` read-only; `annotations`/`recur` visible read-only, not editable.
- Write through CLI keyed by `uuid`: quick-add, popup save, `s` start/stop, `x` done, `d` delete with cascade, `m` section move, `n` subtask, bulk `d/m/t/x`.
- Auto-register `uda.siapsection` via `task config` on launch; missing section renders under Now with no backfill writes.
- Refresh on launch plus re-export after every successful mutation, plus manual `r`; failed commands toast and keep the old list with cursor held by `uuid`.
- **BREAKING**: numeric `id` is no longer stable identity; first launch against live data replaces the 14 mock rows with the user's real pending set.

## Capabilities

### New Capabilities

- `taskwarrior-backend`: CLI-backed task source replacing mocks — export fetching, field mapping, write-through commands, refresh policy, failure toasts, and UDA bootstrap.

### Modified Capabilities

- `task-subtasks`: clarify `depends` direction (parent depends on children) and single-parent enforcement with read mapping.
- `task-sections`: clarify render-only fallback for missing section values (display under Now, write only on explicit move/edit).

## Impact

- `src/main.rs` (sole binary crate): `Task` struct, `mock_tasks()` removal, new backend module, all mutation handlers, `completed_rows()`/`open_rows()` filters, due/overdue logic, keymap (`r` refresh).
- `Cargo.toml`: adds `serde_json` (+ `chrono` for due parsing).
- `docs/main.md`: rewrite backend section from mock to CLI truth.
- No changes to ratatui/crossterm UI chrome, views, or keymap beyond `r`.
