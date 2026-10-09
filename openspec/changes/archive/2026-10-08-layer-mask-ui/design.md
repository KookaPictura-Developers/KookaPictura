# Design

## Approach

The engine and the `layer_masks.rs` bridge already exist; this change only wires
them to the C++ shell. All calls go through the generated free functions from
`pictura_app/src/cxxqt_object/layer_masks.cxxqt.h`, mirroring how
`clipping.cxxqt.h` is included and called in `frame_menus.cpp`:

```cpp
#include "pictura_app/src/cxxqt_object/layer_masks.cxxqt.h"
if (view && layer_mask_add(*view, QStringLiteral("reveal-all"))) { refresh(); }
```

The bridge acts on the **active layer**, so the panel and menus resolve the
active path themselves (the row menu's clicked row becomes current before the
menu opens, exactly as the other per-row commands do).

### Action strip (`layers_panel.cpp`)

`layersStripMask` is created by `stripIconButton`; drop `setEnabled(false)` and
the "not implemented yet" tooltip, connect `QToolButton::clicked`, and pick the
kind from `QApplication::keyboardModifiers()` and `view_->has_selection()`.

### Row context menu (`layers_panel_menu.cpp`)

`kRowSpecs` marks `addLayerMask`, `deleteLayerMask`, `disableLayerMask`
implemented, and gains an `enableLayerMask` row. A single dynamic
`Disable`/`Enable` row is not possible: the bridge exposes no `enabled` read, so
the static table carries the two rows and each applies a fixed target state.
`performRowAction` dispatches the four ids through the bridge.

### Menu bar (`commands.h`, `command_tree.cpp`, `frame_menus.cpp`)

The eleven `Layer ▸ Layer Mask` leaves move from path-derived stubs to frozen
`command_ids`, registered implemented, and `frame_menus.cpp` installs one handler
and one enablement provider each. Add-style leaves are enabled without a mask on
the active layer (selection variants also require a selection); Delete, Apply,
Enable, Disable, Link, and Unlink are enabled only with a mask present.

### Properties panel (`properties_panel.cpp`, `properties_panel.h`)

The Layer page keeps its `QFormLayout` summary; a `maskSection_` widget is added
below it and shown only when the active layer's `layer_row_has_mask` is true.
Its name row reads "Layer Mask" and its buttons call the bridge. The three
parameter rows are disabled labels carrying the `— not implemented yet` tooltip.
The Layer page's layout gains a thin vertical host so the mask section is not
deleted when the summary form is cleared.

### Row indicators — blocked

The delegate link glyph and disabled red cross need per-row mask `linked` and
`disabled`. The bridge exposes only the active layer's `layer_mask_present` and
`layer_mask_linked`; there is no per-row read and no engine `layer_mask_enabled`
read. Adding one would mean changing Rust, which this change forbids. The
requirement is specified but its implementation tasks are left unchecked.

## Risks

- Enabling the mask strip button changes the `panels_rail` self-test (exit 100),
  which asserted the mask button disabled; `stripDisabled` drops `mask` while
  keeping the link/fx assertions and the icon check.
- The row menu gains a fourth mask row; `tst_command_tree`'s expected pixel list
  is updated to match.
