# Spec Delta

## ADDED Requirements

### Requirement: Subtasks can be created from the list

The system SHALL create a subtask on `n` when the cursor rests on a task row, using the same inheritance as the popup: the child receives the parent's project and section and is linked under it.

#### Scenario: Create a subtask from the list

- **WHEN** the user presses `n` with the cursor on a task row
- **THEN** a subtask is created under that task with the parent's project and section

#### Scenario: No cursor means no subtask

- **WHEN** the user presses `n` with no task under the cursor
- **THEN** no task is created and the list is unchanged

#### Scenario: Create a subtask from Search list mode

- **WHEN** the user presses `n` in Search list mode with the cursor on a result
- **THEN** a subtask is created under that result's task
