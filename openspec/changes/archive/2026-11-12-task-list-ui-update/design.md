# Design

## Context

See `proposal.md` for motivation. Current state in `src/main.rs`: `View::Inbox::matches` plus a second inline Inbox filter both require `pending + project empty + due empty`; `open_rows()` sorts section-then-urgency for all views; the list renderer emits section headers for all views; `row_line()` always shows `● project` and appends `● active` text; row background tints on `t.active || cursor || selected`.

## Goals / Non-Goals

**Goals:**
- One filter expression for Inbox shared by matching, counting, and rendering.
- View-aware row rendering (project dot, section headers, sort) with Inbox as the single exception.
- Active state reduced to one signal without weakening cursor/selection visibility.

**Non-Goals:**
- No backend, persistence, or keymap redesign beyond Inbox guards on `m`/`A`/`R`/`X`.
- No popup layout changes (it already shows project; keep as-is).
- No new views, sections model, or sorting keys.

## Decisions

- **Single `is_inbox(t)` helper**: replace both Inbox predicates with `is_open(t) && t.project.is_empty()`. Rationale: the two call sites (`View::matches`, render filter) already drifted; one helper prevents a repeat. Alternative (edit each site separately) rejected as drift-prone.
- **Branch sort + headers on `view == Inbox`**: `open_rows()` returns urgency-desc only for Inbox; renderer skips the `last_section` header block for Inbox but keeps parent-child indent. Rationale: smallest diff, section values stay stored for later project assignment. Alternative (strip section on Inbox entry) rejected — it would destroy organization the user already did.
- **Pass `hide_project: bool` into `row_line()`** (true when `view` is `Project(_)`): suppress only the dot span. Rationale: keeps `row_line` pure and testable; popup code untouched. Alternative (global flag) rejected as hidden coupling.
- **Active = checkbox only**: keep `◉` + mint, delete the `● active` suffix branch and drop `t.active` from the tint condition. Rationale: `◉` is the only signal that survives cursor moves and costs no horizontal space; tint is then unambiguous for cursor/selection. Alternative (keep tint, drop checkbox) rejected — tint collides with cursor exactly as reported.

## Risks / Trade-offs

- [Existing `smoke` tests assert tint + `● active` + Inbox due rule] → Update assertions alongside code; keep one test per changed rule.
- [Cursor on active row could look plain without tint] → Cursor keeps `›` + bright/bold + tint, so active+cursor still reads as selected; covered by new cursor-on-active scenario.
- [Users relying on due-less Inbox triage] → Accepted: due-carrying project-less tasks now stay in Inbox by design; Today still surfaces them by due date.

## Migration Plan

Mock-data only, no data migration. Change code, update `smoke` tests, update `docs/main.md` views/sections/list/active wording in the same change. Rollback is revert of the change.

## Open Questions

None — subtask indent in flat Inbox is kept (nesting is hierarchy, not sections).
