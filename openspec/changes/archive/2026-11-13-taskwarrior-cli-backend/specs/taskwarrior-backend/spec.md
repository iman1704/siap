# Spec Delta

## Purpose

Live taskwarrior data replaces mock tasks so the list, popup, and operations act on the user's real pending and recently completed work.

## ADDED Requirements

### Requirement: Open rows come from pending export

The system SHALL populate open rows from pending and started tasks and exclude deleted tasks from every view.

#### Scenario: Deleted tasks never render

- **WHEN** the data contains tasks with status deleted
- **THEN** no deleted task appears in any view or count

#### Scenario: Started tasks stay open

- **WHEN** a pending task has a start timestamp
- **THEN** it renders as active and stays in open rows

### Requirement: Completed tail shows recent completions

The system SHALL render the dimmed COMPLETED tail from recently completed tasks sorted by completion time descending and capped at ten, still filtered by the current view.

#### Scenario: Tail is capped by recency

- **WHEN** more than ten tasks match the tail filter
- **THEN** only the ten most recently completed render

#### Scenario: Tail respects the current view

- **WHEN** the user opens a project view
- **THEN** the tail shows only recent completions from that project

### Requirement: Identity is stable across refreshes

The system SHALL key selection, cursor, popup, and parent links by stable task identity that survives refreshes and external id shifts.

#### Scenario: Cursor survives refresh

- **WHEN** a refresh reorders numeric ids
- **THEN** the cursor stays on the same logical task

### Requirement: Due display derives from real dates

The system SHALL derive the friendly due label and overdue and due-today markers from the real due timestamp using the local calendar day, with absent due rendering as em-dash.

#### Scenario: Overdue derives from the calendar

- **WHEN** a task's due date is before the local today
- **THEN** it renders an overdue label and marker and appears in Today

#### Scenario: Missing due stays neutral

- **WHEN** a task has no due timestamp
- **THEN** it renders an em-dash with no marker

### Requirement: Priority defaults and urgency stay read-only

The system SHALL render absent priority as P3 and SHALL treat urgency as a read-only computed value that refreshes from the backend.

#### Scenario: Missing priority renders P3

- **WHEN** a task has no priority value
- **THEN** the row shows P3 styling

#### Scenario: Urgency refreshes after save

- **WHEN** the user changes priority and saves
- **THEN** the displayed urgency updates on the next refresh

### Requirement: Writes persist through the backend

The system SHALL persist quick-add, popup saves, start and stop, done, delete with child cascade, section moves, subtask creation, and bulk operations to the backend store.

#### Scenario: Done persists across restart

- **WHEN** the user completes a task and restarts the app
- **THEN** the task remains completed

#### Scenario: Delete removes children

- **WHEN** the user deletes a parent with open subtasks
- **THEN** the parent and its children leave the open list together

### Requirement: Section store self-registers

The system SHALL register the section store on launch when missing without user action and never require manual configuration commands.

#### Scenario: First launch registers silently

- **WHEN** the app launches and the section store is missing
- **THEN** it registers and the list still renders

### Requirement: Refresh follows launch write and manual key

The system SHALL load live data on launch, reload after every successful write, and reload on the manual refresh key for external changes.

#### Scenario: Manual refresh picks up external edits

- **WHEN** the user presses the refresh key after editing tasks elsewhere
- **THEN** the list reflects the external state with cursor held by stable identity

### Requirement: Failures toast and preserve state

The system SHALL report a failed backend command in the toast and keep the existing list, selection, and cursor unchanged.

#### Scenario: Failed write keeps the list

- **WHEN** a write command fails
- **THEN** the toast explains the failure and no rows change

### Requirement: Deferred fields stay read-only

The system SHALL display recurrence and annotation values as read-only and SHALL NOT offer editing for them in this change.

#### Scenario: Deferred fields show without editing

- **WHEN** the popup renders recurrence and annotations
- **THEN** their values are visible and no edit affordance is offered
