# Spec Delta

## MODIFIED Requirements

### Requirement: Popup opens read-only

The system SHALL open a modal popup on `Enter` showing all nine editable fields in one flat list (description, project, due, priority, tags, depends, recur, status, annotation) followed by the read-only urgency and start/end values, in `VIEW` mode. The project field SHALL show the project name only with no `●` dot.

#### Scenario: Open popup from list

- **WHEN** the user presses `Enter` on a highlighted task
- **THEN** the popup opens with all current values populated and marked `VIEW`

#### Scenario: Single list without group headers

- **WHEN** the popup renders its fields
- **THEN** no PRIMARY or SECONDARY group headers render and the project shows `work`, never `● work`

### Requirement: View mode ignores typing

The system SHALL ignore text input while the popup is in `VIEW` mode, leaving the draft clean and showing no unsaved indicator.

#### Scenario: Typing does nothing in view

- **WHEN** the user types characters with the popup in `VIEW`
- **THEN** no field changes and the popup stays clean

### Requirement: Edit mode edits with pickers

The system SHALL enter `EDIT` navigate mode on `e`, where `j`/`k` (or arrows/`Tab`) move between the nine editable fields and `i` starts typing in the focused field. In typing mode the system SHALL edit text on keystrokes, apply the highlighted inline suggestion on `Tab`, move the suggestion selection on `↑`/`↓`, and keep `[`/`]` cycling the project, priority, and status pickers. Urgency and start/end are display-only and skipped. Typing `j`/`k` MUST enter text and MUST NOT move the field.

#### Scenario: Cycle priority picker

- **WHEN** the user focuses priority in `EDIT` typing mode and presses `]`
- **THEN** the priority advances (H → M → L → H) and the draft is marked unsaved

#### Scenario: Modal movement and typing stay separate

- **WHEN** the user presses `e`, then `j`, then `i`, then `j`
- **THEN** the field moves once in navigate mode and the character `j` is typed into the field in typing mode

### Requirement: Save validates and closes

The system SHALL save on `Enter` in `EDIT` mode, land back in EDIT navigate mode with the editor still open and the draft marked clean, and reject an empty description with a hint while keeping the popup open.

#### Scenario: Save requires a description

- **WHEN** the user clears the description and presses `Enter`
- **THEN** the popup stays open and reports that a description is required

#### Scenario: Save stays open

- **WHEN** the user edits the description in `EDIT` typing mode and presses `Enter`
- **THEN** the task updates, the editor stays open in navigate mode, and the draft is marked clean

#### Scenario: Cancel discards edits

- **WHEN** the user edits the description then presses `esc`
- **THEN** the popup steps back to `VIEW` with the original values restored

### Requirement: View closes plainly

The system SHALL close the popup without changes on `esc` or `q` while in `VIEW` mode, and SHALL keep the popup open on `Enter` in `VIEW` mode while showing an edit hint.

#### Scenario: Close from view

- **WHEN** the user presses `esc` with the popup in `VIEW`
- **THEN** the popup closes and the list is unchanged

#### Scenario: Enter stays open in view

- **WHEN** the user presses `Enter` with the popup in `VIEW`
- **THEN** the popup stays open and shows the edit hint

## ADDED Requirements

### Requirement: Inline suggestions complete attributes

The system SHALL render an inline suggestion dropdown under the focused `EDIT` field for project, tags, due, priority, and status, filtered by the text already typed, and apply the highlighted suggestion on `Tab`.

#### Scenario: Project hints list current projects

- **WHEN** the user focuses the project field in `EDIT` typing mode
- **THEN** the dropdown lists current projects and `Tab` applies the highlighted one into the draft
