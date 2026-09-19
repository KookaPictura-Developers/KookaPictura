## 1. Model

- [x] 1.1 Add `LayerBlock { key: [u8;4], data: Vec<u8> }` and `RawChannel { id: i16, data: Vec<u8> }` to `pictura-core`.
- [x] 1.2 Add `Document.color_mode_data`, `.image_resources`, `.global_layer_mask`, `.layer_section_extra` (all `Vec<u8>`).
- [x] 1.3 Add `Layer.blend_key: Option<[u8;4]>`, `.blending_ranges: Vec<u8>`, `.extra_blocks: Vec<LayerBlock>`, `.raw_channels: Vec<RawChannel>`.
- [x] 1.4 Add `LayerMask.extra: Vec<u8>`.
- [x] 1.5 `impl Default for Document/Layer/LayerMask` (sensible values: Layer opacity/fill 255, visible) and `..Default::default()` at every struct literal until `cargo check --workspace --all-targets` is clean. `Document::new` and `Layer`-constructing helpers keep their current values.

## 2. Read (capture)

- [x] 2.1 Header: read the color-mode-data section bytes into `color_mode_data` and the image-resource section bytes into `image_resources` (no longer skip).
- [x] 2.2 Layer section: store the global-layer-mask payload in `global_layer_mask`, and the bytes after it to the section end in `layer_section_extra`.
- [x] 2.3 Layer record: store the raw blend key in `blend_key` only when unrecognized; store the blending-ranges bytes in `blending_ranges`; store mask block bytes after the fixed 18 in `mask.extra`; collect unknown tagged keys into `extra_blocks` in order.
- [x] 2.4 Channel data: ids outside `{0,1,2,-1,-2}` are consumed as `declared_len` raw bytes (compression header included) into `raw_channels` instead of being dropped.

## 3. Write (re-emit)

- [x] 3.1 Header: write `color_mode_data` and `image_resources` with computed lengths (empty -> the current zero lengths).
- [x] 3.2 `write_record`: blend key = `blend_key` when it maps to `layer.blend`, else `layer.blend.to_psd_key()`; channel count includes `raw_channels`; emit raw channels verbatim with `declared_len = data.len()`.
- [x] 3.3 `write_extra`: mask block length 18 + `mask.extra.len()` then the extra bytes; write `blending_ranges` (not empty); after modeled tags write `extra_blocks` with `write_tag`.
- [x] 3.4 Layer section: write `global_layer_mask` (length + bytes) and `layer_section_extra`; the section length accounts for both.

## 4. Tests

- [x] 4.1 Round-trip: a document with non-empty resources/color-mode data/global mask/extra blocks/blending ranges/mask extra/raw channel reads and writes back equal.
- [x] 4.2 Unknown blend key survives read→write→read (and a changed known mode does not resurrect a stale key).
- [x] 4.3 `-3` real-user-mask channel bytes survive read→write→read.
- [x] 4.4 Engine-created default documents still serialize byte-identically (golden).
- [x] 4.5 Malformed/truncated preserved blocks still error without panicking.

## 5. Oracle and verification

- [x] 5.1 Oracle: read a psd-tools fixture, write it, and prove the image resources and any extra blocks are still present with `psd-tools` (or byte comparison of the captured blocks).
- [x] 5.2 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`.
- [x] 5.3 `bash scripts/verify-full.sh` and a headless self-test; record counts.
- [x] 5.4 Update `docs/dev/STATE.md` and the roadmap; commit with `TASK-ALLOWS-DOCS`.
