# Spec Delta — `task-editor`

## MODIFIED Requirements

### Requirement: Popup opens read-only

The system SHALL open a modal popup on `Enter` showing seven editable fields in one flat list (description, project, due, priority, tags, depends, recur) followed by a blank line and the grey read-only block (`id`, `status`, urgency, `start`) plus the interactive `annotation` notes, in `VIEW` mode. Empty values SHALL render blank with no `—` or `-` placeholder. The project field SHALL show the project name only with no `●` dot.

#### Scenario: Open popup from list

- **WHEN** the user presses `Enter` on a highlighted task
- **THEN** the popup opens with all current values populated and marked `VIEW`

#### Scenario: Single list without group headers

- **WHEN** the popup renders its fields
- **THEN** no PRIMARY or SECONDARY group headers render and the project shows `work`, never `● work`

#### Scenario: Empty values stay blank

- **WHEN** the popup renders a task with no project, due, tags, or notes
- **THEN** those values render as blank space with no dash placeholder, and the grey read-only block sits one blank line below the editable fields

#### Scenario: ID leads the read-only block

- **WHEN** the popup renders a task
- **THEN** the read-only block starts with an `id` row showing the short numeric ID, or the UUID prefix when no short ID exists, above `status` and urgency

#### Scenario: Status is read-only

- **WHEN** the popup renders in `VIEW` or `EDIT` mode
- **THEN** `status` renders only in the grey read-only block from the live task, never takes focus, and popup `d` refreshes it without marking the draft dirty

#### Scenario: Recur joins the editable fields

- **WHEN** the popup renders a task with `recur` set to `weekly`
- **THEN** the editable block ends with a `recur` row reading `weekly` and `recur` can take focus, while `annotation` stays outside the editable field list

### Requirement: View mode ignores typing

The system SHALL ignore text input while the popup is in `VIEW` mode, leaving the draft clean and showing no unsaved indicator, except while an annotation draft is active: `a` starts the draft and printable keys edit it, `Enter` adds the note, and `esc` cancels the draft without writing.

#### Scenario: Typing does nothing in view

- **WHEN** the user types characters with the popup in `VIEW` and no note draft active
- **THEN** no field changes and the popup stays clean

#### Scenario: Note draft captures typing

- **WHEN** the user presses `a` in `VIEW` and types characters
- **THEN** those characters join the note draft while the editable fields and dirty flag stay unchanged

#### Scenario: Esc cancels the note draft

- **WHEN** the user presses `esc` while a note draft is active
- **THEN** the draft clears, no annotation is written, and the popup stays open

### Requirement: Save validates and closes

The system SHALL save on `Enter` in `EDIT` mode after applying the highlighted suggestion via a single `task <uuid> modify` write for description, project, due, priority, tags, and recurrence plus per-UUID `depends` add/remove writes, land back in `VIEW` mode with the editor still open and the draft marked clean, and reject an empty description with a hint while keeping the popup open. Recurrence without a due date, unknown dependency IDs, self-links, and links that would close a dependency cycle SHALL toast and keep the draft in `EDIT`. Annotations save only through their inline add/remove path, never through this modify. `status` stays read-only. The save toast SHALL read `saved`.

#### Scenario: Save requires a description

- **WHEN** the user clears the description and presses `Enter`
- **THEN** the popup stays open and reports that a description is required

#### Scenario: Save stays open

- **WHEN** the user edits the description in `EDIT` mode and presses `Enter`
- **THEN** the task updates, the editor stays open in `VIEW` mode, and the draft is marked clean

#### Scenario: Enter applies the hint before saving

- **WHEN** the user types `work` in the project field and presses `Enter` with a matching hint highlighted
- **THEN** the highlighted project is applied and the save persists it

#### Scenario: Cancel discards edits

- **WHEN** the user edits the description then presses `esc`
- **THEN** the popup steps back to `VIEW` with the description restored and any unsaved edits in other fields preserved

#### Scenario: Unknown dependency ID blocks save

- **WHEN** a `depends` draft references an ID that no longer resolves
- **THEN** the toast names the unknown ID and the popup stays in `EDIT`

#### Scenario: Self-link blocks save

- **WHEN** the `depends` draft carries the task's own ID
- **THEN** the toast reads `can't depend on itself` and the popup stays in `EDIT`

#### Scenario: Recurrence needs a due date

- **WHEN** the draft carries `recur: weekly` and an empty due date and the user presses `Enter`
- **THEN** the toast reads `recurrence needs a due date`, the popup stays in `EDIT`, and nothing is written

#### Scenario: Circular dependency blocks save

- **WHEN** the drafted `depends` would make the task reachable from one of its links
- **THEN** the toast reads `circular dependency`, no write happens, and the popup stays in `EDIT`

### Requirement: View closes plainly

The system SHALL close the popup without changes on `esc` or `q` while in `VIEW` mode unless a note draft is active, in which case `esc` cancels only the draft and keeps the popup open, and SHALL keep the popup open with no hint toast on `Enter` in `VIEW` mode.

#### Scenario: Close from view

- **WHEN** the user presses `esc` with the popup in `VIEW`
- **THEN** the popup closes and the list is unchanged

#### Scenario: Enter stays open in view

- **WHEN** the user presses `Enter` with the popup in `VIEW`
- **THEN** the popup stays open

### Requirement: Popup shows dependency relations

The system SHALL render read-only `BLOCKED BY` and `BLOCKS` sections at the bottom of the popup listing open related tasks in the main row format (far-left ID, description, project, tags, due, priority) without the `blocked` trailer; empty sections SHALL stay hidden. `n` in popup `VIEW` SHALL be a no-op.

#### Scenario: Relations close the popup

- **WHEN** a task waits on one open task and blocks another
- **THEN** both related tasks render under their sections with their task ID and description

#### Scenario: No relations means no sections

- **WHEN** a task has no open dependencies in either direction
- **THEN** neither `BLOCKED BY` nor `BLOCKS` renders

#### Scenario: Popup n does nothing

- **WHEN** the user presses `n` in the popup
- **THEN** no task is created and the list is unchanged

### Requirement: Start shows relative age

The system SHALL display the `start` value as a relative age derived from the task's real `start` timestamp (`just now`, `15 min ago`, `1 hour ago`, `3 days ago`), blank when the task was never started. The label SHALL read `start`, never `start/end`.

#### Scenario: Started task shows its age

- **WHEN** the popup renders a task started one hour ago
- **THEN** the start row reads `start   1 hour ago`

#### Scenario: Unstarted task shows blank start

- **WHEN** the popup renders a task with no `start` timestamp
- **THEN** the start value is blank

#### Scenario: Real values use their own color

- **WHEN** the read-only block renders urgency and start values above the annotation notes
- **THEN** the urgency and start values render in their own colors while the empty read-only labels stay grey

## REMOVED Requirements

### Requirement: Edit mode edits with pickers

**Reason**: Editing moved from a separate echo input line with an inline `▌` glyph to in-place caret editing on the field row, and `depends`/`recur` became fixed lists that ignore typed text; the old append-only contract no longer holds.

**Migration**: Use the replacement `Edit mode edits at the field caret` requirement and its scenarios.

### Requirement: Inline suggestions complete attributes

**Reason**: `depends` and `recur` suggestions no longer filter by typed text (they are fixed lists), while the typed fields keep the same completion contract. The replacement requirement records both list kinds.

**Migration**: Use the replacement `Suggestions cover typed and fixed lists` requirement and its scenarios.

### Requirement: Empty edit fields show grey examples

**Reason**: Examples now render on the focused field row at the terminal caret instead of a separate input line, and `depends` no longer has a typing example.

**Migration**: Use the replacement `Empty typed fields show grey examples` requirement and its scenarios.

## ADDED Requirements

### Requirement: Edit mode edits one field at the caret

The system SHALL enter `EDIT` mode on `e` and edit only the focused field at once: printable keys insert at the field's text caret, the field row renders the draft (or the grey shared example while empty), and the native terminal cursor sits at the caret with no separate input line. On text fields, `←`/`→` SHALL move the caret and `Backspace`/`Delete` SHALL remove the character before/after it. `esc` SHALL discard only the focused field.

#### Scenario: Modal movement and typing stay separate

- **WHEN** the user presses `e` on the description field, then `j`
- **THEN** the field does not move and the character `j` is typed into the focused field

#### Scenario: Typing never moves the field

- **WHEN** the user types characters in `EDIT` mode on a text field
- **THEN** the focus stays on the selected field

#### Scenario: Arrows move the text cursor

- **WHEN** the user presses `←` or `→` in `EDIT` mode on a text field
- **THEN** the text caret moves within the draft and the draft text is unchanged except for cursor position

#### Scenario: Focused field carries the terminal caret

- **WHEN** the user focuses a non-empty field in `EDIT` mode
- **THEN** the field row renders the draft at the caret with the native terminal cursor and no inline `▌` glyph or duplicate input line

#### Scenario: Empty field shows its example under the caret

- **WHEN** the user focuses an empty project field in `EDIT` mode
- **THEN** the field row renders `e.g. work` in dim text with the terminal cursor at the example's start

#### Scenario: Esc discards only the focused field

- **WHEN** the user edits one field, moves to a second field, edits it, and presses `esc`
- **THEN** only the second field reverts and the first edit remains unsaved in the draft

#### Scenario: Now reference mirrors the add box

- **WHEN** the user focuses project, priority, or recurrence in `EDIT` mode
- **THEN** a `now:` line renders below the field showing the saved value (`now: @ work`, `now: ! H`, `now: * weekly`; project hides its `now:` when the task has no project, recurrence hides it when cleared)

### Requirement: Fixed lists own j k and ignore typing

The system SHALL render fixed suggestion lists for `priority`, `depends`, and `recur`. `Tab`/`Shift+Tab`/`↑`/`↓` and `j`/`k` SHALL move the highlight, printable typing, `←`/`→`, and `Delete` SHALL NOT change the draft, and `[`/`]` SHALL cycle the project (including `no project`), priority, and recurrence pickers. `Enter` SHALL apply the highlighted row and save.

#### Scenario: Tab moves only the highlight

- **WHEN** the user presses `Tab`, `Shift+Tab`, `↑`, or `↓` in `EDIT` mode
- **THEN** only the suggestion highlight moves and the focused field never changes

#### Scenario: Cycle priority picker

- **WHEN** the user focuses priority in `EDIT` mode and presses `]`
- **THEN** the priority advances (H → M → L → H) and the draft is marked unsaved

#### Scenario: Cycle recurrence picker

- **WHEN** the user focuses `recur` in `EDIT` mode and presses `]`
- **THEN** the field cycles empty → daily → weekly → monthly → yearly → empty and the draft is marked unsaved

#### Scenario: Fixed lists ignore typing

- **WHEN** the user types `9` or `x` while `priority`, `depends`, or `recur` is focused in `EDIT` mode
- **THEN** the draft is unchanged and the highlight stays put

#### Scenario: Fixed lists move on j and k

- **WHEN** the user presses `j` then `k` while `priority`, `depends`, or `recur` is focused in `EDIT` mode
- **THEN** the highlight moves forward then back and no text is inserted

### Requirement: Backspace edits depends and recurrence

The system SHALL pop the last `depends` link on `Backspace` and save while staying in `EDIT`, and SHALL clear the `recur` draft on `Backspace` so the next `Enter` saves the cleared value.

#### Scenario: Depends Backspace pops the last link

- **WHEN** a `depends` draft holds `12, 4` and the user presses `Backspace`
- **THEN** `4` leaves the draft, the save persists only `12`, and the popup stays in `EDIT` for repeated presses

#### Scenario: Recur Backspace clears the draft

- **WHEN** a `recur` draft holds `weekly` and the user presses `Backspace`
- **THEN** the draft clears and the next `Enter` saves the cleared recurrence


### Requirement: Typed suggestions complete attributes

The system SHALL render a capped suggestion dropdown under the focused `EDIT` project, tags, due, and priority fields, filtered by the typed text. Project SHALL lead with `no project` and tags with `no tags`; tags SHALL complete by the in-progress last token, replacing it. `Enter` SHALL apply the highlighted suggestion (a clear row empties the draft) and save, and `Tab` SHALL only move the highlight.

#### Scenario: Project hints list current projects

- **WHEN** the user focuses the project field in `EDIT` typing mode
- **THEN** the dropdown lists `no project` first followed by current projects and `Enter` applies the highlighted one into the draft and saves

#### Scenario: Tags complete the in-progress token

- **WHEN** the tag field contains `work, leet` and the highlighted suggestion is `leetcode`
- **THEN** applying it produces `work, leetcode`, not `work, leet, leetcode`

#### Scenario: Tag filter follows the last token

- **WHEN** the tag field contains `work, lee`
- **THEN** the dropdown filters on `lee`, not on the whole field text

### Requirement: Depends and recurrence offer fixed lists

The system SHALL render a fixed `depends` list of open same-project tasks as `ID  description`, excluding self and already-added IDs and always closing with `no dependencies`, and a fixed `recur` list of `daily` / `weekly` / `monthly` / `yearly` / `no recurrence`. Neither list SHALL filter by typing; `Enter` SHALL apply the highlighted row (clear rows empty the draft) and save at once.

#### Scenario: Depends applies and saves at once

- **WHEN** the user highlights a depends suggestion and presses `Enter`
- **THEN** that ID is appended to the draft, the popup saves back to `VIEW`, and the added row hides from further suggestions

#### Scenario: Depends list never narrows

- **WHEN** the user focuses the fixed `depends` list and types characters
- **THEN** the list keeps every candidate and no draft text changes

#### Scenario: No dependencies clears and saves

- **WHEN** the user highlights `no dependencies` and presses `Enter`
- **THEN** the draft depends clears, the save persists zero dependencies, and the popup lands back in `VIEW` with toast `saved`

#### Scenario: Recurrence lists presets

- **WHEN** the user focuses the recurrence field in `EDIT` mode
- **THEN** the dropdown lists `daily`, `weekly`, `monthly`, `yearly`, and `no recurrence` with no example line

#### Scenario: No recurrence clears and saves

- **WHEN** the user highlights `no recurrence` and presses `Enter`
- **THEN** the draft recurrence clears, the save persists no recurrence, and the popup lands back in `VIEW` with toast `saved`

### Requirement: Empty typed fields show grey examples

While a typed field is focused in `EDIT` and empty, the system SHALL show the shared grey example on the field row at the caret (vanishing on input): description (`e.g. Call the client back`), project (`e.g. work`), due (`e.g. 2026-11-20`), and tags (`e.g. work, urgent`). Picker-only priority and the fixed-list `depends` and `recur` fields SHALL show no example and no caret.

#### Scenario: Empty due shows an example date

- **WHEN** the user focuses an empty due field in `EDIT` mode
- **THEN** the field row shows `e.g. 2026-11-20` in dim text with the terminal cursor at its start

#### Scenario: Recurrence shows no example

- **WHEN** the user focuses an empty recurrence field in `EDIT` mode
- **THEN** no `e.g.` example and no typing caret render, only the preset list and any `now:` reference

#### Scenario: Typing replaces the example

- **WHEN** the user types a character into a focused empty field
- **THEN** the example disappears and the typed text renders bright with the terminal cursor after it

#### Scenario: Picker-only fields have no example

- **WHEN** the user focuses priority in `EDIT` mode
- **THEN** no example text renders, only the `now: ! H` reference and the `H` / `M` / `L` suggestions

### Requirement: Annotations edit inline in view

The system SHALL render the task's annotations below the read-only block as timestamped rows (text, plus a `· YYYY-MM-DD` date when an entry timestamp exists), labelled `annotation (N)`, with the selected row marked. `a` starts an inline draft with the grey example `e.g. call the client back`, `Enter` saves it, `esc` cancels, `[`/`]` move the selection, and `D`/`x` remove the selected note. Empty drafts SHALL toast `annotation required` and stay open.

#### Scenario: Add a note inline

- **WHEN** the user presses `a` in the popup, types `call back`, and presses `Enter`
- **THEN** the note is written, the popup stays open, the new note renders selected, and the toast reads `annotated`

#### Scenario: Empty note is refused

- **WHEN** the user presses `a` then `Enter` with an empty draft
- **THEN** the toast reads `annotation required`, the popup stays open, and no write happens

#### Scenario: Select and remove a note

- **WHEN** a task has two notes, the user presses `[` or `]` to select one, then `D`
- **THEN** that note is removed, the remaining note still renders, and the toast reads `annotation removed`

#### Scenario: Notes show their date

- **WHEN** a note carries an entry timestamp
- **THEN** its row renders the text with the local calendar date appended after `·`

#### Scenario: Empty annotation list stays labelled

- **WHEN** a task has no annotations
- **THEN** the popup renders an `annotation` label with the inline key hint and no note rows

### Requirement: Recurrence edits as a fixed list

The system SHALL expose `recur` as the seventh editable field with a fixed list of `daily`, `weekly`, `monthly`, `yearly`, and `no recurrence`, where `no recurrence` clears the field. Applying recurrence without a due date SHALL be refused, and a cleared recurrence SHALL persist as no recurrence.

#### Scenario: Recurrence presets are fixed

- **WHEN** the user focuses `recur` in `EDIT` mode
- **THEN** the only values offered are `daily`, `weekly`, `monthly`, `yearly`, and `no recurrence`

#### Scenario: Recurrence applies and saves

- **WHEN** the user highlights `weekly` and presses `Enter` on a task with a due date
- **THEN** the task stores `weekly`, the popup returns to `VIEW`, and the toast reads `saved`

#### Scenario: Cleared recurrence drops the value

- **WHEN** the user highlights `no recurrence` and presses `Enter`
- **THEN** the stored recurrence clears and the list stops showing the recurrence badge
