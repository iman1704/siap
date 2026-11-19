# Spec Delta

## MODIFIED Requirements

### Requirement: Popup opens read-only

The system SHALL open a modal popup on `Enter` showing the seven editable fields in one flat list (description, project, due, priority, tags, depends, status) followed by a blank line and the grey read-only block (urgency, `start`, recur, annotation), in `VIEW` mode. Empty values SHALL render blank with no `—` or `-` placeholder. The project field SHALL show the project name only with no `●` dot.

#### Scenario: Open popup from list

- **WHEN** the user presses `Enter` on a highlighted task
- **THEN** the popup opens with all current values populated and marked `VIEW`

#### Scenario: Single list without group headers

- **WHEN** the popup renders its fields
- **THEN** no PRIMARY or SECONDARY group headers render and the project shows `work`, never `● work`

#### Scenario: Empty values stay blank

- **WHEN** the popup renders a task with no project, due, tags, or notes
- **THEN** those values render as blank space with no dash placeholder, and the grey read-only block sits one blank line below the editable fields

### Requirement: View mode ignores typing

The system SHALL ignore text input while the popup is in `VIEW` mode, leaving the draft clean and showing no unsaved indicator.

#### Scenario: Typing does nothing in view

- **WHEN** the user types characters with the popup in `VIEW`
- **THEN** no field changes and the popup stays clean

### Requirement: Edit mode edits with pickers

The system SHALL enter `EDIT` mode on `e` and edit the selected field at once: keystrokes edit text, `Tab` applies the highlighted inline suggestion, `↑`/`↓` move the suggestion selection, and `[`/`]` keep cycling the project, priority, and status pickers. Urgency, start, recur, and annotation are display-only and never take focus. Typing `j`/`k` MUST enter text and MUST NOT move the field.

#### Scenario: Cycle priority picker

- **WHEN** the user focuses priority in `EDIT` mode and presses `]`
- **THEN** the priority advances (H → M → L → H) and the draft is marked unsaved

#### Scenario: Modal movement and typing stay separate

- **WHEN** the user presses `e`, then `j`
- **THEN** the field does not move and the character `j` is typed into the focused field

#### Scenario: Typing never moves the field

- **WHEN** the user types characters in `EDIT` mode
- **THEN** the focus stays on the selected field

### Requirement: Urgency is read-only

The system SHALL display computed urgency as a read-only value with its explanation and MUST NOT allow direct editing.

#### Scenario: Urgency reflects changes

- **WHEN** the user changes priority from M to H and saves
- **THEN** the displayed urgency value increases on next render

### Requirement: Save validates and closes

The system SHALL save on `Enter` in `EDIT` mode, land back in `VIEW` mode with the editor still open and the draft marked clean, and reject an empty description with a hint while keeping the popup open.

#### Scenario: Save requires a description

- **WHEN** the user clears the description and presses `Enter`
- **THEN** the popup stays open and reports that a description is required

#### Scenario: Save stays open

- **WHEN** the user edits the description in `EDIT` mode and presses `Enter`
- **THEN** the task updates, the editor stays open in `VIEW` mode, and the draft is marked clean

#### Scenario: Cancel discards edits

- **WHEN** the user edits the description then presses `esc`
- **THEN** the popup steps back to `VIEW` with the original values restored

### Requirement: View closes plainly

The system SHALL close the popup without changes on `esc` or `q` while in `VIEW` mode, and SHALL keep the popup open with no hint toast on `Enter` in `VIEW` mode.

#### Scenario: Close from view

- **WHEN** the user presses `esc` with the popup in `VIEW`
- **THEN** the popup closes and the list is unchanged

#### Scenario: Enter stays open in view

- **WHEN** the user presses `Enter` with the popup in `VIEW`
- **THEN** the popup stays open

## ADDED Requirements

### Requirement: Start shows relative age

The system SHALL display the `start` value as a relative age derived from the task's real `start` timestamp (`just now`, `15 min ago`, `1 hour ago`, `3 days ago`), blank when the task was never started. The label SHALL read `start`, never `start/end`.

#### Scenario: Started task shows its age

- **WHEN** the popup renders a task started one hour ago
- **THEN** the start row reads `start   1 hour ago`

#### Scenario: Unstarted task shows blank start

- **WHEN** the popup renders a task with no `start` timestamp
- **THEN** the start value is blank

#### Scenario: Real values use their own color

- **WHEN** the read-only block renders urgency and start values alongside an empty recur and annotation
- **THEN** the urgency and start values render in their own colors while the empty labels and values stay grey
