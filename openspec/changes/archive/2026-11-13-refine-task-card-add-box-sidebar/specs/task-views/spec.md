# Spec Delta

## ADDED Requirements

### Requirement: Sidebar lists current taskwarrior projects

The system SHALL list Inbox, Today, and All permanently, followed by every project that currently appears on a task, sorted alphabetically. Projects that no longer appear MUST be removed on refresh. When the active project disappears, the view SHALL fall back to All.

#### Scenario: Projects follow current tasks

- **WHEN** a task gains project `reading`
- **THEN** the sidebar lists `reading` after the permanent views without a restart

#### Scenario: Stale project leaves the sidebar

- **WHEN** the last task carrying `reading` is completed, deleted, or moved to another project, and the app refreshes
- **THEN** `reading` no longer appears in the sidebar

#### Scenario: Permanent views always remain

- **WHEN** the task list is empty
- **THEN** Inbox, Today, and All still render in the sidebar

#### Scenario: Projects sort alphabetically

- **WHEN** projects `zeta` and `alpha` exist
- **THEN** `alpha` renders before `zeta` after the permanent views

#### Scenario: Active project disappears

- **WHEN** the user is viewing `reading` and the last `reading` task is removed on refresh
- **THEN** the view falls back to All and the sidebar highlight follows it

#### Scenario: Project count follows its tasks

- **WHEN** a project has open top-level tasks
- **THEN** its sidebar count matches those tasks
