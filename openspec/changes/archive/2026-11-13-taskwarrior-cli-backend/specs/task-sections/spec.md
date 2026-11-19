# Spec Delta

## MODIFIED Requirements

### Requirement: App owns one section UDA

The system SHALL maintain a single user-defined attribute (`uda.siapsection`) and auto-register it via `task config` on first run without user action.

#### Scenario: First run registers UDA

- **WHEN** the app launches and the section UDA is missing
- **THEN** it registers the UDA and existing tasks without a value display under Now without any backfill write

#### Scenario: Missing value writes only on explicit change

- **WHEN** a task without a section value is displayed
- **THEN** it renders under Now and its stored value stays empty until the user moves or edits that task
