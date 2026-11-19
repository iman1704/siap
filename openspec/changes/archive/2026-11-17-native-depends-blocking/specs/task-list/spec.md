# Spec Delta

## MODIFIED Requirements

### Requirement: Rows never show subtask counts

The system SHALL NOT render `n/m` progress anywhere in the list. A task waiting on open `depends` shows a dimmed description with a `blocked` marker instead.

#### Scenario: Parent shows blocked, not counts

- **WHEN** a blocked task waits on one open and one completed dependency
- **THEN** the row shows `blocked` and no numbers

### Requirement: Overdue has no row trailer

The system SHALL indicate an overdue task through its red due label and Today inclusion only, and MUST NOT append an `overdue` text trailer to the row. The `blocked` trailer SHALL remain for tasks waiting on open dependencies.

#### Scenario: Overdue row reads without a trailer

- **WHEN** an overdue task renders in a list
- **THEN** its due date renders in red and no `overdue` text trails the row

#### Scenario: Blocked trailer still renders

- **WHEN** a task waits on an open dependency
- **THEN** its row shows the `blocked` trailer

## REMOVED Requirements

### Requirement: Children render indented under parents
**Reason**: Native `depends` never nests; every row is top-level.
**Migration**: Blocked and blocker tasks render as adjacent flat rows in sort order; see `task-dependencies`.

## ADDED Requirements

### Requirement: Blocked rows stay flat and dimmed

The system SHALL render every open task as its own top-level row in the view's sort order with no indentation. A blocked task's description SHALL render dimmed with the magenta `blocked` trailer; blockers carry no trailer.

#### Scenario: Flat blocked pair

- **WHEN** a blocked task and its open blocker share a view
- **THEN** both render as top-level rows and only the waiter shows `blocked`
