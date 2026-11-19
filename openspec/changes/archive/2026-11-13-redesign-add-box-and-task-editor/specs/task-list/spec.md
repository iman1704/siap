# Spec Delta

## MODIFIED Requirements

### Requirement: Quick-add creates one task

The system SHALL provide a top `Add a task…` line where the description keeps focus and `Enter` creates exactly one `status:pending` task, joining pasted multi-line input into a single title and applying the current view's project/due defaults. Typing `#`, `~`, `@`, `!`, or `+` SHALL open the tags, due, project, priority, or more picker; each picker SHALL show only its own hints. The picker list SHALL move on `↑`/`↓`/`Tab` and apply on `Enter` (tags toggle and stay for multi-entry; the rest apply back to the description), so typed `j`/`k` filter instead of moving. Set attributes SHALL render as colored chips in the add row (`#` cyan, `~` yellow, project and `!` priority in their own colors) while unset ones stay grey, and the footer SHALL stay minimal. `Enter` on the description SHALL keep the add box open for rapid entry; `esc` SHALL leave it.

#### Scenario: Paste joins into one task

- **WHEN** the user pastes three lines and presses `Enter`
- **THEN** exactly one task is created with the lines joined by spaces

#### Scenario: Trigger opens the tag picker

- **WHEN** the user types `#` in the add box
- **THEN** the tag picker opens listing existing tags and `Enter` toggles the highlighted tag without leaving the picker

#### Scenario: Typed j filters instead of moving

- **WHEN** the user types `j` in the tag picker filter
- **THEN** the character is added to the filter and the picker stays open on the same focus

#### Scenario: Set attributes show colored chips

- **WHEN** the draft carries a tag and a due date
- **THEN** the add row shows the tag chip in cyan and the due chip in yellow instead of grey

#### Scenario: Commit stays open for rapid entry

- **WHEN** the user presses `Enter` on a non-empty description
- **THEN** exactly one task is created, the draft clears, and the add box stays open
