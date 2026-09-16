## Context

M20 built the Layers panel as a flat table over the top-level layer list
(`crates/pictura-app/cpp/panels/layers_panel.cpp`): a `QAbstractTableModel`
(`LayersModel`) with five columns (visibility, thumbnail, name, mode, opacity),
a header `QComboBox` (blend) + `QSpinBox` (opacity), and four buttons (Add
Adjustment / Delete / Move Up / Move Down). The bridge
(`crates/pictura-app/src/cxxqt_object.rs`) exposes `layer_count`, `layer_name`,
`layer_kind`, `layer_visible`/`set_layer_visible`, `layer_blend`/`set_layer_blend`,
`layer_opacity`/`set_layer_opacity`, `set_layer_name`, `move_layer`,
`layer_thumbnail`, `add_adjustment`, `remove_layer`. The compositor's per-sample
alpha is `src_a × opacity × mask` (`blend_into`, `crates/pictura-render/src/lib.rs:423`;
the GPU does the same in `cs_main`, `crates/pictura-render/src/gpu.rs:626`). The
codec's additional-layer loop handles only `luni` and `lsct`
(`crates/pictura-codec/src/lib.rs:529`).

`docs/05-layers/layer-management-ui.md` (`LAY-002`) is the contract; the
researched CS6 behaviour is in `docs/dev/layers-panel-program.md`. The relevant
frozen facts:

- **Fill** is layer content opacity, distinct from Opacity (which also scales
  styles); `docs/05-layers/blend-modes.md` §Parameters, `LAY-002` §"Visibility,
  lock, opacity/fill, blend mode".
- **Locks** are four: Lock Transparent Pixels, Lock Image Pixels, Lock Position,
  Lock All; `/` toggles Lock Transparency per the repo contract. The lock strip
  sits below the blend/opacity/fill strip; a partial lock dims, a full lock is
  solid, a fully locked layer is locked.
- **Color labels** are set from the CS6 row context menu; the palette is
  `None, Red, Orange, Yellow, Green, Blue, Violet, Gray`.
- **PSD tags** (from the Adobe File Formats Specification and psd-tools 1.19,
  installed as the canonical oracle): `lspf` is a 4-byte integer whose low bits
  are transparency (`0x01`), image pixels/composite (`0x02`), position (`0x04`);
  `lclr` is `H6x` (2-byte color value + 6 padding = 8 bytes); `iOpa` is a
  1-byte fill-opacity value, default 255.

Constraints: the CPU compositor is the frozen oracle, the GPU path is ±1 LSB
against it, `write_psd`'s layout must not change for default documents, no new
dependency, and `docs/` is the long-form contract. `cxx-qt-lib 0.10` carries
only scalar bridge types, so the new controls use `i32`/`bool`/`QString`.

## Goals / Non-Goals

**Goals:**

- Add `fill`, `lock`, and `color` to `pictura_core::Layer` and update every
  construction site in the workspace (no `..Default::default()` escape hatch).
- Fold Fill into the composited source alpha identically on the CPU oracle and
  the GPU path; exact when fill is default.
- Read/write `lspf`, `lclr`, `iOpa`; defaults omitted, so default documents
  serialize byte-for-byte as before.
- Expose the attributes through the bridge with history/undo, and add the Fill
  spinbox, lock strip, and color chooser to the panel with the documented
  enablement rules.
- Ship runnable checks: codec round-trip, compositor fill equivalence, and a
  panel self-test step.

**Non-Goals:**

- Group expansion, indentation, multi-selection, drag-reorder, inline rename,
  filter/search, panel options, the bottom quick-action strip, solo visibility
  (M37–M39 in `docs/dev/layers-panel-program.md`).
- Layer styles/effects, blend-if badge, smart filters (M40).
- Smart objects, vector masks, clipping, artboards, layer comps (M41).
- A first-class Background layer flag and type/shape layer kinds. M36 detects
  the Background row from the model's existing signals (see Decision 7) and
  cannot yet model the forced type/shape locks.
- Blend-If, knockout, advanced-blending flags, and styles — Fill's interaction
  with effects is therefore not yet observable.

## Decisions

### 1. `ColorLabel` and `LockFlags` (frozen)

```rust
/// PSD `lclr` sheet color. Values match psd-tools `SheetColorType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorLabel {
    None = 0,
    Red = 1,
    Orange = 2,
    Yellow = 3,
    Green = 4,
    Blue = 5,
    Violet = 6,
    Gray = 7,
}

impl ColorLabel {
    pub fn from_byte(v: u8) -> ColorLabel; // 0..=7 named, anything else None
    pub fn to_byte(self) -> u8;
}

/// The three CS6 layer locks. A `u8` bit set, not an enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LockFlags(u8);

impl LockFlags {
    pub const TRANSPARENCY: u8 = 0x01;
    pub const PIXELS: u8 = 0x02;      // PSD "composite"/image pixels
    pub const POSITION: u8 = 0x04;
    pub const fn bits(self) -> u8;
    pub fn contains(self, flag: u8) -> bool;
    pub fn with(self, flag: u8, on: bool) -> LockFlags;
    /// All three lockable bits (the panel's "Lock All" toggle).
    pub fn all() -> LockFlags { LockFlags(0x01 | 0x02 | 0x04) }
    pub fn is_all(self) -> bool { self.0 & 0x07 == 0x07 }
}
```

- `LockFlags::all()` is **derived** (all three bits), matching the task scope
  and `docs/05-layers/layer-management-ui.md`. The PSD `lspf` Lock All encoding
  is a risk (Decision 5).
- `Layer` derives `PartialEq, Eq, Clone` today; the newtypes do too, so
  existing `assert_eq!` on `Document` keeps working. `fill: u8` is plain.
- Defaults: `fill = 255`, `lock = LockFlags(0)`, `color = ColorLabel::None`.

### 2. Wire fields and update every literal (frozen)

```rust
pub struct Layer {
    pub name: String,
    pub rect: PsdRect,
    pub blend: BlendMode,
    pub opacity: u8,
    pub fill: u8,          // NEW
    pub lock: LockFlags,   // NEW
    pub color: ColorLabel, // NEW
    pub clipping: bool,
    pub visible: bool,
    pub mask: Option<LayerMask>,
    pub adjustment: Option<AdjustmentData>,
    pub channels: Vec<Channel>,
    pub children: Vec<Layer>,
    pub is_group: bool,
}
```

All 44 literal sites across 14 files gain `fill: 255, lock: LockFlags::default(),
color: ColorLabel::None`. The three codec/UI constructors that already exist
(`empty_layer`, `adjustment_layer`, `new_document`, `layer_thumbnail`'s helper)
must be updated too. Add `impl Default for Layer` **only** if it removes churn
without hiding intent; the decision is to add the fields explicitly so no
construction site silently inherits an attribute.

### 3. Compositor: effective alpha `opacity × fill` (frozen)

CPU (`blend_into`):

```rust
let opacity = layer.opacity as f32 / 255.0;
let fill = if layer.is_group { 1.0 } else { layer.fill as f32 / 255.0 };
let masked = mask_alpha(layer, x as i32, y as i32) as f32 / 255.0;
let mut as_ = src_a * opacity * fill * masked;
```

- `fill == 255` gives `fill == 1.0`, and `x * 1.0` is exact in IEEE-754 for
  every finite `x`, so the default path is byte-identical.
- Multiplication order `src_a * opacity * fill * masked` mirrors the GPU
  shader exactly, minimising the fill≠255 CPU/GPU delta.
- `is_group` ignores Fill in compositing (CS6 has no group Fill). The field
  still round-trips through PSD.
- Adjustment layers carry `fill = 255`; the shader's adjustment branch also
  multiplies by `params.fill`, which is 1.0 for them.
- Function shape and the mask/clipping order are unchanged, so groups,
  adjustment layers, Dissolve, and off-canvas clipping behave exactly as today.

GPU:

```wgsl
struct Params { mode: u32, opacity: f32, fill: f32, count: u32, /* + rest */ };
...
let as_ = ab * params.opacity * params.fill * mask_a;   // adjustment branch
let as_ = s.a * params.opacity * params.fill * mask_a;  // source branch
```

`Gpu::dispatch` gains a `fill: u8` argument and writes
`(fill as f32 / 255.0)` into `Params.fill`; `composite_layer` passes
`if layer.is_group { 255 } else { layer.fill }`. `Params` grows by one `f32`;
the uniform layout is by-value, so this is a one-line struct change.

### 4. PSD read/write (frozen)

Read in `read_layer_record`'s tagged-block loop (alongside `luni`/`lsct`):

| tag | parse | default when absent |
|---|---|---|
| `lspf` | first 4 bytes `u32` big-endian; lock bits = `value & 0x07` | `LockFlags(0)` |
| `lclr` | first 2 bytes `u16` big-endian; `ColorLabel::from_byte(value as u8)` | `ColorLabel::None` |
| `iOpa` | first payload byte (length-agnostic: some writers emit 1, some 4) | `255` |

Also fold the legacy layer-record `flags` bit 0 (transparency-protected) into
the transparency lock, because Photoshop ≤5-era files (and psd-tools'
`LayerFlags.transparency_protected`) set it and the model has no other home for
it.

Write in `write_extra`, after `luni` and before/around `adjustment`:

- `lspf` only when `lock.bits() != 0`: a 4-byte `u32` = `lock.bits()` (write the
  low 3 bits; high bits zero). `write_record` sets record `flags` bit 0 when the
  transparency lock is set, since the legacy bit and the `lspf` bit are the same
  concept.
- `lclr` only when `color != None`: an 8-byte payload `[value:u16 BE, 0,0,0,0,0,0]`,
  matching psd-tools' `H6x`.
- `iOpa` only when `fill != 255`: a 1-byte payload `[fill]` (the codec's
  `write_tag` pads odd lengths to even, so the on-disk tag is 2 bytes; the
  reader is length-agnostic).

Because every tag is omitted at its default, `write_extra`'s output for a
default-only document is byte-identical to today, so `write_psd` is unchanged
for existing tests and fixtures. Round-trip is asserted against the **psd-tools
oracle**: a document carrying non-default fill/lock/color written by the codec
is opened by `psd-tools` and its `fill_opacity`, `locks`, and `sheet_color`
match.

### 5. Lock All and `lspf` encoding — risk, resolved conservatively

psd-tools models Lock All as `ProtectedFlags.COMPLETE = 0x80000000`, while the
Adobe spec text and the model here describe three low bits. The conservative
choice: **read** the low three bits into `LockFlags` and ignore any high bits;
**write** `0x07` for Lock All. Round-trip is therefore stable for our own files
and lossy only for a Lock All marker encoded as a high bit. This is called out
in `## Open Questions` and in the program doc's PSD-interop risk column. It does
not affect default documents.

### 6. Bridge API (frozen)

```rust
#[qinvokable] fn layer_fill(&self, i: i32) -> i32;                    // 0..=255, 0 OOR
#[qinvokable] fn set_layer_fill(self: Pin<&mut Self>, i: i32, value: i32) -> bool;
#[qinvokable] fn layer_lock(&self, i: i32) -> i32;                    // bitmask
#[qinvokable] fn set_layer_lock(self: Pin<&mut Self>, i: i32, flag: &QString, on: bool) -> bool;
#[qinvokable] fn layer_color(&self, i: i32) -> i32;                   // ColorLabel byte
#[qinvokable] fn set_layer_color(self: Pin<&mut Self>, i: i32, value: i32) -> bool;
```

- `flag` is `"transparency" | "pixels" | "position" | "all"` (case-sensitive,
  matching the PSD/model names); an unknown flag returns `false`.
- `set_layer_fill` clamps to `0..=255`; `set_layer_color` rejects values
  outside `0..=7`.
- Every successful setter calls `recomposite()` then `record(label)`, matching
  the existing pattern (`crates/pictura-app/src/cxxqt_object.rs:773`), so undo
  restores the pre-edit state. Labels: `"Fill Opacity"`, `"Lock"`, `"Layer Color"`.
- **Refusal rules** (return `false`, leave state unchanged):
  - `set_layer_fill`: group, Background row, or `lock.is_all()`.
  - `set_layer_opacity` (existing): Background row or `lock.is_all()`.
  - `set_layer_lock`: Background row.
  - `set_layer_color`: Background row.
- `layer_kind` gains `"background"` (see Decision 7); `layer_kind` is otherwise
  unchanged.

### 7. Background detection (frozen heuristic, upgraded in M37)

The model has no Background kind or flag: `new_document` creates a layer named
`"Layer 0"` and `read_psd` carries whatever name the file has. M36 therefore
defines a Background row as `i == 0 && !is_group && adjustment.is_none() &&
name == "Background"` — the CS6 label, and the only signal the current model
carries. The bridge's `layer_kind(i)` returns `"background"` for it. A
first-class `is_background` field / `NodeKind::Background` is an M37 item; this
heuristic is marked `// ponytail:` in the code with the M37 upgrade path.

### 8. Panel (frozen)

- Header strip becomes blend | opacity | **fill** (all in the same `QHBoxLayout`),
  then the existing controls. Fill is a `QSpinBox` range `0..=255`, disabled
  when no layer is selected, when `kind == "group"`, when `kind == "background"`,
  or when the layer is fully locked. Opacity gains the same Background /
  fully-locked disable.
- Lock strip: four checkable `QToolButton`s (transparency, pixels, position,
  all) driven by `layer_lock`; clicking calls `set_layer_lock`. The strip is
  disabled for the Background row. Because M36 has no type/shape kinds, the
  forced-lock rule is not yet reachable; it is an M37 dependency.
- Row context menu: right-clicking a row opens a menu with `Color Label ▸` (the
  eight entries) calling `set_layer_color`. There is no row context menu today;
  this is new.
- `syncControls` reads the three attributes and sets enablement; the model rows
  gain `fill`, `lock`, and `color` so a repaint shows them.

### 9. Tests and checks (frozen)

- `pictura-core`: `ColorLabel::from_byte`/`to_byte` round-trip and out-of-range
  mapping; `LockFlags::{with,contains,is_all,all}`; a default `Layer` has
  fill 255 / no lock / no color.
- `pictura-codec`: a document with non-default fill/lock/color round-trips
  (`read_psd(write_psd(doc)) == doc`); a default document serializes
  byte-identically to today (golden bytes or a `write_psd` equality against a
  pre-change fixture); an `lspf`/`lclr`/`iOpa`-bearing file is read with the
  expected values; psd-tools opens codec output and reports the attributes.
- `pictura-render`: `fill == 255` composite is byte-identical to the pre-change
  CPU composite for a masked, a group, and an adjustment scene; `fill < 255`
  scales the layer's contribution by the expected factor over a transparent and
  an opaque backdrop; the GPU fill scene is within ±1 LSB of the CPU oracle.
- App self-test (`main.cpp`): set fill to 128, toggle a lock, set a color, and
  observe the bridge getters, one history state per edit, and an undo that
  restores the prior values.

### 10. Process (frozen)

Waves: (1) brief + frozen interfaces, this change; (2) `pictura-core` model +
every literal (mechanical, one agent); (3) codec read/write + oracle round-trip;
(4) CPU/GPU compositor fill; (5) bridge + panel; (6) tests + self-test +
evidence; (7) close-out. The model change (wave 2) blocks waves 3–5, so it lands
first and alone.

## Risks / Trade-offs

- **Struct-literal churn.** Adding three fields breaks every `Layer` literal
  (44 sites). The update is mechanical but must be exhaustive; the compiler
  finds them, and `cargo test --workspace` proves it. No `Default` escape hatch
  is added, so a missed site is a compile error, never a silent default.
- **`lspf` Lock All encoding.** See Decision 5: read low bits, write `0x07`.
  A CS6 file that encodes Lock All as `0x80000000` reads as no lock. The
  round-trip test uses the codec's own output and psd-tools, so it cannot catch
  this; the program doc records it as a PSD-interop risk. Resolution requires a
  CS6-authored fully-locked PSD.
- **`iOpa` payload length.** psd-tools reads a 1-byte `ByteElement`; an
  ag-psd issue reports a 4-byte payload (byte + 3 pad). Reading the first byte
  is correct for both; our writer emits 1 byte (padded to 2 by `write_tag`).
  psd-tools parses it; a stricter reader might not. Low risk, documented.
- **Color-label range.** `ColorLabel` models 0–7. Photoshop CC adds 8–11
  (Seafoam/Indigo/Magenta/Fuschia); those read as `None` and are lost on
  rewrite. CS6 has no such labels, so this is a post-CS6 interop limit, not a
  parity loss.
- **Background heuristic.** A layer at index 0 named `Background` is treated as
  the Background; a renamed background or a lower-case `background` is not.
  M37 replaces it. The rule only affects refusal/enablement, never pixels.
- **Group Fill.** The model stores it and PSD round-trips it, but the
  compositor ignores it (CS6 has no group Fill). A PSD with a non-default group
  `iOpa` will not render it; `LAY-003`'s open question already flags that CS6's
  group-fill serialization is unsourced.
- **Fill vs. effects.** Fill is defined as "pixels only, not styles". Styles
  do not exist yet, so applying Fill uniformly to the source alpha is the
  correct subset; when M40 lands, Fill must move after the effect passes and
  Opacity must scale the effect result. Noted as an M40 dependency.
- **Bridge scalar types.** `cxx-qt-lib` has no enum type; lock/color cross as
  `i32`/`QString`. This matches the existing `layer_blend`-as-key convention.

## Migration Plan

Additive for the model (new fields) and the codec (new tags omitted at
defaults), so no existing document changes bytes. The compositor is a
one-factor change that is exact at the default. Rollback: remove the three
fields and the tags, and drop the panel controls; the fill factor disappears
because the field does. No on-disk migration is needed.

## Open Questions

- **Lock All `lspf` encoding** (Decision 5): `0x07` vs `0x80000000`. Resolve
  with a CS6-authored fully-locked PSD inspected byte-for-byte.
- **`lclr` exact payload**: the spec/psd-tools agree on 8 bytes (`H6x`), but the
  Adobe spec text is terse. Confirm against a CS6 color-labelled PSD.
- **`iOpa` presence rule**: does CS6 always write `iOpa` (even at 255) or only
  when non-default? `write_psd` omits at 255 to preserve the byte-identical
  default output; confirm the CS6 writer's rule before claiming byte parity
  with a CS6 save.
- **Group `iOpa` semantics**: whether CS6 writes a meaningful group fill or a
  constant belongs to `LAY-003`; M36 preserves the value and ignores it in
  compositing.
- **Background flag**: M36's name/index heuristic is a documented ceiling; a
  first-class flag is M37.
