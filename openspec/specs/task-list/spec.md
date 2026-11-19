# task-list Specification

## Purpose

Defines the whitespace-driven Main list: primary-only rows, cursor navigation, quick-add, and the dimmed completed tail.

## Requirements

### Requirement: Task rows render aligned primary attributes

The system SHALL render each open task row in one aligned line: a marker column (`›` cursor, `▌` selected), a start indicator only when active, a far-left ID column (short numeric ID, else an 8-character UUID prefix), description, then aligned project, tags, due, and priority. The project SHALL be a colored name with no dot, hidden in its own project view. Secondary attributes and urgency numbers MUST NOT render inline, except a blue `↻ <recur>` badge on recurring tasks and the magenta `blocked` trailer. The same ID-led row format SHALL render completed rows and popup relation rows. The Enter popup SHALL always show the project field.

#### Scenario: Row hides secondary fields

- **WHEN** the list renders a task with depends, annotations, and recurrence
- **THEN** only the recurrence badge joins description, project, due, priority, and tags, while depends and annotations stay in the popup

#### Scenario: Row leads with the task ID

- **WHEN** the list renders open, completed, or relation rows
- **THEN** each row renders the short numeric ID, or the 8-character UUID prefix when no short ID exists, before the description

#### Scenario: Project view hides the project name

- **WHEN** the list renders a `work` task inside the `work` project view
- **THEN** no project name or dot renders in the row, while the Enter popup still shows the project field

#### Scenario: Mixed views color the project name

- **WHEN** the list renders tasks in Today or All
- **THEN** each row with a non-empty project shows its project name in that project's color, with no `●` dot

#### Scenario: Metadata aligns across rows

- **WHEN** two rows carry IDs, projects, tags, due dates, and priorities of different lengths
- **THEN** their ID, project, tag, due, and priority columns start at the same positions and long values truncate with `…`

### Requirement: Rows never show subtask counts

The system SHALL NOT render `n/m` progress anywhere in the list. A task waiting on open `depends` shows a dimmed description with a `blocked` marker instead.

#### Scenario: Parent shows blocked, not counts

- **WHEN** a blocked task waits on one open and one completed dependency
- **THEN** the row shows `blocked` and no numbers

### Requirement: Active tasks show a single start indicator

The system SHALL render the mint `◉` start indicator only for a task whose status is active. Pending tasks MUST NOT render a start placeholder. Active state MUST NOT add full-row tint or `● active` text; tint is reserved for cursor and selection.

#### Scenario: Started task shows the start indicator

- **WHEN** the user starts a task
- **THEN** its start column shows the mint `◉` with no tint background and no `● active` text

#### Scenario: Pending task shows no start indicator

- **WHEN** a task is pending
- **THEN** its start column is blank

#### Scenario: Cursor keeps its own highlight

- **WHEN** the cursor rests on an active task
- **THEN** the row shows the cursor `›` marker and highlight plus the mint `◉`, with no `● active` text

### Requirement: Selected rows show an accent marker

The system SHALL mark a selected row with a `▌` accent and a tint. The cursor `›` SHALL take precedence in the marker column when a row is both selected and under the cursor; the selection count bar SHALL still reflect it.

#### Scenario: Selected row shows the accent

- **WHEN** the user presses `space` on a row
- **THEN** the row shows `▌` and a tint while the cursor remains a single `›`

#### Scenario: Cursor and selection coincide

- **WHEN** the cursor rests on a selected row
- **THEN** the marker column shows `›` and the row stays tinted, with the selection count unchanged

### Requirement: Cursor highlights without expanding

The system SHALL move a single `›` cursor with `j/k` (or arrows) across task rows and section headers and highlight the focused row without revealing extra lines. `Enter` on a section header SHALL collapse or expand it instead of opening a popup.

#### Scenario: Cursor moves across sections

- **WHEN** the user presses `j` on the last task of Now
- **THEN** the cursor lands on the Next header, and a further `j` lands on the first task of Next

#### Scenario: Cursor moves row to row in Inbox

- **WHEN** the user presses `j` in Inbox
- **THEN** the cursor lands on the next row with no section boundary involved

#### Scenario: Enter toggles a header

- **WHEN** the user presses `Enter` with the cursor on a section header
- **THEN** the section collapses (its rows hide) or expands, and no popup opens

### Requirement: Sorting is section then urgency

The system SHALL order tasks in project views grouped by the active section registry order (Now/Next/Later when present, then custom sections alphabetically) with taskwarrior urgency descending within each section. Unsectioned tasks SHALL render as a flat urgency-descending list above the first section header. Search, Inbox, Today, and All SHALL order by taskwarrior urgency descending with no section grouping.

#### Scenario: Urgent task surfaces first

- **WHEN** two tasks share the same section
- **THEN** the higher-urgency task renders first

#### Scenario: Inbox ignores sections

- **WHEN** two Inbox tasks hold different section values
- **THEN** the higher-urgency task renders first regardless of section

#### Scenario: Today and All ignore sections

- **WHEN** Today or All contains tasks carrying section values
- **THEN** no section headers render and rows sort by urgency descending

#### Scenario: Search ignores sections

- **WHEN** Search matches tasks carrying section values
- **THEN** results render as one urgency-descending list with no headers

#### Scenario: Unsectioned tasks lead

- **WHEN** a project view contains unsectioned tasks and sectioned tasks
- **THEN** the unsectioned tasks render as a flat list before the first section header

#### Scenario: Custom sections sort alphabetically

- **WHEN** two custom sections are present after Now, Next, and Later
- **THEN** they render in alphabetical order

### Requirement: Blocked rows stay flat and dimmed

The system SHALL render every open task as its own top-level row in the view's sort order with no indentation. A blocked task's description SHALL render dimmed with the magenta `blocked` trailer; blockers carry no trailer.

#### Scenario: Flat blocked pair

- **WHEN** a blocked task and its open blocker share a view
- **THEN** both render as top-level rows and only the waiter shows `blocked`

### Requirement: Recurring tasks show a badge

The system SHALL render a blue `↻ <recur>` badge before the `blocked` trailer on a recurring task's row and MUST NOT render it when recurrence is absent, cleared, or `none`.

#### Scenario: Badge shows recurrence

- **WHEN** a task carries `recur: weekly`
- **THEN** its row renders `↻ weekly` in blue before any `blocked` trailer

#### Scenario: Clearing recurrence drops the badge

- **WHEN** the user clears a task's recurrence and the list refreshes
- **THEN** no `↻` badge renders on that row

#### Scenario: Badge precedes the blocked trailer

- **WHEN** a recurring task also waits on an open dependency
- **THEN** the row renders the recurrence badge before the magenta `blocked` trailer

### Requirement: Quick-add creates one pending task

The system SHALL provide a top `Add a task…` line where the description keeps focus and `Enter` creates exactly one `status:pending` task, joining pasted lines and applying the view's project/due defaults. `Enter` SHALL keep the add box open for rapid entry and `esc` SHALL leave it. Quick-add MUST NOT write a section value, and Search MUST NOT show an add box.

#### Scenario: Paste joins into one task

- **WHEN** the user pastes three lines and presses `Enter`
- **THEN** exactly one task is created with the lines joined by spaces

#### Scenario: Commit stays open for rapid entry

- **WHEN** the user presses `Enter` on a non-empty description
- **THEN** exactly one task is created, the draft clears, and the add box stays open

#### Scenario: Added task carries no section

- **WHEN** the user commits a task from the add box
- **THEN** the new task has no `siapsection` value

#### Scenario: Search offers no add box

- **WHEN** the Search view renders
- **THEN** no add box or `a` trigger is available there

### Requirement: Quick-add pickers carry every attribute

The system SHALL let `#`, `~`, `@`, `!`, `&`, `*`, and `+` open capped pickers moved by `↑`/`↓` and `Tab`/`Shift+Tab`. Project SHALL lead with `no project`, tags with `no tags`, and depends SHALL close with `no dependencies`; due SHALL offer `today`/`tomorrow`/`no date` and recurrence `daily`/`weekly`/`monthly`/`yearly`/`no recurrence`. `+ more` SHALL list every picker; `Enter` SHALL apply the highlighted row.

#### Scenario: Trigger opens the tag picker

- **WHEN** the user types `#` in the add box
- **THEN** the tag picker opens with `no tags` first followed by existing tags and `Enter` toggles the highlighted tag without leaving the picker

#### Scenario: Custom tag is created from typed text

- **WHEN** the user types `brand-new-tag` in the tag picker filter and presses `Enter`
- **THEN** `brand-new-tag` is added to the draft tags and the picker stays open with a cleared filter

#### Scenario: No project clears the project

- **WHEN** the user highlights `no project` in the project picker and presses `Enter`
- **THEN** the draft project clears and the created task has no project

#### Scenario: No tags clears the tags

- **WHEN** the user highlights `no tags` in the tag picker with draft tags set and presses `Enter`
- **THEN** the draft tags clear

#### Scenario: Empty text pickers show grey examples

- **WHEN** the Tags, Project, or Due picker is focused with an empty filter
- **THEN** the input line shows `e.g. work, urgent`, `e.g. work`, or `e.g. 2026-11-20` in dim text with the terminal caret at its start, and typing replaces it

#### Scenario: Typed j filters instead of moving

- **WHEN** the user types `j` in the tag picker filter
- **THEN** the character is added to the filter and the picker stays open on the same focus

#### Scenario: j and k move fixed lists

- **WHEN** the priority, depends, recurrence, or `+ more` picker is focused and the user presses `j` or `k`
- **THEN** the highlight moves instead of typing a character

#### Scenario: Set attributes show colored chips

- **WHEN** the draft carries a tag and a due date
- **THEN** the add row shows the tag chip in cyan and the due chip in yellow instead of grey

#### Scenario: Recurrence chip always renders

- **WHEN** the draft has no recurrence
- **THEN** the add row shows a grey `* recur` chip, and with `weekly` set it shows a blue `* weekly` chip

#### Scenario: Depends picker appends links

- **WHEN** the user highlights a depends candidate and presses `Enter`
- **THEN** its ID joins the draft `depends`, the `& <ids>` chip renders, and the picker stays open for another pick

#### Scenario: No dependencies clears links

- **WHEN** the user highlights `no dependencies` and presses `Enter`
- **THEN** the draft `depends` clears and its chip disappears

#### Scenario: Recurrence picker applies

- **WHEN** the user highlights `weekly` in the recurrence picker and presses `Enter`
- **THEN** the draft recurrence is `weekly`, the chip turns blue, and focus returns to the description

#### Scenario: More opens the chosen picker

- **WHEN** the user presses `Enter` on `@ project`, `& depends`, or `* recur` in the `+ more` list
- **THEN** that picker opens and remains focused

#### Scenario: Shift+Tab moves the picker backward

- **WHEN** the user presses `Shift+Tab` in a picker
- **THEN** the highlight moves to the previous suggestion and clamps at the first item

#### Scenario: Priority chips use H M L

- **WHEN** the draft carries priority H
- **THEN** the add row shows the `! H` chip and the priority picker lists `H` / `M` / `L`

### Requirement: Quick-add accepts custom project and due values

The system SHALL apply a typed non-empty project or due filter as-is on `Enter`, creating the project when missing and passing the date through, and SHALL clear the due when the user types `no date`.

#### Scenario: Custom project is created from typed text

- **WHEN** the user types `garden` in the project picker and presses `Enter`
- **THEN** the draft project is `garden` and the created task carries `project:garden` even though no such project existed

#### Scenario: Custom due date is used as-is

- **WHEN** the user types `2026-12-01` in the due picker and presses `Enter`
- **THEN** the draft due is `2026-12-01` and the created task carries that due date

#### Scenario: No date clears the due

- **WHEN** the user types `no date` in the due picker and presses `Enter`
- **THEN** the draft due clears and the created task has no due date

### Requirement: Quick-add edits text in place

The system SHALL edit the description and tag/project/due filters at a real caret: printable keys SHALL insert at it, `←`/`→` SHALL move it, and `Backspace`/`Delete` SHALL remove around it, with no inline `▌` glyph. The description caret SHALL survive picker switches, and trigger characters SHALL stay reserved even mid-description.

#### Scenario: Focused inputs place the terminal caret

- **WHEN** the add-box description is focused with text `buy milk`
- **THEN** the line renders `buy milk` with the native terminal cursor at the caret and no inline `▌` glyph

#### Scenario: Typing inserts at the caret

- **WHEN** the description reads `abc` with the caret after `a` and the user types `X`
- **THEN** the description becomes `aXbc` with the caret after `X`

#### Scenario: Arrows move the insertion point

- **WHEN** the user presses `←` or `→` in the description or a text filter
- **THEN** the caret moves one character without changing the text

#### Scenario: Backspace edits before the caret

- **WHEN** the filter contains `abc` with the caret after `b` and the user presses `Backspace`
- **THEN** the filter becomes `ac` with the caret after `a`

#### Scenario: Delete edits after the caret

- **WHEN** the filter contains `abc` with the caret after `b` and the user presses `Delete`
- **THEN** the filter becomes `ac` with the caret after `b`

#### Scenario: Trigger chars stay reserved mid-description

- **WHEN** the caret sits inside the description and the user types `#`
- **THEN** the tag picker opens and `#` never enters the description, and `esc` returns to the description with its caret unchanged

### Requirement: Quick-add validates before writing

The system SHALL validate a non-empty description, a due date whenever recurrence is set, and resolvable `depends` IDs before writing. A failed check SHALL toast and keep the add-box draft.

#### Scenario: Description is required

- **WHEN** the user presses `Enter` with an empty or whitespace-only description
- **THEN** the toast reads `description required` and no task is created

#### Scenario: Recurrence needs a due date

- **WHEN** the draft carries `weekly` and no due date and the user presses `Enter` on the description
- **THEN** the toast reads `recurrence needs a due date`, no task is created, and the draft stays

#### Scenario: Unknown dependency is refused

- **WHEN** a drafted `depends` link no longer resolves
- **THEN** the toast names the unknown ID, no task is created, and the draft stays

### Requirement: Added attributes can be removed

The system SHALL remove a set add-box attribute on `Backspace` when the focused picker's filter is empty. Project, due, priority, and recurrence SHALL clear; tags and depends SHALL remove the last entry; unset attributes and the More picker SHALL be a no-op. The footer SHALL hint the removal when a set attribute is focused.

#### Scenario: Backspace clears the project

- **WHEN** the project picker has a project set and an empty filter, and the user presses `Backspace`
- **THEN** the project chip clears and the project picker stays focused

#### Scenario: Backspace removes the last tag

- **WHEN** the tag picker has an empty filter and at least one tag, and the user presses `Backspace`
- **THEN** the most recently added tag is removed from the draft

#### Scenario: Backspace pops the last dependency

- **WHEN** the depends picker has an empty filter and at least one link, and the user presses `Backspace`
- **THEN** the last link leaves the draft and the chip updates

#### Scenario: Backspace clears recurrence

- **WHEN** the recurrence picker has a value set and the user presses `Backspace`
- **THEN** the draft recurrence clears and the chip returns to grey `* recur`

#### Scenario: Backspace edits a non-empty filter

- **WHEN** the project picker filter contains text and the user presses `Backspace`
- **THEN** the last character before the caret is removed and the project chip is unchanged

#### Scenario: Unset attribute is a no-op

- **WHEN** the due picker has no due value and an empty filter, and the user presses `Backspace`
- **THEN** the draft is unchanged

### Requirement: Completed tail is a dimmed list at the bottom

The system SHALL show completed tasks in a `COMPLETED n` section at the bottom as dimmed, struck-through rows that are cursor-navigable and collapsible like any section. `Enter` on the `COMPLETED` header SHALL collapse or expand the tail.

#### Scenario: Done work rests below

- **WHEN** tasks are completed
- **THEN** they leave the open rows and appear dimmed under the COMPLETED header

#### Scenario: Cursor reaches completed rows

- **WHEN** the user moves the cursor past the last open task
- **THEN** the cursor lands on the COMPLETED header and then on completed rows

#### Scenario: Collapsed tail hides its rows

- **WHEN** the user presses `Enter` on the COMPLETED header
- **THEN** the completed rows hide and a second `Enter` shows them again

### Requirement: Overdue has no row trailer

The system SHALL indicate an overdue task through its red due label and Today inclusion only, and MUST NOT append an `overdue` text trailer to the row. The `blocked` trailer SHALL remain for tasks waiting on open dependencies.

#### Scenario: Overdue row reads without a trailer

- **WHEN** an overdue task renders in a list
- **THEN** its due date renders in red and no `overdue` text trails the row

#### Scenario: Blocked trailer still renders

- **WHEN** a task waits on an open dependency
- **THEN** its row shows the `blocked` trailer
