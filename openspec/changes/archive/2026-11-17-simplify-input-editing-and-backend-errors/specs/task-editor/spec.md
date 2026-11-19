# Spec Delta

## MODIFIED Requirements

### Requirement: Edit mode edits with pickers
The system SHALL enter `EDIT` mode on `e` and edit only the focused field at once: keystrokes append at the end of the field's text, `Backspace` removes the last character, and the focused line renders a trailing inline `▌` block glyph at the text cursor (at the end of the example text when empty, which vanishes on input). `←`/`→` SHALL NOT move the insertion point, and `Tab`/`Shift+Tab`/`↑`/`↓` move only the highlighted suggestion. `Enter` applies the highlighted suggestion and saves, `esc` discards only the focused field, and `[`/`]` keep cycling the project, priority, and status pickers. Urgency, start, recur, and annotation are display-only and never take focus. Typing `j`/`k` MUST insert text and MUST NOT move the field.

#### Scenario: Cycle priority picker
- **WHEN** the user focuses priority in `EDIT` mode and presses `]`
- **THEN** the priority advances (H → M → L → H) and the draft is marked unsaved

#### Scenario: Modal movement and typing stay separate
- **WHEN** the user presses `e`, then `j`
- **THEN** the field does not move and the character `j` is typed into the focused field

#### Scenario: Typing never moves the field
- **WHEN** the user types characters in `EDIT` mode
- **THEN** the focus stays on the selected field

#### Scenario: Tab moves only the highlight
- **WHEN** the user presses `Tab`, `Shift+Tab`, `↑`, or `↓` in `EDIT` mode
- **THEN** only the suggestion highlight moves and the focused field never changes

#### Scenario: Arrows move the text cursor
- **WHEN** the user presses `←` or `→` in `EDIT` mode
- **THEN** the key is consumed with no movement: the insertion point stays at the end of the field and the text is unchanged

#### Scenario: Focused field shows the inline block cursor
- **WHEN** the user focuses a non-empty field in `EDIT` mode
- **THEN** the line renders the text with `▌` inserted at the end and no native terminal cursor is placed

#### Scenario: Esc discards only the focused field
- **WHEN** the user edits one field, moves to a second field, edits it, and presses `esc`
- **THEN** only the second field reverts and the first edit remains unsaved in the draft

### Requirement: Save validates and closes
The system SHALL save on `Enter` in `EDIT` mode after applying the highlighted suggestion via a single `task <uuid> modify` write for description, project, due, priority, and tags, land back in `VIEW` mode with the editor still open and the draft marked clean, and reject an empty description with a hint while keeping the popup open.

#### Scenario: Save requires a description
- **WHEN** the user clears the description and presses `Enter`
- **THEN** the popup stays open and reports that a description is required

#### Scenario: Save stays open
- **WHEN** the user edits the description in `EDIT` mode and presses `Enter`
- **THEN** the task updates, the editor stays open in `VIEW` mode, and the draft is marked clean

#### Scenario: Enter applies the hint before saving
- **WHEN** the user types `work` in the project field and presses `Enter` with a matching hint highlighted
- **THEN** the highlighted project is applied and the save persists it

#### Scenario: Cancel discards edits
- **WHEN** the user edits the description then presses `esc`
- **THEN** the popup steps back to `VIEW` with the description restored and any unsaved edits in other fields preserved
