# taskwarrior-backend Specification

## Purpose

Live taskwarrior data replaces mock tasks so the list, popup, and operations act on the user's real pending and recently completed work.

## Requirements

### Requirement: Open rows come from pending export

The system SHALL populate open rows from pending and started tasks and exclude deleted tasks from every view.

#### Scenario: Deleted tasks never render

- **WHEN** the data contains tasks with status deleted
- **THEN** no deleted task appears in any view or count

#### Scenario: Started tasks stay open

- **WHEN** a pending task has a start timestamp
- **THEN** it renders as active and stays in open rows

### Requirement: Completed tail shows recent completions

The system SHALL render the dimmed COMPLETED tail from recently completed tasks sorted by completion time descending and capped at ten, still filtered by the current view.

#### Scenario: Tail is capped by recency

- **WHEN** more than ten tasks match the tail filter
- **THEN** only the ten most recently completed render

#### Scenario: Tail respects the current view

- **WHEN** the user opens a project view
- **THEN** the tail shows only recent completions from that project

### Requirement: Identity is stable across refreshes

The system SHALL key selection, cursor, and popup by stable task UUID identity that survives refreshes and external id shifts.

#### Scenario: Cursor survives refresh

- **WHEN** a refresh reorders numeric ids
- **THEN** the cursor stays on the same logical task

### Requirement: Due display derives from real dates

The system SHALL derive the friendly due label and overdue and due-today markers from the real due timestamp using the local calendar day, with absent due rendering as em-dash.

#### Scenario: Overdue derives from the calendar

- **WHEN** a task's due date is before the local today
- **THEN** it renders an overdue label and marker and appears in Today

#### Scenario: Missing due stays neutral

- **WHEN** a task has no due timestamp
- **THEN** it renders an em-dash with no marker

### Requirement: Priority defaults and urgency stay read-only

The system SHALL render absent priority as L and SHALL treat urgency as a read-only computed value that refreshes from the backend.

#### Scenario: Missing priority renders P3

- **WHEN** a task has no priority value
- **THEN** the row shows L styling (renamed from `P3`)

#### Scenario: Urgency refreshes after save

- **WHEN** the user changes priority and saves
- **THEN** the displayed urgency updates on the next refresh

### Requirement: Writes persist through the backend

The system SHALL persist quick-add (including `depends` and recurrence), popup saves (including per-UUID `depends` add/remove and recurrence set/clear), start and stop, done, undone (`modify status:pending`), undo, single-only delete, section moves, annotation add/remove (`annotate`/`denotate`), and bulk operations to the backend store.

#### Scenario: Done persists across restart

- **WHEN** the user completes a task and restarts the app
- **THEN** the task remains completed

#### Scenario: Undone persists across restart

- **WHEN** the user reopens a completed task and restarts the app
- **THEN** the task remains pending

#### Scenario: Delete removes children

- **WHEN** the user deletes a task that participates in `depends`
- **THEN** only that task leaves the open list and relations resolve on refresh

#### Scenario: Delete removes only the confirmed tasks

- **WHEN** the user deletes a task that participates in `depends`
- **THEN** only that task leaves the open list and relations resolve on refresh

#### Scenario: Recurrence persists across restart

- **WHEN** the user sets `weekly` recurrence with a due date and restarts the app
- **THEN** the task still reads `weekly` and still shows the recurrence badge

#### Scenario: Annotation persists across restart

- **WHEN** the user adds an annotation and restarts the app
- **THEN** the note still renders on the task

### Requirement: Depends maps flat from export

The system SHALL read native `depends` UUID arrays from `task export` directly into each task with no parent inversion, dropping `deleted` rows; a task whose dependency UUIDs are all completed or deleted SHALL render unblocked.

#### Scenario: Flat export keeps both rows

- **WHEN** the export shows task A depending on open task B
- **THEN** both render as top-level rows with A marked `blocked`

#### Scenario: Deleted rows never render

- **WHEN** the export contains a deleted task
- **THEN** it appears in no view and holds no relations

### Requirement: Section store self-registers

The system SHALL register the section store on launch when missing without user action and never require manual configuration commands.

#### Scenario: First launch registers silently

- **WHEN** the app launches and the section store is missing
- **THEN** it registers and the list still renders

### Requirement: Refresh follows launch write and manual key

The system SHALL load live data on launch, reload after every successful write, and reload on the manual refresh key for external changes.

#### Scenario: Manual refresh picks up external edits

- **WHEN** the user presses the refresh key after editing tasks elsewhere
- **THEN** the list reflects the external state with cursor held by stable identity

### Requirement: Failures toast and preserve state

The system SHALL define a failed backend command by non-zero process exit status only, report it in the toast with the trimmed `stderr` (falling back to trimmed `stdout`), and keep the existing list, selection, and cursor unchanged. Taskwarrior validation text printed with exit status 0 SHALL be treated as success.

#### Scenario: Failed write keeps the list

- **WHEN** a write command exits non-zero
- **THEN** the toast explains the failure and no rows change

#### Scenario: Exit-zero output counts as success

- **WHEN** a write command exits zero even with warning-like text on stdout or stderr
- **THEN** the write counts as successful and the list refreshes from the backend

### Requirement: Undo reverts the last backend change

The system SHALL revert the most recent backend mutation on `u` (or `Ctrl+z`) via the backend `undo` command, then re-export and hold the cursor by stable identity. A failed undo SHALL toast and preserve the list.

#### Scenario: Undo after done

- **WHEN** the user completes a task then presses `u`
- **THEN** the task returns to pending

#### Scenario: Failed undo preserves state

- **WHEN** the backend undo command fails
- **THEN** the toast explains the failure and no rows change

### Requirement: Start display derives from the real start timestamp

The system SHALL read the task's real `start` timestamp from the export and derive the popup `start` row from it; tasks with no `start` value SHALL show a blank start.

#### Scenario: Started task shows age from export

- **WHEN** the export carries a `start` timestamp one hour in the past
- **THEN** the popup shows a relative age for that task

#### Scenario: Missing start stays blank

- **WHEN** the export carries no `start` timestamp
- **THEN** the popup start value is blank

### Requirement: Recurrence maps to native recur

The system SHALL write recurrence as taskwarrior's native `recur` attribute: quick-add skips the attribute when recurrence is cleared and otherwise adds `recur:<value>`, while popup saves always write `recur:<value>` to set it or `recur:` to clear it. A recurrence without a due date SHALL be refused before any write, and recurring templates SHALL stay out of the views because only pending and completed tasks are exported.

#### Scenario: Add carries recurrence

- **WHEN** the user commits a quick-add with recurrence `weekly` and a due date
- **THEN** the single `task add` write includes `recur:weekly`

#### Scenario: Modify sets and clears recurrence

- **WHEN** the user saves `monthly` in the popup and later saves `no recurrence`
- **THEN** the first write includes `recur:monthly` and the second includes a bare `recur:`

#### Scenario: Recurrence without a due date never writes

- **WHEN** recurrence is set with no due date
- **THEN** no backend write starts and the toast reads `recurrence needs a due date`

#### Scenario: Recurring templates stay hidden

- **WHEN** taskwarrior holds generated recurrence templates
- **THEN** no template row appears in any view because exports stay limited to pending and completed tasks

### Requirement: Annotations keep timestamps and exact text

The system SHALL read native `annotations` from `task export` into ordered notes carrying the raw `entry` timestamp and description text, ignoring entries without text. Adding SHALL write `annotate <text>` and removing SHALL write `denotate <text>` with the note's exact description so only the selected note is deleted.

#### Scenario: Export notes keep entry and text

- **WHEN** the export carries an annotation with `entry` and `description`
- **THEN** both values load and the popup can render the note with its local date

#### Scenario: Adding a note writes annotate

- **WHEN** the user commits an inline note
- **THEN** the backend runs `annotate <text>` for that task and the refreshed task includes the new note

#### Scenario: Removing a note writes denotate by exact text

- **WHEN** the user removes one of several notes
- **THEN** the backend runs `denotate <text>` with that note's exact description and the other notes remain

#### Scenario: Empty export means no notes

- **WHEN** a task export has no annotations array or an empty one
- **THEN** the task loads with no notes and the popup shows the empty annotation label
