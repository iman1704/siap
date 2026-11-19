# Spec Delta

## Purpose

Defines single and bulk task lifecycle operations (start/stop, done, delete, move, tags) with the minimal vim keymap.

## ADDED Requirements

### Requirement: Single start and stop

The system SHALL support `s` to toggle the highlighted task between `status:active` and `status:pending`, updating its active marker and full-row tint.

#### Scenario: Start a task

- **WHEN** the user presses `s` on a pending task
- **THEN** it becomes active, gains the tint, and appears in Today while active

### Requirement: Single complete

The system SHALL support `x` to complete the highlighted task, moving it from the open rows to the dimmed `COMPLETED` tail.

#### Scenario: Complete a task

- **WHEN** the user presses `x` on a pending task
- **THEN** its status becomes completed and it renders under COMPLETED

### Requirement: Delete confirms

The system SHALL support `d` to request deletion of the highlighted task, applying it only on `y` and cancelling on `n` or `esc`.

#### Scenario: Cancel a delete

- **WHEN** the user presses `d` then `n`
- **THEN** the task remains unchanged

### Requirement: Space and visual multi-select

The system SHALL toggle selection with `space` and support `v` VISUAL mode where `j/k` extend the range, showing a bar with the selection count and available bulk actions.

#### Scenario: Select three tasks

- **WHEN** the user presses `space` on three separate rows
- **THEN** the bar shows `3 selected` with the bulk actions

### Requirement: Bulk delete move and tags

The system SHALL apply bulk `d` delete, `m` move-project, `t` edit-tags, and `x` complete to every selected task, then clear the selection. `esc` cancels without changes.

#### Scenario: Bulk move project

- **WHEN** the user selects two Inbox tasks and moves them to `personal`
- **THEN** both gain `project:personal` and leave Inbox

### Requirement: List edit key only hints

The system SHALL NOT edit inline from the list on `e`; it reports that editing happens after opening the task with `Enter`.

#### Scenario: Edit from the list

- **WHEN** the user presses `e` with the cursor on a row
- **THEN** a hint explains to open the task first

### Requirement: Minimal keymap only

The system SHALL implement exactly `j/k` move, `space` select, `Enter` open, `a` new, `e` hint, `x` done, `s` start/stop, `d` delete, `m` move, `t` tags, `/` find, `?` help, `q` quit, plus VISUAL `v` range, popup `n` subtask, and `esc` cancel, with the bar hints and `?` overlay matching the implementation.

#### Scenario: Help matches keys

- **WHEN** the user presses `?`
- **THEN** the overlay lists only the keys above with correct descriptions
