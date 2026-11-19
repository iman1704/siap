# Spec Delta

## MODIFIED Requirements

### Requirement: Identity is stable across refreshes

The system SHALL key selection, cursor, and popup by stable task UUID identity that survives refreshes and external id shifts.

#### Scenario: Cursor survives refresh

- **WHEN** a refresh reorders numeric ids
- **THEN** the cursor stays on the same logical task

### Requirement: Writes persist through the backend

The system SHALL persist quick-add, popup saves (including per-UUID `depends` add/remove), start and stop, done, undone (`modify status:pending`), undo, single-only delete, section moves, and bulk operations to the backend store.

#### Scenario: Done persists across restart

- **WHEN** the user completes a task and restarts the app
- **THEN** the task remains completed

#### Scenario: Undone persists across restart

- **WHEN** the user reopens a completed task and restarts the app
- **THEN** the task remains pending

#### Scenario: Delete removes children

- **WHEN** the user deletes a task that participates in `depends`
- **THEN** only that task leaves the open list and relations resolve on refresh

#### Scenario: Delete removes only the confirmed tasks

- **WHEN** the user deletes a task that participates in `depends`
- **THEN** only that task leaves the open list and relations resolve on refresh

## ADDED Requirements

### Requirement: Depends maps flat from export

The system SHALL read native `depends` UUID arrays from `task export` directly into each task with no parent inversion, dropping `deleted` rows; a task whose dependency UUIDs are all completed or deleted SHALL render unblocked.

#### Scenario: Flat export keeps both rows

- **WHEN** the export shows task A depending on open task B
- **THEN** both render as top-level rows with A marked `blocked`

#### Scenario: Deleted rows never render

- **WHEN** the export contains a deleted task
- **THEN** it appears in no view and holds no relations
