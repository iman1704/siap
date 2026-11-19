# `main`

Source: `src/main.rs`

## Purpose

Single coherent TUI for siap: a tabbed task list with a split task inspector.
Covers `openspec/changes/siap-v2` and the E-hybrid redesign in
`mockups/E-hybrid.png`: a top tab bar with live counts (one tab per project),
two-line task rows with a priority dot and right-aligned urgency, a single-line
quick-add field, and a detail pane that opens on `Enter`. The list header
stays fixed while the task rows scroll. Search is a `/` mode rather than a
tab. Blank space and hairlines -- not a sidebar or row grid -- carry the
structure.

## What it handles

- Views: Inbox (open tasks with no project), Today (overdue / due-today /
  active), All (all open), and one tab per project (exact `project:` match).
  Search is a hotkey-only mode over every open + completed task; text fields
  use case-insensitive substring first, then `nucleo-matcher` fuzzy matching,
  and ID queries match a short-ID substring or UUID prefix. Projects are
  derived from current tasks, sorted A–Z and rebuilt on every refresh; stale
  projects leave and an active project that disappears falls back to All.
  The launch view is Inbox with the list focused.
- Top bar: pinned `Inbox`, `Today`, `All` tabs (never scroll) plus one
  tab per project with live counts, then right-pinned
  `open · due · active · synced HH:MM` totals for the current view. Search
  shows a `SEARCH n` pill at the bar start. The active tab uses a tinted background and cyan text;
  sync failures turn yellow. When project tabs overflow, only they scroll:
  the bar keeps the active project visible and shows `‹`/`›` for hidden neighbours; stats stay pinned.
- Navigation: `Tab`/`Shift+Tab` cycle header tabs with no wrap-around
  (never `h`/`l`); `1`–`3`
  jump to Inbox / Today / All and `4`+ pick project tabs. `/` opens Search
  from any tab, including with the inspector open (the inspector stays open
  and Search input owns keys while typing); `esc` leaves Search back to the tab it came from (clearing
  the query first when needed). Switching views clears the selection and
  parks the cursor on the first task. `→` opens (or toggles) exactly like
  `Enter`; `h`/`l` stay off view switching entirely.
- Project tabs: each project is a first-class header tab, sharing the same
  active tint, count, and `h`/`l` cycling as Inbox/Today/All. Long names
  truncate in the tab (the list title keeps the full name) and the tab window
  scrolls around the active project.
- List: flush-left, two lines per task. The title and quick-add/search field
  stay fixed while the task rows scroll vertically, keeping the cursor row
  fully visible with a two-line cushion. The first line carries the
  cursor/selection accent, a priority dot (`H` red, `M` yellow, `L` dim), the
  mint `◉` start indicator for active tasks, the description, and
  the right-aligned urgency (whole numbers stay bare, fractions keep one
  decimal), with a dim `urgency` label right-aligned above the column.
  The second line carries secondary metadata starting with `id · due` (ID in
  dim, due red overdue / yellow today / `INK2` otherwise; absent due leaves
  no trace, no dash, no leading separator; separators sit only between present segments),
  then project in its
  stable palette color (hidden inside its own project view), tags as `#tag`,
  plain blue recur text for recurring tasks (no icon), and a red `blocked by <ids>`
  trailer listing the open blockers' short IDs.
  The title prefix is bar · dot · start · space (`● desc`) and the metadata
  sits under it at the full prefix width, starting with the task ID.
  The title sits directly above the add box with no gap row.
  The main list carries no ID column; the ID lives in the metadata (`id · due`),
  and short IDs survive in Search matching,
  the inspector read-only `id` row, and the `BLOCKED BY` / `BLOCKS` relations.
  Blocked descriptions stay dimmed. Long values truncate with `…`.
- Sections: still one invisible UDA (`uda.siapsection`), derived from task
  values plus names added in this run. Project views render unsectioned tasks
  as a flat urgency-desc list, then each registry section with its count and
  `▾`/`▸` collapse arrow; Search, Inbox, Today, and All stay flat. Section
  headers are cursor stops. `a` on a section header (or a task inside it)
  starts the add box with that section prefilled so the committed task lands
  there; elsewhere the draft stays unsectioned. `m` moves the cursor task to the next section
  (wrapping), `A`/`R`/`X` open the inline footer prompt to add, rename, or
  remove empty named sections. Flat views refuse section actions with a hint.
- Completed tail: a hairline rule and a `COMPLETED n` header at the bottom of
  every view, with dim struck-through rows carrying `✓` and the
  description. The header is a cursor stop and collapses/expands; `d` toggles
  done/undone and the cursor follows into or out of the tail. The tail is
  capped at 10 recent completions.
- Quick-add: an outlined capture box with no fill (`a`) expands into the
  same rounded boundary: description, attribute chips (`[+ more] # tags ~ due`
  plus `* recur` one space after due, with project/priority/depends chips
  only when set), focused picker input,
  and suggestion list all inside the outline on black, TINT only on the
  focused row and the hot suggestion (the input line itself stays black so
  only one highlight shows besides the focused row; rows are fitted/padded
  to the inner width so long chips can never push the right border off-screen).
  While the box is open the list cursor highlight hides so the terminal caret
  in the box is the only cursor. Description keeps
  chips, a focused picker input, and a suggestion list. Description keeps
  immediate focus; `←`/`→` move its caret and `Backspace`/`Delete` edit around
  it. `#` tags / `~` due / `@` project / `!` priority / `&` depends / `*` recur
  / `+ more` jump to pickers; trigger chars stay reserved mid-description.
  `↑`/`↓`/`Tab` move pickers (and `j`/`k` in fixed lists) so typing `j`/`k`
  still works in text filters. `Shift+Tab` moves backward, `Backspace` on an
  empty filter removes that attribute, unset chips stay grey while set chips
  take their semantic color, and examples are shared with the inspector.
  Typed description and filter inputs scroll horizontally so long text keeps
  the caret on-screen instead of overflowing the box.
  Enter commits one task and stays open; `esc` leaves.
- Inspector: `Enter`/`i`/`→` on a task opens a read-only split pane on the right
  (fixed 54 columns with a blank gutter) and `Enter`/`esc`/`q` close it. Below 100
  terminal columns the same detail renders as a full-body overlay. `h`/`←`
  hand focus to the task list behind the split (`j`/`k` walk rows and the
  preview follows the cursor, holding its row over headers including
  COMPLETED and re-seating when the cursor returns to a task; every list
  hotkey (`space`, `s`, `d`, `u`, `D`, `m`, `t`, `v`, `r`, `A`/`R`/`X`,
  `1`–`9`, `/`, `?`) works there and a clean preview re-seats on the cursor
  task after each one); `i`/`Enter`/`l`
  hand focus back to the inspector, which never closes on a focus shift. `a`
  in list focus opens the add box behind the open inspector (add mode then
  owns keys until Esc, and the inspector caret yields to the add-box caret).
  `Tab`/`Shift+Tab` cycle views from both inspector focus and list-behind-inspector
  focus. View switches re-seat an open inspector on the new view's cursor task (only
  a view with no task rows at all closes it); dirty, editing, and annotating
  drafts never follow. The box
  border brightens (`INK2`) while the inspector holds focus and falls back
  to the hairline (`LINE`) when the list does. The pane
  has a `TASK` header with the current `VIEW` / `EDIT` mode and a cyan
  underline, the description as title (also the editable description field),
  editable rows for project / due / priority / tags / depends / recur, then a
  grey read-only block (`id`, `status`, `urgency`, `start`), the interactive
  `annotation` list, and `BLOCKED BY` / `BLOCKS` relations (regular spacing)
  as compact single rows (`ID  desc · due` only, empty due shows just
  `ID  desc`, shared ID width and padded desc so both sections align,
  truncated with `…`, never wrapping).
- Inspector editing: `e` (or `i`) enters `EDIT` for the focused field;
  `j`/`k` and `↑`/`↓` move fields in `VIEW`, while `Tab`/`Shift+Tab` toggle
  views even with the inspector open, and
  `Tab`/`Shift+Tab`/`↑`/`↓` move the focused picker highlight in `EDIT`. Typing lands at the text caret (`←`/`→` move
  it, `Backspace`/`Delete` edit around it, long text scrolls to keep the caret
  visible); empty text fields show a grey
  shared example that vanishes on input. Enter applies the highlighted
  suggestion and saves back to `VIEW` (validating non-empty description and
  recur-needs-due); `esc` discards the focused field. `depends` shows open
  same-project tasks and appends/removes one link at a time; `recur` clears
  via `Backspace` (highlight parks on `no recurrence` so Enter keeps the
  clear) or by picking `no recurrence` directly. Clearing recurrence on a
  generated occurrence keeps that task and every earlier pending occurrence
  (same UUIDs), makes each a one-off, and deletes the hidden recurring
  template plus every later pending occurrence; `recur`/`parent` are
  stripped from each kept task, while completed history is untouched;
  self-links and cycles toast and stay in `EDIT`. `annotation` supports `a`
  add, `[`/`]` select, and `D`/`x` remove. `d` toggles done/undone without
  closing; `E` toasts the `$EDITOR` placeholder.
- Footer: a bare `LINE` rule then short `·`-separated key hints.
  A toast, selection count, delete confirmation, or section prompt
  temporarily replaces the hints; the cursor position (`n / open-count`) sits
  at the right when the cursor is on an open task. The section prompt types
  directly on the status row with a real terminal caret, scrolling long names
  to keep the caret visible. Search input scrolls the same way.
- Operations: `s` start/stop, `d` done/undone toggle, `D` delete behind a
  `y/n` confirm, `space` toggle selection, `v` VISUAL range with bulk
  `D/m/t/d`, `u`/`Ctrl+z` single-level undo, `m` section cycle, `A`/`R`/`X`
  section prompts, `r` manual refresh, `/` Search, `?` grouped help, `q` quit.
- Backend reads/writes cover: `task +PENDING export` and
  `task +COMPLETED export`, section-UDA bootstrap with
  `rc.confirmation=off`, quick-add through `task add`, inspector saves through
  `task <uuid> modify`, `start`/`stop`/`done`/`modify status:pending`/`undo`/
  `delete`, annotation `annotate`/`denotate`, depends add/remove, and section
  moves/renames. Clearing recurrence on an occurrence detects the hidden
  template and the earlier/later pending occurrences: earlier ones are
  detached with `task <uuid> edit`, the template and later ones are deleted
  with `rc.recurrence.confirmation=no`, and the edited task is detached
  last. Every successful write re-exports and holds the cursor by `uuid`;
  failures toast and preserve list, selection, and cursor.

## What it does not handle

- No mouse input or scrollbar; the task rows scroll, but there is no thumb.
- No background sync daemon or file watcher; refresh is launch, after-write,
  and the manual `r` key.
- No `status` editing; that field stays read-only.
- No configurable theme or keymap (the palette is fixed).
- No project management screens beyond opening a project tab; project
  renames and deletions live in taskwarrior.

## Dependencies

- `ratatui` — layout, widgets, rounded borders, terminal cursor placement, and
  `TestBackend` smoke tests.
- `crossterm` — raw mode, alternate screen, and key events.
- `chrono` — local-day due labels and relative `start` ages.
- `serde_json` — parse `task export` JSON in mapping and backend tests.
- `nucleo-matcher` — fuzzy Search matching.
- `backend` (`src/backend.rs`) — `task` subprocess (`export_pending`,
  `export_completed`, `run_write` with stderr capture, `ensure_section_uda`).
- `std::io`, `std::time` — terminal setup, event loop, toast expiry.
- `std::collections` — `HashSet` for multi-select, `HashMap` for project
  colors.
- `std::cell` — the persisted task-list scroll offset.

## Public API

Binary crate, so there is no library API. The extension points are:

```rust
fn render(f: &mut ratatui::Frame, app: &App);
fn render_header(f: &mut ratatui::Frame, area: Rect, app: &App);
fn render_footer(f: &mut ratatui::Frame, area: Rect, app: &App);
fn render_list(f: &mut ratatui::Frame, area: Rect, app: &App);
fn render_inspector(f: &mut ratatui::Frame, area: Rect, p: &Popup, app: &App);
fn render_popup(f: &mut ratatui::Frame, area: Rect, p: &Popup, app: &App);
fn render_help(f: &mut ratatui::Frame, area: Rect);
fn popup_key(app: &mut App, code: KeyCode, mods: KeyModifiers);
fn add_key(app: &mut App, code: KeyCode, mods: KeyModifiers);
fn search_key(app: &mut App, code: KeyCode, mods: KeyModifiers) -> bool;
fn route_key(app: &mut App, code: KeyCode) -> bool;
fn tasks_from_export(json: &str, now: DateTime<Local>) -> Vec<Task>;
fn format_due_label(raw: Option<&str>, now: DateTime<Local>) -> String;
fn search_pattern(query: &str) -> Option<NucleoPattern>;
fn task_matches_search_query(t: &Task, query: &str, pattern: &NucleoPattern, matcher: &mut NucleoMatcher) -> bool;
```

- `render` draws one frame: tab bar, split list/inspector (or overlay below
  100 columns), footer, then the help overlay.
- `render_inspector` is the split pane; `render_popup` is the narrow-terminal
  full-body overlay. Both share `render_inspector_body`.
- `switch_to_view`, `cycle_view` (clamped, no wrap), `switch_view`, `open_search`, and
  `leave_search` own tab changes; `tab_views` is the header-tab order
  (Inbox, Today, All, projects A–Z, never Search).
- `popup_key` owns `VIEW` / single-field `EDIT` handling; `add_key` owns the
  quick-add pickers; `search_key` owns Search input/list modes; `route_key`
  owns the section prompt.
- `task_row` and `completed_row` build the two-line/metadata row spans
  (no ID column in the main list); `task_meta_spans` owns the
  id/due/project/tag/recur/blocked-by ordering (recur renders as plain text,
  no icon; `due_span` owns the colored due label); `rel_row` builds the shared compact single-row
  `ID  desc · due` line for both `BLOCKED BY` and `BLOCKS`.
- `visible_input` scrolls every typed input (add description/filter, Search
  query, inspector title/fields, annotation draft, section prompt) so the
  caret stays on-screen; `cursor_section` resolves the section under the
  cursor for section-aware quick-add.
- `format_start_ago` maps an export `start` timestamp to a relative age;
  `start_display` prefers it.
- `render_list` renders a fixed header (title + add/search field) and a
  separate stateful task list whose `ListState` offset follows the cursor.
  The offset lives in `App::list_offset` (`Cell<usize>`) so scrolling stays
  smooth between frames while the header remains still.
- `smoke` tests render with `TestBackend` and cover fixed header tabs with a
  scrolling project window and pinned stats, tab cycling with no wrap, Search pill/origin/return, scrolling with a fixed header,
  two-line row content and truncation, priority-dot colors, start
  positions, section headers/collapse, completed tail, outlined quick-add
  caret positions, split inspector field editing, examples/suggestions,
  annotations, dependency relations, footer modes, Search input/list modes,
  and the existing backend round trips.

## Example

```sh
cargo run            # the UI (live taskwarrior reads; `r` refreshes)
cargo run -- --help  # keys
cargo test           # mapping + backend + headless render checks
```
