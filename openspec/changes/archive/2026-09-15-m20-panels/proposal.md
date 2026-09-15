## Why

M16–M19 gave the shell a frame, menus, file lifecycle, tools, and imagery, but
its only panel is the M16 debug dock: a flat `QListWidget` with layer names and
adjacent debug buttons. The engine exposes far more than the UI shows (blend
modes, opacity, masks, a 20-state history), and there is no Navigator, Color,
Info, or Histogram surface. M20 replaces the debug dock with real panels backed
by the document and history models.

## What Changes

- Add a **Layers panel** with per-row visibility, thumbnail, editable name, blend
  mode, opacity, and lock, plus selection, add-adjustment, delete, and move
  up/down. Group expansion, drag-reorder, clipping/link/labels, and the
  filter/search row are deferred.
- Add a **History panel** with labeled states (newest last), a highlighted
  current state, click-to-jump, and a snapshot of the current state. This needs
  labels on the history stack and jump/snapshot operations in the bridge.
- Add a **Navigator panel** with a document thumbnail, a viewport proxy
  rectangle, a zoom slider (0.01×–32×), and Fit / 100% buttons.
- Add **Color and Swatches panels**: a foreground/background colour pair, RGB/HSB
  sliders with a hex field, a spectrum, and a default swatch grid that sets the
  foreground colour. The Eyedropper (M18) feeds the same state.
- Add **Info and Histogram panels**: Info shows cursor position, the colour under
  the cursor, selection size, and document dimensions; Histogram draws the
  composite's per-channel distribution with a channel selector.
- Extend `--self-test` with panel checks.

Out of scope: group-tree expansion, layer drag-reorder, layer styles/smart
objects, swatch library file I/O (`.aco`/`.ase`), sampling multiple points, and
Histogram source/channel-cache semantics.

## Capabilities

### New Capabilities

- `layers-panel`: the Layers panel rows, property editing, selection, and
  add/delete/move operations, and the bridge layer-property/thumbnail support.
- `history-panel`: labeled history states, jump, and snapshots, and the bridge
  history-label/jump support.
- `navigator-panel`: the Navigator thumbnail, proxy rectangle, and zoom controls.
- `color-swatches-panel`: the foreground/background colour state, the Color
  panel controls, and the default Swatches grid.
- `info-histogram-panel`: the Info readouts and the Histogram view.

### Modified Capabilities

_None._

## Impact

- Crates: `pictura-app` gains a `panels/` directory with per-panel widgets, frame
  wiring, and dock registrations; the cxx-qt bridge gains layer property
  setters/thumbnails and labeled history access.
- Build: `CMakeLists.txt` gains the panel sources. No new dependencies.
- Verification: `--self-test` grows panel checks and exit codes; existing
  engine/tool behavior is unchanged.
