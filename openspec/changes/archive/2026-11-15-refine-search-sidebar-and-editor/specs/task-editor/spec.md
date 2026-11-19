# Spec Delta

## MODIFIED Requirements

### Requirement: Popup opens read-only

The system SHALL open a modal popup on `Enter` showing the seven editable fields in one flat list (description, project, due, priority, tags, depends, status) followed by a blank line and the grey read-only block (`id`, urgency, `start`, recur, annotation), in `VIEW` mode. Empty values SHALL render blank with no `—` or `-` placeholder. The project field SHALL show the project name only with no `●` dot.

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
- **THEN** the read-only block starts with an `id` row showing the short numeric ID, or the UUID prefix when no short ID exists, above urgency

### Requirement: Edit mode edits with pickers

The system SHALL enter `EDIT` mode on `e` and edit only the focused field at once: keystrokes insert at the field's text cursor, `←`/`→` move that cursor, and `Tab`/`Shift+Tab`/`↑`/`↓` move only the highlighted suggestion. `Enter` applies the highlighted suggestion and saves, `esc` discards only the focused field, and `[`/`]` keep cycling the project, priority, and status pickers. Urgency, start, recur, and annotation are display-only and never take focus. Typing `j`/`k` MUST insert text and MUST NOT move the field.

#### Scenario: Cycle priority picker

- **WHEN** the user focuses priority in `EDIT` mode and presses `]`
- **THEN** the priority advances (H → M → L → H) and the draft is marked unsaved

#### Scenario: Modal movement and typing stay separate

- **WHEN** the user presses `e`, then `j`
- **THEN** the field does not move and the character `j` is typed into the focused field

#### Scenario: Typing never moves the field

- **WHEN** the user types characters in `EDIT` mode
- **THEN** the focus stays on the selected field

#### Scenario: Tab moves only the highlight

- **WHEN** the user presses `Tab`, `Shift+Tab`, `↑`, or `↓` in `EDIT` mode
- **THEN** only the suggestion highlight moves and the focused field never changes

#### Scenario: Arrows move the text cursor

- **WHEN** the user presses `←` or `→` in `EDIT` mode
- **THEN** the insertion point moves inside the focused field and typing lands there

#### Scenario: Esc discards only the focused field

- **WHEN** the user edits one field, moves to a second field, edits it, and presses `esc`
- **THEN** only the second field reverts and the first edit remains unsaved in the draft

### Requirement: Save validates and closes

The system SHALL save on `Enter` in `EDIT` mode after applying the highlighted suggestion, land back in `VIEW` mode with the editor still open and the draft marked clean, and reject an empty description with a hint while keeping the popup open.

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

### Requirement: Inline suggestions complete attributes

The system SHALL render an inline suggestion dropdown under the focused `EDIT` field for project, tags, due, priority, and status, filtered by the text already typed. Tags SHALL filter and complete by the in-progress last token, replacing it rather than appending a duplicate. `Enter` SHALL apply the highlighted suggestion, and `Tab` SHALL only move the highlight.

#### Scenario: Project hints list current projects

- **WHEN** the user focuses the project field in `EDIT` typing mode
- **THEN** the dropdown lists current projects and `Enter` applies the highlighted one into the draft

#### Scenario: Tags complete the in-progress token

- **WHEN** the tag field contains `work, leet` and the highlighted suggestion is `leetcode`
- **THEN** applying it produces `work, leetcode`, not `work, leet, leetcode`

#### Scenario: Tag filter follows the last token

- **WHEN** the tag field contains `work, lee`
- **THEN** the dropdown filters on `lee`, not on the whole field text

## ADDED Requirements

### Requirement: Overdue tasks open their raw due date

When opening an overdue task, the editor SHALL show the due field as the raw local calendar date (`YYYY-MM-DD`) derived from the due timestamp, never the `Xd overdue` label. A task carrying only the friendly label SHALL open the field blank.

#### Scenario: Overdue opens the calendar date

- **WHEN** the user opens an overdue task with a due timestamp
- **THEN** the due field reads `2026-11-10` and editing it changes the real due date

#### Scenario: Label is never edited as a date

- **WHEN** an overdue task carries only the label form
- **THEN** the due field opens blank instead of showing `3d overdue`

### Requirement: Empty edit fields show grey examples

While a typed field is focused in `EDIT` and empty, the system SHALL show a grey example that vanishes on input: description, project (`e.g. work`), due (`e.g. 2026-11-20`), tags (`e.g. work, urgent`), and depends (`enter task ID`, with no `e.g.`). Picker-only priority and status SHALL show no example.

#### Scenario: Empty due shows an example date

- **WHEN** the user focuses an empty due field in `EDIT` mode
- **THEN** the line shows `e.g. 2026-11-20` in dim text

#### Scenario: Depends names a task ID

- **WHEN** the user focuses an empty depends field in `EDIT` mode
- **THEN** the line shows `enter task ID` without an `e.g.` prefix

#### Scenario: Typing replaces the example

- **WHEN** the user types a character into a focused empty field
- **THEN** the example disappears and the typed text renders bright

#### Scenario: Picker-only fields have no example

- **WHEN** the user focuses priority or status in `EDIT` mode
- **THEN** no example text renders

### Requirement: Done toggles without closing the popup

The system SHALL toggle the popup task between done and undone on `d` in `VIEW` mode without closing the popup, refreshing the displayed status and dirty state.

#### Scenario: Complete inside the popup

- **WHEN** the user presses `d` in the popup `VIEW` mode on an open task
- **THEN** the task completes, the popup stays open, and its status row reads completed

#### Scenario: Reopen inside the popup

- **WHEN** the user presses `d` again on the completed popup task
- **THEN** the task returns to pending and the popup stays open
