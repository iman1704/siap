# task-views Specification

## Purpose

Defines the four main navigation views and their taskwarrior filters so users always know where a task lives, restated against the single-UI chrome.

## Requirements

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

### Requirement: Search input edits the query at the caret

The system SHALL let Search input mode insert printable keys at the query caret instead of firing list hotkeys, move that caret with `←`/`→`, and remove around it with `Backspace`/`Delete`, placing the native terminal cursor at the caret with no inline `▌` glyph. It SHALL leave input on `esc` while keeping the query and SHALL focus input on `i` with the caret at the query's end.

#### Scenario: Typing never fires hotkeys

- **WHEN** the user types `q`, `l`, or `?` in Search input mode
- **THEN** those characters enter the query and no view switch, focus change, or help overlay occurs

#### Scenario: Typing inserts at the query caret

- **WHEN** the query reads `abc` with the caret after `a` and the user types `X`
- **THEN** the query becomes `aXbc` and the caret follows the inserted character

#### Scenario: Arrows move the query caret

- **WHEN** the user presses `←` or `→` in Search input mode
- **THEN** the query caret moves one character without changing the text and focus stays in the query

#### Scenario: Backspace edits before the caret

- **WHEN** the query reads `abc` with the caret after `b` and the user presses `Backspace`
- **THEN** the query becomes `ac` with the caret after `a`

#### Scenario: Delete edits after the caret

- **WHEN** the query reads `abc` with the caret after `b` and the user presses `Delete`
- **THEN** the query becomes `ac` with the caret after `b`

#### Scenario: Focus input from list mode

- **WHEN** the user presses `i` on Search in list mode
- **THEN** input mode focuses with the caret at the query's end and the query stays editable

#### Scenario: Exit input keeps the query

- **WHEN** the user presses `esc` in Search input mode
- **THEN** list mode returns with the query and its results intact

#### Scenario: Input mode shows the terminal caret

- **WHEN** the query is `nadia` in input mode
- **THEN** the line renders `nadia` with the native terminal cursor after it, and list mode renders `nadia` with no cursor

#### Scenario: Empty query starts under the caret

- **WHEN** input mode is focused with an empty query
- **THEN** `Type to search…` renders with the terminal cursor at its start

### Requirement: Search list mode clears the query

The system SHALL clear a non-empty query on `esc` in Search list mode without leaving Search, and SHALL move the results cursor with `Tab`/`Shift+Tab`/`↑`/`↓` in both modes. List mode SHALL render the plain query with an `(i input)` hint when empty.

#### Scenario: Esc clears from list mode

- **WHEN** the user presses `esc` in Search list mode with a non-empty query
- **THEN** the query clears, all open tasks return, and the Search view stays active

#### Scenario: Tab moves the results cursor

- **WHEN** the user presses `Tab` or `Shift+Tab` in either Search mode
- **THEN** the list cursor moves to the next or previous row without editing the query

#### Scenario: List mode hints at input

- **WHEN** Search is in list mode with an empty query
- **THEN** the line renders `Type to search…  (i input)` with no terminal cursor

### Requirement: Search query clears on exit

The system SHALL clear the Search query when the user leaves the Search view, so matches never leak into another view.

#### Scenario: Leaving clears results

- **WHEN** the user switches from Search to Inbox
- **THEN** the query clears and Inbox shows all its tasks

#### Scenario: Re-entering starts empty

- **WHEN** the user returns to Search after leaving it
- **THEN** the query is empty and all open tasks are listed

### Requirement: Sidebar previews the view under the cursor

While the sidebar is focused, moving the cursor SHALL switch the active view immediately without leaving sidebar focus. `Enter` or `l` SHALL confirm the view and focus the list, and entering Search SHALL start in input mode. On launch the sidebar highlight SHALL match the initial view (first permanent row).

#### Scenario: Arrow previews each view

- **WHEN** the user presses `j` or `k` in the focused sidebar
- **THEN** the list shows the view under the cursor while the sidebar stays focused

#### Scenario: Enter confirms the preview

- **WHEN** the user presses `Enter` or `l` on a sidebar row
- **THEN** that view stays active and the list gains focus

#### Scenario: Entering Search starts in input mode

- **WHEN** the user selects Search from the sidebar
- **THEN** Search opens in input mode with the cursor ready to type

#### Scenario: Launch highlight matches the initial view

- **WHEN** the app launches
- **THEN** the highlighted sidebar row is the same view that the list shows
