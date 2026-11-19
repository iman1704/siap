# Tasks

## 1. Task editor modal and flat list

- [ ] 1.1 Flatten the popup to one attribute list with the name-only project field and verify the rendered popup contains no PRIMARY/SECONDARY headers and no `●` project marker.
- [ ] 1.2 Implement the `e` navigate / `i` type modal (`Enter` saves back to navigate, `e`/`esc` discards to VIEW, `j`/`k` types in type mode) and verify each transition with a popup key-handling test.
- [ ] 1.3 Add the filtered inline suggestion dropdown (`Tab` applies, `↑`/`↓` picks) for project/tags/due/priority/status and verify project and tag hints apply into the draft.
- [ ] 1.4 Update the editor `smoke` tests and `docs/main.md` editor section and verify the editor tests pass.

## 2. Add-task box pickers and chips

- [ ] 2.1 Implement trigger-char focus (`#`/`~`/`@`/`!`/`+`) with per-picker suggestion lists and verify each trigger opens its picker with only its own hints.
- [ ] 2.2 Move picker-list control to `↑`/`↓`/`Tab`, keep `j`/`k` typable, add `1`/`2`/`3` priority shortcuts, and verify typing `jk` filters while arrows move the cursor.
- [ ] 2.3 Render set-state chip colors with project/priority chips only when set, remove the extra hint row, shrink the footer, and verify set chips render colored and unset chips grey.
- [ ] 2.4 Update the add-box `smoke` tests and `docs/main.md` add-box section and verify the add-box tests pass.

## 3. Integration checks

- [ ] 3.1 Refresh the `?` help overlay and `--help` output for the new flows and verify every footer hint string appears in the overlay or help text.
- [ ] 3.2 Run `cargo fmt` and `cargo clippy -- -D warnings` plus the full `smoke` suite and verify all green before review.
