# Spec Delta

## MODIFIED Requirements

### Requirement: Sorting is section then urgency

The system SHALL order tasks in project views grouped by the active section registry order (Now/Next/Later when present, then custom sections alphabetically) with taskwarrior urgency descending within each section. Unsectioned tasks SHALL render as a flat urgency-descending list above the first section header. Search, Inbox, Today, and All SHALL order by taskwarrior urgency descending with no section grouping.

#### Scenario: Urgent task surfaces first

- **WHEN** two tasks share the same section
- **THEN** the higher-urgency task renders first

#### Scenario: Inbox ignores sections

- **WHEN** two Inbox tasks hold different section values
- **THEN** the higher-urgency task renders first regardless of section

#### Scenario: Today and All ignore sections

- **WHEN** Today or All contains tasks carrying section values
- **THEN** no section headers render and rows sort by urgency descending

#### Scenario: Search ignores sections

- **WHEN** Search matches tasks carrying section values
- **THEN** results render as one urgency-descending list with no headers

#### Scenario: Unsectioned tasks lead

- **WHEN** a project view contains unsectioned tasks and sectioned tasks
- **THEN** the unsectioned tasks render as a flat list before the first section header

#### Scenario: Custom sections sort alphabetically

- **WHEN** two custom sections are present after Now, Next, and Later
- **THEN** they render in alphabetical order

### Requirement: Quick-add creates one task

The system SHALL provide a top `Add a task…` line where description keeps focus and `Enter` creates exactly one `status:pending` task, joining pasted lines and applying the view's project/due defaults. `#`, `~`, `@`, `!`, or `+` SHALL open their pickers; each picker shows only its hints, moves on `↑`/`↓` and `Tab`/`Shift+Tab`, and applies on `Enter` (tags toggle and stay; `+ more` opens the chosen picker and stays; the rest return to the description). Typed `j`/`k` filter instead of moving. A typed non-empty project or due filter SHALL apply as-is on `Enter`, creating the project or using the typed date, and `no date` SHALL clear the due. Empty Tags and Due filters SHALL show a grey example (`e.g. work, urgent` / `e.g. 2026-11-20`) that vanishes on input. Set chips use their colors, unset chips stay grey, and the footer stays minimal. `Enter` keeps the add box open; `esc` leaves it. Quick-add MUST NOT write a section value, and Search MUST NOT show an add box.

#### Scenario: Paste joins into one task

- **WHEN** the user pastes three lines and presses `Enter`
- **THEN** exactly one task is created with the lines joined by spaces

#### Scenario: Trigger opens the tag picker

- **WHEN** the user types `#` in the add box
- **THEN** the tag picker opens listing existing tags and `Enter` toggles the highlighted tag without leaving the picker

#### Scenario: Custom tag is created from typed text

- **WHEN** the user types `brand-new-tag` in the tag picker filter and presses `Enter`
- **THEN** `brand-new-tag` is added to the draft tags and the picker stays open with a cleared filter

#### Scenario: Custom project is created from typed text

- **WHEN** the user types `garden` in the project picker and presses `Enter`
- **THEN** the draft project is `garden` and the created task carries `project:garden` even though no such project existed

#### Scenario: Custom due date is used as-is

- **WHEN** the user types `2026-12-01` in the due picker and presses `Enter`
- **THEN** the draft due is `2026-12-01` and the created task carries that due date

#### Scenario: No date clears the due

- **WHEN** the user types `no date` in the due picker and presses `Enter`
- **THEN** the draft due clears and the created task has no due date

#### Scenario: Empty pickers show grey examples

- **WHEN** the Tags or Due picker is focused with an empty filter
- **THEN** the input line shows `e.g. work, urgent` or `e.g. 2026-11-20` in dim text, and typing replaces it

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

#### Scenario: Search offers no add box

- **WHEN** the Search view renders
- **THEN** no add box or `a` trigger is available there

## ADDED Requirements

### Requirement: Overdue has no row trailer

The system SHALL indicate an overdue task through its red due label and Today inclusion only, and MUST NOT append an `overdue` text trailer to the row. The `blocked` trailer SHALL remain for parents with open children.

#### Scenario: Overdue row reads without a trailer

- **WHEN** an overdue task renders in a list
- **THEN** its due date renders in red and no `overdue` text trails the row

#### Scenario: Blocked trailer still renders

- **WHEN** a parent has open subtasks
- **THEN** its row shows the `blocked` trailer

## REMOVED Requirements

### Requirement: Find filters live

**Reason**: Replaced by the dedicated Search view with its own sidebar entry, input/list modes, and a query that clears on exit.

**Migration**: Use the Search row above Inbox and type to filter; `/` no longer opens a find mode and the `FIND` bar label is gone.
