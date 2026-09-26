## Context

The Layers-panel header already wires Opacity/Fill to `set_layers_opacity` /
`set_layers_fill` (bridge, 0–255) and a four-entry lock strip to
`set_layers_lock` (bridge). `LockFlags` has three bits (`0x01` transparency,
`0x02` pixels, `0x04` position); `all()` is `0x07`. This change extends the
input model to CS6 percentages, adds a fourth lock bit, and draws the clipping
indicator. No document/composite semantics change for opacity/fill; grouping
gains one refusal for the new lock.

## Goals / Non-Goals

- **Goals:** percent Opacity/Fill (text + popup slider + label scrub); five-icon
  lock strip; a `nesting` lock that pins a layer's structural parent; a
  clipping-mask row indicator.
- **Non-Goals:** drag-reorder across containers (a later change consumes the
  nesting refusal); type/shape forced locks; a mask/vector-mask/link/fx slot
  (those engine stages are separate); persisting any new panel state.

## Decisions

### D1 — Percent boundary
The view is the only percent-aware layer. `PercentField` exposes
`value()`/`setValue(int pct)` and `valueChanged(int pct)`; conversion is
`byte = qRound(pct * 255.0 / 100.0)` and `pct = qRound(byte * 100.0 / 255.0)`.
`255`↔`100`, `128`↔`50`, `0`↔`0`. `set_layers_opacity`/`set_layers_fill` are
unchanged.

### D2 — One reusable `PercentField`
`percent_field.{h,cpp}` is a `QWidget` holding a `QLineEdit` (validated 0–100)
and a popup `QSlider` opened from a small arrow button; pressing and dragging on
the field itself scrubs the value by the horizontal mouse delta. It emits only
on user input, so `syncControls` can `setValue` without feedback (the panel
keeps its `syncing_` guard). Used twice: Opacity and Fill.

### D3 — Nesting lock bit
`LockFlags::NESTING = 0x08`; `all()` = `0x0F`; `is_all()` tests `0x0F`. The PSD
`lspf` reader/writer already round-trips the raw `u32`; only the `& 0x07` mask
in `read.rs` widens to `& 0x0F` (`lock_from_bits` gains the fourth flag). The
legacy layer-record `flags` transparency bit is unchanged. `lock_bit("nesting")`
is added; the panel's full-lock tests read `0x0F`.

### D4 — Nesting refusal
The nesting lock pins a node's parent. `group_paths` (Group Layers) and
`ungroup_paths` (Ungroup Layers) SHALL refuse the whole operation when the
selection contains a nesting-locked node, exactly as they already refuse a
fully locked node. Within-container `move_path` (Move Up/Down) SHALL remain
allowed. `move_path` is the hook a future cross-container drag will extend; this
change records the rule and tests it at the bridge/engine level.

### D5 — Clipping indicator
The delegate draws a small downward-curve clipping-mask glyph (`layers.clipMask`)
left of the thumbnail when `ClippingRole` is set, keeping the M39 indentation
and base underline. Asset-missing is safe (glyph omitted).

### D6 — Icons
Five original lock SVGs (`layers.lockAlpha`, `layers.lockPaint`,
`layers.lockPosition`, `layers.lockNesting`, `layers.lockAll`) plus
`layers.clipMask`, 24×24 stroke `#c8c8c8` to match the existing set; regenerate
`assets/pictura.qrc`.

## Interface freeze

```
pictura_core::LockFlags      NESTING = 0x08, all() = 0x0F, is_all() => 0x0F
pictura-codec read.rs        lock_from_bits(u8 & 0x0F) sets four flags
bridge lock_bit(flag)        "transparency"|"pixels"|"position"|"nesting"|"all"
pictura_render group_paths   refuse if a selected node has NESTING
pictura_render ungroup_paths refuse if a selected node has NESTING
class pictura::PercentField  int value() const; void setValue(int pct);
                             signal valueChanged(int pct)
LayersPanel                  keeps Opacity/Fill as percent fields; 5 lock toggles
```

## Risks / Trade-offs

- **`lspf` bit 0x08** is defined by `psd-tools` `ProtectedFlags` but CS6's exact
  meaning is unverified; a default document omits the block, so no default-save
  byte changes. Marked with a `// ponytail:` note.
- **Percent rounding** loses 1/255 granularity at some values; accepted (the
  stored value is still exact for the common 0/25/50/75/100 points).
- **Nesting refusal is engine-only** until the drag-reorder change; the UI has
  no cross-container gesture yet, so the test drives the bridge.

## Migration

No stored-format change for a default document. A PSD carrying `lspf` bit 0x08
now reads as nesting-locked instead of dropping the bit; re-saving preserves it.
