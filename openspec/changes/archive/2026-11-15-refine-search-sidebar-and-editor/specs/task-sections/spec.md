# Spec Delta

## MODIFIED Requirements

### Requirement: Moving a task sets the section

The system SHALL move a task between sections by updating its section UDA value when the user presses `m` on a highlighted task, cycling the active registry order. An unsectioned task SHALL move to the first registry section. In any non-project view (`Search`, `Inbox`, `Today`, `All`) `m` SHALL be disabled with a hint and the hidden section value preserved.

#### Scenario: Move task to Next

- **WHEN** the user moves a task from `Now` to the next registry section
- **THEN** its section value updates and it renders under that section on the next refresh

#### Scenario: Unsectioned task moves to the first section

- **WHEN** the user presses `m` on an unsectioned task
- **THEN** its section becomes the first registry section

#### Scenario: Move is refused in Inbox

- **WHEN** the user presses `m` in Search, Inbox, Today, or All
- **THEN** the section value stays unchanged and a hint explains sections live in project views

### Requirement: Sections are separated by blank space

The system SHALL render each section, including empty user-added sections, as a quiet uppercase label with its count, cushioned by blank lines above and below, with no boxes, rules, or borders. This requirement does not apply to Search, Inbox, Today, or All, which have no section labels.

#### Scenario: Sections breathe

- **WHEN** the list shows `Now` and `Next`
- **THEN** a blank line separates the last `Now` row from the `Next` header

#### Scenario: Empty section renders count 0

- **WHEN** a user-added section holds no tasks
- **THEN** its uppercase label renders with `0` and blank lines above and below

#### Scenario: Inbox has no separators for sections

- **WHEN** the list shows Search, Inbox, Today, or All tasks
- **THEN** no uppercase section labels or section separator blanks render between rows

### Requirement: Quick-add leaves the section unset

The system SHALL NOT write a `siapsection` value when quick-add creates a task; the task SHALL render in the unsectioned flat list until the user moves it.

#### Scenario: Added task has no section value

- **WHEN** the user commits a task from the add box
- **THEN** the stored task has no `siapsection` attribute

#### Scenario: Added task starts unsectioned

- **WHEN** the user commits a task while viewing a project with sections
- **THEN** the task renders in the flat unsectioned list until an explicit move

## REMOVED Requirements

### Requirement: Unsectioned tasks show a display-only group

**Reason**: Unsectioned tasks now render as a flat list above the section headers instead of under an `UNSECTIONED` group, and flat views render no headers at all.

**Migration**: No user action. Unsectioned tasks stay visible at the top of project views and no section UDA value is written to produce the list.

## ADDED Requirements

### Requirement: Unsectioned tasks render flat above sections

In project views the system SHALL render tasks without a `siapsection` value as a flat urgency-descending list above the first section header, with no `UNSECTIONED` header. It MUST NOT write a section value merely to render them.

#### Scenario: Unsectioned tasks lead without a header

- **WHEN** a project view contains unsectioned tasks and sectioned tasks
- **THEN** the unsectioned tasks render as a flat list before `NOW`, with no `UNSECTIONED` label or group count

#### Scenario: No sections means an all-flat list

- **WHEN** a project view has no sections in its registry
- **THEN** all its tasks render as one urgency-descending flat list

#### Scenario: Rendering never writes a section

- **WHEN** a task has no section value and the user only views it
- **THEN** the stored task still has no `siapsection` attribute

### Requirement: Section actions are limited to project views

The system SHALL open the `A`, `R`, and `X` section prompts only in project views. In Search, Inbox, Today, or All these keys SHALL be refused with a hint and MUST NOT change the registry.

#### Scenario: Add is refused in All

- **WHEN** the user presses `A` in All
- **THEN** no prompt opens and a hint explains sections live in project views

#### Scenario: Rename and remove are refused in Inbox

- **WHEN** the user presses `R` or `X` in Inbox
- **THEN** no prompt opens and the section registry is unchanged

#### Scenario: Add works in a project view

- **WHEN** the user presses `A` in a project view and enters a name
- **THEN** the new empty section joins the registry
