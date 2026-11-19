# Spec Delta

## Purpose

Defines native flat Taskwarrior `depends` blocking as a multi-blocker DAG with no hierarchy, so every task stays visible top-level with a `blocked` marker until its open dependencies complete.

## ADDED Requirements

### Requirement: Blocking uses a flat dependency DAG

The system SHALL store zero or more blocking UUIDs per task in native `depends` with no parent, nesting, or single-link limit.

#### Scenario: Multiple blockers block together

- **WHEN** a task lists two open tasks in `depends`
- **THEN** it renders `blocked` until every listed task completes

#### Scenario: No hierarchy is inferred

- **WHEN** task A lists task B in `depends`
- **THEN** B gains no children, A gains no parent, and both stay top-level rows

### Requirement: Blocked means waiting on open tasks

The system SHALL treat a task as blocked when any listed `depends` UUID resolves to an open task, ignoring completed, deleted, and unknown UUIDs for display.

#### Scenario: Completed deps lift the block

- **WHEN** the last open dependency of a blocked task completes
- **THEN** the task loses its `blocked` marker

#### Scenario: Completed blockers hide from relations

- **WHEN** a blocked task's dependency completes
- **THEN** that task leaves `BLOCKED BY` and `BLOCKS` while the UUID stays stored until edited

### Requirement: Deletes are single-only

The system SHALL delete exactly the confirmed tasks with no cascade; the backend strips each deleted UUID from other tasks' `depends` on refresh.

#### Scenario: Blocker survives a blocked delete

- **WHEN** the user deletes a blocked task
- **THEN** its blockers stay open and listed

#### Scenario: Blocked task survives a blocker delete

- **WHEN** the user deletes a blocker
- **THEN** the blocked task stays open and unblocks on refresh

### Requirement: Blocked rows stay visible flat

The system SHALL keep every open task as its own top-level row in sort order with a dimmed description and magenta `blocked` trailer, never nesting or hiding blocked rows.

#### Scenario: Blocked task keeps its row

- **WHEN** a task waits on an open dependency
- **THEN** both tasks render as separate top-level rows with only the waiter marked `blocked`
