# Design

## Context

See `proposal.md` for why. Current staged code already implements the simplification: `src/backend.rs` dropped `task_output_is_error` / `clean_task_output`; `src/main.rs` dropped `SetCursorStyle`, `place_native_cursor`, `insert_at` / `backspace_at`, per-input cursors, and multi-batch popup save; `docs/backend.md` and `docs/main.md` (unstaged) already describe the simplified behavior. This design records the approach so review can confirm the staged diff matches the deltas.

## Goals / Non-Goals

**Goals:**
- Document how append-only editing, inline `▌` rendering, exit-status-only errors, and single-batch save fit together.
- Preserve existing picker, view, and refresh behavior outside text-cursor movement.

**Non-Goals:**
- No restoration of mid-text editing, native cursor, or exit-0 sniffing.
- No change to suggestion filtering, sorting, section registry, or undo semantics.

## Decisions

- **Inline `▌` over native cursor:** render `format!("{}▌", text)` on focused lines and `e.g. ...▌` on empty examples; drop `f.set_cursor_position` and `SetCursorStyle` handling. Alternative (restore steady-bar native cursor) rejected: extra terminal-state management and off-screen clamping for little benefit.
- **Append-only (`push`/`pop`) over char-index cursors:** remove all `*_cursor` fields and helpers; `Left`/`Right` become no-ops (consumed in Search so sidebar focus never steals typing). Alternative (keep mid-text insert) rejected: multiplied state and render-offset bugs.
- **Exit-status-only backend errors:** `run_write_with_env` returns `Ok(stdout)` on success, else trimmed `stderr`/`stdout`. Alternative (keep substring sniffing) rejected: fragile keyword list, stripped override lines, and false positives.
- **Single-batch popup save:** one `build_modify_args` call; no separate `status:` write. Alternative (keep status batch + depends guard) rejected: extra write path; depends remains unwritten because `build_modify_args` never includes it. Trade-off: popup status edits made in the same save are not persisted — recorded explicitly in the spec delta.

## Risks / Trade-offs

- [Status edits silently dropped] → Mitigation: spec delta states single-modify scope; follow-up change should either wire status through `build_modify_args` or make status picker-only with its own write.
- [Exit-0 validation now counts as success] → Mitigation: spec delta makes exit-status the contract; invalid dates/priorities that exit 0 will refresh as if saved until taskwarrior returns non-zero.
- [No mid-text correction] → Mitigation: accepted UX cost; users retype or use external `$EDITOR` via `E`.
- [Staged code already implements this] → Mitigation: tasks below are verification + doc-alignment, not re-implementation; no new logic unless review finds gaps.

## Migration Plan

- Land staged `src/` changes as-is after spec/design review.
- Land unstaged `docs/` + `spec-update.md` alignment in the same change.
- Rollback: revert the two `src/` files; specs/design/tasks revert with the change directory.

## Open Questions

- None — popup status persistence needs a separate decision and is out of scope here.
