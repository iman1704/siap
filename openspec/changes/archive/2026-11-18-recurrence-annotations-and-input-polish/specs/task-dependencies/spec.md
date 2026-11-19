# Spec Delta — `task-dependencies`

## ADDED Requirements

### Requirement: Links never close a cycle

The system SHALL refuse a `depends` edit when any new link would make the edited task reachable by following existing blocking links from that candidate, covering direct self-links, mutual pairs, and longer transitive chains. A refused edit SHALL toast and keep the draft unchanged, and existing links SHALL stay untouched.

#### Scenario: Mutual pair is refused

- **WHEN** task A waits on task B and the user makes B wait on A
- **THEN** the save is refused with `circular dependency`, B keeps its previous links, and A keeps its links

#### Scenario: Transitive chain is refused

- **WHEN** A waits on B, B waits on C, and the user makes C wait on A
- **THEN** the save is refused with `circular dependency` and no link changes

#### Scenario: Valid link still saves

- **WHEN** the new link cannot reach the edited task
- **THEN** the link saves and both tasks stay top-level rows with the waiter marked `blocked`
