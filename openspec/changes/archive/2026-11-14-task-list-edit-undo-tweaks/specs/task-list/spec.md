# Spec Delta

## MODIFIED Requirements

### Requirement: Cursor highlights without expanding

The system SHALL move a single `›` cursor with `j/k` (or arrows) across task rows and section headers and highlight the focused row without revealing extra lines. `Enter` on a section header SHALL collapse or expand it instead of opening a popup.

#### Scenario: Cursor moves across sections

- **WHEN** the user presses `j` on the last task of Now
- **THEN** the cursor lands on the Next header, and a further `j` lands on the first task of Next

#### Scenario: Cursor moves row to row in Inbox

- **WHEN** the user presses `j` in Inbox
- **THEN** the cursor lands on the next row with no section boundary involved

#### Scenario: Enter toggles a header

- **WHEN** the user presses `Enter` with the cursor on a section header
- **THEN** the section collapses (its rows hide) or expands, and no popup opens

### Requirement: Quick-add creates one task

The system SHALL provide a top `Add a task…` line where description keeps focus and `Enter` creates exactly one `status:pending` task, joining pasted lines and applying the view's project/due defaults. `#`, `~`, `@`, `!`, or `+` SHALL open their pickers; each picker shows only its hints, moves on `↑`/`↓` and `Tab`/`Shift+Tab`, and applies on `Enter` (tags toggle and stay; `+ more` opens the chosen picker and stays; the rest return to the description). A non-empty tag filter plus `Enter` SHALL create custom tags from the typed words (split on comma/space, stripping leading `+`/`#`) instead of only toggling existing suggestions. Typed `j`/`k` filter instead of moving. Set chips use their colors, unset chips stay grey, and the footer stays minimal. `Enter` keeps the add box open; `esc` leaves it. Quick-add MUST NOT write a section value.

#### Scenario: Paste joins into one task

- **WHEN** the user pastes three lines and presses `Enter`
- **THEN** exactly one task is created with the lines joined by spaces

#### Scenario: Trigger opens the tag picker

- **WHEN** the user types `#` in the add box
- **THEN** the tag picker opens listing existing tags and `Enter` toggles the highlighted tag without leaving the picker

#### Scenario: Custom tag is created from typed text

- **WHEN** the user types `brand-new-tag` in the tag picker filter and presses `Enter`
- **THEN** `brand-new-tag` is added to the draft tags and the picker stays open with a cleared filter

#### Scenario: Typed j filters instead of moving

- **WHEN** the user types `j` in the tag picker filter
- **THEN** the character is added to the filter and the picker stays open on the same focus

#### Scenario: Set attributes show colored chips

- **WHEN** the draft carries a tag and a due date
- **THEN** the add row shows the tag chip in cyan and the due chip in yellow instead of grey

#### Scenario: Commit stays open for rapid entry

- **WHEN** the user presses `Enter` on a non-empty description
- **THEN** exactly one task is created, the draft clears, and the add box stays open

#### Scenario: Shift+Tab moves the picker backward

- **WHEN** the user presses `Shift+Tab` in a picker
- **THEN** the highlight moves to the previous suggestion and clamps at the first item

#### Scenario: More opens the chosen picker

- **WHEN** the user presses `Enter` on `@ project` in the `+ more` list
- **THEN** the project picker opens with its hint row and remains focused

#### Scenario: Added task carries no section

- **WHEN** the user commits a task from the add box
- **THEN** the new task has no `siapsection` value

### Requirement: Completed tail is a dimmed list at the bottom

The system SHALL show completed tasks in a `COMPLETED n` section at the bottom as dimmed, struck-through rows that are cursor-navigable and collapsible like any section. `Enter` on the `COMPLETED` header SHALL collapse or expand the tail.

#### Scenario: Done work rests below

- **WHEN** tasks are completed
- **THEN** they leave the open rows and appear dimmed under the COMPLETED header

#### Scenario: Cursor reaches completed rows

- **WHEN** the user moves the cursor past the last open task
- **THEN** the cursor lands on the COMPLETED header and then on completed rows

#### Scenario: Collapsed tail hides its rows

- **WHEN** the user presses `Enter` on the COMPLETED header
- **THEN** the completed rows hide and a second `Enter` shows them again
