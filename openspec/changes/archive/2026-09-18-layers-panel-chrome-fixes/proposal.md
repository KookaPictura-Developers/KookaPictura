## Why

The shipped Layers-panel header does not match the requested layout: the lock
strip sits below the layer list, Fill shares the blend row with no label,
Opacity and Fill have no labels and their picker is right-aligned, the
visibility toggle is pushed right by the tree's indentation, the filter toggle
is a text button that starts off, and the header carries a redundant Qt menu
button the panel-group menu already covers.

## What Changes

- **Header order.** Top to bottom: the filter/search row, then a row of blend
  mode and a labeled Opacity field, then a row of the five lock toggles and a
  labeled Fill field, then the layer list, then the action strip.
- **Opacity / Fill labels and picker.** Both fields get a leading label, and the
  popup slider is centred under the field instead of aligned to its right edge.
- **Left-anchored visibility toggle.** The eye is drawn at the panel's left
  edge for every row; the nesting indentation applies to the thumbnail/name, and
  an expand/collapse chevron is drawn for a group. **BREAKING** (internal): the
  tree's default branch indicator is suppressed and the delegate owns the
  indentation and chevron.
- **Filter lightswitch.** The filter on/off control is an icon (a lightswitch)
  rather than a text button, and it starts **on**.
- **One menu entry point.** The panel's own header menu button is removed; the
  panel-group widget menu (`▾`) remains the single menu path for the wired
  Layer commands.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `layers-panel`: the header layout order and labels, the left-anchored
  visibility toggle with content indentation and a group chevron, and the
  removal of the panel's separate menu button.
- `layers-filtering-search`: the filter on/off toggle is a lightswitch icon and
  defaults to on.

## Impact

- `crates/pictura-app/cpp/panels/layers_panel.{h,cpp}` — header layout, labels,
  tree type, removal of `panelMenu_`.
- `crates/pictura-app/cpp/panels/layers_panel_internal.h` — `LayersTreeView`
  (branch suppression) and the delegate layout.
- `crates/pictura-app/cpp/panels/layers_filter_bar.{h,cpp}` — icon toggle,
  default on.
- `crates/pictura-app/cpp/panels/percent_field.cpp` — centred popup.
- `crates/pictura-app/cpp/theme.cpp`, a new self-test TU (keeps
  `selftest.cpp` at its allowance), `docs/dev/STATE.md`.
- No Rust change; no new dependencies.
