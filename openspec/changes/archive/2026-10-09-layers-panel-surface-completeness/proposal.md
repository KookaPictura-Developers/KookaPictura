# Proposal

## Why

The Layers panel is the one UI surface still missing a handful of self-contained
CS6 affordances. The row delegate only tints the eye with a color label and
paints no smart-object mark, the Panel Options dialog omits the two CS6 toggles,
the `Layer > Lock Layers` / `Lock All Layers In Group…` commands are absent or
greyed, and `Tab`/`Shift+Tab` rename navigation is specified but not wired. Each
gap is small and independent, and the engine primitives already exist, so this
change closes them together.

## What Changes

- Paint a CS6 color-label chip on each labeled row (a small bar at the row
  content edge) in addition to the existing eye-toggle tint.
- Paint the CS6 smart-object badge on the lower-right thumbnail corner; add a
  `layer_row_is_smart_object(i)` bridge role so the panel can distinguish every
  smart object (embedded included), not only placed/external ones.
- Add the two CS6 `Panel Options` toggles: `Add "copy" to Copied Layers and
  Groups` (affects the duplicate-layer name) and `Use Default Masks on Fill
  Layers` (a fill/adjustment layer created with an active selection gets that
  selection as a layer mask). Both default on and persist with the other
  panel-option settings.
- Wire `Tab` / `Shift+Tab` while inline-renaming: commit the edit and move the
  editor to the next / previous visible row, without wrapping at the ends.
- Add `Layer > Lock Layers` (`All`, `Transparency`, `Image`, `Position`) over
  the panel selection and wire the greyed `Layer > Lock All Layers In Group…` to
  lock every layer inside the current group. Each is one undo state.
- Defer, and record in `design.md`, the items that need unbuilt models:
  the Blend-If badge and `Alt`-click FX show/hide-all (need the layer-effects
  authoring model), the vector-mask thumbnail (needs the vector-mask change),
  and the Filter Effect dimension (needs effects).

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `ui/layers-panel`: the row delegate paints a color-label chip and a
  smart-object badge; `Panel Options` gains the copy-suffix and default-mask
  toggles; inline rename gains `Tab`/`Shift+Tab` navigation.
- `compositing/layer-locks`: add the lock-command requirements (`Layer > Lock
  Layers` over the selection and `Lock All Layers In Group…`).

## Impact

- Bridge: a new secondary cxx-qt bridge file exposing `layer_row_is_smart_object`
  and the panel-option setter, keeping `cxxqt_object.rs` at its size ceiling.
- App: `panels/layers_panel_internal.h` (row projection and paint),
  `panels/layers_panel.{h,cpp}` (options, rename event filter, session push),
  `panels/layers_panel_test.cpp` (test hooks), `command_tree.cpp` / `commands.h`
  / `frame_menus*.cpp` (lock commands), `session.{h,cpp}` (two new persisted
  options).
- Rust: `impl_layers.rs` / `impl_layers_rasterize.rs` read the two view-level
  options; a new bridge file; engine unit tests where behaviour changed.
- Tests: Qt Test cases in `cpp/tests/tst_layers_panel.cpp` and
  `cpp/tests/tst_command_tree.cpp`, plus Rust bridge/engine tests.
- No new dependencies. No `docs/` changes.
