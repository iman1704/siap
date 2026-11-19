# Spec Delta

## MODIFIED Requirements

### Requirement: Single complete

The system SHALL support `d` to toggle the highlighted task between done and undone: an open task completes and moves to the dimmed `COMPLETED` tail, while a completed task reopens to `status:pending` and returns to the open rows. The cursor follows the toggled task.

#### Scenario: Complete a task

- **WHEN** the user presses `d` on a pending task
- **THEN** its status becomes completed and it renders under COMPLETED

#### Scenario: Reopen a completed task

- **WHEN** the user presses `d` on a completed row
- **THEN** its status becomes pending and it returns to the open rows

### Requirement: Delete confirms

The system SHALL support `D` to request deletion of the highlighted task, applying it only on `y` and cancelling on `n` or `esc`.

#### Scenario: Cancel a delete

- **WHEN** the user presses `D` then `n`
- **THEN** the task remains unchanged

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

### Requirement: Minimal keymap only

The system SHALL implement exactly `j/k` move, `space` select, `Enter` open/toggle, `a` new, `d` done/undone, `s` start/stop, `D` delete, `u` undo, `m` move, `t` tags, `/` find, `?` help, `q` quit, plus `Ctrl+z` undo, VISUAL `v` range, popup `n` subtask, and `esc` cancel, with the bar hints and `?` overlay matching the implementation.

#### Scenario: Help matches keys

- **WHEN** the user presses `?`
- **THEN** the overlay lists only the keys above with correct descriptions

## ADDED Requirements

### Requirement: Undo reverts the last change

The system SHALL revert the most recent list mutation on `u` (or `Ctrl+z`) and report it, with only one level of history: a second undo without an intervening change SHALL report that there is nothing to undo.

#### Scenario: Undo restores a toggled task

- **WHEN** the user completes a task then presses `u`
- **THEN** the task returns to its previous status

#### Scenario: Single-level undo exhausts

- **WHEN** the user presses `u` twice with no change in between
- **THEN** the second press reports that there is nothing to undo
