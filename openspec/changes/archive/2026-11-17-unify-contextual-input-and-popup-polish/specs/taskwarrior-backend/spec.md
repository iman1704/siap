# Spec Delta — `taskwarrior-backend`

## MODIFIED Requirements

### Requirement: Priority defaults and urgency stay read-only

The system SHALL render absent priority as L and SHALL treat urgency as a read-only computed value that refreshes from the backend.

#### Scenario: Missing priority renders P3

- **WHEN** a task has no priority value
- **THEN** the row shows L styling (renamed from `P3`)

#### Scenario: Urgency refreshes after save

- **WHEN** the user changes priority and saves
- **THEN** the displayed urgency updates on the next refresh
