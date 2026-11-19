# Tasks

## 1. Model and backend scaffold

- [x] 1.1 Add `serde_json` and `chrono` to `Cargo.toml` and verify `cargo build` succeeds
- [x] 1.2 Switch `Task` to `uuid` identity with display-only short id and re-key selection/cursor/children/popup, verify `cargo test smoke` still compiles against mocks
- [x] 1.3 Create backend module skeleton (`export_pending`, `export_completed`, `ensure_section_uda`, `run_write` with stderr capture) behind temp `TASKDATA` fixtures and verify unit tests use fixtures without touching live data

## 2. Live reads and mapping

- [x] 2.1 Implement two-export fetch (`+PENDING` open, `+COMPLETED` by `end` desc capped at 10, `deleted` excluded) with view filtering and verify tail cap plus view filter tests pass
- [x] 2.2 Implement field mapping (ISO due to friendly label with local-day overdue/today, absent priority to P3, urgency read-only, section fallback display-without-write, parent from inverted `depends`) and verify mapping unit tests pass including Kuching/UTC edge cases
- [x] 2.3 Render deferred `recur`/`annotations` read-only in the popup with no edit path and verify popup test shows values without edit affordance
- [x] 2.4 Update `docs/main.md` backend section from mock truth to CLI reads and verify documented behavior matches the build

## 3. Write-through and refresh

- [x] 3.1 Implement UDA bootstrap plus quick-add and popup save via `task <uuid> modify` with re-export on success and verify round-trip test creates then reads back a fixture task
- [x] 3.2 Implement `s` start/stop, `x` done, `d` delete with child cascade, and `m` section move, verify each operation's test passes and cursor holds by `uuid`
- [x] 3.3 Implement `n` subtask (add child then parent `depends+=child`, single-parent move semantics) and bulk `d/m/t/x` per-`uuid` loop, verify subtask link plus blocked marker and bulk tests pass
- [x] 3.4 Implement refresh on launch plus after-write plus `r` key with failure toast preserving list/selection/cursor, verify failure-injection test keeps state and shows stderr

## 4. Integration hardening

- [x] 4.1 Extend `TestBackend` smoke coverage for live mapping, tail cap, section fallback, and refresh hold, verify `cargo test smoke` passes headless
- [x] 4.2 Run `cargo fmt` and `cargo clippy -- -D warnings` clean and verify both stay clean with no warnings
- [x] 4.3 Finalize `docs/main.md` for write-through truth and verify every documented key and behavior runs as written
