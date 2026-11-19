# Spec Delta — `task-views`

## REMOVED Requirements

### Requirement: Search input captures typing

**Reason**: The query is now edited at a real text caret with `←`/`→` movement and `Delete`, and the inline `▌` glyph was replaced by the native terminal cursor, so the old append-only, cursor-frozen contract no longer holds.

**Migration**: Use the replacement requirements `Search input edits the query at the caret` and `Search list mode clears the query`.

## ADDED Requirements

### Requirement: Search input edits the query at the caret

The system SHALL let Search input mode insert printable keys at the query caret instead of firing list hotkeys, move that caret with `←`/`→`, and remove around it with `Backspace`/`Delete`, placing the native terminal cursor at the caret with no inline `▌` glyph. It SHALL leave input on `esc` while keeping the query and SHALL focus input on `i` with the caret at the query's end.

#### Scenario: Typing never fires hotkeys

- **WHEN** the user types `q`, `l`, or `?` in Search input mode
- **THEN** those characters enter the query and no view switch, focus change, or help overlay occurs

#### Scenario: Typing inserts at the query caret

- **WHEN** the query reads `abc` with the caret after `a` and the user types `X`
- **THEN** the query becomes `aXbc` and the caret follows the inserted character

#### Scenario: Arrows move the query caret

- **WHEN** the user presses `←` or `→` in Search input mode
- **THEN** the query caret moves one character without changing the text and focus stays in the query

#### Scenario: Backspace edits before the caret

- **WHEN** the query reads `abc` with the caret after `b` and the user presses `Backspace`
- **THEN** the query becomes `ac` with the caret after `a`

#### Scenario: Delete edits after the caret

- **WHEN** the query reads `abc` with the caret after `b` and the user presses `Delete`
- **THEN** the query becomes `ac` with the caret after `b`

#### Scenario: Focus input from list mode

- **WHEN** the user presses `i` on Search in list mode
- **THEN** input mode focuses with the caret at the query's end and the query stays editable

#### Scenario: Exit input keeps the query

- **WHEN** the user presses `esc` in Search input mode
- **THEN** list mode returns with the query and its results intact

#### Scenario: Input mode shows the terminal caret

- **WHEN** the query is `nadia` in input mode
- **THEN** the line renders `nadia` with the native terminal cursor after it, and list mode renders `nadia` with no cursor

#### Scenario: Empty query starts under the caret

- **WHEN** input mode is focused with an empty query
- **THEN** `Type to search…` renders with the terminal cursor at its start

### Requirement: Search list mode clears the query

The system SHALL clear a non-empty query on `esc` in Search list mode without leaving Search, and SHALL move the results cursor with `Tab`/`Shift+Tab`/`↑`/`↓` in both modes. List mode SHALL render the plain query with an `(i input)` hint when empty.

#### Scenario: Esc clears from list mode

- **WHEN** the user presses `esc` in Search list mode with a non-empty query
- **THEN** the query clears, all open tasks return, and the Search view stays active

#### Scenario: Tab moves the results cursor

- **WHEN** the user presses `Tab` or `Shift+Tab` in either Search mode
- **THEN** the list cursor moves to the next or previous row without editing the query

#### Scenario: List mode hints at input

- **WHEN** Search is in list mode with an empty query
- **THEN** the line renders `Type to search…  (i input)` with no terminal cursor
