# Spec Delta

## MODIFIED Requirements

### Requirement: Save validates and closes

The system SHALL save on `Enter` in `EDIT` mode after applying the highlighted suggestion via a single `task <uuid> modify` write for description, project, due, priority, and tags plus per-UUID `depends` add/remove writes, land back in `VIEW` mode with the editor still open and the draft marked clean, and reject an empty description with a hint while keeping the popup open. Unknown dependency IDs and self-links SHALL toast and keep the draft in `EDIT`.

#### Scenario: Save requires a description

- **WHEN** the user clears the description and presses `Enter`
- **THEN** the popup stays open and reports that a description is required

#### Scenario: Save stays open

- **WHEN** the user edits the description in `EDIT` mode and presses `Enter`
- **THEN** the task updates, the editor stays open in `VIEW` mode, and the draft is marked clean

#### Scenario: Enter applies the hint before saving

- **WHEN** the user types `work` in the project field and presses `Enter` with a matching hint highlighted
- **THEN** the highlighted project is applied and the save persists it

#### Scenario: Cancel discards edits

- **WHEN** the user edits the description then presses `esc`
- **THEN** the popup steps back to `VIEW` with the description restored and any unsaved edits in other fields preserved

#### Scenario: Unknown dependency ID blocks save

- **WHEN** the user types `999` in `DEPENDS` and presses `Enter`
- **THEN** the toast reads `unknown id: 999` and the popup stays in `EDIT`

#### Scenario: Self-link blocks save

- **WHEN** the user types the task's own ID in `DEPENDS` and presses `Enter`
- **THEN** the toast reads `can't depend on itself` and the popup stays in `EDIT`

### Requirement: Inline suggestions complete attributes

The system SHALL render an inline suggestion dropdown under the focused `EDIT` field for project, tags, due, priority, status, and depends, filtered by the text already typed. Tags SHALL filter and complete by the in-progress last token, replacing it rather than appending a duplicate. `Enter` SHALL apply the highlighted suggestion, and `Tab` SHALL only move the highlight. Depends suggestions SHALL list open same-project tasks (empty matches empty) as `ID  description` excluding self and already-added IDs, scroll at up to 8 rows, and `Enter` on a match SHALL add that ID and stay in `EDIT`.

#### Scenario: Project hints list current projects

- **WHEN** the user focuses the project field in `EDIT` typing mode
- **THEN** the dropdown lists current projects and `Enter` applies the highlighted one into the draft

#### Scenario: Tags complete the in-progress token

- **WHEN** the tag field contains `work, leet` and the highlighted suggestion is `leetcode`
- **THEN** applying it produces `work, leetcode`, not `work, leet, leetcode`

#### Scenario: Tag filter follows the last token

- **WHEN** the tag field contains `work, lee`
- **THEN** the dropdown filters on `lee`, not on the whole field text

#### Scenario: Depends adds one and stays open

- **WHEN** the user highlights a depends suggestion and presses `Enter`
- **THEN** that ID is appended to the draft, the popup stays in `EDIT`, and the added row hides from further suggestions

### Requirement: Empty edit fields show grey examples

While a typed field is focused in `EDIT` and empty, the system SHALL show a grey example that vanishes on input: description, project (`e.g. work`), due (`e.g. 2026-11-20`), tags (`e.g. work, urgent`), and depends (`e.g. 12, 14`). Picker-only priority and status SHALL show no example.

#### Scenario: Empty due shows an example date

- **WHEN** the user focuses an empty due field in `EDIT` mode
- **THEN** the line shows `e.g. 2026-11-20` in dim text

#### Scenario: Depends names a task ID

- **WHEN** the user focuses an empty depends field in `EDIT` mode
- **THEN** the line shows `e.g. 12, 14`

#### Scenario: Typing replaces the example

- **WHEN** the user types a character into a focused empty field
- **THEN** the example disappears and the typed text renders bright

#### Scenario: Picker-only fields have no example

- **WHEN** the user focuses priority or status in `EDIT` mode
- **THEN** no example text renders

## REMOVED Requirements

### Requirement: Subtask creation inherits project
**Reason**: Popup `n` creation removed with the hierarchy.
**Migration**: Create tasks from the list and link blockers via `DEPENDS`.

## ADDED Requirements

### Requirement: Popup shows dependency relations

The system SHALL render read-only `BLOCKED BY` and `BLOCKS` sections at the bottom of the popup listing open related tasks as plain list rows without the `blocked` trailer; empty sections SHALL stay hidden. `n` in popup `VIEW` SHALL be a no-op.

#### Scenario: Relations close the popup

- **WHEN** a task waits on one open task and blocks another
- **THEN** both related tasks render under their sections with ID and description

#### Scenario: No relations means no sections

- **WHEN** a task has no open dependencies in either direction
- **THEN** neither `BLOCKED BY` nor `BLOCKS` renders

#### Scenario: Popup n does nothing

- **WHEN** the user presses `n` in the popup
- **THEN** no task is created and the list is unchanged
