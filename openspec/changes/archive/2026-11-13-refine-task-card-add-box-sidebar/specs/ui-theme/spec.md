# Spec Delta

## MODIFIED Requirements

### Requirement: Text glyph markers

The system SHALL use text glyphs for state: `›` cursor, `▌` selection, `◉` start indicator only for active tasks, and `✓` completed. The UI MUST NOT render `[ ]`, `[×]`, `○`, or a project-dot glyph. Project identity SHALL be carried by the project name's color.

#### Scenario: State reads without color

- **WHEN** a selected active task renders
- **THEN** its `▌` accent and `◉` start indicator read correctly even ignoring color

#### Scenario: Pending row has no start glyph

- **WHEN** a pending task renders
- **THEN** its start column is blank and no `○` appears

#### Scenario: Project name carries identity

- **WHEN** a `work` task renders in a mixed view
- **THEN** `work` renders in its project color with no `●` dot

## ADDED Requirements

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
