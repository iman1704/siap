# Spec Delta — `ui-theme`

## MODIFIED Requirements

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
