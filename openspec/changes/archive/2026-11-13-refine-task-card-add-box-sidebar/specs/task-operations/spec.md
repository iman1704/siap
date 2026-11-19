# Spec Delta

## MODIFIED Requirements

### Requirement: Single start and stop

The system SHALL support `s` to toggle the highlighted task between `status:active` and `status:pending`, updating its start indicator. Active state MUST NOT add full-row tint; tint is reserved for cursor and selection.

#### Scenario: Start a task

- **WHEN** the user presses `s` on a pending task
- **THEN** it becomes active, gains the mint `◉` start indicator, and appears in Today while active with no full-row tint

#### Scenario: Stop a task

- **WHEN** the user presses `s` on an active task
- **THEN** it returns to pending, its start column becomes blank, and it leaves Today unless it is otherwise due or overdue
