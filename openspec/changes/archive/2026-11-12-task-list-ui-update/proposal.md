# Proposal

## Why

The task list UI over-signals and over-groups: Inbox mixes project-less scoping with due-date filtering, Inbox inherits section chrome it does not need, project rows repeat context the sidebar already gives, and active tasks shout three times (tint + mint checkbox + text). Simplifying these four display rules makes each view's meaning obvious at a glance.

## What Changes

- Inbox filter becomes "open tasks with no project, due date ignored" (pending or active, empty `project`).
- Inbox renders as a flat urgency-ordered list with no section headers or blank separators; `m` / `A` / `R` / `X` are disabled in Inbox with a hint, hidden `section` values are preserved.
- Project views hide the `● project` dot in list rows; Today, All keep it; the Enter popup always shows the project field in every view.
- Active tasks use exactly one indicator: the mint `◉` checkbox. Full-row tint and `● active` text are removed; tint returns to cursor/selection only.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `task-views`: Inbox filter requirement changes (drop due-date condition, include active).
- `task-sections`: default/move/spacing requirements gain an Inbox exception (flat, no headers, actions disabled).
- `task-list`: row-attribute requirement (conditional project dot), active-indicator requirement (checkbox only), sorting requirement (Inbox urgency-only exception).

## Impact

- `src/main.rs`: `View::Inbox::matches`, `open_rows()` sort, list render section-header branch, `row_line()` project-dot branch + active-marker branch, tint assignment, `m`/`A`/`R`/`X` guards, `smoke` tests.
- `docs/main.md`: views/sections/list/active wording follows the spec deltas.
- No taskwarrior backend, persistence, or keymap changes beyond Inbox guards.
