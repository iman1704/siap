# Spec Delta

## ADDED Requirements

### Requirement: Headers collapse on Enter

The system SHALL let the user collapse or expand any section header (including `COMPLETED`) by pressing `Enter` with the cursor on it. A collapsed section SHALL hide its rows and show a collapsed marker; expanding SHALL restore them. Collapse state SHALL survive refreshes within the session.

#### Scenario: Collapse a section

- **WHEN** the user presses `Enter` on the `NOW` header
- **THEN** the Now rows hide and the header shows the collapsed marker

#### Scenario: Collapse the completed tail

- **WHEN** the user presses `Enter` on the `COMPLETED` header
- **THEN** the completed rows hide and a second `Enter` shows them again

#### Scenario: Collapse survives refresh

- **WHEN** the user collapses `NOW` and presses `r`
- **THEN** `NOW` stays collapsed after the refresh
