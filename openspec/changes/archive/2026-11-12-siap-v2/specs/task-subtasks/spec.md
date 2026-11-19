# Spec Delta

## Purpose

Defines Todoist-style subtasks backed by native taskwarrior depends blocking, with indented display and no progress counts.

## ADDED Requirements

### Requirement: Subtasks use depends blocking

The system SHALL implement subtasks by setting the parent's `depends` field to its children's IDs so the parent is blocked until children complete.

#### Scenario: Parent blocked until children done

- **WHEN** a parent has two incomplete subtasks
- **THEN** the parent shows a blocked marker until both are completed

### Requirement: Subtasks inherit project and section

The system SHALL assign a new subtask the same project and section as its parent by default.

#### Scenario: Create subtask under a project parent

- **WHEN** the user adds a subtask to a `project:work` parent
- **THEN** the child is created with `project:work` in the parent's section

### Requirement: Indented display without counts

The system SHALL render subtasks indented under their parent with checkboxes, and SHALL NOT render `n/m` progress counts anywhere.

#### Scenario: Children nest quietly

- **WHEN** a parent has open and completed children
- **THEN** each child renders indented below the parent with its checkbox state and no numbers

### Requirement: Deleting a parent deletes its children

The system SHALL mark a parent's subtasks deleted when the parent is deleted, so no orphaned children remain.

#### Scenario: Delete cascades

- **WHEN** the user deletes a parent with two subtasks
- **THEN** both children leave the list along with the parent
