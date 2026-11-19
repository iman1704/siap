# Spec Delta — `taskwarrior-backend`

## REMOVED Requirements

### Requirement: Deferred fields stay read-only

**Reason**: Recurrence and annotations are now first-class editable data: `recur` writes through `add`/`modify` and annotations write through `annotate`/`denotate`, with both surfaced in the list and popup.

**Migration**: No data migration. Existing stored recurrence and annotation values render as before, and users edit them through the popup `recur` field and the inline annotation row.

## MODIFIED Requirements

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

## ADDED Requirements

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
