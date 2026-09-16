## Why

The app ships 10 toolbox tools with text-labeled slots and a two-column grid,
while CS6 presents ~71 tools in 23 single-column flyout slots. Only 8 tool
cursors exist, the toolbox unreels no hidden-tool triangles, the unimplemented
tools are invisible rather than shown-disabled, the right panel rail renders
two-letter text glyphs instead of icons, and the Layers/History action buttons
are text-only. A user cannot see the shape of the CS6 toolbox or recognise a
panel by its icon.

This is a user-requested interruption to the layers-panel program. It takes the
**M38** number, so the layers-panel stages shift by one (panel anatomy → M39,
filtering → M40, management → M41, styles → M42, smart objects → M43); the
numbering note is recorded in `docs/dev/layers-panel-program.md` and
`docs/dev/STATE.md`. The frozen contract is
`docs/dev/m38-icon-cursor-library.md`.

## What Changes

- **Frozen tool catalogue (71 tools).** A single table in `tools.{h,cpp}`:
  asset id (`tool.<name>`), display label, shortcut char, flyout group + slot,
  implemented flag (true for the 10 existing tools only), `Qt::CursorShape`
  fallback, and cursor hotspot. The 10 existing asset ids are unchanged.
- **Full asset set.** 61 new tool icons + 63 new tool cursors (the 10 existing
  icons/cursors are reused), ~28 panel icons, 7 Layers-strip icons, and 1 History
  snapshot icon, all following `assets/icons/tool.move.svg` /
  `assets/cursors/tool.move.svg` and bundled in `assets/pictura.qrc`.
- **Per-tool cursor hotspots.** `icons.cpp` resolves the hotspot from the
  catalogue instead of the hardcoded `eyedropper ? (2,22) : (12,12)`.
- **CS6 toolbox chrome.** The two-column grid becomes a single column of 23
  flyout slots: a corner triangle where a slot has hidden tools, hold to reveal,
  `Alt`-click or `Shift`+shortcut to cycle, and the slot shows the last-used
  member. Unimplemented tools are disabled with the tooltip
  `<label> — not implemented yet`; the 10 implemented tools behave as today.
  The fg/bg widget and Screen-Mode button stay below the grid.
- **Panel icons.** The right rail's five buttons, the dock tabs, the Layers-panel
  action strip (link, fx, mask, fill/adjustment, group, layer, delete), and the
  History `Create Snapshot` button carry the documented SVG icons.
- **Verification.** A new `m38_icons` C++ self-test proves every catalogue tool
  has a non-null icon and cursor and a defined hotspot, every bundled `.svg`
  parses, the toolbox shows all 71 tools, implemented tools are enabled and
  unimplemented disabled, and every panel icon resolves.

## Capabilities

### New Capabilities

None. This is an asset + toolbox-chrome milestone; no new document, codec, or
compositor behaviour.

### Modified Capabilities

- `application-shell`: add the frozen toolbox catalogue, the 23-slot flyout
  presentation, the last-used-slot rule, and the disabled-unimplemented-tool
  contract.
- `tool-framework`: the Tools panel requirement changes from a two-column grid
  to a single column of flyout slots with disabled unimplemented tools.
- `panel-rail`: rail buttons and dock tabs render real panel icons instead of
  text glyphs.
- `layers-panel`: the panel's action strip carries the documented icons.
- `svg-cursors`: per-tool cursor hotspots replace "the eye-dropper at lower-left,
  every other tool centred".

## Impact

- `crates/pictura-app/cpp/tools.{h,cpp}` — the 71-row catalogue table and
  `allToolIds`/lookup helpers; the `ToolId` enum keeps the 10 implemented values.
- `crates/pictura-app/cpp/toolbox.cpp` — the single-column flyout grid, disabled
  state, and tooltips.
- `crates/pictura-app/cpp/icons.{h,cpp}` — the hotspot lookup table.
- `crates/pictura-app/cpp/frame.cpp` — rail buttons consume panel icons; dock
  tabs set their icon; Layers/History buttons set their icons.
- `crates/pictura-app/cpp/panels/{layers_panel,history_panel,panel_rail}.{h,cpp}`
  — icon wiring.
- `crates/pictura-app/cpp/main.cpp` — the `m38_icons` self-test step with fresh
  exit codes.
- `assets/icons/*.svg`, `assets/cursors/*.svg`, `assets/pictura.qrc` — the new
  assets and the regenerated resource list.
- `scripts/gen_qrc.sh` (new) — regenerates `pictura.qrc` from the asset dirs.
- `docs/dev/m38-icon-cursor-library.md`, `docs/dev/layers-panel-program.md`,
  `docs/dev/STATE.md` — the contract and the numbering note.
- No new dependency, no Rust change, no PSD or compositor change. Selecting an
  unimplemented tool remains impossible, so canvas routing and the options bar
  are untouched.
