# Spec Delta

## MODIFIED Requirements

### Requirement: Subtasks use depends blocking

The system SHALL implement blocking as native flat `depends` with zero or more blockers per task and no parent link, nesting, or single-parent limit; see `task-dependencies` for the blocking contract. The legacy single-parent subtask hierarchy is retired.

#### Scenario: Parent blocked until children done

- **WHEN** a task lists two open tasks in `depends`
- **THEN** it shows a blocked marker until both complete

#### Scenario: Single parent enforced

- **WHEN** a task lists blockers from different projects
- **THEN** all links persist and no link is moved or dropped

#### Scenario: Extra dependencies stay untouched

- **WHEN** a task carries several `depends` values
- **THEN** every open value counts toward blocking and renders via `BLOCKED BY`

## REMOVED Requirements

### Requirement: Subtasks inherit project and section
**Reason**: Hierarchy retired; native blockers have no inheritance.
**Migration**: Create blockers as normal tasks; link them via `DEPENDS`.

### Requirement: Indented display without counts
**Reason**: Rows stay flat; nesting removed.
**Migration**: Blocked and blocker tasks render as separate top-level rows with the `blocked` trailer.

### Requirement: Deleting a parent deletes its children
**Reason**: Deletes are single-only in the native model.
**Migration**: Delete each task explicitly; the backend strips its UUID from others' `depends`.

### Requirement: Subtasks can be created from the list
**Reason**: `n` subtask creation removed from list and popup.
**Migration**: Use quick-add for new tasks and `DEPENDS` to declare blocking.
