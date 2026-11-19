# Spec Delta — `task-sections`

## ADDED Requirements

### Requirement: Section prompts edit at the caret

The system SHALL edit the `A`, `R`, and `X` footer prompt buffer at a real caret: printable keys insert at it, `←`/`→` move it, and `Backspace`/`Delete` remove around it. The native terminal cursor SHALL sit at the caret in the footer instead of an inline `▌` glyph.

#### Scenario: Caret follows the typed name

- **WHEN** the user presses `A` and types `Someday`
- **THEN** the footer renders the prompt prefix followed by `Someday` and the terminal cursor sits after the last character

#### Scenario: Caret moves and edits in place

- **WHEN** the prompt reads `Now`, the user presses `←`, then types `X`
- **THEN** the buffer becomes `NoXw` with the caret after `X`

#### Scenario: Backspace and Delete edit at the caret

- **WHEN** the prompt reads `Now` with the caret after `No`
- **THEN** `Backspace` produces `Nw` and `Delete` from the same state produces `No`

#### Scenario: Prompt shows no block glyph

- **WHEN** any section prompt renders
- **THEN** no inline `▌` glyph appears and the terminal cursor marks the insertion point
