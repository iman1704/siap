# Spec Delta — `task-views`

## MODIFIED Requirements

### Requirement: Top line and sidebar show counts

The system SHALL render a top line with the wordmark, open / due / active totals, and sync state, with no current view title, plus per-view open counts in the sidebar for Search, Inbox, Today, All, and each project, all reflecting the current filter. Sync state SHALL read ` · synced HH:MM:SS` in dim after a successful refresh, ` · sync failed` in yellow after a failed refresh, and ` · not synced` before the first refresh. The Search count SHALL reflect the current query.

#### Scenario: Counts match filters

- **WHEN** the user switches from Inbox to Today
- **THEN** the sidebar counts and top-line totals reflect each view's filter results

#### Scenario: Empty views explain themselves

- **WHEN** a view has no matching tasks
- **THEN** the list shows that view's empty hint and no rows

#### Scenario: Top bar omits the view name

- **WHEN** any view renders
- **THEN** the top line shows the wordmark, totals, and sync state only, with no view title

#### Scenario: Search count follows its query

- **WHEN** the user types `nadia` in Search
- **THEN** the Search sidebar count equals the number of matching open tasks

#### Scenario: Sync state follows refresh

- **WHEN** a backend refresh succeeds
- **THEN** the top line appends ` · synced HH:MM:SS` with the local refresh time

#### Scenario: Failed sync warns

- **WHEN** a backend refresh fails
- **THEN** the top line appends ` · sync failed` in yellow and the list, selection, and cursor stay unchanged

### Requirement: Sidebar lists current taskwarrior projects

The system SHALL list Search, Inbox, Today, and All permanently, followed by every project that currently appears on a task, sorted alphabetically. A dim `· · · · · · ·` rule SHALL separate the permanent views from the project rows. The sidebar SHALL NOT render the `taskwarrior, tamed` tagline or the trailing `uda.siapsection` label. Projects that no longer appear MUST be removed on refresh. When the active project disappears, the view SHALL fall back to All.

#### Scenario: Projects follow current tasks

- **WHEN** a task gains project `reading`
- **THEN** the sidebar lists `reading` after the permanent views without a restart

#### Scenario: Stale project leaves the sidebar

- **WHEN** the last task carrying `reading` is completed, deleted, or moved to another project, and the app refreshes
- **THEN** `reading` no longer appears in the sidebar

#### Scenario: Permanent views always remain

- **WHEN** the task list is empty
- **THEN** Search, Inbox, Today, and All still render in the sidebar

#### Scenario: Search sits above Inbox

- **WHEN** the sidebar renders
- **THEN** `Search` is the first permanent row and `Inbox` follows it

#### Scenario: Projects sort alphabetically

- **WHEN** projects `zeta` and `alpha` exist
- **THEN** `alpha` renders before `zeta` after the permanent views

#### Scenario: Active project disappears

- **WHEN** the user is viewing `reading` and the last `reading` task is removed on refresh
- **THEN** the view falls back to All and the sidebar highlight follows it

#### Scenario: Project count follows its tasks

- **WHEN** a project has open tasks
- **THEN** its sidebar count matches those tasks
