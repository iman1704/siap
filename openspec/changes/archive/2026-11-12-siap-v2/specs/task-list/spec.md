# Spec Delta

## Purpose

Defines the whitespace-driven Main list: primary-only rows, cursor navigation, quick-add, and the dimmed completed tail.

## ADDED Requirements

### Requirement: Rows show primary attributes only

The system SHALL render each task row with checkbox state, description, due, priority (H/M/L displayed as P1/P2/P3), tags, and project marker. Secondary attributes and urgency numbers MUST NOT render inline.

#### Scenario: Row hides secondary fields

- **WHEN** the list renders a task with depends, recur, and annotations
- **THEN** only description, due, priority, tags, and project are visible until Enter opens the popup

### Requirement: Rows never show subtask counts

The system SHALL NOT render `n/m subtasks` progress anywhere in the list. A parent with open children shows a `blocked` marker instead.

#### Scenario: Parent shows blocked, not counts

- **WHEN** a parent has one of two subtasks completed
- **THEN** the row shows `blocked` and no numbers

### Requirement: Active tasks carry a full-row tint

The system SHALL render every active task row on the tint background with an active marker, whether or not the cursor is on it.

#### Scenario: Started task is tinted

- **WHEN** the user starts a task
- **THEN** its row gains the full-row tint and the active marker

### Requirement: Cursor highlights without expanding

The system SHALL move a single `›` cursor with `j/k` (or arrows) and highlight the row without revealing extra lines.

#### Scenario: Cursor moves across sections

- **WHEN** the user presses `j` on the last task of Now
- **THEN** the cursor lands on the first task of Next

### Requirement: Sorting is section then urgency

The system SHALL order tasks grouped by section (Now, Next, Later, then custom) and by taskwarrior urgency descending within each section.

#### Scenario: Urgent task surfaces first

- **WHEN** two tasks share the Now section
- **THEN** the higher-urgency task renders first

### Requirement: Children render indented under parents

The system SHALL render each open subtask indented directly beneath its parent with a checkbox, in a stable order.

#### Scenario: Subtask nests visually

- **WHEN** a parent with one open subtask renders
- **THEN** the child row appears indented on the line below the parent

### Requirement: Quick-add creates one task

The system SHALL provide a top `Add a task…` line where `Enter` creates exactly one `status:pending` task, joining pasted multi-line input into a single title and applying the current view's project/due defaults.

#### Scenario: Paste joins into one task

- **WHEN** the user pastes three lines and presses `Enter`
- **THEN** exactly one task is created with the lines joined by spaces

### Requirement: Completed tail is a dimmed list at the bottom

The system SHALL show completed tasks in a `COMPLETED n` section at the bottom as dimmed, struck-through rows that are never cursor-navigable.

#### Scenario: Done work rests below

- **WHEN** tasks are completed
- **THEN** they leave the open rows and appear dimmed under the COMPLETED header

### Requirement: Find filters live

The system SHALL filter rows live while `/` find mode is active, keeping the filter on `Enter` and clearing it on `esc`.

#### Scenario: Find filters rows

- **WHEN** the user types `/` followed by `invoice`
- **THEN** only matching rows remain visible until find is cleared
