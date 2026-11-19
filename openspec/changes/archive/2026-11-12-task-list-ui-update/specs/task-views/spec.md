# Spec Delta

## MODIFIED Requirements

### Requirement: Inbox view shows unsorted intake

The system SHALL show in Inbox only open tasks (`status:pending` or `status:active`) that have no project, regardless of due date.

#### Scenario: Fresh task appears in Inbox

- **WHEN** a user creates a task with no project and no due date
- **THEN** it appears in Inbox and not in any project view

#### Scenario: Task with due date but no project stays in Inbox

- **WHEN** a task has no project and gains a due date
- **THEN** it still appears in Inbox

#### Scenario: Assigned task leaves Inbox

- **WHEN** a task gains a project
- **THEN** it no longer appears in Inbox

#### Scenario: Active project-less task stays in Inbox

- **WHEN** a user starts a task that has no project
- **THEN** it still appears in Inbox while active
