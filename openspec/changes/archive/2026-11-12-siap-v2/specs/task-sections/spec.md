# Spec Delta

## Purpose

Defines app-managed sections backed by a single invisible UDA so users organize tasks without ever configuring taskwarrior directly.

## ADDED Requirements

### Requirement: App owns one section UDA

The system SHALL maintain a single user-defined attribute (e.g. `uda.siap.section`) and auto-register it via `task config` on first run without user action.

#### Scenario: First run registers UDA

- **WHEN** the app launches and the section UDA is missing
- **THEN** it registers the UDA and existing tasks without a value behave as unsectioned

### Requirement: Default sections exist

The system SHALL provide default sections Now, Next, and Later for every view.

#### Scenario: New task gets a default section

- **WHEN** a task is created and has no section value
- **THEN** it appears under Now

### Requirement: User manages sections without config syntax

The system SHALL let users add, rename, and remove sections through UI actions only (`A` adds, `R` renames, `X` removes an empty section), never requiring `task config` commands.

#### Scenario: User adds a section

- **WHEN** the user adds a section named `Someday`
- **THEN** a new empty section appears and tasks can be moved into it

#### Scenario: Removing a non-empty section is refused

- **WHEN** the user removes a section that still holds tasks
- **THEN** the section stays and a hint explains tasks must move out first

### Requirement: Moving a task sets the section

The system SHALL move a task between sections by updating its section UDA value when the user presses `m` on a highlighted task.

#### Scenario: Move task to Next

- **WHEN** the user moves a task from Now to Next
- **THEN** its section value updates and it renders under Next on next refresh

### Requirement: Sections are separated by blank space

The system SHALL render each section as a quiet uppercase label with its count, cushioned by blank lines above and below, with no boxes, rules, or borders.

#### Scenario: Sections breathe

- **WHEN** the list shows Now and Next
- **THEN** a blank line separates the last Now row from the Next header
