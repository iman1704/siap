# Design

## Context

See `proposal.md` — Why. The staged `src/main.rs` diff already implements this change; the design records the approach so review and follow-ups stay consistent. Current state: popup `Popup` holds seven draft fields including `status`; priority helpers map `H/M/L` to `P1/P2/P3`; add box and popup each hardcode their own examples, clear strings, and `truncate(5)`; `depends` Enter adds one and stays in `EDIT`; header renders only wordmark plus totals; `toggle_ids` toasts plain counts; `save_popup` toasts `saved · esc closes`.

## Goals / Non-Goals

**Goals:**
- Unify add box and popup contextual input on one constant set with identical user-visible strings and cap.
- Make popup `status` read-only without losing visibility, and keep done-toggles saved-clean.
- Surface sync health and blocked-completion signal without new views or keys.

**Non-Goals:**
- No backend schema, filter, sort, or write-path changes beyond label mapping already staged.
- No new pickers, keys, or views; `[`/`]` and `Enter`/`esc` semantics stay except as specified.
- No migration of stored `P1/P2/P3` values (labels only; stored `H/M/L` unchanged).

## Decisions

- **Single constant module for contextual language (`CTX_SUGGEST_CAP`, `CTX_CLEAR_*`, `CTX_EXAMPLE_*`) over per-surface literals.** Staged code already does this; rationale: one edit changes both surfaces and tests can assert equality (e.g. add-box `no tags` equals popup `no tags`). Alternative (duplicate strings) rejected — it caused the prior drift.
- **Clear rows as data, not footer hints (`no project` index 0, `no tags` index 0, `no dependencies` last).** Rationale: one highlight + `Enter` clears, matching how `no date` already worked; keeps mouse-free flow. Alternative (dedicated clear key) rejected — new keymap for a rare action.
- **Depends `Enter` = apply + save (returns `bool`), not add-and-stay.** Rationale: every other field already applies-then-saves; the two-step `Enter…Enter again` was the only exception and confused the dirty flag. Trade-off: multi-add needs re-entering `EDIT`, accepted as rarer than single-add.
- **Status read-only (field count 7→6, `F_STATUS` removed).** Rationale: `status` is derived from done/active flows and backend `start`; editing it directly bypassed `start` timestamps. `d` inside the popup remains the supported toggle and now recomputes dirty so the title never shows stale `● unsaved`.
- **Header sync fields (`last_sync: Option<DateTime<Local>>`, `sync_ok: bool`).** Rationale: cheapest observable health signal; set on both success and failure paths of `refresh_from_backend`, rendered dim/yellow. No background polling introduced.
- **Priority relabel at presentation (`prio_label` → `H/M/L`) only.** Stored chars `H/M/L` already; this only changes what the user sees plus suggestion/cycling strings. Tests assert ` H ` / ` M ` / ` L ` row chips.

## Risks / Trade-offs

- [Risk] `H/M/L` rename breaks muscle memory and golden-string tests → Mitigation: specs pin new strings; staged smoke tests already updated.
- [Risk] Depends single-step save surprises users adding several deps → Mitigation: footer explicitly reads `Enter apply+save`; toast no longer promises `Enter again`.
- [Risk] Sync `HH:MM:SS` causes snapshot churn → Mitigation: assert presence of `synced` / `sync failed` prefixes, not exact time, except format check.
- [Risk] Removing `status` draft field orphans external scripts setting popup status → Mitigation: no external API; backend `modify status:` paths untouched, only the popup draft loses the field.

## Migration Plan

- Land staged `src/main.rs` as-is; no data migration.
- Rollback: revert the change directory deltas plus the staged file; old `P1/P2/P3` and seven-field specs in `openspec/specs/` remain the fallback contract until archive.
- Follow-ups (out of scope): background sync polling, blocked-completion confirmation prompt (currently warn-but-complete).

## Open Questions

- None that change specs or tasks. Wording of `· N blocked!` versus a blocking confirm can be revisited after usage.
