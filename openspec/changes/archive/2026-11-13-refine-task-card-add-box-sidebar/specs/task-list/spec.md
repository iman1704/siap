# Spec Delta

## MODIFIED Requirements

### Requirement: Children render indented under parents

The system SHALL render each open subtask indented directly beneath its parent with the same marker, start, and metadata rules as top-level rows, in a stable order.

#### Scenario: Subtask nests visually

- **WHEN** a parent with one open subtask renders
- **THEN** the child row appears indented on the line below the parent

#### Scenario: Child follows the row marker rules

- **WHEN** an active subtask is selected
- **THEN** its indented row shows the `▌` selection accent and the mint `◉` start indicator

### Requirement: Sorting is section then urgency

The system SHALL order tasks grouped by the active section registry order (unsectioned first, then Now/Next/Later when present, then custom sections alphabetically) and by taskwarrior urgency descending within each section, except in Inbox which SHALL order by taskwarrior urgency descending with no section grouping.

#### Scenario: Urgent task surfaces first

- **WHEN** two tasks share the same section
- **THEN** the higher-urgency task renders first

#### Scenario: Inbox ignores sections

- **WHEN** two Inbox tasks hold different section values
- **THEN** the higher-urgency task renders first regardless of section

#### Scenario: Unsectioned tasks lead

- **WHEN** a view contains unsectioned tasks and sectioned tasks
- **THEN** the unsectioned group renders before the first section header

#### Scenario: Custom sections sort alphabetically

- **WHEN** two custom sections are present after Now, Next, and Later
- **THEN** they render in alphabetical order

### Requirement: Quick-add creates one task

The system SHALL provide a top `Add a task…` line where description keeps focus and `Enter` creates exactly one `status:pending` task, joining pasted lines and applying the view's project/due defaults. `#`, `~`, `@`, `!`, or `+` SHALL open their pickers; each picker shows only its hints, moves on `↑`/`↓` and `Tab`/`Shift+Tab`, and applies on `Enter` (tags toggle and stay; `+ more` opens the chosen picker and stays; the rest return to the description). Typed `j`/`k` filter instead of moving. Set chips use their colors, unset chips stay grey, and the footer stays minimal. `Enter` keeps the add box open; `esc` leaves it. Quick-add MUST NOT write a section value.

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

#### Scenario: Shift+Tab moves the picker backward

- **WHEN** the user presses `Shift+Tab` in a picker
- **THEN** the highlight moves to the previous suggestion and clamps at the first item

#### Scenario: More opens the chosen picker

- **WHEN** the user presses `Enter` on `@ project` in the `+ more` list
- **THEN** the project picker opens with its hint row and remains focused

#### Scenario: Commit stays open for rapid entry

- **WHEN** the user presses `Enter` on a non-empty description
- **THEN** exactly one task is created, the draft clears, and the add box stays open

#### Scenario: Added task carries no section

- **WHEN** the user commits a task from the add box
- **THEN** the new task has no `siapsection` value

## REMOVED Requirements

### Requirement: Active tasks show a single checkbox marker

**Reason**: The pending `○` placeholder and checkbox language no longer match the row; the card now shows a start indicator only for active tasks.
**Migration**: Render the mint `◉` in the start column only when the task is active. Pending tasks render no start glyph. Do not render `● active` text or full-row tint for active state.

### Requirement: Rows show primary attributes only

**Reason**: The row is replaced by aligned columns, an active-only start indicator, and colored project names; the old requirement names a checkbox and project dot.
**Migration**: Use the new `Task rows render aligned primary attributes` requirement. Project identity moves from the `●` dot to the project name color, and the selection box is replaced by the `▌` accent.

## ADDED Requirements

### Requirement: Task rows render aligned primary attributes

The system SHALL render each open task row in one aligned line: a marker column (`›` cursor, `▌` selected), a start indicator only when active, description, then aligned project, tags, due, and priority. The project SHALL be a colored name with no dot, hidden in its own project view. Secondary attributes and urgency numbers MUST NOT render inline. The Enter popup SHALL always show the project field.

#### Scenario: Row hides secondary fields

- **WHEN** the list renders a task with depends, recur, and annotations
- **THEN** only description, project, due, priority, and tags are visible until Enter opens the popup

#### Scenario: Project view hides the project name

- **WHEN** the list renders a `work` task inside the `work` project view
- **THEN** no project name or dot renders in the row, while the Enter popup still shows the project field

#### Scenario: Mixed views color the project name

- **WHEN** the list renders tasks in Today or All
- **THEN** each row with a non-empty project shows its project name in that project's color, with no `●` dot

#### Scenario: Metadata aligns across rows

- **WHEN** two rows carry projects, tags, due dates, and priorities of different lengths
- **THEN** their project, tag, due, and priority columns start at the same positions and long values truncate with `…`

### Requirement: Active tasks show a single start indicator

The system SHALL render the mint `◉` start indicator only for a task whose status is active. Pending tasks MUST NOT render a start placeholder. Active state MUST NOT add full-row tint or `● active` text; tint is reserved for cursor and selection.

#### Scenario: Started task shows the start indicator

- **WHEN** the user starts a task
- **THEN** its start column shows the mint `◉` with no tint background and no `● active` text

#### Scenario: Pending task shows no start indicator

- **WHEN** a task is pending
- **THEN** its start column is blank

#### Scenario: Cursor keeps its own highlight

- **WHEN** the cursor rests on an active task
- **THEN** the row shows the cursor `›` marker and highlight plus the mint `◉`, with no `● active` text

### Requirement: Selected rows show an accent marker

The system SHALL mark a selected row with a `▌` accent and a tint. The cursor `›` SHALL take precedence in the marker column when a row is both selected and under the cursor; the selection count bar SHALL still reflect it.

#### Scenario: Selected row shows the accent

- **WHEN** the user presses `space` on a row
- **THEN** the row shows `▌` and a tint while the cursor remains a single `›`

#### Scenario: Cursor and selection coincide

- **WHEN** the cursor rests on a selected row
- **THEN** the marker column shows `›` and the row stays tinted, with the selection count unchanged

### Requirement: Added attributes can be removed

The system SHALL remove a set add-box attribute on `Backspace` when the focused picker's filter is empty. Project, due, and priority SHALL clear; tags SHALL remove the last added tag; unset attributes and the More picker SHALL be a no-op. The footer SHALL hint the removal when a set attribute is focused.

#### Scenario: Backspace clears the project

- **WHEN** the project picker has a project set and an empty filter, and the user presses `Backspace`
- **THEN** the project chip clears and the project picker stays focused

#### Scenario: Backspace removes the last tag

- **WHEN** the tag picker has an empty filter and at least one tag, and the user presses `Backspace`
- **THEN** the most recently added tag is removed from the draft

#### Scenario: Backspace edits a non-empty filter

- **WHEN** the project picker filter contains text and the user presses `Backspace`
- **THEN** the last filter character is removed and the project chip is unchanged

#### Scenario: Unset attribute is a no-op

- **WHEN** the due picker has no due value and an empty filter, and the user presses `Backspace`
- **THEN** the draft is unchanged
