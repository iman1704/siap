# Spec Delta — `task-editor`

## MODIFIED Requirements

### Requirement: Popup opens read-only

The system SHALL open a modal popup on `Enter` showing the six editable fields in one flat list (description, project, due, priority, tags, depends) followed by a blank line and the grey read-only block (`id`, `status`, urgency, `start`, recur, annotation), in `VIEW` mode. Empty values SHALL render blank with no `—` or `-` placeholder. The project field SHALL show the project name only with no `●` dot.

#### Scenario: Open popup from list

- **WHEN** the user presses `Enter` on a highlighted task
- **THEN** the popup opens with all current values populated and marked `VIEW`

#### Scenario: Single list without group headers

- **WHEN** the popup renders its fields
- **THEN** no PRIMARY or SECONDARY group headers render and the project shows `work`, never `● work`

#### Scenario: Empty values stay blank

- **WHEN** the popup renders a task with no project, due, tags, or notes
- **THEN** those values render as blank space with no dash placeholder, and the grey read-only block sits one blank line below the editable fields

#### Scenario: ID leads the read-only block

- **WHEN** the popup renders a task
- **THEN** the read-only block starts with an `id` row showing the short numeric ID, or the UUID prefix when no short ID exists, above `status` and urgency

#### Scenario: Status is read-only

- **WHEN** the popup renders in `VIEW` or `EDIT` mode
- **THEN** `status` renders only in the grey read-only block from the live task, never takes focus, and popup `d` refreshes it without marking the draft dirty

### Requirement: Edit mode edits with pickers

The system SHALL enter `EDIT` mode on `e` and edit only the focused field at once: keystrokes append at the end of the field's text, `Backspace` removes the last character, and the focused field keeps its saved body on the field row while a separate contextual input line below it renders the draft with `▌` at the text cursor (grey example when empty, which vanishes on input). `←`/`→` SHALL move the text cursor within the draft, and `Tab`/`Shift+Tab`/`↑`/`↓` move only the highlighted suggestion. `Enter` applies the highlighted suggestion and saves, `esc` discards only the focused field, and `[`/`]` keep cycling the project (including the `no project` clear row first) and priority pickers. Urgency, status, start, recur, and annotation are display-only and never take focus. Typing `j`/`k` MUST insert text and MUST NOT move the field.

#### Scenario: Cycle priority picker

- **WHEN** the user focuses priority in `EDIT` mode and presses `]`
- **THEN** the priority advances (H → M → L → H) and the draft is marked unsaved

#### Scenario: Modal movement and typing stay separate

- **WHEN** the user presses `e`, then `j`
- **THEN** the field does not move and the character `j` is typed into the focused field

#### Scenario: Typing never moves the field

- **WHEN** the user types characters in `EDIT` mode
- **THEN** the focus stays on the selected field

#### Scenario: Tab moves only the highlight

- **WHEN** the user presses `Tab`, `Shift+Tab`, `↑`, or `↓` in `EDIT` mode
- **THEN** only the suggestion highlight moves and the focused field never changes

#### Scenario: Arrows move the text cursor

- **WHEN** the user presses `←` or `→` in `EDIT` mode
- **THEN** the text cursor moves within the draft input line and the draft text is unchanged except for cursor position

#### Scenario: Focused field shows the inline block cursor

- **WHEN** the user focuses a non-empty field in `EDIT` mode
- **THEN** the field row keeps its body text and exactly one input line below it renders the draft with `▌` at the text cursor and no native terminal cursor is placed

#### Scenario: Focused field shows the separate input line

- **WHEN** the user focuses a field in `EDIT` mode
- **THEN** the field row keeps its body text with no inline cursor, and exactly one input line below it renders the draft with `▌` (or the grey shared example plus `▌` when empty); priority shows no input line

#### Scenario: Esc discards only the focused field

- **WHEN** the user edits one field, moves to a second field, edits it, and presses `esc`
- **THEN** only the second field reverts and the first edit remains unsaved in the draft

#### Scenario: Now reference mirrors the add box

- **WHEN** the user focuses project or priority in `EDIT` mode
- **THEN** a `now:` line renders below the input line showing the saved value (`now: @ work`, `now: ! H`; project `now:` hides when the task has no project)

### Requirement: Save validates and closes

The system SHALL save on `Enter` in `EDIT` mode after applying the highlighted suggestion via a single `task <uuid> modify` write for description, project, due, priority, and tags plus per-UUID `depends` add/remove writes, land back in `VIEW` mode with the editor still open and the draft marked clean, and reject an empty description with a hint while keeping the popup open. Unknown dependency IDs and self-links SHALL toast and keep the draft in `EDIT`. Status, recur, and annotations stay read-only and are never written from the draft. The save toast SHALL read `saved`.

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

The system SHALL render an inline suggestion dropdown under the focused `EDIT` field for project, tags, due, priority, and depends, filtered by the text already typed, capped at the shared count of 5 with the add box. Project lists SHALL lead with `no project`, tags with `no tags`, and depends SHALL always close with `no dependencies` even from an empty draft. Tags SHALL filter and complete by the in-progress last token, replacing it rather than appending a duplicate. `Enter` SHALL apply the highlighted suggestion (a clear row empties the draft field) and save at once, and `Tab` SHALL only move the highlight. Depends suggestions SHALL list open same-project tasks (empty matches empty) as `ID  description` excluding self and already-added IDs, and `Enter` on a match SHALL append that ID and save back to `VIEW` at once.

#### Scenario: Project hints list current projects

- **WHEN** the user focuses the project field in `EDIT` typing mode
- **THEN** the dropdown lists `no project` first followed by current projects and `Enter` applies the highlighted one into the draft and saves

#### Scenario: Tags complete the in-progress token

- **WHEN** the tag field contains `work, leet` and the highlighted suggestion is `leetcode`
- **THEN** applying it produces `work, leetcode`, not `work, leet, leetcode`

#### Scenario: Tag filter follows the last token

- **WHEN** the tag field contains `work, lee`
- **THEN** the dropdown filters on `lee`, not on the whole field text

#### Scenario: Depends adds one and stays open

- **WHEN** the user highlights a depends suggestion and presses `Enter`
- **THEN** that ID is appended to the draft, the popup saves back to `VIEW` at once, and the added row hides from further suggestions

#### Scenario: Depends applies and saves at once

- **WHEN** the user highlights a depends suggestion and presses `Enter`
- **THEN** that ID is appended to the draft, the popup saves back to `VIEW`, and the added row hides from further suggestions

#### Scenario: No dependencies clears and saves

- **WHEN** the user highlights `no dependencies` and presses `Enter`
- **THEN** the draft depends clears, the save persists zero dependencies, and the popup lands back in `VIEW` with toast `saved`

### Requirement: Empty edit fields show grey examples

While a typed field is focused in `EDIT` and empty, the system SHALL show the shared grey example on the separate input line (vanishing on input): description (`e.g. Call the client back`), project (`e.g. work`), due (`e.g. 2026-11-20`), tags (`e.g. work, urgent`), and depends (`e.g. 12, 14`). Picker-only priority SHALL show no example and no input line.

#### Scenario: Empty due shows an example date

- **WHEN** the user focuses an empty due field in `EDIT` mode
- **THEN** the input line shows `e.g. 2026-11-20▌` in dim text

#### Scenario: Depends names a task ID

- **WHEN** the user focuses an empty depends field in `EDIT` mode
- **THEN** the input line shows `e.g. 12, 14▌`

#### Scenario: Typing replaces the example

- **WHEN** the user types a character into a focused empty field
- **THEN** the example disappears and the typed text renders bright with `▌` at the cursor

#### Scenario: Picker-only fields have no example

- **WHEN** the user focuses priority in `EDIT` mode
- **THEN** no example text and no input line render, only the `now: ! H` reference and the `H` / `M` / `L` suggestions

### Requirement: Done toggles without closing the popup

The system SHALL toggle the popup task between done and undone on `d` in `VIEW` mode without closing the popup, refreshing the displayed read-only status and recomputing dirty from the remaining draft fields so a saved toggle never shows a stale unsaved indicator.

#### Scenario: Complete inside the popup

- **WHEN** the user presses `d` in the popup `VIEW` mode on an open task
- **THEN** the task completes, the popup stays open, its read-only status row reads completed, and the draft stays clean

#### Scenario: Reopen inside the popup

- **WHEN** the user presses `d` again on the completed popup task
- **THEN** the task returns to pending, the popup stays open, and the draft stays clean
