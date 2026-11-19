# task-sections Specification

## Purpose

Defines app-managed sections backed by a single invisible UDA so users organize tasks without ever configuring taskwarrior directly.

## Requirements

### Requirement: App owns one section UDA

The system SHALL maintain a single user-defined attribute (e.g. `uda.siapsection`) and auto-register it via `task config` on first run without user action. Registration MUST run non-interactively; a failed registration MUST toast and MUST NOT let the attribute text enter a task description.

#### Scenario: First run registers UDA

- **WHEN** the app launches and the section UDA is missing
- **THEN** it registers the UDA and existing tasks without a value behave as unsectioned

#### Scenario: Registration failure is safe

- **WHEN** the UDA cannot be registered
- **THEN** the app shows a failure toast and a quick-added task contains no `siapsection` text in its description or attributes

### Requirement: User manages sections without config syntax

The system SHALL let users add, rename, and remove sections through UI actions only: `A` opens a footer prompt for a new section name, `R` prompts to rename the cursor task's section, and `X` prompts to remove an empty section. No operation may require `task config` commands.

#### Scenario: User adds a section

- **WHEN** the user presses `A`, types `Someday`, and presses `Enter`
- **THEN** a new empty section appears in the registry, renders with count 0, and tasks can be moved into it

#### Scenario: User renames a section

- **WHEN** the user presses `R` on a task in `Later`, types `Someday`, and presses `Enter`
- **THEN** `Later` is renamed and every task with that section moves to `Someday`

#### Scenario: Rename refuses an unsectioned task

- **WHEN** the user presses `R` on a task with no `siapsection` value
- **THEN** the prompt does not open and a hint explains the task has no section

#### Scenario: Adding a duplicate is refused

- **WHEN** the user adds a section name already in the registry
- **THEN** no duplicate is created and a hint explains the section exists

#### Scenario: Renaming to an existing section is refused

- **WHEN** the user renames a section to a name already in the registry
- **THEN** the rename is refused and every task keeps its current section

#### Scenario: Removing a non-empty section is refused

- **WHEN** the user presses `X` and names a section that still holds tasks
- **THEN** the section stays and a hint explains tasks must move out first

#### Scenario: Removing an empty section succeeds

- **WHEN** the user presses `X` and names an empty section
- **THEN** the section leaves the registry and no task changes

#### Scenario: Prompt cancel leaves sections unchanged

- **WHEN** the user presses `esc` in the `A`, `R`, or `X` prompt
- **THEN** no section is added, renamed, or removed

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

### Requirement: Sections seed from task values

The system SHALL seed the section registry from the distinct `siapsection` values on tasks plus any sections the user adds. It MUST NOT create default Now, Next, or Later sections.

#### Scenario: Existing values seed the registry

- **WHEN** tasks carry `siapsection:Now` and `siapsection:Reading`
- **THEN** the registry contains `Now` and `Reading` with no other default section

#### Scenario: Empty data has no sections

- **WHEN** no task carries a `siapsection` value and the user has added none
- **THEN** the registry is empty

### Requirement: Quick-add leaves the section unset

The system SHALL NOT write a `siapsection` value when quick-add creates a task; the task SHALL render in the unsectioned flat list until the user moves it.

#### Scenario: Added task has no section value

- **WHEN** the user commits a task from the add box
- **THEN** the stored task has no `siapsection` attribute

#### Scenario: Added task starts unsectioned

- **WHEN** the user commits a task while viewing a project with sections
- **THEN** the task renders in the flat unsectioned list until an explicit move

### Requirement: Section registration never leaks into descriptions

The system SHALL ensure a section UDA registration or write failure cannot append `siapsection:` text to a task description. A failed write MUST surface an error and leave the description unchanged.

#### Scenario: Failed section write leaves the description intact

- **WHEN** a section write cannot be applied because the UDA is unavailable
- **THEN** the task description is unchanged and the app reports the failure

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

### Requirement: Section prompts edit at the caret

The system SHALL edit the `A`, `R`, and `X` footer prompt buffer at a real caret: printable keys insert at it, `←`/`→` move it, and `Backspace`/`Delete` remove around it. The native terminal cursor SHALL sit at the caret in the footer instead of an inline `▌` glyph.

#### Scenario: Caret follows the typed name

- **WHEN** the user presses `A` and types `Someday`
- **THEN** the footer renders the prompt prefix followed by `Someday` and the terminal cursor sits after the last character

#### Scenario: Caret moves and edits in place

- **WHEN** the prompt reads `Now`, the user presses `←`, then types `X`
- **THEN** the buffer becomes `NoXw` with the caret after `X`

#### Scenario: Backspace and Delete edit at the caret

- **WHEN** the prompt reads `Now` with the caret after `No`
- **THEN** `Backspace` produces `Nw` and `Delete` from the same state produces `No`

#### Scenario: Prompt shows no block glyph

- **WHEN** any section prompt renders
- **THEN** no inline `▌` glyph appears and the terminal cursor marks the insertion point
