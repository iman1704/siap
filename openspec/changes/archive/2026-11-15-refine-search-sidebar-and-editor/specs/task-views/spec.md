# Spec Delta

## MODIFIED Requirements

### Requirement: Top line and sidebar show counts

The system SHALL render a top line with the wordmark and open / due / active totals, with no current view title, plus per-view open counts in the sidebar for Search, Inbox, Today, All, and each project, all reflecting the current filter. The Search count SHALL reflect the current query.

#### Scenario: Counts match filters

- **WHEN** the user switches from Inbox to Today
- **THEN** the sidebar counts and top-line totals reflect each view's filter results

#### Scenario: Empty views explain themselves

- **WHEN** a view has no matching tasks
- **THEN** the list shows that view's empty hint and no rows

#### Scenario: Top bar omits the view name

- **WHEN** any view renders
- **THEN** the top line shows the wordmark and totals only, with no view title

#### Scenario: Search count follows its query

- **WHEN** the user types `nadia` in Search
- **THEN** the Search sidebar count equals the number of matching open tasks

### Requirement: Sidebar lists current taskwarrior projects

The system SHALL list Search, Inbox, Today, and All permanently, followed by every project that currently appears on a task, sorted alphabetically. Projects that no longer appear MUST be removed on refresh. When the active project disappears, the view SHALL fall back to All.

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

- **WHEN** a project has open top-level tasks
- **THEN** its sidebar count matches those tasks

## ADDED Requirements

### Requirement: Search view filters open tasks live

The system SHALL provide a Search view listing matching tasks: open tasks whose description, project, or tags contains the query case-insensitively, plus matching completed tasks in the dimmed tail. An empty query SHALL list every open task and the normal completed tail, and the view SHALL render no add box.

#### Scenario: Description match

- **WHEN** the user types `nadia` in Search
- **THEN** only open tasks whose description contains `nadia` render

#### Scenario: Project and tag match

- **WHEN** the query matches a project name or a tag
- **THEN** open tasks carrying that project or tag render

#### Scenario: Empty query lists open tasks

- **WHEN** the Search view opens with an empty query
- **THEN** every open task renders in urgency-descending order

#### Scenario: Search has no add box

- **WHEN** the Search view renders
- **THEN** no add box or `a` trigger appears in the list

#### Scenario: Completed matches stay in the tail

- **WHEN** the query matches a completed task
- **THEN** that task renders dimmed under the `COMPLETED` tail alongside the open matches

### Requirement: Search input captures typing

The system SHALL give Search an input mode and a list mode. In input mode printable keys type into the query instead of firing list hotkeys, `i` focuses input from list mode, and `esc` leaves input while keeping the query. `esc` in list mode SHALL clear a non-empty query without leaving Search. `Tab`/`Shift+Tab`/`↑`/`↓` SHALL move the list cursor in both modes.

#### Scenario: Typing never fires hotkeys

- **WHEN** the user types `q`, `l`, or `?` in Search input mode
- **THEN** those characters enter the query and no view switch, focus change, or help overlay occurs

#### Scenario: Focus input from list mode

- **WHEN** the user presses `i` on Search in list mode
- **THEN** input mode focuses and the query stays editable

#### Scenario: Exit input keeps the query

- **WHEN** the user presses `esc` in Search input mode
- **THEN** list mode returns with the query and its results intact

#### Scenario: Esc clears from list mode

- **WHEN** the user presses `esc` in Search list mode with a non-empty query
- **THEN** the query clears, all open tasks return, and the Search view stays active

#### Scenario: Tab moves the results cursor

- **WHEN** the user presses `Tab` or `Shift+Tab` in either Search mode
- **THEN** the list cursor moves to the next or previous row without editing the query

### Requirement: Search query clears on exit

The system SHALL clear the Search query when the user leaves the Search view, so matches never leak into another view.

#### Scenario: Leaving clears results

- **WHEN** the user switches from Search to Inbox
- **THEN** the query clears and Inbox shows all its tasks

#### Scenario: Re-entering starts empty

- **WHEN** the user returns to Search after leaving it
- **THEN** the query is empty and all open tasks are listed

### Requirement: Sidebar previews the view under the cursor

While the sidebar is focused, moving the cursor SHALL switch the active view immediately without leaving sidebar focus. `Enter` or `l` SHALL confirm the view and focus the list, and entering Search SHALL start in input mode.

#### Scenario: Arrow previews each view

- **WHEN** the user presses `j` or `k` in the focused sidebar
- **THEN** the list shows the view under the cursor while the sidebar stays focused

#### Scenario: Enter confirms the preview

- **WHEN** the user presses `Enter` or `l` on a sidebar row
- **THEN** that view stays active and the list gains focus

#### Scenario: Entering Search starts in input mode

- **WHEN** the user selects Search from the sidebar
- **THEN** Search opens in input mode with the cursor ready to type
