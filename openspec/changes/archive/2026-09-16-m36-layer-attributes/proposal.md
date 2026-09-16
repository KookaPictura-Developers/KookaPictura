## Why

CS6's Layers panel exposes layer attributes the engine does not model: **Fill**
opacity, the four **lock** flags, and **color labels**. Today
`pictura_core::Layer` carries only name/rect/blend/opacity/clipping/visible/mask/
adjustment/channels/children/is_group (`crates/pictura-core/src/lib.rs:274`), the
panel renders a **flat 5-column table** (visibility, thumbnail, name, mode,
opacity) with four placeholder buttons (`crates/pictura-app/cpp/panels/layers_panel.cpp`),
and the codec reads only `luni` and `lsct`
(`crates/pictura-codec/src/lib.rs:515`). None of this can be fixed in the panel
alone: the compositor must fold Fill into the source alpha, the PSD codec must
carry the tags, and the bridge must expose getters/setters that are undoable.
A CS6-parity Layers panel cannot be built until these three attributes are
end-to-end.

This is the first milestone of the layers-panel program
(`docs/dev/layers-panel-program.md`). It deliberately ships only the attributes
that are already implied by `openspec/specs/layers-panel/spec.md`,
`docs/05-layers/layer-management-ui.md` (`LAY-002`), and
`docs/05-layers/blend-modes.md` (`LAY-010`) — no tree, no filtering, no styles.

## What Changes

- **Model.** `pictura_core::Layer` gains `fill: u8` (default `255`), `lock:
  LockFlags` (a hand-rolled `u8` newtype: transparency/pixels/position bits plus
  a derived `all`), and `color: ColorLabel` (a `u8` enum: `None`, `Red`,
  `Orange`, `Yellow`, `Green`, `Blue`, `Violet`, `Gray`). Every `Layer { .. }`
  literal in the workspace is updated (**44 sites across 14 files**, listed in
  `tasks.md`). No new dependency.
- **Compositing.** A layer's effective source alpha SHALL be
  `opacity/255 × fill/255` (masks and clipping fold in exactly as today), in both
  the CPU oracle (`blend_into`, `crates/pictura-render/src/lib.rs:423`) and the
  GPU compositor (`SHADER`, `crates/pictura-render/src/gpu.rs:302`). It MUST be
  byte-identical to today when `fill == 255`; the GPU path stays within the
  existing ±1 LSB CPU-parity contract. A group's `fill` is ignored in
  compositing (CS6 exposes no group Fill); the value still round-trips.
- **PSD I/O.** `read_psd`/`write_psd` read and write the `lspf` (protected/lock),
  `lclr` (color label) and `iOpa` (fill opacity) additional-layer-info blocks,
  with round-trip tests. Absent tags mean the defaults (no lock, no color,
  fill 255). `write_psd` MUST NOT change its output for documents that use only
  defaults.
- **Bridge.** `PictureView` gains `layer_fill`/`set_layer_fill`,
  `layer_lock`/`set_layer_lock` (per flag), and `layer_color`/`set_layer_color`;
  `layer_kind` is extended to report a Background row. Every setter recomposites,
  records a labelled history state, and is undoable. Fill is refused for groups,
  the Background layer, and fully locked layers; lock and color are refused for
  the Background layer; opacity is refused for the Background layer and fully
  locked layers.
- **Panel.** A **Fill** spinbox (0–255) beside Opacity, a **4-button lock strip**,
  and a **color-label chooser** on the row's context menu. Fill is disabled for
  groups, the Background layer, and fully locked layers.

## Capabilities

### New Capabilities

None. M36 extends existing capabilities.

### Modified Capabilities

- `layers-panel`: layer property editing SHALL cover fill, lock flags, and color
  label in addition to visibility/name/blend/opacity, each applied through the
  bridge and undoable; the panel SHALL expose a Fill control beside Opacity, a
  four-button lock strip, and a color-label chooser; Fill SHALL be disabled for
  groups and the Background layer and refused for fully locked layers, and
  opacity SHALL be refused for the Background layer and fully locked layers.
- `psd-layer-io`: the codec SHALL read and write `lspf`, `lclr`, and `iOpa` with
  absent-tag defaults, and `write_psd` SHALL be byte-identical for documents
  that use only defaults.
- `layer-compositing`: a layer's effective source alpha SHALL be
  `opacity/255 × fill/255`, byte-identical when `fill == 255`, within the
  existing GPU ±1 LSB contract.

## Impact

- `crates/pictura-core/src/lib.rs` — `ColorLabel`, `LockFlags`, `Layer::{fill,
  lock, color}`, updated tests.
- `crates/pictura-render/src/lib.rs` — `blend_into` fill factor.
- `crates/pictura-render/src/gpu.rs` — `Params.fill` uniform and the shader's
  per-branch fill factor; `dispatch` gains a `fill` argument.
- `crates/pictura-codec/src/lib.rs` — read `lspf`/`lclr`/`iOpa` in
  `read_layer_record`, write them in `write_extra`, sync the legacy record
  transparency bit in `write_record`, `empty_layer` defaults.
- `crates/pictura-app/src/cxxqt_object.rs` — the six new bridge methods,
  `layer_kind`, `set_layer_opacity` refusal, and every `Layer` literal.
- `crates/pictura-app/cpp/panels/layers_panel.{h,cpp}` — Fill spinbox, lock
  strip, row context menu; opacity enablement.
- `crates/pictura-app/cpp/main.cpp` — a self-test step for the new controls.
- All other crates/tests that construct a `Layer` literal (paint, render
  document ops, oracle tests).
- No new dependency; `composite_rgba`/`composite_active` and the layer stack
  semantics are otherwise unchanged. Layer-tree expansion, filtering, styles,
  mask/clip badges, merge/rasterize, and smart objects stay deferred to
  M37–M41.
