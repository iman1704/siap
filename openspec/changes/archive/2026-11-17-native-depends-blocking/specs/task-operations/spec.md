# Spec Delta

## MODIFIED Requirements

### Requirement: Minimal keymap only

The system SHALL implement exactly `j/k` move, `space` select, `Enter` open/toggle, `a` new (except in Search), `d` done/undone, `s` start/stop, `D` delete, `u` undo, `m` move section, `t` tags, `?` help, `q` quit, `1`-`5` view shortcuts with `5` selecting the first project, and `i`/`esc` for Search input, plus `Ctrl+z` undo, VISUAL `v` range, and `esc` cancel, with the bar hints and `?` overlay matching the implementation. `/` MUST NOT open a find mode. `n` SHALL be a no-op everywhere.

#### Scenario: Help matches keys

- **WHEN** the user presses `?`
- **THEN** the overlay lists only the keys above with correct descriptions

#### Scenario: No find hotkey

- **WHEN** the user presses `/`
- **THEN** no find mode opens and the list is unchanged

#### Scenario: Subtask hotkey works from the list

- **WHEN** the user presses `n` with the cursor on a task row
- **THEN** no task is created and the list is unchanged

#### Scenario: List n does nothing

- **WHEN** the user presses `n` with the cursor on a task row
- **THEN** no task is created and the list is unchanged

## ADDED Requirements

### Requirement: Delete is single-only

The system SHALL delete exactly the confirmed tasks on `D` + `y` with no cascade to other tasks; related `depends` links resolve on the next backend refresh.

#### Scenario: Delete leaves blockers alone

- **WHEN** the user deletes a blocked task with an open blocker
- **THEN** only the confirmed task leaves and the blocker stays open
