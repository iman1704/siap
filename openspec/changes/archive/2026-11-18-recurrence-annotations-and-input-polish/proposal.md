# Proposal

## Why

The staged `src/main.rs` and `src/backend.rs` work turns the two previously deferred taskwarrior fields into editable data: recurrence gets a list badge, add-box picker, editor field, and native `recur:` writes, while annotations get timestamped display plus inline add/remove through `annotate`/`denotate`. The same batch completes the add box (`depends` and `recur` parity, a far-left task ID column) and replaces every duplicated `▌` echo line with the real terminal caret. Current specs still pin read-only recur/annotations, append-only quick-add input, typed `depends`, an `Enter`/`r`-heavy hint bar, and no `/` search shortcut. Without a spec, the staged behavior would drift from the recorded contract.

## What Changes

- Recurrence becomes a seventh editable popup field and a `* recur` add-box picker: fixed `daily` / `weekly` / `monthly` / `yearly` presets plus `no recurrence`, a blue `↻ <recur>` list badge, `[`/`]` cycling in the editor, a `now: * …` reference, and `recur:` / cleared `recur:` backend writes. Setting recurrence without a due date is refused with `recurrence needs a due date` in both surfaces.
- Annotations become structured notes (`entry` timestamp + `text`) rendered under the popup read-only block as timestamped rows. `a` starts an inline note draft, `Enter` writes `task <uuid> annotate`, `[`/`]` select a note, and `D`/`x` removes the selected note via `task <uuid> denotate`; empty drafts and missing selections toast without writing.
- Quick-add gains a fixed-list `& depends` picker that appends same-project open tasks, stays open for multi-entry, and carries blocking links in one `depends:u1,u2` attribute; `Backspace` pops the last link. `depends` is no longer typed by hand in either surface.
- Dependency saves reject links that would close a direct, mutual, or transitive cycle with a `circular dependency` toast, in addition to the existing unknown-ID and self-link checks.
- The main rows, the `COMPLETED` tail, and the popup `BLOCKED BY` / `BLOCKS` rows gain a far-left ID column (short numeric ID, else an 8-character UUID prefix) with the description columns resized around it.
- Every typing surface uses the native terminal caret instead of an inline `▌` glyph: the add-box description and text filters, the Search query, the section prompt, popup `EDIT` fields, and the popup annotation draft. `←`/`→` move the caret, `Backspace`/`Delete` edit around it, the popup no longer renders a duplicate input line under the focused field, and the add-box description caret survives picker switches.
- The popup's fixed lists (priority, `depends`, `recur`) take `j`/`k` as highlight moves and ignore printable typing; `Backspace` in `depends` pops the last link and saves while staying in `EDIT`, and `Backspace` in `recur` clears the draft.
- Hotkey chrome is trimmed and regrouped: `/` jumps to Search from any list view, the list footer drops the `Enter` and `r` hints and adds `D delete` and `/ search`, the `?` overlay groups keys under List / Edit / Search, and `print_help` follows.
- Due presets shrink to `today` / `tomorrow` / `no date`, and the plain quick-add refusal reads `description required`.

## Capabilities

### New Capabilities

- None — every behavior lands in an existing capability.

### Modified Capabilities

- `task-editor`: seven-field editor with editable `recur`, interactive annotations, fixed-list `depends`/`recur`, field-row caret with `Delete`, dependency-cycle and recur-due validation, ID in relation rows.
- `task-list`: far-left ID column, recurrence badge, quick-add `depends`/`recur` fixed pickers and chips, in-place caret editing, due preset trim, quick-add validation.
- `task-dependencies`: blocking links never close a cycle (direct, mutual, or transitive).
- `task-operations`: `/` Search shortcut, trimmed footer hints with `D delete`, and the grouped List / Edit / Search help overlay.
- `task-views`: Search query caret movement and in-place editing with the native caret, plus `/` entry.
- `task-sections`: section prompts edit at the native caret in the footer.
- `taskwarrior-backend`: writable recurrence and annotations, structured annotation export, and `recur:` / `annotate` / `denotate` writes.
- `ui-theme`: the `↻` recurrence glyph joins the state glyph set.

## Impact

- Affected code: `src/main.rs` (staged diff, ~2,036 insertions / 377 deletions) and `src/backend.rs` (~77 / 3), with `docs/main.md` and `docs/backend.md` updated in the same worktree.
- Backend surface grows by `task <uuid> annotate <text>` / `denotate <text>` plus `recur:` on `add`/`modify`; no new crates, config keys, or data migrations.
- `Task.annotations` changes from a joined `String` to `Vec<Annotation>`; existing fixtures and mock tasks move to the structured shape.
- Risk: caret placement depends on exact column widths, so wide-glyph and narrow-terminal smoke tests now assert cursor positions and may need layout attention; cycle detection walks the in-memory graph and must stay in sync with the backend store.
