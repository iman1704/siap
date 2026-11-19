# Spec Delta

## Purpose

Defines the Enter popup editor that exposes taskwarrior attributes as a labeled form with a read-only default, so users never memorize modify syntax.

## ADDED Requirements

### Requirement: Popup opens read-only

The system SHALL open a modal popup on `Enter` showing primary fields (description, project, due, priority, tags) on top and secondary fields (depends, recur, urgency, start/end, status, annotations) below, in `VIEW` mode.

#### Scenario: Open popup from list

- **WHEN** the user presses `Enter` on a highlighted task
- **THEN** the popup opens with all current values populated and marked `VIEW`

### Requirement: View mode ignores typing

The system SHALL ignore text input while the popup is in `VIEW` mode, leaving the draft clean and showing no unsaved indicator.

#### Scenario: Typing does nothing in view

- **WHEN** the user types characters with the popup in `VIEW`
- **THEN** no field changes and the popup stays clean

### Requirement: Edit mode edits with pickers

The system SHALL enter `EDIT` mode on `e` (or `i`), where `tab`/`j`/`k` move between the nine editable fields, typing edits text fields, and `[`/`]` cycle the project, priority, and status pickers. Urgency and start/end are display-only and skipped.

#### Scenario: Cycle priority picker

- **WHEN** the user focuses priority in `EDIT` and presses `]`
- **THEN** the priority advances (H → M → L → H) and the draft is marked unsaved

### Requirement: Urgency is read-only

The system SHALL display computed urgency as a read-only value with its explanation and MUST NOT allow direct editing.

#### Scenario: Urgency reflects changes

- **WHEN** the user changes priority from M to H and saves
- **THEN** the displayed urgency value increases on next render

### Requirement: Save validates and closes

The system SHALL save on `Enter` in `EDIT` mode, rejecting an empty description with a hint while keeping the popup open.

#### Scenario: Save requires a description

- **WHEN** the user clears the description and presses `Enter`
- **THEN** the popup stays open and reports that a description is required

#### Scenario: Cancel discards edits

- **WHEN** the user edits the description then presses `esc`
- **THEN** the popup steps back to `VIEW` with the original values restored

### Requirement: View closes plainly

The system SHALL close the popup without changes on `esc`, `q`, or `Enter` while in `VIEW` mode.

#### Scenario: Close from view

- **WHEN** the user presses `esc` with the popup in `VIEW`
- **THEN** the popup closes and the list is unchanged

### Requirement: Subtask creation inherits project

The system SHALL create a subtask on `n` in the popup, assigning it the parent's project and section and linking it as a child of the current task.

#### Scenario: Create subtask in popup

- **WHEN** the user adds a subtask to a `project:work` parent
- **THEN** the child is created with `project:work` and linked via depends

### Requirement: Editor key opens the system editor

The system SHALL open the description/annotations in `$EDITOR` on `E` and report it, without closing the popup.

#### Scenario: Open editor from popup

- **WHEN** the user presses `E` in the popup
- **THEN** the system editor opens for the task text
