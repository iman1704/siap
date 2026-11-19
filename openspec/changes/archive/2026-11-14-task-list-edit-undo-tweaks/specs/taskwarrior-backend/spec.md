# Spec Delta

## MODIFIED Requirements

### Requirement: Writes persist through the backend

The system SHALL persist quick-add, popup saves, start and stop, done, undone (`modify status:pending`), undo, delete with child cascade, section moves, subtask creation, and bulk operations to the backend store.

#### Scenario: Done persists across restart

- **WHEN** the user completes a task and restarts the app
- **THEN** the task remains completed

#### Scenario: Undone persists across restart

- **WHEN** the user reopens a completed task and restarts the app
- **THEN** the task remains pending

#### Scenario: Delete removes children

- **WHEN** the user deletes a parent with open subtasks
- **THEN** the parent and its children leave the open list together

## ADDED Requirements

### Requirement: Undo reverts the last backend change

The system SHALL revert the most recent backend mutation on `u` (or `Ctrl+z`) via the backend `undo` command, then re-export and hold the cursor by stable identity. A failed undo SHALL toast and preserve the list.

#### Scenario: Undo after done

- **WHEN** the user completes a task then presses `u`
- **THEN** the task returns to pending

#### Scenario: Failed undo preserves state

- **WHEN** the backend undo command fails
- **THEN** the toast explains the failure and no rows change

### Requirement: Start display derives from the real start timestamp

The system SHALL read the task's real `start` timestamp from the export and derive the popup `start` row from it; tasks with no `start` value SHALL show a blank start.

#### Scenario: Started task shows age from export

- **WHEN** the export carries a `start` timestamp one hour in the past
- **THEN** the popup shows a relative age for that task

#### Scenario: Missing start stays blank

- **WHEN** the export carries no `start` timestamp
- **THEN** the popup start value is blank
