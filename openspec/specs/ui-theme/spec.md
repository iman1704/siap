# ui-theme Specification

## Purpose

Pins the WarGames Tokyo Night palette and the whitespace-first layout language so every screen stays coherent.

## Requirements

### Requirement: Tokyo Night palette mapping

The system SHALL use the WarGames Tokyo Night palette with fixed roles: night background `#0a0f1e`, tint `#1e2742`, primary text `#a8b6d4`, bright `#d6e2ff`, secondary `#8a9abf`, faint `#3a4a6d`, focus blue `#3da9ff`, tags cyan `#5ac8ff`, active mint `#5cffc6`, completed green `#4af28a`, danger red `#ff5c6c`, warm yellow `#ffcc66`, numbers orange `#ffbd5e`, and blocked magenta `#ff7a9e`.

#### Scenario: Priority colors carry meaning

- **WHEN** rows show P1, P2, and P3
- **THEN** P1 renders red, P2 yellow, and P3 faint, matching urgency at a glance

### Requirement: Blank space groups information

The system SHALL separate the sidebar from the list with a blank gutter, cushion the quick-add, find line, sections, and completed tail with blank lines, and use no row boxes, column rules, or list borders.

#### Scenario: Airy list

- **WHEN** the list renders sections and the completed tail
- **THEN** blank lines — not lines or boxes — separate each group

### Requirement: Bright-versus-dim hierarchy

The system SHALL render the current and important elements bright (cursor row, titles, wordmark), secondary information mid-tone (counts, metadata), and faint chrome dim (hints, gutters, dividers).

#### Scenario: Cursor pops quietly

- **WHEN** the cursor rests on a row
- **THEN** its description renders bold bright with a blue marker while neighbors stay mid-tone

### Requirement: Text glyph markers

The system SHALL use text glyphs for state: `›` cursor, `▌` selection, `◉` start indicator only for active tasks, `✓` completed, and a blue `↻` recurrence badge. The UI MUST NOT render `[ ]`, `[×]`, `○`, or a project-dot glyph. Project identity SHALL be carried by the project name's color.

#### Scenario: State reads without color

- **WHEN** a selected active task renders
- **THEN** its `▌` accent and `◉` start indicator read correctly even ignoring color

#### Scenario: Pending row has no start glyph

- **WHEN** a pending task renders
- **THEN** its start column is blank and no `○` appears

#### Scenario: Recurring row carries the recurrence glyph

- **WHEN** a task repeats weekly
- **THEN** its row shows `↻ weekly` in the recurrence blue before any `blocked` trailer

#### Scenario: Project name carries identity

- **WHEN** a `work` task renders in a mixed view
- **THEN** `work` renders in its project color with no `●` dot

### Requirement: Project colors are deterministic

The system SHALL assign each project a color deterministically from the palette's non-danger accents (blue, mint, yellow, magenta, cyan, and orange). The color SHALL remain stable across refreshes and SHALL be used in the sidebar and on row project names. When two project names collide, the system SHALL resolve to the next free palette color so currently listed projects have distinct colors.

#### Scenario: Same project keeps its color

- **WHEN** the app refreshes and the project set is unchanged
- **THEN** every project keeps the same color

#### Scenario: Current projects get distinct colors

- **WHEN** two projects would resolve to the same palette slot
- **THEN** the later project in sorted order takes the next free palette color

#### Scenario: Sidebar and row agree

- **WHEN** a project renders in the sidebar and on a task row
- **THEN** both use the same project color
