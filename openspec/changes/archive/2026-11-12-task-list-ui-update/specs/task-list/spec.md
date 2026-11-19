# Spec Delta

## MODIFIED Requirements

### Requirement: Rows show primary attributes only

The system SHALL render each task row with checkbox state, description, due, priority (H/M/L displayed as P1/P2/P3), tags, and project marker, except in project views where the project marker SHALL be hidden because the view already scopes to one project. Secondary attributes and urgency numbers MUST NOT render inline. The Enter popup SHALL always show the project field regardless of view.

#### Scenario: Row hides secondary fields

- **WHEN** the list renders a task with depends, recur, and annotations
- **THEN** only description, due, priority, tags, and project are visible until Enter opens the popup

#### Scenario: Project view hides the project dot

- **WHEN** the list renders a `work` task inside the `work` project view
- **THEN** no `● work` marker renders in the row, while the Enter popup still shows the project field

#### Scenario: Mixed views keep the project dot

- **WHEN** the list renders tasks in Today or All
- **THEN** each row with a non-empty project still shows its `● project` marker

### Requirement: Cursor highlights without expanding

The system SHALL move a single `›` cursor with `j/k` (or arrows) and highlight the row without revealing extra lines.

#### Scenario: Cursor moves across sections

- **WHEN** the user presses `j` on the last task of Now
- **THEN** the cursor lands on the first task of Next

#### Scenario: Cursor moves row to row in Inbox

- **WHEN** the user presses `j` in Inbox
- **THEN** the cursor lands on the next row with no section boundary involved

### Requirement: Sorting is section then urgency

The system SHALL order tasks grouped by section (Now, Next, Later, then custom) and by taskwarrior urgency descending within each section, except in Inbox which SHALL order by taskwarrior urgency descending with no section grouping.

#### Scenario: Urgent task surfaces first

- **WHEN** two tasks share the Now section
- **THEN** the higher-urgency task renders first

#### Scenario: Inbox ignores sections

- **WHEN** two Inbox tasks hold different section values
- **THEN** the higher-urgency task renders first regardless of section

## REMOVED Requirements

### Requirement: Active tasks carry a full-row tint

**Reason**: Replaced by a single checkbox indicator; the tint collided with cursor/selection highlighting and tripled the active signal.
**Migration**: Active state is now shown only by the mint `◉` checkbox defined below; remove tint and `● active` text from list rows.

## ADDED Requirements

### Requirement: Active tasks show a single checkbox marker

The system SHALL render an active task with exactly one active indicator: the mint `◉` checkbox. Full-row tint and `● active` text MUST NOT render for active state; tint is reserved for cursor and selection only.

#### Scenario: Started task shows checkbox only

- **WHEN** the user starts a task
- **THEN** its checkbox becomes the mint `◉` with no tint background and no `● active` text

#### Scenario: Cursor keeps its own highlight

- **WHEN** the cursor rests on an active task
- **THEN** the row shows the cursor `›` marker and highlight plus the mint `◉`, with no `● active` text
