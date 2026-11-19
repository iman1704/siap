# Spec Delta

## MODIFIED Requirements

### Requirement: Search input captures typing
The system SHALL give Search an input mode and a list mode. In input mode printable keys append to the end of the query instead of firing list hotkeys, `Backspace` removes the last query character, and the query line renders with a trailing inline `▌` block glyph when in input mode (plain text with an `(i input)` hint in list mode). `←`/`→` SHALL be consumed without moving any cursor so sidebar focus never steals typing. `i` focuses input from list mode, and `esc` leaves input while keeping the query. `esc` in list mode SHALL clear a non-empty query without leaving Search. `Tab`/`Shift+Tab`/`↑`/`↓` SHALL move the list cursor in both modes.

#### Scenario: Typing never fires hotkeys
- **WHEN** the user types `q`, `l`, or `?` in Search input mode
- **THEN** those characters enter the query and no view switch, focus change, or help overlay occurs

#### Scenario: Focus input from list mode
- **WHEN** the user presses `i` on Search in list mode
- **THEN** input mode focuses and the query stays editable

#### Scenario: Exit input keeps the query
- **WHEN** the user presses `esc` in Search input mode
- **THEN** list mode returns with the query and its results intact

#### Scenario: Esc clears from list mode
- **WHEN** the user presses `esc` in Search list mode with a non-empty query
- **THEN** the query clears, all open tasks return, and the Search view stays active

#### Scenario: Tab moves the results cursor
- **WHEN** the user presses `Tab` or `Shift+Tab` in either Search mode
- **THEN** the list cursor moves to the next or previous row without editing the query

#### Scenario: Input mode shows the inline block cursor
- **WHEN** the query is `nadia` in input mode
- **THEN** the line renders `nadia▌` and list mode renders `nadia` with no cursor plus the `(i input)` hint only when empty

#### Scenario: Arrows never move the query cursor
- **WHEN** the user presses `←` or `→` in Search input mode
- **THEN** the query text is unchanged and focus stays in the query

### Requirement: Sidebar previews the view under the cursor
While the sidebar is focused, moving the cursor SHALL switch the active view immediately without leaving sidebar focus. `Enter` or `l` SHALL confirm the view and focus the list, and entering Search SHALL start in input mode. On launch the sidebar highlight SHALL match the initial view (first permanent row).

#### Scenario: Arrow previews each view
- **WHEN** the user presses `j` or `k` in the focused sidebar
- **THEN** the list shows the view under the cursor while the sidebar stays focused

#### Scenario: Enter confirms the preview
- **WHEN** the user presses `Enter` or `l` on a sidebar row
- **THEN** that view stays active and the list gains focus

#### Scenario: Entering Search starts in input mode
- **WHEN** the user selects Search from the sidebar
- **THEN** Search opens in input mode with the cursor ready to type

#### Scenario: Launch highlight matches the initial view
- **WHEN** the app launches
- **THEN** the highlighted sidebar row is the same view that the list shows
