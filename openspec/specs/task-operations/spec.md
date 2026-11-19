# task-operations Specification

## Purpose

Defines single and bulk task lifecycle operations (start/stop, done, delete, move, tags) with the minimal vim keymap.

## Requirements

### Requirement: Single start and stop

The system SHALL support `s` to toggle the highlighted task between `status:active` and `status:pending`, updating its start indicator. Active state MUST NOT add full-row tint; tint is reserved for cursor and selection.

#### Scenario: Start a task

- **WHEN** the user presses `s` on a pending task
- **THEN** it becomes active, gains the mint `◉` start indicator, and appears in Today while active with no full-row tint

#### Scenario: Stop a task

- **WHEN** the user presses `s` on an active task
- **THEN** it returns to pending, its start column becomes blank, and it leaves Today unless it is otherwise due or overdue

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

### Requirement: Delete confirms

The system SHALL support `D` to request deletion of the highlighted task, applying it only on `y` and cancelling on `n` or `esc`.

#### Scenario: Cancel a delete

- **WHEN** the user presses `D` then `n`
- **THEN** the task remains unchanged

### Requirement: Space and visual multi-select

The system SHALL toggle selection with `space` and support `v` VISUAL mode where `j/k` extend the range, showing a bar with the selection count and available bulk actions.

#### Scenario: Select three tasks

- **WHEN** the user presses `space` on three separate rows
- **THEN** the bar shows `3 selected` with the bulk actions

### Requirement: Bulk delete move and tags

The system SHALL apply bulk `D` delete, `m` move-project, `t` edit-tags, and `d` done-toggle to every selected task, then clear the selection. `esc` cancels without changes.

#### Scenario: Bulk move project

- **WHEN** the user selects two Inbox tasks and moves them to `personal`
- **THEN** both gain `project:personal` and leave Inbox

#### Scenario: Bulk done-toggle mixes states

- **WHEN** the user selects one open and one completed task and presses `d`
- **THEN** the open task completes and the completed task reopens

### Requirement: List edit key only hints

The system SHALL NOT edit inline from the list on `e` and SHALL NOT show any hint toast; list editing happens only after opening the task with `Enter`.

#### Scenario: Edit from the list

- **WHEN** the user presses `e` with the cursor on a row
- **THEN** no toast appears, no popup opens, and the list is unchanged

### Requirement: List keymap stays minimal

The system SHALL keep the list keymap to `j/k`, `space`, `Enter`, `a`, `d`, `s`, `D`, `u`, `m`, `t`, `?`, `q`, `1`-`5`, `/`, `i`/`esc` for Search input, `v` range, and `esc`, plus `Ctrl+z` undo. `r` SHALL stay available as the unhinted manual refresh key, and `n` SHALL be a no-op everywhere. The `?` overlay SHALL list the current keymap with correct descriptions.

#### Scenario: Help matches keys

- **WHEN** the user presses `?`
- **THEN** the overlay lists the current keymap with correct descriptions

#### Scenario: Slash opens Search

- **WHEN** the user presses `/` in a list view
- **THEN** the Search view opens in input mode with the query ready to type and no separate find mode opens

#### Scenario: Subtask hotkey works from the list

- **WHEN** the user presses `n` with the cursor on a task row
- **THEN** no task is created and the list is unchanged

#### Scenario: List n does nothing

- **WHEN** the user presses `n` with the cursor on a task row
- **THEN** no task is created and the list is unchanged

### Requirement: Footer hints stay minimal

The system SHALL render the list footer as `j/k move · space select · a new · d done · s start · D delete · u undo · / search · ? help · q quit`, without the old `Enter` or `r` hints.

#### Scenario: Footer hints stay minimal

- **WHEN** the list footer renders
- **THEN** it advertises `D delete` and `/ search` and omits the old `Enter open/toggle` and `r refresh` hints

### Requirement: Help overlay groups keys by surface

The system SHALL render the `?` overlay with `List`, `Edit`, and `Search` group headings, listing the current list, popup, and Search keys under them and omitting the removed `Enter open/toggle` and `r refresh from backend` lines.

#### Scenario: Group headings render

- **WHEN** the user opens the `?` overlay
- **THEN** `List`, `Edit`, and `Search` headings separate the key groups

#### Scenario: Removed hints are gone

- **WHEN** the help overlay renders
- **THEN** no `Enter open/toggle` or `r refresh` line appears

#### Scenario: Note keys are listed

- **WHEN** the help overlay renders
- **THEN** it lists `a note`, `D del note`, and `[/] select` for the popup annotation row

### Requirement: Delete is single-only

The system SHALL delete exactly the confirmed tasks on `D` + `y` with no cascade to other tasks; related `depends` links resolve on the next backend refresh.

#### Scenario: Delete leaves blockers alone

- **WHEN** the user deletes a blocked task with an open blocker
- **THEN** only the confirmed task leaves and the blocker stays open

### Requirement: Undo reverts the last change

The system SHALL revert the most recent list mutation on `u` (or `Ctrl+z`) and report it, with only one level of history: a second undo without an intervening change SHALL report that there is nothing to undo.

#### Scenario: Undo restores a toggled task

- **WHEN** the user completes a task then presses `u`
- **THEN** the task returns to its previous status

#### Scenario: Single-level undo exhausts

- **WHEN** the user presses `u` twice with no change in between
- **THEN** the second press reports that there is nothing to undo

### Requirement: Progress toasts expire

The system SHALL clear a progress or result toast after about 2.5 seconds so messages such as `saved`, `edits discarded`, or `completed 1 task(s)` never persist indefinitely.

#### Scenario: Toast clears itself

- **WHEN** a toast is set and the user makes no further change
- **THEN** the bar clears after about 2.5 seconds

#### Scenario: New feedback restarts the timer

- **WHEN** a second toast replaces the first before it expires
- **THEN** the second toast stays visible for its own ~2.5 seconds
