# Spec Delta

## MODIFIED Requirements

### Requirement: Failures toast and preserve state
The system SHALL define a failed backend command by non-zero process exit status only, report it in the toast with the trimmed `stderr` (falling back to trimmed `stdout`), and keep the existing list, selection, and cursor unchanged. Taskwarrior validation text printed with exit status 0 SHALL be treated as success.

#### Scenario: Failed write keeps the list
- **WHEN** a write command exits non-zero
- **THEN** the toast explains the failure and no rows change

#### Scenario: Exit-zero output counts as success
- **WHEN** a write command exits zero even with warning-like text on stdout or stderr
- **THEN** the write counts as successful and the list refreshes from the backend
