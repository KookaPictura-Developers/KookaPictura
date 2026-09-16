## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/layers-panel-program.md`: the CS6 behavior summary
  (cited), the current-state inventory, the gap-analysis table, the M36–M41
  staged plan (dependencies, size, Core/Extended, PSD-interop risk), and the
  risks/open questions
- [x] 1.2 Write `proposal.md`, `design.md`, and `tasks.md`
- [x] 1.3 Freeze in `design.md`: `ColorLabel` and `LockFlags`; the three `Layer`
  fields; the CPU/GPU effective-alpha formula (byte-exact at fill 255, ±1 LSB
  otherwise, groups ignore fill); the `lspf`/`lclr`/`iOpa` layouts, defaults, and
  omit-at-default rule; the bridge signatures and refusal rules; the Background
  heuristic and its M37 upgrade; the panel controls
- [x] 1.4 Update `docs/dev/STATE.md` "Next" with the layers-panel program
  (M36–M41) pointing at `docs/dev/layers-panel-program.md`, claiming none of it
  is done; note the label collision with the deferred canvas-perf M36–M38 tracks

## 2. pictura-core: model + every construction site

- [x] 2.1 Add `ColorLabel` (`#[repr(u8)]`: None/Red/Orange/Yellow/Green/Blue/
  Violet/Gray) with `from_byte`/`to_byte` (out-of-range → None); derive
  `Debug, Clone, Copy, PartialEq, Eq`
- [x] 2.2 Add `LockFlags(u8)` with `TRANSPARENCY=0x01`, `PIXELS=0x02`,
  `POSITION=0x04`, `bits`, `contains`, `with(flag, on)`, `all()` (derived 0x07),
  `is_all()`; derive `Debug, Clone, Copy, PartialEq, Eq, Default`
- [x] 2.3 Add `fill: u8`, `lock: LockFlags`, `color: ColorLabel` to `Layer`
- [x] 2.4 Update every `Layer { .. }` literal (37 explicit sites plus the
  `..bare` update-syntax site in `pictura-core/src/lib.rs`'s `masked` test
  layer; 14 files):
  `pictura-core/src/lib.rs` (5), `pictura-codec/src/lib.rs` (6, incl.
  `empty_layer`), `pictura-app/src/cxxqt_object.rs` (6, incl.
  `adjustment_layer` and `new_document`), `pictura-render/src/lib.rs` (4),
  `pictura-render/src/gpu.rs` (2), `pictura-render/src/filter.rs` (1),
  `pictura-render/src/document_ops/{canvas,crop,orient,resize}.rs` (1/2/1/1),
  `pictura-paint/src/stroke.rs` (2), `pictura-render/tests/{oracle,
  document_oracle,gpu_parity}.rs` (1/2/4)
- [x] 2.5 Unit tests: default `Layer` is fill 255 / no lock / no color;
  `ColorLabel` byte round-trip and out-of-range → None; `LockFlags::all` is
  `0x07` and `is_all`; `with`/`contains`
- [x] 2.6 `cargo test -p pictura-core`; `cargo fmt`/`clippy` clean

## 3. pictura-codec: lspf, lclr, iOpa

- [x] 3.1 Read in `read_layer_record`'s tagged-block loop: `lspf` (first 4 bytes
  BE → lock bits `& 0x07`), `lclr` (first 2 bytes BE → `ColorLabel::from_byte`),
  `iOpa` (first payload byte, length-agnostic → fill); fold the legacy record
  `flags` bit 0 into the transparency lock
- [x] 3.2 Write in `write_extra`, omitted at defaults: `lspf` (4-byte `u32` =
  `lock.bits()`) when locked, `lclr` (8 bytes: `u16` value + 6 zeros) when
  colored, `iOpa` (1 byte) when `fill != 255`; set record `flags` bit 0 with the
  transparency lock in `write_record`
- [x] 3.3 `empty_layer` and any codec-side `Layer` literal carry the defaults
- [x] 3.4 Tests: `read_psd(write_psd(doc)) == doc` for non-default fill/lock/
  color; a default document's `write_psd` bytes are unchanged (compare against
  the pre-change output for a fixed fixture); an `lspf`/`lclr`/`iOpa`-bearing
  file reads the expected values; `psd-tools` opens codec output and reports
  `fill_opacity`, `locks`, and `sheet_color`
- [x] 3.5 `cargo test -p pictura-codec`; `cargo fmt`/`clippy` clean

## 4. pictura-render: opacity × fill (CPU + GPU)

- [x] 4.1 `blend_into`: `fill = if layer.is_group { 1.0 } else { layer.fill as f32 / 255.0 }`,
  `as_ = src_a * opacity * fill * masked`
- [x] 4.2 GPU `Params` gains `fill: f32`; the shader multiplies `as_` by
  `params.fill` in both the adjustment and source branches; `Gpu::dispatch`
  gains `fill: u8` and writes `fill as f32 / 255.0`; `composite_layer` passes
  `255` for groups
- [x] 4.3 Tests: fill 255 is byte-identical to the pre-change CPU composite for
  a plain, masked, group, and adjustment scene; fill < 255 scales the
  contribution over a transparent and an opaque backdrop; GPU fill scene within
  ±1 LSB of the CPU oracle (extend the existing `gpu_parity` tests)
- [x] 4.4 `cargo test --workspace`; `cargo fmt`/`clippy` clean

## 5. Bridge and panel

- [x] 5.1 Add the six bridge methods (`layer_fill`/`set_layer_fill`,
  `layer_lock`/`set_layer_lock`, `layer_color`/`set_layer_color`) with the
  `_`/`set_` naming, clamping/validation, and `flag` string parsing
  (`transparency|pixels|position|all`)
- [x] 5.2 Refusal rules: fill refused for group/Background/fully-locked; opacity
  refused for Background/fully-locked; lock and color refused for Background;
  return `false` and leave state unchanged
- [x] 5.3 Each successful setter `recomposite()`s then `record()`s
  (`"Fill Opacity"`/`"Lock"`/`"Layer Color"`); `layer_kind` reports
  `"background"` for the index-0 `Background` row (with a `// ponytail:` note
  naming the M37 upgrade)
- [x] 5.4 Panel: add the Fill `QSpinBox` (0–255) beside Opacity; a four-button
  lock strip; a row context menu with the eight color-label entries; extend
  `LayerRow` and `syncControls`; Fill/Opacity/lock-strip enablement follows the
  frozen rules. The panel derives the full/partial lock state from
  `layer_lock(i)`, so no `layer_full_lock`/`layer_locked_any` getters were
  needed, and the color context menu is not greyed on the Background row — the
  bridge refuses instead
- [x] 5.5 `cmake --build build`; wire the controls to the new bridge calls
- [x] 5.6 `main.cpp` self-test step: set fill 128, toggle a lock, set a color,
  and assert the getters, one history state per edit, and undo restoring the
  prior values; print `m36_attrs fill=… lock=… color=… undo=…`

## 6. Verification and evidence

- [x] 6.1 `cargo test --workspace` (all new tests, no weakened goldens)
- [x] 6.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`
- [x] 6.3 `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` on the
  GPU default and the CPU fallback (and the fixture variant)
- [x] 6.4 `openspec validate m36-layer-attributes --strict`;
  `openspec validate --all --strict`
- [x] 6.5 Update `docs/dev/STATE.md` with the M36 result and the new test count

## 7. Close-out

- [ ] 7.1 Archive the change (`openspec archive m36-layer-attributes`) and
  commit with the `TASK-ALLOWS-DOCS` marker where docs are touched — **deferred**:
  this close-out is docs-only and explicitly does not commit; archive/commit is
  left to a separate requested step
