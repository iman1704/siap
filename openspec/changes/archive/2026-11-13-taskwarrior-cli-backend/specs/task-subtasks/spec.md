# Spec Delta

## MODIFIED Requirements

### Requirement: Subtasks use depends blocking

The system SHALL implement subtasks by adding each child's ID to the parent's `depends` field so the parent is blocked until children complete. Each child SHALL have at most one parent, and the parent link SHALL be read as the single task whose `depends` contains the child.

#### Scenario: Parent blocked until children done

- **WHEN** a parent has two incomplete subtasks
- **THEN** the parent shows a blocked marker until both are completed

#### Scenario: Single parent enforced

- **WHEN** a child is already linked to one parent
- **THEN** linking it to another parent moves the link instead of adding a second parent

#### Scenario: Extra dependencies stay untouched

- **WHEN** a task carries dependency values beyond the single parent link
- **THEN** the list renders the first parent link and leaves the remaining values unmodified until the user edits that task
