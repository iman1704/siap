# `backend`

Source: `src/backend.rs`

## Purpose

Taskwarrior CLI subprocess behind the TUI. Runs `task export` for reads
and `task <uuid> modify/add/done/delete/start/stop/edit` plus `task config`
for writes, capturing stderr so failures can toast without losing list
state. Respects `TASKDATA`/`TASKRC` so tests never touch live data.

## What it handles

- Export fetching: `+PENDING` for open rows, `+COMPLETED` for the tail.
- Write runner with stderr capture for toasts, plus a per-command
  `TASKDATA`/`TASKRC` override so tests isolate without mutating env.
- Quick-add, popup-save, and op arg builders (`task add` with an
  optional section stamp, `task <uuid> modify` with tag diffs, recur handling, and
  section stamp,
  `start`/`stop`, `done`, `modify status:pending` undone, `undo`,
  `delete` with `rc.confirmation=no` first,
  `siapsection:`, `project:`, `depends:` add-if-missing / `depends:-`
  remove-one, `recur:` set / `recur:` clear, `annotate` add / `denotate`
  remove-one-by-exact-text) plus
  re-export round-trip tests.
- Recurring-series cleanup: `task <uuid> delete` with
  `rc.recurrence.confirmation=no` deletes a template or one occurrence
  without touching its siblings, and `task <uuid> edit` with a
  non-interactive `sed` editor strips `Recur:`/`Parent:` from each kept
  occurrence. Taskwarrior refuses `modify recur:` on recurring tasks, so
  the edit path is the only way to detach an occurrence while preserving
  its UUID. Completed occurrences are never edited: `task edit` reopens a
  completed task, so history is left as-is with its inert recurrence data.
- Section UDA bootstrap via `task config` with `rc.confirmation=off`
  (`uda.siapsection` as string; the dotted spec example is unusable —
  task 3.5 folds it into the description, so the concrete attribute drops
  the dot).
- Data-dir resolution honoring the `TASKDATA` override with a `~/.task`
  fallback, kept pure for unit testing.

## What it does not handle

- No field mapping (due labels, priority defaults, section fallback).
  That lives in the read path built in `2.x`.
- No refresh policy, cursor hold, or toast UI. That lives in `src/main`.
- No `until`/`wait`/`mask` handling; recurrence templates (`status:recurring`)
  stay hidden since only `+PENDING`/`+COMPLETED` are exported.

## Dependencies

- `std::process` — run the `task` subprocess and capture output.
- `std::env`, `std::path` — `TASKDATA` override and data-dir fallback.
- `serde_json` — parse export fixtures in unit tests only.

## Public API

```rust
pub fn data_dir() -> PathBuf;
pub fn resolve_data_dir(taskdata: Option<String>) -> PathBuf;
pub fn is_recur_cleared(recur: &str) -> bool;
pub fn run_write(args: &[&str]) -> Result<String, String>;
pub fn run_write_with_env(args: &[&str], taskdata: Option<&Path>, taskrc: Option<&Path>) -> Result<String, String>;
pub fn build_add_args(desc: &str, project: &str, due: &str, prio: Option<char>, tags: &[String], depends: &[String], recur: &str, section: &str) -> Vec<String>;
pub fn build_modify_args(uuid: &str, desc: &str, project: &str, due: &str, prio: char, old_tags: &[String], new_tags: &[String], section: &str, recur: &str) -> Vec<String>;
pub fn build_start_args(uuid: &str) -> Vec<String>;
pub fn build_stop_args(uuid: &str) -> Vec<String>;
pub fn build_done_args(uuid: &str) -> Vec<String>;
pub fn build_undone_args(uuid: &str) -> Vec<String>;
pub fn build_undo_args() -> Vec<String>;
pub fn build_delete_args(uuid: &str) -> Vec<String>;
pub fn build_recur_delete_args(uuid: &str) -> Vec<String>;
pub fn build_detach_recur_args(uuid: &str) -> Vec<String>;
pub fn build_section_args(uuid: &str, section: &str) -> Vec<String>;
pub fn build_project_args(uuid: &str, project: &str) -> Vec<String>;
pub fn build_tag_args(uuid: &str, add: &[String], remove: &[String]) -> Vec<String>;
pub fn build_depends_add_args(task_uuid: &str, dep_uuid: &str) -> Vec<String>;
pub fn build_depends_remove_args(task_uuid: &str, dep_uuid: &str) -> Vec<String>;
pub fn build_annotate_args(uuid: &str, text: &str) -> Vec<String>;
pub fn build_denotate_args(uuid: &str, text: &str) -> Vec<String>;
pub fn export_pending() -> Result<String, String>;
pub fn export_completed() -> Result<String, String>;
pub fn section_uda_args() -> Vec<String>;
pub fn ensure_section_uda() -> Result<String, String>;
```

- `resolve_data_dir` is the pure override check; `data_dir` reads `TASKDATA`.
- `run_write` runs `task <args>`, returning `Ok(stdout)` or `Err(stderr)`;
  `run_write_with_env` pins per-command dirs for isolated tests.
- `build_add_args` / `build_modify_args` are pure arg builders (add
  stamps `siapsection:` only when the draft carries a section; empty project/due clears on modify; tags diffed
  `+added`/`-removed`; add carries blocking links as one
  `depends:u1,u2` attribute so the create stays a single write; recur is
  skipped when cleared on add and always written on modify — `recur:<value>`
  to set, `recur:` to clear; taskwarrior requires a due date for recurrence,
  so the UI refuses recur-without-due before the write).
- `build_start_args` / `build_stop_args` / `build_done_args` /
  `build_undone_args` key the lifecycle commands by `uuid` (`done` completes,
  `modify status:pending` reopens); `build_undo_args` reverts the last
  backend change; `build_delete_args` prefixes
  `rc.confirmation=no` since the UI already confirms (the subprocess
  has no stdin to answer the prompt).
- `build_recur_delete_args` adds `rc.recurrence.confirmation=no` so
  deleting a template or one occurrence answers "no" to taskwarrior's
  "delete all pending recurrences?" prompt; the caller deletes the
  remaining occurrences explicitly. `build_detach_recur_args` runs
  `task <uuid> edit` with `rc.editor=sed` to strip `Recur:` and `Parent:`
  from one kept occurrence, preserving its UUID and every other field; the
  save path calls it for the selected occurrence and its earlier pending
  siblings.
- `build_section_args` / `build_project_args` stamp one field via
  `modify`; `build_tag_args` diffs `+added`/`-removed`.
- `build_depends_add_args` uses `depends:<dep>` (add-if-missing) and
  `build_depends_remove_args` uses `depends:-<dep>` (remove-one), so
  surrounding dependencies stay untouched; `depends+=` is avoided
  (task 3.5 folds it into the description). Used by popup `depends` save
  (project-scoped picker, unknown/self rejected with a toast).
- `build_annotate_args` uses `annotate <text>` and `build_denotate_args`
  uses `denotate <text>` (exact-text remove). Used by the popup
  `annotation` row (`a` add, `D` remove, empty drafts refused).
- `export_pending` / `export_completed` run the two JSON exports.
- `section_uda_args` is the pure `task config` arg list for registering
  `uda.siapsection.type` as string; the `src/main` bootstrap runs it through
  the env-aware write runner so tests stay isolated.
- `ensure_section_uda` registers the same UDA with `rc.confirmation=off`,
  so a default taskrc cannot block the bootstrap; per-task section writes
  (`m` move or `R` rename) still use `build_section_args`.

## Example

```rust
let pending = backend::export_pending()?;
let out = backend::run_write(&["12", "modify", "project:work"])?;
```

Unit tests use temp `TASKDATA` fixtures and fixture JSON strings, never
live data:

```sh
cargo test backend     # skeleton + fixture isolation
cargo test smoke       # headless render against mocks
```
