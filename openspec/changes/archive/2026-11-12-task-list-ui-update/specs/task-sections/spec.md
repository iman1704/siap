# Spec Delta

## MODIFIED Requirements

### Requirement: Default sections exist

The system SHALL provide default sections Now, Next, and Later for every view except Inbox. Inbox SHALL render as a flat list with no section headers.

#### Scenario: New task gets a default section

- **WHEN** a task is created and has no section value
- **THEN** it appears under Now

#### Scenario: Inbox shows no sections

- **WHEN** the user opens Inbox with tasks holding Now, Next, and Later values
- **THEN** no section headers render and rows appear as one flat list ordered by urgency

### Requirement: Moving a task sets the section

The system SHALL move a task between sections by updating its section UDA value when the user presses `m` on a highlighted task, except in Inbox where `m` is disabled with a hint and the hidden section value is preserved.

#### Scenario: Move task to Next

- **WHEN** the user moves a task from Now to Next
- **THEN** its section value updates and it renders under Next on next refresh

#### Scenario: Move is refused in Inbox

- **WHEN** the user presses `m` in Inbox
- **THEN** the section value stays unchanged and a hint explains sections live in project views

### Requirement: Sections are separated by blank space

The system SHALL render each section as a quiet uppercase label with its count, cushioned by blank lines above and below, with no boxes, rules, or borders. This requirement does not apply to Inbox, which has no section labels.

#### Scenario: Sections breathe

- **WHEN** the list shows Now and Next
- **THEN** a blank line separates the last Now row from the Next header

#### Scenario: Inbox has no separators for sections

- **WHEN** the list shows Inbox tasks
- **THEN** no uppercase section labels or section separator blanks render between rows
