# Spec Delta — `task-operations`

## MODIFIED Requirements

### Requirement: Single complete

The system SHALL support `d` to toggle the highlighted task between done and undone: an open task completes and moves to the dimmed `COMPLETED` tail, while a completed task reopens to `status:pending` and returns to the open rows. The cursor follows the toggled task. When any completed task was waiting on open dependencies, the toast SHALL append ` · N blocked!` where N counts those tasks; completion still succeeds because the backend enforces no blocking.

#### Scenario: Complete a task

- **WHEN** the user presses `d` on a pending task
- **THEN** its status becomes completed and it renders under COMPLETED

#### Scenario: Reopen a completed task

- **WHEN** the user presses `d` on a completed row
- **THEN** its status becomes pending and it returns to the open rows

#### Scenario: Blocked completion warns but succeeds

- **WHEN** the user completes two tasks where one waits on an open dependency
- **THEN** both complete and the toast reads `completed 2 task(s) · 1 blocked!`

### Requirement: Progress toasts expire

The system SHALL clear a progress or result toast after about 2.5 seconds so messages such as `saved`, `edits discarded`, or `completed 1 task(s)` never persist indefinitely.

#### Scenario: Toast clears itself

- **WHEN** a toast is set and the user makes no further change
- **THEN** the bar clears after about 2.5 seconds

#### Scenario: New feedback restarts the timer

- **WHEN** a second toast replaces the first before it expires
- **THEN** the second toast stays visible for its own ~2.5 seconds
