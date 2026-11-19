# Spec Delta

## MODIFIED Requirements

### Requirement: Minimal keymap only

The system SHALL implement exactly `j/k` move, `space` select, `Enter` open/toggle, `a` new (except in Search), `n` new subtask (list and popup), `d` done/undone, `s` start/stop, `D` delete, `u` undo, `m` move section, `t` tags, `?` help, `q` quit, `1`-`5` view shortcuts with `5` selecting the first project, and `i`/`esc` for Search input, plus `Ctrl+z` undo, VISUAL `v` range, and `esc` cancel, with the bar hints and `?` overlay matching the implementation. `/` MUST NOT open a find mode.

#### Scenario: Help matches keys

- **WHEN** the user presses `?`
- **THEN** the overlay lists only the keys above with correct descriptions

#### Scenario: No find hotkey

- **WHEN** the user presses `/`
- **THEN** no find mode opens and the list is unchanged

#### Scenario: Subtask hotkey works from the list

- **WHEN** the user presses `n` with the cursor on a task row
- **THEN** a subtask is created under that task

## ADDED Requirements

### Requirement: Progress toasts expire

The system SHALL clear a progress or result toast after about 2.5 seconds so messages such as `saved · esc closes`, `edits discarded`, or `completed 1 task(s)` never persist indefinitely.

#### Scenario: Toast clears itself

- **WHEN** a toast is set and the user makes no further change
- **THEN** the bar clears after about 2.5 seconds

#### Scenario: New feedback restarts the timer

- **WHEN** a second toast replaces the first before it expires
- **THEN** the second toast stays visible for its own ~2.5 seconds
