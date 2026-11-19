# Spec Delta

## Purpose

Pins the WarGames Tokyo Night palette and the whitespace-first layout language so every screen stays coherent.

## ADDED Requirements

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

The system SHALL use text glyphs for state: `›` cursor, `[×]`/`[ ]` selection, `○`/`◉`/`✓` checkbox, and `●` for project dots and the active marker.

#### Scenario: State reads without color

- **WHEN** a selected active task renders
- **THEN** its `[×]` box and `◉` mark read correctly even ignoring color
