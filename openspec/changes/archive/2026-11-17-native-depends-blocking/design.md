# Design

## Context

See proposal.md Why. Current code inverts native `depends` into a single-parent tree (`parent`, `parent_of`, `children()`, `descendants_of()`, indented rows, cascade delete, `n` creation). Staged changes already replace this with `Task.depends: Vec<String>` plus open-only `blocked_by()` / `blocking()` indexes and flat rendering. This design records those decisions for implementation.

## Goals / Non-Goals

**Goals:**
- Codify flat-DAG blocking, ID-draft editing, and single delete as the implemented approach.
- Keep popup/list/backend boundaries stable (pure builders in `backend.rs`, derived indexes in `App`).

**Non-Goals:**
- No hierarchy migration UI, no multi-level undo, no cross-project blocking rules beyond the dropdown scope.
- No changes to section registry, quick-add, search, or theme.

## Decisions

- **Flat `depends` as source of truth over parent inversion.** Store native UUID arrays directly; derive `is_blocked` as any-open-dep. Alternative (first-link inversion) rejected: it hides multi-blockers and forces cascade semantics Taskwarrior does not need.
- **Short-ID draft with UUID resolution.** Display `depends` as sorted `12, 14`; parse tokens on `[, ]`, resolve short-ID exact match first then unique UUID prefix/full UUID; unknown/self toasts without writes. Alternative (raw UUID editing) rejected: untypable and error-prone.
- **Project-scoped, self-excluding candidates.** Dropdown lists open same-project tasks sorted short-ID-first, hiding already-added IDs; `Enter` adds one and stays in EDIT for multi-add. Alternative (global list) rejected: leaks unrelated projects into scoped views.
- **Save diffs per UUID.** After the base `modify`, add missing UUIDs via `task <id> modify depends:<dep>` and remove dropped ones via `depends:-<dep>`; refresh then re-render draft IDs (short IDs can shift). Failures toast `depends failed` and refresh. Alternative (single rewrite) rejected: Taskwarrior has no atomic set for `depends`.
- **Single delete, no cascade.** Delete batches are one arg-set per confirmed ID; relations resolve on re-export. Alternative (descendant walk) rejected: flat DAG has no descendants.
- **Relations as read-only list rows.** `BLOCKED BY` / `BLOCKS` reuse `row_line` without trailer (dim kept); empty sections hidden. Alternative (interactive relation rows) rejected: editing lives in `DEPENDS`.

## Risks / Trade-offs

- [Short-ID shifts after refresh] → Re-render draft display IDs from live UUIDs after save/refresh.
- [Ambiguous UUID prefixes] → Require uniqueness; otherwise toast `unknown id`.
- [Stale drafts across external edits] → Refresh-driven indexes always recompute; draft dirty-check compares UUID sets, unresolvable text counts dirty.
- [Breaking UX for subtask users] → Documented as BREAKING in proposal; help/footer/`?` updated, `n` no-ops.

## Migration Plan

- Land staged `src/backend.rs` + `src/main.rs` as-is; update smoke tests to flat expectations (already in diff).
- No data migration: existing `depends` values render as blockers automatically.
- Rollback: revert change; parent inversion would re-hide extra deps (lossy display only, store untouched).

## Open Questions

- None that change specs or tasks. Follow-up polish (e.g. blocker completion shortcuts) stays out of this change.
