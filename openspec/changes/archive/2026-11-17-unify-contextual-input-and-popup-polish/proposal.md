# Proposal

## Why

The staged `src/main.rs` work already unifies the add box and popup editor around one contextual-input language (shared examples, explicit clear rows, shared suggestion cap) while making popup `status` read-only, relabeling priorities `P1/P2/P3` → `H/M/L`, and surfacing sync and blocked-task feedback. Without a spec, the existing `task-editor`, `task-list`, `task-views`, `task-operations`, and `taskwarrior-backend` requirements still describe the old seven-field editor, `P3` default, `no date`-only clearing, silent sync, and silent blocked completion. This proposal records the staged behavior as intended spec deltas.

## What Changes

- Popup editor drops to six editable fields (description, project, due, priority, tags, depends); `status` becomes read-only in the grey block with urgency, start, recur, and annotations.
- Priority labels render as `H` / `M` / `L` everywhere (list rows, add-box chips, popup field, suggestions, `[`/`]` cycling); absent priority renders `L`.
- Shared contextual-input language between add box and popup: constants for examples (`e.g. work`, `e.g. work, urgent`, `e.g. 2026-11-20`), clear rows (`no project`, `no tags`, `no date`, `no dependencies`), and shared suggestion cap of 5.
- Clear rows lead project (`no project` first) and tags (`no tags` first) suggestion lists in both surfaces; `no dependencies` always closes the depends list even from an empty draft; applying a clear row empties that draft field.
- Popup `depends` `Enter` applies the highlight (candidate appends, `no dependencies` clears) then saves back to `VIEW` at once; the old add-one-and-stay-in-`EDIT` flow is removed.
- Popup `EDIT` renders a contextual block under the focused field mirroring the add box: separate input line with `▌` cursor (grey example when empty), `now:` reference for project/priority, then the capped suggestion list; priority is picker-only with no input line.
- Header top line appends sync state (`synced HH:MM:SS` dim, `sync failed` yellow, `not synced` initial); successful and failed backend refreshes update it.
- Completing blocked tasks still completes but toasts `completed N task(s) · M blocked!`; save toast shortens from `saved · esc closes` to `saved`; popup title and footers drop the `esc closes` hints.
- Popup `d` done-toggle recomputes dirty from remaining draft fields so a saved toggle never shows a stale `● unsaved`; project `[`/`]` cycling includes the `no project` clear row.
- Sidebar chrome trims the `taskwarrior, tamed` tagline and trailing `uda.siapsection` label, replacing the section separator blank with a dim `· · · · · · ·` rule.

## Capabilities

### New Capabilities

- None — this change only revises existing editor, list, views, operations, and backend behaviors.

### Modified Capabilities

- `task-editor`: six-field editor with read-only `status`, `H/M/L` labels, shared examples/clear rows/cap, contextual edit block, `depends` apply+save, `saved` toast, dirty-safe done toggle.
- `task-list`: quick-add `no project` / `no tags` clear rows, shared examples and cap, `H/M/L` chips and labels.
- `task-operations`: blocked-completion `· N blocked!` suffix, `saved` toast wording.
- `task-views`: header sync state, sidebar separator/tagline cleanup.
- `taskwarrior-backend`: absent priority renders `L` (was `P3`).

## Impact

- Affected code: `src/main.rs` only (staged diff, ~474 insertions / 220 deletions); smoke tests in the same file updated to match.
- No new dependencies, APIs, config keys, or data migrations; backend writes unchanged (`modify`, per-UUID `depends` add/remove).
- Risk: label rename `P1/P2/P3` → `H/M/L` and seven-to-six field change touch many golden strings and keymap expectations; specs pin the new strings so regressions surface in tests.
