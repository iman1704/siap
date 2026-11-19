# Tasks

## 1. Editor model — read-only status and H/M/L labels

- [x] 1.1 Remove `status` from the popup draft (`Popup.status`, `F_STATUS`, `field_count` 7→6) and render it read-only under `id`, verifying the popup opens with six fields and status never takes focus.
- [x] 1.2 Relabel `prio_label`/`prio_color` and all suggestion/cycling/chip strings from `P1/P2/P3` to `H/M/L` with absent priority rendering `L`, verifying list rows show ` H ` / ` M ` / ` L ` and cycling wraps H→M→L→H.
- [x] 1.3 Update popup save/dirty paths (`save_popup`, `popup_is_dirty`, `revert_popup_field`, `popup_field_mut`) to ignore status/recur/annotations and shorten the save toast to `saved`, verifying `cargo test` popup save cases pass.

## 2. Shared contextual input

- [x] 2.1 Introduce shared constants (`CTX_SUGGEST_CAP`, `CTX_CLEAR_PROJECT/TAGS/DATE/DEPENDS`, `CTX_EXAMPLE_*`) and use them in both add-box and popup examples, suggestions, and truncation, verifying both surfaces render identical strings and cap at 5.
- [x] 2.2 Lead project suggestions with `no project` and tags with `no tags` in both surfaces (filter-aware, canonical-casing preserved), verifying applying a clear row empties the draft and `Enter` on typed `no project`/`no tags` clears as well.
- [x] 2.3 Render the popup contextual block under the focused `EDIT` field (separate `▌` input line, `now:` reference for project/priority, capped list below), verifying empty project shows `e.g. work▌` + `now: @ work` with exactly one `▌`, and priority shows `now: ! H` with no input line.

## 3. Depends and done-toggle behavior

- [x] 3.1 Make `no dependencies` always close the depends list and change `apply_depends_suggestion` to return `bool` (clear returns true, duplicate/unknown returns false with toast kept), verifying empty-draft still offers the clear row.
- [x] 3.2 Change popup `Enter` on depends to apply-then-save back to `VIEW` at once and update the `EDIT depends` footer to `Enter apply+save`, verifying one `Enter` on a candidate appends and saves with toast `saved`.
- [x] 3.3 Recompute dirty after popup `d` done-toggle (hold field, clamp, recompute via `popup_is_dirty`) so saved toggles stay clean, verifying `d`/`d` completes then reopens with `dirty == false`.

## 4. Header, toasts, operations, and sidebar chrome

- [x] 4.1 Track `last_sync`/`sync_ok` on every backend refresh path (success and failure) and render `· synced HH:MM:SS` / `· sync failed` / `· not synced` in the header, verifying a failed refresh toasts `refresh failed` and keeps rows unchanged.
- [x] 4.2 Append `· N blocked!` to done-toggle toasts when completing blocked tasks (list, bulk, and popup paths) while still completing, verifying completing one blocked of two toasts `completed 2 task(s) · 1 blocked!`.
- [x] 4.3 Trim popup title/footer `esc closes` hints and sidebar chrome (drop `taskwarrior, tamed` tagline and `uda.siapsection` footer, use dim `· · · · · · ·` separator), verifying `cargo test` render snapshots match.

## 5. Integration verification

- [x] 5.1 Run `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`, verifying all green and updating `docs/main.md` per `AGENTS.md` if module behavior changed.
