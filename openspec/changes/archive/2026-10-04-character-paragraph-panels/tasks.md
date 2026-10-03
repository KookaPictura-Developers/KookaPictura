# Tasks

## 1. Character panel

- [x] 1.1 Create `panels/character_panel.{h,cpp}`: a `QWidget` with `setView(PictureView*)` / `refresh()`, controls for font family, size, leading (auto + value), kerning (mode + value), tracking, horizontal/vertical scale, baseline shift, anti-aliasing, colour, and the caps/super/sub/underline/strike toggles; verify the app builds
- [x] 1.2 Populate the controls from `type_layer_character_setting` with signals blocked, show `type_default_character_setting()` and disable editing when the active layer is not a type layer, and verify refresh does not emit an edit (no history state)
- [x] 1.3 Commit an edit through `type_update_layer` (registering the resolved font family first), re-rendering the layer and recording exactly one `"Edit Type Layer"` state; verify with a test that an edit + one undo restores the previous value

## 2. Paragraph panel

- [x] 2.1 Create `panels/paragraph_panel.{h,cpp}`: controls for alignment/justification, left/right/first-line indents, space before/after, hanging punctuation, hyphenation, and composer; populate from `type_layer_paragraph_setting` with signals blocked
- [x] 2.2 Commit an edit through `type_update_layer` with one history state; verify with a test

## 3. Menu and options-bar wiring

- [x] 3.1 Add `window.panels.character` / `window.panels.paragraph` to `commands.h` and replace the disabled `leaf` Window and Type `Character`/`Paragraph` stubs in `command_tree.cpp` with enabled checkable commands sharing those ids
- [x] 3.2 Construct, `registerPanel`, and group both panels in `frame_build.cpp` (hidden by default) and add their members to `frame.h`
- [x] 3.3 Append both to the `kPanelToggles` table in `frame_menus.cpp` and wire the type options bar's Panel button to toggle them
- [x] 3.4 Add the two `.cpp` files to `CMakeLists.txt`

## 4. Tests

- [x] 4.1 Add `cpp/tests/tst_character_paragraph_panels.cpp` covering: the panel reflects the active type layer; the Window/Type entries are enabled and their checked state follows visibility; toggling shows/hides the panel; an edit records one history state and one undo restores the value
- [x] 4.2 Register the suite in `CMakeLists.txt` and verify it passes under `QT_QPA_PLATFORM=offscreen ctest -R '^tst_'`

## 5. Verification

- [x] 5.1 Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `bash scripts/verify-fast.sh`; verify all pass
