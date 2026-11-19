# task-subtasks Specification

## Purpose

Retired single-parent subtask hierarchy; blocking is now native flat Taskwarrior `depends` with no parent, nesting, or cascade. See `task-dependencies` for the blocking contract.

## Requirements

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
