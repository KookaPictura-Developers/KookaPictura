## Why

Three CS6 selection tools (Elliptical Marquee, Polygonal Lasso, Magic Wand) are
visible but disabled, the marquee/lasso options bars expose only a label, and
the Select menu is almost entirely unwired even though `pictura-select` already
implements every underlying operation (`invert`, `expand`, `contract`, `border`,
`smooth`, `feather`, `grow`, `similar`, channel save/load, combine). The gap is
UI wiring and options, not new selection math. Closing it makes the selection
half of the CS6 toolbox and Select menu usable in one coherent change.

## What Changes

- Enable the **Elliptical Marquee** tool: drag rasterises an ellipse through the
  existing `select_ellipse` bridge; options bar gains combine mode, Anti-alias,
  Feather, and Style (Normal / Fixed Ratio / Fixed Size).
- Enable the **Polygonal Lasso** tool: click anchors, preview the path, close on
  the first point / double-click / Enter; options bar gains combine mode, Feather,
  Anti-alias.
- Enable the **Magic Wand** tool: click runs the existing `magic_wand` bridge;
  options bar gains Tolerance (0-255), Contiguous, Anti-alias, Sample All Layers,
  combine mode.
- Give the **Marquee**, **Lasso**, and **Quick Selection** options bars their real
  CS6 controls (mode, Feather, for Quick Selection tolerance plus Sample All
  Layers / Auto-Enhance placeholders marked inferred).
- Wire the **Select menu** through the bridge: Reselect, Inverse, Modify
  (Border / Smooth / Expand / Contract / Feather dialogs), Grow, Similar, Save
  Selection…, Load Selection…, and the panel-level All Layers / Deselect Layers /
  Similar Layers commands, with document/selection enable rules and one undo
  state per applied command.
- Defer **Magnetic Lasso**, **Color Range…**, **Refine Edge…**, and **Transform
  Selection**: remain visible but disabled with a documented reason.
- **BREAKING**: none. Disabled tools stay disabled; no existing requirement is
  removed.

## Capabilities

### New Capabilities

- `select-menu`: the Select menu surface (Reselect, Inverse, Modify submenu, Grow,
  Similar, Save/Load Selection, All/Deselect/Similar Layers), its enable rules,
  dialog parameters, undo semantics, and the bridge operations behind each row.

### Modified Capabilities

- `shape-selection-tools`: the Elliptical Marquee and Polygonal Lasso become
  enabled tools with CS6 option semantics, and the marquee/lasso combine-mode and
  rubber-band requirements extend to them. Quick Selection options extend with
  Sample All Layers / Auto-Enhance.
- `selection-tools`: the Magic Wand gains its full options set (Anti-alias,
  Contiguous, Sample All Layers) and the shared modified-selection operations
  (invert, modify, grow, similar, save/load) gain bridge-level requirements
  tying the existing engine functions to menu commands.

## Impact

- Engine reuse only: `pictura-select` already exposes every operation; no new
  selection algorithm is required. Anti-alias and (tool-time) Feather are not
  modelled by the engine and are documented as inferred ceilings.
- `crates/pictura-app/src/cxxqt_object/impl_selection.rs`: new bridge methods
  (`invert_selection`, `reselect`, `modify_selection`, `grow_selection`,
  `similar_selection`, `save_selection`, `load_selection`, `select_all_layers`,
  `deselect_layers`, `select_similar_layers`) and extended wand/options arguments.
- `crates/pictura-app/cpp/tools.{h,cpp}`: enable `EllipticalMarquee`,
  `PolygonalLasso`, `MagicWand`; new polygonal-lasso interaction state.
- `crates/pictura-app/cpp/options_bar.{h,cpp}`: new selection option pages.
- `crates/pictura-app/cpp/commands.h`, `command_tree.cpp`, `frame_menus.cpp`:
  new command ids, wired Select-menu rows, handlers, and enable providers.
- `crates/pictura-app/cpp/selftest_*`: new self-test codes per tool and command.
