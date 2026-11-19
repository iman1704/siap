# Spec Delta

## Purpose

Defines the four main navigation views and their taskwarrior filters so users always know where a task lives, restated against the single-UI chrome.

## ADDED Requirements

### Requirement: Inbox view shows unsorted intake

The system SHALL show in Inbox only tasks with `status:pending` that have no project and no due date.

#### Scenario: Fresh task appears in Inbox

- **WHEN** a user creates a task with no project and no due date
- **THEN** it appears in Inbox and not in any project view

#### Scenario: Assigned task leaves Inbox

- **WHEN** a task gains a project or a due date
- **THEN** it no longer appears in Inbox

### Requirement: Today view shows time-sensitive tasks

The system SHALL show in Today tasks that are overdue, due today, or active (started and not stopped), limited to `status:pending` or `status:active`.

#### Scenario: Overdue task appears in Today

- **WHEN** a pending task has a due date before today
- **THEN** it appears in Today with an overdue marker

#### Scenario: Active task appears in Today

- **WHEN** a user starts a task with a future due date
- **THEN** it appears in Today while active

### Requirement: All and Projects views filter predictably

The system SHALL show in All every `status:pending` and `status:active` task, and in each project view only tasks whose `project` field matches exactly.

#### Scenario: Project view isolates tasks

- **WHEN** the user selects the `work` project
- **THEN** only tasks with `project:work` are listed

### Requirement: Top line and sidebar show counts

The system SHALL render a top line with the wordmark, current view title, and open / due / active totals, plus per-view open counts in the sidebar, all reflecting the current filter.

#### Scenario: Counts match filters

- **WHEN** the user switches from Inbox to Today
- **THEN** the sidebar counts and top-line totals reflect each view's filter results

#### Scenario: Empty views explain themselves

- **WHEN** a view has no matching tasks
- **THEN** the list shows that view's empty hint and no rows
