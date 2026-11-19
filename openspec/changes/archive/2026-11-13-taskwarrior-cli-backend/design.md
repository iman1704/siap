# Design

## Context

See proposal.md Why. Current: `src/main.rs` is a single-file binary (~4000 lines) with `mock_tasks()` and string-based due checks (`contains("overdue")`). Live env: task 3.5.0, data at `~/.config/task/data`, 12 pending of ~71 total, no `uda.siapsection` yet, `priority` often absent, `depends` is a UUID list, `start` timestamp marks actives, `urgency` is computed `f64`. Event loop is single-threaded with 120ms key poll; all mutations are in-memory today.

## Goals / Non-Goals

**Goals:**
- Replace mocks with CLI-backed reads/writes keyed by `uuid` with zero surprise writes on first launch.
- Keep the existing UI chrome, views, and keymap intact (plus `r` refresh).

**Non-Goals:**
- No background sync daemon, file watcher, or taskchampion replica.
- No `recur`/`annotations` editing; no sync indicator or alerts.

## Decisions

- **CLI subprocess over taskchampion library.** `task export` JSON for reads, `task <uuid> modify/add/done/delete/start/stop` for writes, `task config` for UDA bootstrap. Rationale: matches installed 3.5.0 semantics (UDA, urgency, GC) with no replica/sync state; subprocess cost trivial at this scale. Alternative (taskchampion) rejected: faster but adds sync model and a heavy new dep against AGENTS.md simplicity rule.
- **New `backend` module in `src/main.rs` split.** Extract `Task { uuid, short_id: Option<u32>, ... }` with `parent: Option<String>` plus `export_pending()`, `export_completed()`, `ensure_section_uda()`, `run_write()` returning `Result` with stderr captured for toasts. Re-key `selected: HashSet<String>`, `cursor_id() -> Option<String>`, `children()`, `Popup.task_id`. Alternative (keep `u32` + uuid map) rejected: two keys drift after every add/complete.
- **Two exports.** `task +PENDING export` for open rows, `task +COMPLETED export` sorted by `end` desc capped at 10 for the tail; `deleted` never fetched. Rationale: bounds the tail against full history while reusing current view filters.
- **Render-only section fallback.** Missing/empty `siapsection` displays under Now (first section per view) but is written only on explicit `m`/edit. UDA registered as string type with values unconstrained so `A`/`R`/`X` stay UI-only. Alternative (backfill on launch) rejected: would rewrite `modified` on every legacy task. Concrete name drops the dot: task 3.5 parses a dotted `siap.section` attribute as description text, so `siapsection` is the usable form (reads still accept dotted keys as fixtures).
- **Subtask direction: parent depends on children.** New subtask flow: `task add` child, then `task <parent-uuid> modify depends+=<child-uuid>`; single-parent enforced by moving the link. Read: invert `depends` (find the task listing this child) rather than reading `child.depends.first()`. Rationale: matches taskwarrior blocking semantics and the existing `blocked` marker; preserves delete-cascade. Extra deps beyond the link are left untouched until that task is edited.
- **Due via `chrono`, local calendar.** Parse `YYYYMMDDTHHMMSSZ` to local day; display `today HH:MM` / `today` / `tomorrow` / weekday / `YYYY-MM-DD` / `Xd overdue`, absent to `—`. Overdue = due-date < local today; due-today = same local day. Replaces substring hacks; Today-view rule unchanged.
- **Refresh = launch + after-write + `r`.** Synchronous re-export after each successful write; failures toast with stderr and keep old state. No polling thread. `r` is free (only `R` in use). Rationale: fits the single-threaded loop and stays readable; avoids list-jump UX.
- **Tests via temp `TASKDATA`.** Headless `TestBackend` smoke tests set `TASKDATA`/`TASKRC` to a temp dir with fixture JSON so no live data is touched. Covers mapping, section fallback, tail cap, uuid cursor hold, and failure toast.

## Risks / Trade-offs

- [Risk] `task` binary missing or old → Mitigation: launch check with clear toast and empty-list state; document MSRV on task 3.x.
- [Risk] Concurrent external mutation between write and re-export → Mitigation: last-writer-wins with immediate re-export; `r` recovers; no merge UI in this change.
- [Risk] `depends` values created externally with multiple parents → Mitigation: render first link, preserve rest, overwrite only on explicit parent edit with toast.
- [Risk] Timezone edge (due near midnight UTC) → Mitigation: all day math in local time; tests pin Asia/Kuching plus UTC cases.
- [Risk] `export` schema drift across task versions → Mitigation: `serde_json` with optional fields and defaults; unknown fields ignored.

## Migration Plan

1. Land behind no flag; first launch registers UDA and shows live data (mocks removed).
2. No data migration; missing sections render without writes.
3. Rollback: revert binary; task data remains valid taskwarrior data (section UDA harmless if left registered).

## Open Questions

- None blocking. Tail cap (10) and due weekday window are constants tunable after dogfooding without spec changes.
