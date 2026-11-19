# Spec Delta — `task-operations`

## REMOVED Requirements

### Requirement: Minimal keymap only

**Reason**: The keymap gained the `/` Search shortcut, the footer dropped the `Enter` and `r` hints in favor of `D delete` and `/ search`, and the `?` overlay was regrouped, so the old enumeration and "no find hotkey" contract no longer hold.

**Migration**: Use the replacement requirements `List keymap stays minimal` and `Footer hints stay minimal`.

## ADDED Requirements

### Requirement: List keymap stays minimal

The system SHALL keep the list keymap to `j/k`, `space`, `Enter`, `a`, `d`, `s`, `D`, `u`, `m`, `t`, `?`, `q`, `1`-`5`, `/`, `i`/`esc` for Search input, `v` range, and `esc`, plus `Ctrl+z` undo. `r` SHALL stay available as the unhinted manual refresh key, and `n` SHALL be a no-op everywhere. The `?` overlay SHALL list the current keymap with correct descriptions.

#### Scenario: Help matches keys

- **WHEN** the user presses `?`
- **THEN** the overlay lists the current keymap with correct descriptions

#### Scenario: Slash opens Search

- **WHEN** the user presses `/` in a list view
- **THEN** the Search view opens in input mode with the query ready to type and no separate find mode opens

#### Scenario: Subtask hotkey works from the list

- **WHEN** the user presses `n` with the cursor on a task row
- **THEN** no task is created and the list is unchanged

#### Scenario: List n does nothing

- **WHEN** the user presses `n` with the cursor on a task row
- **THEN** no task is created and the list is unchanged

### Requirement: Footer hints stay minimal

The system SHALL render the list footer as `j/k move · space select · a new · d done · s start · D delete · u undo · / search · ? help · q quit`, without the old `Enter` or `r` hints.

#### Scenario: Footer hints stay minimal

- **WHEN** the list footer renders
- **THEN** it advertises `D delete` and `/ search` and omits the old `Enter open/toggle` and `r refresh` hints

### Requirement: Help overlay groups keys by surface

The system SHALL render the `?` overlay with `List`, `Edit`, and `Search` group headings, listing the current list, popup, and Search keys under them and omitting the removed `Enter open/toggle` and `r refresh from backend` lines.

#### Scenario: Group headings render

- **WHEN** the user opens the `?` overlay
- **THEN** `List`, `Edit`, and `Search` headings separate the key groups

#### Scenario: Removed hints are gone

- **WHEN** the help overlay renders
- **THEN** no `Enter open/toggle` or `r refresh` line appears

#### Scenario: Note keys are listed

- **WHEN** the help overlay renders
- **THEN** it lists `a note`, `D del note`, and `[/] select` for the popup annotation row
