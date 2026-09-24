# Tasks: knockout-composite

## 1. Canvas coverage flag

- [x] 1.1 Add `cover: Option<Vec<bool>>` to `Canvas` in `crates/pictura-render/src/composite.rs`; `new`/`new_region` set it `None`.
- [x] 1.2 Add `Canvas::with_cover_from(base: &Canvas) -> Canvas` that clones `base.px` and sets `cover = Some(vec![false; n])` (same `w`/`h`/`ox`/`oy`).
- [x] 1.3 In `blend_parts`, after the `as_ <= 0.0`/dissolve early-returns, set `cover[i] = true` when `canvas.cover` is `Some`.

## 2. CPU knockout

- [x] 2.1 Rename the current `composite_layer` body to `composite_layer_inner(canvas, layer, doc)`; group recursion inside calls `composite_layer(inner, child, doc, None)`.
- [x] 2.2 Add `composite_layer(canvas, layer, doc, base: Option<&Canvas>)`: if `layer.knockout != Knockout::None` and `base` is `Some`, run `composite_knockout(...)` and return; else `composite_layer_inner`.
- [x] 2.3 Add `composite_knockout(canvas, layer, doc, base)`: `tmp = Canvas::with_cover_from(base)`; `composite_layer_inner(&mut tmp, layer, doc)`; for each output pixel copy `tmp.px[i]` into `canvas.px[i]` only where `tmp.cover[i]`.
- [x] 2.4 In `composite_rgba` / `composite_rgba_region`, if any layer below the top has `knockout != None`, build `base` = a same-region canvas with `doc.layers[0]` composited alone; pass `Some(&base)` for index `> 0` and `None` for index `0`.
- [x] 2.5 Mark the inferred mechanism and the nested-group/clipping/`Transparency Shapes` ceilings with `ponytail:` comments.

## 3. GPU decline

- [x] 3.1 In `check_supported`'s `walk` (`crates/pictura-render/src/gpu/mod.rs`), return `GpuError::UnsupportedAdvancedBlending` when `layer.knockout != Knockout::None`.

## 4. Tests and gates

- [x] 4.1 3-layer 1x1 test: `[red, green, blue(fill 128)]` — `None` shows green, `Deep` drops the green channel (punch-through to red).
- [x] 4.2 `Shallow` equals `Deep` at the root; `None` byte-identical to no field.
- [x] 4.3 A transparent knockout pixel leaves the backdrop unchanged.
- [x] 4.4 GPU test: `composite_gpu` returns `UnsupportedAdvancedBlending` for a knockout stack.
- [x] 4.5 `cargo nextest run -p pictura-render -p pictura-core`, `cargo fmt --all --check`, `cargo clippy -p pictura-render -p pictura-core --all-targets -- -D warnings`, `openspec validate knockout-composite --strict`.
- [x] 4.6 Confirm the GPU parity suites and existing goldens still pass (no knockout in their fixtures).
