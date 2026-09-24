# STATE — project resume anchor

Snapshot for resuming after a context break. Update after each milestone.

## Where things are

- Repo: `github.com/Zawaro/kooka-pictura`, branch `main`. Docs-only corpus +
  a working Rust/Qt engine.
- Toolchain: Rust 1.98 (`rust-toolchain.toml`), system Qt **6.11.1**, cxx-qt
  **0.10.0**, wgpu **30.0.1**, lcms2 **6.2.0** (system Little CMS 2.19).
- Oracles installed for tests: `psd-tools` 1.19, ImageMagick 7.1.2, `magick`.
- Test suite: **1231 tests, 0 failed, 8 skipped** (the `move_profile_*` pair,
  `region_move_timing_4000`, `region_refresh_profile_4000`, `undo_profile_4000`,
  the `composite_profile_*` pair, and `filter_profile_1024`; counted from
  `cargo nextest run --workspace`, which excludes the pre-existing ignored
  `pictura-render` doctest that `cargo test --workspace` reports as the ninth
  skip). The C++ self-test reports **234 passed, 0 failed, 0 skipped**. The
  full gate (`scripts/verify-full.sh`) reports **1503 passed, 9 skipped,
  0 failed**.
- OpenSpec **1.3.1** (`/usr/bin/openspec`). M0–M47 archived plus the
  content-named `layers-panel-controls`, `layers-filtering-search`,
  `layers-panel-chrome-fixes`, `layers-panel-row-interactions`,
  `layers-panel-control-polish`, `layers-panel-management`,
  `selection-tools-and-menu`, `psd-interop-compression`,
  `psd-opaque-preservation`, `psd-smart-object-roundtrip`,
  `smart-object-source-render`, `adjustment-payload-decode`, and
  `psd-rle-write`, `convert-to-smart-object`, `rasterize-smart-object`,
  `place-smart-object`, `replace-smart-object-contents`,
  `open-as-smart-object`, `export-smart-object-contents`,
  `photo-filter-adjustment-decode`, `gradient-map-adjustment-decode`,
  `solid-color-fill-descriptor`, `gradient-fill-layer`,
  `edit-smart-object-contents`, `image-import`, `file-drop-routing`,
    `free-transform-mode`, `psb-write`, `color-balance-adjustment-decode`,
    `pattern-fill-layer`, `layer-effects-drop-shadow`,
    `layer-effects-outer-glow`, `layer-effects-inner-shadow`,
    `layer-effects-inner-glow`, `layer-effects-stroke`,
    `layer-effects-overlays`, `layer-effects-satin`,
    `layer-effects-bevel`, `layer-effects-legacy-lrfx`, and
    `channel-mixer-adjustment-decode`, `curves-adjustment-decode`,
    `selective-color-adjustment-decode`, `layer-effects-stroke-fills`,
    `vector-mask-render`, `vector-fill-content`, `color-mode-read`,
    `depth-read`, `color-lookup-adjustment-decode`, `psd-image-resources`,
    `psd-icc-convert`, `psd-file-info`, `psd-iptc-write`, `assign-convert-profile`,
    `xmp-metadata`, `metadata-templates`, `psd-zip-write`, `color-settings`,
    `app-control-server`, `depth-preserve`, `agentic-control-vision`,
    `agentic-control-actions`, `color-mode-write-back`, `cmyk-write-back`,
    `icc-output-mode-consistency`, `indexed-write-back`, `bitmap-write-back`,
    `agentic-control-input`, `depth-color-mode-write-back`,
    `agentic-control-e2e`, `phfl-v3-xyz-decode`, `type-layer-kind`,
    `tysh-model-roundtrip`, `multichannel-duotone-read`, and
    `knko-blend-if-model`, and
    `crs-xmp-edit`, and
    `blend-if-render`, and
    `type-engine-data`, and
    `text-render-seam`, and
    `text-rasterize-bundled`
    changes;
    canonical specs are in `openspec/specs/` (93 specs, `validate --all --strict`
   green), change history under `openspec/changes/archive/`; no change is open.
   The panel-program stage **layer styles / effects** is complete:
   `layer-effects-drop-shadow`, `layer-effects-outer-glow`,
   `layer-effects-inner-shadow`, `layer-effects-inner-glow`,
   `layer-effects-stroke`, `layer-effects-overlays`, `layer-effects-satin`,
   `layer-effects-bevel`, and `layer-effects-legacy-lrfx` cover the object-based
   effect kinds (drop shadow, outer glow, inner shadow, inner glow, stroke,
   color/gradient/pattern overlay, satin, and bevel & emboss) plus the legacy
   `lrFX` block.
- PSD interop roadmap (`docs/dev/psd-support-roadmap.md`): P1 (ZIP/ZIP-prediction
  read) and P2 (opaque lossless open→save) and P2.5 are shipped. P2.5 adds a
  smart-object model and the Camera Raw Filter view on top of the preserved
  blocks: `SmartObject`/`SmartFilter` in `pictura-core`, `descriptor.rs`
  (Photoshop descriptor DOM), `smart_object.rs` (resolve `SoLd`/`SoLE`/`lnk*`),
  `smart_filter.rs` (`set_camera_raw_option` preserving object class identity),
  and `smart_writer.rs` (author `SoLd` v4 + embedded `lnk2`). It covers smart
  objects CS6→current CC by *tolerant read + byte-preserving write*, proven only
  on the a reference build fixtures `assets/test_with_smart_object0{1,2}.psd`; the Camera
  Raw settings model targets the earliest CC (ACR 8 / PV2012) `Fltr` key set,
  later-CC keys preserved, and `crs:` XMP is lifted to a typed `CrsSettings`
  view and editable in place (`crs-xmp-edit`). A CS6/earliest-CC
  fixture and the manual Photoshop reopen are deferred follow-ups.
- **Color-mode read** (roadmap P4/G2/G3, change `color-mode-read`, archived):
  `read_psd` now opens Bitmap (depth 1), Indexed, CMYK, and Lab 8-bit documents
  in addition to Grayscale/RGB and normalizes each to the working mode on load.
  Bitmap and Indexed are exact (1-bit expansion and a 768-byte palette lookup);
  CMYK and Lab are profile-free approximations marked `ponytail:`. CMYK floors
  `color * black / 255` (psd-tools/Pillow rounds, a ≤1 difference); Lab is the
  exact CIELAB(D50)→sRGB(D65) transform and matches lcms2's unoptimized transform
  within 1 LSB — psd-tools' optimized `.convert("RGB")` LUT can differ by up to
  ~20 in-gamut, so the oracle uses the exact transform, not the composite. The
  normalized document's `mode` is `Rgb`, `depth` is `Eight`, and the new
  `Document.source_mode` carries the original mode (`Some` for a normalized file);
  the Indexed palette is consumed and `write_psd` still writes the working mode,
  so open→save is a documented lossy-in-mode save. A depth-1 Bitmap layer's color
  channel is bit-unpacked like the composite. Multichannel/Duotone and all 16/32-bit
  depth stay `PsdError::Unsupported`. Four new fixtures
  (`indexed/cmyk/lab/bitmap.psd`) are compared to psd-tools/lcms2 by
  `tests/color_mode_oracle.rs`; the app shows a status-bar "Converted from …"
  notice and the C++ self-test `color_mode_open` (code 297) covers it. No new
  dependency. A depth-1 layer channel with ZIP compression is `Unsupported` (as
  for the composite); the Lab tolerance ceiling is 1 LSB against the exact
  transform. `scripts/generate-fixtures.py` regenerated `lab.psd`/`cmyk.psd`;
  `bitmap.psd`/`indexed.psd` stayed byte-stable.
- Smart-object source rendering (roadmap P3, archived
  `2026-09-19-smart-object-source-render`): `pictura-render` now depends on
  `pictura-codec` and `composite_rgba` renders an `Embedded` smart object's
  payload when the layer has no raster proxy (`composite.rs::composite_smart_source`):
  decode the payload, prefer its stored merged composite, else composite its
  layers; sample nearest-neighbour into the layer rect; `External`/`Alias`/
  `Unresolved`/empty/undecodable are no-ops. `Document.merged_composite_present`
  distinguishes a real merged composite from the codec's zero-filled placeholder
  so the layers-fallback is reachable. `Trnf`/warp and bilinear resampling are a
  `ponytail:` ceiling. A layer with a raster proxy still renders from the proxy.
- Adjustment payload decoding (roadmap P3/G8, archived
  `2026-09-19-adjustment-payload-decode`): `pictura-render::composite::decode_adjustment`
  now decodes `expA` (fixed `H3f` struct → `ExposureParams`), `vibA` (descriptor
  keys `vibrance`/`Strt` → `VibranceParams`), and `blwh` (descriptor keys
  `Rd  `/`Yllw`/`Grn `/`Cyn `/`Bl  `/`Mgnt`, `useTint`, `tintColor` 0..1 →
  `BlackWhiteParams`). `pictura-codec` exposes the descriptor DOM
  (`read_descriptor`/`write_descriptor`/`DescValue`) publicly for this. Malformed
  payloads are a no-op. Deferred because the schema is not confidently groundable
  or the op is missing: `phfl` (colour-space conversion), `mixr` (layout), `curv`
  (list/format), `selc`/`clrL`/`grdm`.
- Photo Filter adjustment decoding (roadmap P3/G8, archived
  `2026-09-19-photo-filter-adjustment-decode`): a version-2 `phfl` payload now
  decodes to `PhotoFilterParams` (R, G, B taken from the first three of the four
  `u16` colour components, `u32` density in `0..=100`, `u8` luminosity), and
  `pictura-render` exposes `encode_photo_filter` so the app can author one. The
  app maps the `photo-filter` kind to a warming layer (density 25, luminosity
  preserved) and the Adjustments panel menu offers a `Photo Filter` entry
  (`adjustment:photo-filter`). Only version 2 is decoded; a truncated payload, a
  version-3 `phfl` (CIE XYZ), a component above 255, or a density above 100 stays
  a no-op. **Curves (`curv`) is deferred**: the model is single-composite versus
  Photoshop's per-channel curves, and the legacy channel-bitmap order is
  ungrounded (no real Photoshop fixture has any adjustment key). `mixr` (layout)
  remains deferred too.
- Gradient Map adjustment decoding (roadmap P3/G8, archived
  `2026-09-19-gradient-map-adjustment-decode`): a `grdm` payload (version 1 or 3)
  now decodes to `Adjustment::GradientMap(GradientMapParams)` (16-bit colour stops
  reduced with `>> 8`, sampling a linear luminance LUT), and `pictura-render`
  exposes `encode_gradient_map` (the `* 257` inverse) so the app can author one.
  The app maps the `gradient-map` kind to a black→white layer and the Adjustments
  panel menu offers a `Gradient Map` entry (`adjustment:gradient-map`); the C++
  self-test check is code 285. The key is `grdm` — the codec whitelist previously
  misspelled it `gdrm`, so real Photoshop `grdm` blocks were unrecognized (both
  spellings are now accepted on read). Ceilings: linear-only interpolation,
  midpoint/dither/transparency stops ignored, and only RGB (no non-RGB colour
  models).
- Solid-color fill descriptor (roadmap P3/G8, archived
  `2026-09-19-solid-color-fill-descriptor`): the real Photoshop `SoCo` descriptor
  (version-16 `Clr `/`RGBC` with `Rd  `/`Grn `/`Bl  ` doubles on the 0–255 scale)
  now decodes to `Adjustment::SolidFill([r, g, b, 255])`; the 4-byte in-house form
  stays readable (length-first dispatch). `pictura-render::encode_solid_color_fill`
  and `document_ops::add_solid_fill` now author the standard descriptor, and
  `is_fill_content_layer`/`rasterize_fill_content` accept both forms through one
  decoder. Alpha ceiling: the descriptor is RGB-only, so all app callers stay
  opaque (`// ponytail:` note in `create.rs`).
- Gradient fill descriptor (roadmap P3/G8, archived `2026-09-19-gradient-fill-layer`):
  the real Photoshop `GdFl` gradient-fill descriptor now decodes to
  `Adjustment::GradientFill(GradientFillParams)` (kind Linear/Radial/Angle/
  Reflected/Diamond, angle, scale, reverse, reusing `GradientStop`), composites
  generatively over the layer rect (psd-tools' geometry), and is fill content for
  rasterize through the same `decode_adjustment`. `pictura-render::encode_gradient_fill`
  and the `Layer > New Fill Layer > Gradient…` command author a black-to-white
  Linear fill; C++ self-test codes 286/287. Ceilings: colour-noise gradients
  (`ClNs`), transparency stops, midpoint, non-linear interpolation, and non-RGB
  are ignored.
- RLE write (roadmap P3/G12, archived `2026-09-19-psd-rle-write`): `write_psd`
  now PackBits-encodes the merged composite (color + document extra channels),
  layer color channels, and the raster mask (compression 1), instead of raw.
  Preserved `Layer.raw_channels` and all unmodeled blocks stay byte-for-byte.
  The byte golden `crates/pictura-codec/tests/fixtures/default_before.psd` was
  regenerated for the RLE output (`default_document_matches_rle_golden`); the
  P2 lossless-open→save guarantee is unaffected because only engine-encoded
  channel compression changes. ZIP **write** is still missing.
- Smart-object layer actions (app, archived `2026-09-19-convert-to-smart-object`
  and `2026-09-19-rasterize-smart-object`): `Layer > Smart Objects > Convert to
  Smart Object` builds an embedded PSD source for a raster layer, keeps the proxy
  (rendering unchanged), and authors `SoLd`/`lnk2`; `Layer > Rasterize > Smart
  Object` materializes the content and drops the preserved `SoLd`/`SoLE`/`plLd`
  block plus the document `lnk*` record (`pictura-codec::remove_linked_source`).
  The embedded-source render was refactored to `render_smart_source(so, rect,
  region)` so compositing allocates only the canvas-clipped region. `File >
  Place…` (archived `2026-09-19-place-smart-object`) inserts a PSD/PSB as a
  channel-less top smart-object layer (native size at the origin, no transform
  session) that renders from its embedded source; the GPU compositor declines a
  channel-less smart-object layer so `composite_active` falls back to the CPU
  oracle. `Layer > Smart Objects > Replace Contents…` (archived
  `2026-09-19-replace-smart-object-contents`) swaps the embedded source while
  keeping the layer's geometry, clears the proxy, and drops the preserved
  `SoLd`/`lnk*` so the save re-authors the new payload. The C++ smart-object
  self-tests (277-281) live in `selftest_layers_smart_object.{cpp,h}`, taking
  `selftest_layers_controls.cpp` from the 1200 cap to 1009. `File > Open As Smart
  Object…` (archived `2026-09-19-open-as-smart-object`) opens a PSD/PSB as a new
  untitled document (path `None`, so Save cannot overwrite the source) with one
  embedded smart-object layer. `Layer > Smart Objects > Export Contents…`
  (archived `2026-09-19-export-smart-object-contents`) writes the stored payload
  byte-for-byte to a file and records no history state. Deferred: linked objects
  and the placeholder menu commands New Smart Object via Copy and Stack Mode.
- `Layer > Smart Objects > Edit Contents` (archived
  `2026-09-19-edit-smart-object-contents`) opens the embedded source as a new
  untitled editor tab; saving the editor re-embeds the edited document into the
  originating layer (`commit_smart_object_edit`, labelled `"Edit Contents"`,
  exactly one origin undo state). The implementation reuses the existing
  export/open/save/replace bridge methods through a per-session `QTemporaryDir`
  temp file (no new byte-crossing FFI). Eligibility requires an `Embedded` object
  with a non-empty payload that parses as PSD/PSB; C++ self-test codes 288/289.
- Move-tool drag start is instant: `begin_move_preview` reuses a cached base
  composite keyed by `content_revision` + topmost-layer index. The base is the
  document with the topmost layer hidden, which does not depend on that layer's
  position, so a committed move leaves it valid (`record_move` deliberately does
  not bump `content_revision`) and the next press is a pure cache hit. The
  earlier key also included the layer's clamped rect, which threw the base away
  on every drag and forced a 140–200 ms region recomposite (GPU device + layer
  upload + 64 MB clone + convert) on the next press. `applyToolPolicy` warms it
  when Move is selected or the canvas rebinds. The `move_preview_cache` self-test
  (exit 197) covers hit, byte-identical reuse, hit-after-commit, and
  miss-after-content-change; the fresh-compute path stays byte-identical.
- Drag-start latency probe: `ImageView` records the press-handler, OS
  press→move, post-handler→move, and move→paint gaps and prints a `[press-trace]`
  line to stderr when a drag start exceeds one frame (or always with
  `PICTURA_PRESS_TRACE=1`); `ToolController` logs `[move-press]` with the cache
  hit flag and `begin_move_preview` time. This is what located the replaced cache
  key above.
- Mouse-move no longer pulls the composite across the FFI:
  `PictureView::document_width()` / `document_height()` read `doc.width` /
  `doc.height` directly, so `InfoPanel::refresh()` (called on every
  `ImageView::mouseMoved`) and `PicturaMainWindow::updateStatus()` no longer pay
  a full ~64 MB `image()` copy for a size readout. `InfoPanel` still samples the
  hovered pixel through the O(1) `sample_argb`. The `document_size` self-test
  (exit 198) asserts the accessors equal `image()` and track a `resize_canvas`.
  The histogram panel still pulls `image()` because it bins the pixels; the
  document load/switch paths (`addDocument`, `refresh`) still pull it to fill the
  canvas.
- The C++ app needs **Qt6::Svg** (`Qt6Svg` CMake package) alongside the other Qt
  modules; icons and cursors render through `QSvgRenderer`.
- Image import (non-PSD Open/Place, archived `2026-09-19-image-import`):
  `File > Open` and `File > Place…` accept raster images (PNG/JPEG/GIF/BMP/TIFF/
  WebP) beside the native PSD/PSB path. Qt decodes at the app boundary to packed
  RGBA8888 (`decode_image_rgba`), so the engine stays Qt-free. A pure
  `pictura-codec::probe_image` header probe + `ImageBudget` guards the edge
  (recognized headers are dimension/allocation pre-checked, unknown containers
  deferred to Qt with a cap on the actual decoded allocation). New
  `Document::from_rgba` and `add_raster_layer_from_rgba` build engine structures
  from pixels; `place_image` reuses `convert_to_smart_object` (baked proxy,
  embedded PSD source) so Place keeps its one-undo-state contract. PSD/PSB never
  leaves the native `read_psd` path. The modified capability is
  `smart-object-layer-actions`'s Place filter; `image-import` is the new
  capability; C++ self-test `lpr_image_import` (code 290). Ceilings
  (`// ponytail:`): original-file-bytes fidelity is deferred (Export Contents
  yields the authored proxy), frame 0 only, no ICC/EXIF, and a colour image
  placed into a Grayscale document renders red-as-gray. This was Phase 1 of the
  place/drop program; OS file drag-and-drop and free transform on place have
  since shipped, completing the program.
- OS file drag-and-drop (archived `2026-09-19-file-drop-routing`): a
  `FileDropRouter` event filter on the frame routes each drop by target — a drop
  on a document `ImageView` (canvas or its surrounding space) **places** each file
  into the current document, falling back to opening tabs when no document is
  open; a drop on the tab strip, menu bar, or options bar **opens** each file as
  its own tab. PSD stays on the native `openPath`/`place_smart_object` branch and
  other images reuse the Phase 1 `open_image`/`place_image` routing; a URL-less or
  non-file drag is never consumed, so the Layers-panel internal DnD and tab
  reordering are untouched, and failures are skipped per file. C++ self-test
  `lpr_file_drop` (code 291). This was Phase 2 of the place/drop program.
- Free Transform (move/scale/rotate, archived `2026-09-19-free-transform-mode`):
  a per-`PictureView` modal session with a bounding quad, 8 handles, and a rotate
  affordance; the live preview reuses the Move-tool overlay (cached base + a
  `QTransform`-drawn layer image, no recomposite per move). Enter commits one
  `"Free Transform"` state, Escape cancels bit-identically. The engine gained
  `transform_layer` — inverse-mapped bilinear resample of every channel plane and
  the mask into the transformed bounding rect, a refusal contract (missing path,
  group, adjustment, Background, position-locked, zero-area, bad scale), and
  channel-less smart objects materialized from their payload then consumed (the
  `SoLd`/`plLd` blocks and linked-source record dropped). `Edit > Free Transform`
  (`Ctrl+T`, `edit.freeTransform`) begins a session on the current layer; a
  successful `File > Place…` or canvas drop selects the new layer and enters the
  session (cancel keeps the `"Place"` state). C++ self-test code 292. Ceilings:
  bilinear-only, no skew/distort/perspective/warp. This completes the place/drop
  program (image import + OS file drop + free transform).
- PSB **write** (roadmap P5/G9, archived `2026-09-19-psb-write`): `write_psd`
  now emits a version-2 PSB when `Document.is_psb` is set or either dimension
  exceeds 30 000, and a new `write_psb` always forces a PSB; both share one
  container writer and accept dimensions up to 300 000, above which the write
  returns `PsdError::Unsupported`. The PSB container widens the layer-and-mask
  section length, the layer-info length, and each per-channel data length to
  `u64`, and the RLE scanline byte-count table entries to `u32`; the global
  layer-mask info length stays `u32`. The bundled correctness fixes it required:
  PSB "big key" additional-layer-info blocks (`lnk2`/`lnk3`/`lnkE`,
  `Lr16`/`Lr32`/`Layr`, `LMsk`, `Alph`, `FMsk`, `PxSD`, `pths`,
  `Mtrn`/`Mt16`/`Mt32`, `cinf`, `extd`/`extn`, `artd`, `FXid`/`FEid`/`FELS`)
  carry an 8-byte length in a PSB (`common::is_psb_big_key`, mirroring
  psd-tools' `_BIG_KEYS`); per-layer tagged blocks declare an even length with
  the pad inside it, while document-level blocks declare the exact length and are
  padded externally to 4 (`reframe_document_extra` re-frames a preserved
  document-level block to the output container's width); and `iOpa` is written as
  a 4-byte `[fill, 0, 0, 0]` block. `Document` gained `is_psb` (set by the
  reader, false for new documents) and the reader gained the matching big-key
  widths. Proven by the psd-tools oracle: an authored-smart-object PSB, a
  reframed PSD→PSB big key, and odd/non-multiple-of-4 block framing all decode.
  Ceilings: an `8B64` document-level signature is normalized to `8BIM` on
  re-frame; a small PSB is preserved as a PSB via `is_psb`; and opening a
  >30 000 PSB through the app is still capped by the import probe budget
  (`probe.rs`, 30 000), a distinct follow-up.
- Color Balance adjustment decode (roadmap P3/G8, archived
  `2026-09-19-color-balance-adjustment-decode`): `blnc` is now a recognized
  adjustment key; `pictura-render` exposes `decode_color_balance` (nine `i16`
  shifts plus a luminosity byte, each in `-100..=100`, pad ignored) and
  `encode_color_balance` (clamps to range). The app maps kind `color-balance` to
  the neutral Photoshop default (all-zero shifts, preserve-luminosity on) and the
  Adjustments panel offers `Color Balance`; C++ self-test `lpr_color_balance`
  (code 293). The engine op (`Adjustment::ColorBalance`) already existed — only
  PSD decode/encode and app wiring were added. Remaining P3 adjustment keys:
  `curv`, `mixr`, `selc`, `clrL`, and version-3 `phfl`.
- Pattern fill layers (roadmap P3/G8, archived `2026-09-19-pattern-fill-layer`):
  the `PtFl` pattern-fill descriptor and the document `Patt`/`Pat2`/`Pat3`
  pattern library are decoded (`pictura-codec::decode_patterns`), pattern-fill
  layers composite as tiled content (`pictura-render::composite_pattern_fill`)
  and are rasterizable, and `PtFl` joined `ADJUSTMENT_KEYS` so a real block is
  recognized instead of dropped to `extra_blocks`. Missing or unsupported
  patterns fall back to a grey placeholder; authoring (`Layer > New Fill Layer >
  Pattern…`) is deferred. The change brought security hardening: `pictura-codec`'s
  `inflate` now bounds ZIP/Deflate channel decompression to the expected plane
  size (a crafted pattern/channel can no longer balloon memory), and pattern
  parsing has a mode-derived channel-count gate and a pixel cap. Remaining P3:
  `curv`, `mixr`, `selc`, `clrL`, `phfl` v3, layer effects (`lfx2`/`lrFX`), text,
  vector masks.
- Layer effects — Drop Shadow (roadmap P3/G6, archived
  `2026-09-19-layer-effects-drop-shadow`): the object-based effects `lfx2`
  descriptor is now decoded for a **Drop Shadow**
  (`pictura-render::layer_effects::decode_drop_shadow`), and a layer with an
  enabled present shadow composites it behind its content, offset by
  `Distance`/`Angle`, with a `Spread` dilate, a Gaussian `Size`, and the shadow
  colour/opacity/blend, shaped by the layer mask. The matte follows pixel alpha,
  fill coverage (`SoCo`/`GdFl`/`PtFl`), or a channel-less smart source. The GPU
  rejects the document and falls back to the CPU
  (`GpuError::UnsupportedLayerEffect`). A psd-tools-authored `drop_shadow.psd`
  fixture proves decode + round-trip + rendering. Ceilings: only Drop Shadow
  (other effects, legacy `lrFX`, group/adjustment-layer effects, `Scale
  Effects`, knock-out application, and a GPU shader are deferred); the
  global-light resource (id 1037) is not decoded (the stored local angle is
  used); a canvas-filling layer at the maximum `Size` is O(canvas·size). The
  change also did a pure move: the blend-mode math moved out of `composite.rs`
  into `blend.rs` and the source-over blend was factored into a `blend_parts`
  helper (behaviour unchanged), keeping `composite.rs` under the cap.
- Layer effects — Outer Glow (roadmap P3/G6, archived
  `2026-09-19-layer-effects-outer-glow`): the object-based `lfx2` **Outer Glow**
  (`OrGl`) is now decoded and composited — exterior matte (`1 − matte`),
  `Spread` (`Ckmt`) dilate then Gaussian `Size`, colour/opacity/blend (default
  Screen, `#FFFFBE`, opacity 75, size 5, Softer), composited behind the content
  through the same bbox/early-out pipeline as Drop Shadow; the GPU rejects an
  enabled+present glow (`UnsupportedLayerEffect`) and falls back to CPU; a
  psd-tools-authored `outer_glow.psd` fixture proves decode/round-trip/render.
  Ceilings: `Precise` technique renders as `Softer`; range/contour/noise/jitter/
  anti-alias/gradient-mode glows and a GPU shader are deferred; the global-light
  resource is still not decoded.
- Layer effects — Inner Shadow (roadmap P3/G6, archived
  `2026-09-19-layer-effects-inner-shadow`): the object-based `lfx2` **Inner
  Shadow** (`IrSh`) is now decoded and composited — an interior matte
  (`M · blurred(offset, erode)` confined to the content), `Distance`/`Angle`
  offset, `Choke` (`Ckmt`) erode then Gaussian `Size`, colour/opacity/blend
  (default Multiply, black, 75), composited above the content; the GPU rejects an
  enabled+present inner shadow (`UnsupportedLayerEffect`) and falls back to CPU;
  a psd-tools-authored `inner_shadow.psd` fixture proves decode/round-trip/
  render. `composite.rs` now runs a below-content and an above-content effects
  pass (drop shadow/outer glow below, inner shadow above); the split was verified
  byte-identical to the single-pass composite. Ceilings: `knocks_out`
  (`layerConceals`) is decoded but inert; contour/noise/anti-alias, the
  global-light resource, and a GPU shader are deferred.
- Layer effects — Inner Glow (roadmap P3/G6, archived
  `2026-09-19-layer-effects-inner-glow`): the object-based `lfx2` **Inner Glow**
  (`IrGl`) is now decoded and composited — an interior matte confined to the
  content, `Source` Edge (`glwS` typeID `IGSr`, `SrcE`) or Center (`SrcC`),
  `Choke` (`Ckmt`) erode then Gaussian `Size`, colour/opacity/blend (default
  Screen, white, 75), composited above the content; a `(size 0, choke 0, Edge)`
  pair is a strict no-op. The GPU rejects an enabled+present inner glow
  (`UnsupportedLayerEffect`) and falls back to CPU; a psd-tools-authored
  `inner_glow.psd` fixture proves decode/round-trip/render. The change also did a
  pure move: `layer_effects.rs` split into `layer_effects/{mod,shadows,glows}.rs`
  (drop shadow / inner shadow in `shadows`, outer/inner glow in `glows`, shared
  helpers in `mod`), verified behavior-preserving. Ceilings: `Center` is a
  complement-of-edge approximation, linear choke, `Precise` renders as `Softer`,
  contour/noise/anti-alias ignored, and the legacy `IGsr` typeID is accepted
  leniently.
- Layer effects — Stroke (roadmap P3/G6, archived
  `2026-09-20-layer-effects-stroke`): the object-based `lfx2` **Stroke** (`FrFX`)
  solid-colour stroke is now decoded and composited as a band at the content
  edge — `Position` Outside/Inside/Centre (`Styl` typeID `FStl`), `Sz  ` size
  `1..=250`, colour/opacity/blend (default Normal, black, 100), composited above
  the content; gradient/pattern strokes are deferred (a non-solid `FrFX` decodes
  `None`); the GPU rejects an enabled+present solid stroke and falls back to CPU;
  a psd-tools-authored `stroke.psd` fixture proves decode/round-trip/render; the
  new `layer_effects/strokes.rs` holds the kind. Ceilings: gradient/pattern
  fills, contour/anti-alias/overprint, and the integer max/min band (psd-tools
  uses a doubled-radius edge mask) are deferred.
- Layer effects — Color/Gradient/Pattern Overlay (roadmap P3/G6, archived
  `2026-09-20-layer-effects-overlays`): the object-based `lfx2` overlay effects
  are now decoded and composited — **Color Overlay** (`SoFi`), **Gradient
  Overlay** (`GrFl`), and **Pattern Overlay** (`patternFill`) into typed params.
  Each fills the layer's own content coverage: the source (solid colour;
  gradient via the shared gradient geometry, layer-rect anchored when
  `align_with_layer` and canvas otherwise; pattern via the document pattern
  library, grey placeholder when the id is absent) is gated by the masked
  content matte `M` and composited **above** the content with the effect's own
  blend mode and `M · opacity/100` alpha. Overlay defaults: absent `Angl` → 0,
  absent `Type` → Linear. The GPU rejects an enabled+present overlay
  (`UnsupportedLayerEffect`, no shader) and falls back to the CPU; psd-tools
  authored `{color,gradient,pattern}_overlay.psd` fixtures prove
  decode/round-trip/render; the new `layer_effects/overlays.rs` holds the three
  kinds. The change also extracted the shared `fill.rs` helpers
  `gradient_params_from_desc`/`pattern_params_from_desc` (a pure move), which
  tightens the strict `GdFl` `Type`/`GrdF` typeID checks and rejects an `f64`
  that overflows `f32` (no existing fixture regressed). Ceilings: gradient
  `Ofst`/noise/`Dither`, pattern rotation (`Angl` decoded but not applied); the
  remaining kinds — bevel and the legacy `lrFX` block — are deferred.
- Layer effects — Satin (roadmap P3/G6, archived
  `2026-09-20-layer-effects-satin`): the object-based `lfx2` **Satin** (`ChFX`)
  is now decoded and composited — an interior directional band from the blurred
  content matte (`|B(x−dx, y−dy) − B(x+dx, y+dy)|`, offset by `Distance`/`Angle`),
  optionally inverted, tinted colour/opacity/blend (defaults Multiply, black, 50,
  angle 19, distance 11, size 14), confined to the content and composited above
  it; the GPU rejects an enabled+present satin (`UnsupportedLayerEffect`) and
  falls back to CPU; a psd-tools-authored `satin.psd` fixture proves
  decode/round-trip/render; the new `layer_effects/satin.rs` holds the kind.
  Ceilings: contour (`MpgS`)/anti-alias/global-light ignored, one `M` confinement
  (libpsd squares it), and the rounding differs from libpsd by ≤1 px.
- Layer effects — Bevel & Emboss (roadmap P3/G6, archived
  `2026-09-20-layer-effects-bevel`): the object-based `lfx2` **Bevel & Emboss**
  (`ebbl`) is now decoded and, for the **Inner + Smooth** slice, composited — a
  height field from the blurred content matte lit from `Angle`/`Altitude`
  produces highlight/shadow intensities (tinted by `hglM`/`hglC`/`hglO` and
  `sdwM`/`sdwC`/`sdwO`), confined to the content and composited above it; other
  styles/techniques decode but render a no-op; the GPU rejects an enabled+present
  renderable bevel (`UnsupportedLayerEffect`); a psd-tools-authored `bevel.psd`
  fixture proves decode/round-trip/render; the new `layer_effects/bevel.rs` holds
  the kind. Ceilings: chisel techniques, the Outer/Emboss/Pillow/Stroke styles,
  contour/texture, and the global-light resource.
- Effect blend-mode fix (cross-cutting, in `2026-09-20-layer-effects-bevel`): all
  `lfx2` effect blend modes (`Md  `, and a bevel's `hglM`/`sdwM`) now decode the
  `BlnM` descriptor vocabulary (`Nrml`/`Mltp`/`Scrn`/`Ovrl`/…) via a shared
  `effect_blend_mode`; previously they used the layer-key decoder, so real
  Photoshop effect modes silently fell back to defaults. This required
  regenerating the effect fixtures' goldens (`drop_shadow`, `outer_glow`,
  `inner_shadow`, `inner_glow`, `color`/`gradient`/`pattern_overlay`, `satin`;
  `bevel` is new) — non-effect fixtures are unchanged.
- Layer effects — legacy `lrFX` (roadmap P3/G6, archived
  `layer-effects-legacy-lrfx`): the legacy `lrFX` block (Photoshop 5.0–6.0; CS
  writes it for compatibility) is now decoded into the existing typed effect
  model (`DropShadow`/`InnerShadow`/`OuterGlow`/`InnerGlow`/`BevelEmboss`/
  `ColorOverlay`) in the new `layer_effects/legacy.rs`, and rendered through the
  shipped `lfx2` renderers. A single resolver `decode_layer_effects` prefers
  `lfx2` and falls back to `lrFX` — never both, so no effect is double-applied.
  `lrFX` uses the layer blend vocabulary (`BlendMode::from_psd_key`), unlike
  `lfx2`'s capitalized `BlnM`; gaps (technique/soften/altitude/knockout) take
  the typed defaults. The GPU rejects a renderable legacy effect through the same
  resolved set (`UnsupportedLayerEffect`). A psd-tools-authored
  `legacy_effects.psd` fixture proves decode, round-trip and render; new
  `layer_effects/legacy.rs` test module. Ceilings: only the classic `lrFX`
  record set (no satin/stroke/gradient/pattern record), contour/noise/anti-alias,
  and the shadow `blur` width ambiguity (psd-tools u32 vs libpsd u16+u16).
- Channel Mixer adjustment decode (roadmap P3/G8, archived
  `channel-mixer-adjustment-decode`): the `mixr` block now decodes into the
  existing `Adjustment::ChannelMixer` (no `pictura-adjust` change) and composites
  through the adjustment-layer path, and `pictura-render` exposes
  `encode_channel_mixer`. The layout is grounded on **ag-psd** because psd-tools'
  `ChannelMixer` reads only the red row; a committed `channel_mixer.psd` fixture
  (monochrome + non-monochrome) is proven by an ag-psd oracle plus a psd-tools
  partial check. The app gains a neutral-identity `channel-mixer` kind and
  Adjustments panel row. The adjustment self-test checks moved (pure) into
  `selftest_layers_adjustments.cpp`, where `lpr_channel_mixer` takes code 294.
  Ceilings: version 1 only; the non-monochrome gray row is read by ag-psd but
  ignored by the decoder; the two reserved bytes per channel are ignored; no
  Adobe pixel parity claim.
- Layer effects — Stroke gradient/pattern fills (roadmap P3/G6, archived
  `2026-09-20-layer-effects-stroke-fills`): the `lfx2` Stroke (`FrFX`) now
  decodes and composites **gradient** (`PntT` `GrFl`, `Grad`) and **pattern**
  (`PntT` `Ptrn`, `Ptrn`) fills in addition to solid, reusing the shipped overlay
  gradient/pattern samplers over the stroke band; the pattern link key is `Lnkd`
  (with `Algn` fallback); the GPU rejects a renderable gradient/pattern stroke;
  two psd-tools-authored fixtures `stroke_gradient.psd`/`stroke_pattern.psd`
  prove decode/render. Ceilings: the aligned-gradient edge clamp and the
  layer-rect pattern anchor are marked approximations (Photoshop's exact
  stroke-fill extent is ungrounded); the solid render path is unchanged.
- Curves adjustment decode (roadmap P3/G8, archived `curves-adjustment-decode`):
  `curv` now decodes/encodes into an extended `CurvesParams` (composite `points`
  plus optional per-channel `red`/`green`/`blue` curves), applied per-channel
  then composite. The layout is grounded on **ag-psd** with a psd-tools partial
  check; a committed `curves.psd` fixture proves decode. The legacy `is_map`
  bitmap form and the duplicate `Crv ` v4 section are ignored. The
  per-channel-then-composite order is a marked assumption (not
  Photoshop-verified). Ceilings: point counts `2..=14`; no pixel-parity claim.
- Selective Color adjustment decode (roadmap P3/G8, archived
  `selective-color-adjustment-decode`): `selc` now decodes/encodes into a new
  `Adjustment::SelectiveColor` (relative/absolute, nine ranges), implemented from
  libpsd's integer CMYK algorithm; the block layout is grounded three ways
  (libpsd plate 0 reserved + nine named ranges; ag-psd; psd-tools framing); a
  committed `selective_color.psd` fixture is proven by psd-tools + ag-psd
  oracles; the app gains a neutral `selective-color` kind/panel row. Ceilings:
  profile-free CMYK round-trip, the all-zero early-return deviation from libpsd,
  no Photoshop pixel parity, no GPU shader.
- **Color Lookup adjustment decode** (roadmap P3/G8, archived
  `2026-09-22-color-lookup-adjustment-decode`): `clrL` now decodes to a new
  `Adjustment::ColorLookup(ColorLookupParams)` (kind plus an optional parsed
  `Lut3d`), closing the whitelisted adjustment-key set. A `3DLUT` whose embedded
  `LUT3DFileData` is a `.CUBE` is parsed (`pictura_adjust::parse_cube`, red index
  fastest) and sampled trilinearly by a new `pictura_adjust::lut` kernel;
  abstract-profile, device-link, non-`.CUBE`, and malformed payloads are no-ops.
  `pictura-render` exposes `decode_color_lookup`/`encode_color_lookup`/
  `identity_cube`; the app maps kind `color-lookup` to an identity-cube layer
  (neutral composite) with an Adjustments-panel `Color Lookup` row (C++ self-test
  code 455). The committed `color_lookup.psd` fixture is proven by psd-tools and
  the ag-psd oracle. Ceilings (`// ponytail:`): `dataOrder`/`tableOrder` are
  metadata only, a non-default `DOMAIN_MIN`/`DOMAIN_MAX` is treated as `0..1`,
  and there is no Adobe pixel-parity claim. Remaining P3 adjustment key at the
  time: version-3 `phfl` (now shipped, below).
- **Version-3 Photo Filter decode** (roadmap P3/G8, change
  `phfl-v3-xyz-decode`): `decode_photo_filter` now accepts version 3 as well as
  version 2. The v3 payload is three big-endian `u32` CIE XYZ values, a `u32`
  density, and a `u8` luminosity flag (offsets 2/6/10, 14, 18 — layout grounded
  on psd-tools). XYZ is read as 16.16 fixed-point relative to D50 and converted
  with the same profile-free matrix as Lab document read
  (`pictura_codec::xyz_d50_to_srgb_u8`, shared with `lab_to_rgb`).
  `encode_photo_filter` stays version 2, so open→save of a v3 layer re-emits v2
  with the decoded colour. Truncated payloads, versions other than 2/3, and
  density `> 100` remain `None`. Ceilings (`ponytail:`): the 16.16 scale and
  D50 white are unproven without a CS6-authored v3 fixture; out-of-gamut
  components clip. This closes the last deferred adjustment key (G8); remaining
  P3 is the text (`TySh`) kind. Proven by unit tests
  (`phfl_decodes_version_three`, v2 still green, encoder round-trip); no
  app/UI change, no new dependency.
- **Type-tool model** (roadmap P3/G6, change `tysh-model-roundtrip`): a layer's
  preserved `TySh` now decodes into a derived `pictura_core::TypeTool` view
  (6×`f64` transform, `Txt ` string, four `i32` bounds, raw text/warp
  descriptor bytes). `encode_type_tool` rebuilds framing-only for an edited
  view; unmodified open→save still re-emits `extra_blocks` bytes. Malformed
  `TySh` leaves the view `None` without failing the document; kind detection
  stays presence-only (`type-layer-kind`). Ceilings (`ponytail:`): bounds are
  `i32` per psd-tools (Adobe table says "4 * 8"); EngineData styles/fonts are
  now decoded (`type-engine-data`); no glyph rasterization or Type tool.
  Proven by synthetic TySh unit tests (decode, encode round-trip, open→save,
  malformed). No app UI change, no new dependency.
- **EngineData text styles** (roadmap P3, change `type-engine-data`, archived):
  `pictura_codec::parse_engine_data` decodes the `Txt ` descriptor's `EngineData`
  (`tdta`) blob into a typed `EngineValue` tree (bounded depth/token/byte caps;
  UTF-16BE strings with escapes; MacRoman names; malformed → error, never a
  panic). `TypeTool.fonts` holds the font-set names and `TypeTool.style`
  (`pictura_core::TextStyle`: resolved font, size, fill colour, tracking,
  justification) the first run's effective values with the style/paragraph
  defaults applied. Proven by a real Photoshop-2021 text-layer EngineData
  fixture (`tests/fixtures/engine_data.bin`) and a psd-tools differential
  oracle, plus unit tests. Ceilings (`ponytail:`): first run only, no per-run
  layout or font-file resolution, EngineData2 (`TEXT_ENGINE_DATA`) not decoded,
  no glyph rasterization. No app UI change, no new dependency.
- **Text render seam** (roadmap P3, change `text-render-seam`, archived): the
  ADHD architecture decision for live text — the Qt-free engine owns a
  deterministic layout pass, and glyph rasterization sits behind a POD port.
  `pictura_core::layout_lines(lines, params) -> TextLayout` positions
  already-shaped glyphs (advance scaling by `size/units_per_em`, 1/1000-em
  tracking, leading, left/center/right alignment inside a `wrap_width`) with
  pure `f32` arithmetic and no font parsing; `TextAlign::from_justification`
  maps the EngineData paragraph byte. `RasterRequest`/`GlyphMask`/`trait
  Rasterizer` are the font- and Qt-free seam (a test backend ships);
  `FontPolicy` and `TextProvenance` (requested/resolved family, font content
  hash, backend+version) make substitution auditable. Ceilings (`ponytail:`):
  justification variants beyond 0/1/2 map to Left; automatic line-break/wrap
  is out of scope (the host supplies lines); no rasterizer backend, font, or
  pixels yet. Proven by layout/port/provenance unit tests. No app UI change,
  no new dependency.
- **Bundled text rasterizer** (roadmap P3, change `text-rasterize-bundled`,
  archived): `pictura-render` now depends on `fontdue` 0.9.4 and bundles
  `LiberationSans-Regular.ttf` (SIL OFL 1.1, license + provenance in
  `crates/pictura-render/assets/`). `BundledText::shape_line` maps chars to
  glyph ids + device-pixel advances; `BundledRasterizer` implements the core
  `Rasterizer` port via `rasterize_indexed` (guarded against the out-of-range
  panic); `BundledText::provenance` records the requested→Liberation-Sans
  substitution with a non-crypto font hash. `render_text_layer(doc, path)`
  shapes/lays out a type layer's `TypeTool` text+style, paints the glyph
  coverage in the fill colour into fresh `0/1/2/-1` channels, removes `TySh`,
  and clears `type_tool`; a refusal (no style, zero area, nothing painted)
  leaves the document unchanged. Ceilings (`ponytail:`): no GSUB/GPOS
  shaping/kerning, no warp/rotation, integer glyph placement ignores subpixel,
  the file name is a substitution. Proven by six render unit tests. The app
  `Rasterize Type` command and the optional Qt backend are deferred follow-ups.
  Adds the `fontdue` dependency (pure Rust, no C deps, keeps the engine
  Qt-free) and the bundled font asset.
- **Multichannel and Duotone read** (roadmap P4/G2/G3, change
  `multichannel-duotone-read`): header modes 7 and 8 now open. Duotone normalizes
  like grayscale, retains the plane and `color_mode_data` (the undocumented
  duotone spec), and an unchanged flat document saves back as mode 8.
  Multichannel opens only for header channel counts 1 or 3 at depth 8: 1 channel
  maps like grayscale; 3 channels map as profile-free CMY→RGB (`255-x`, marked
  ungrounded). Unchanged flat Multichannel saves back as mode 7 with retained
  plates; edited/layered falls back to RGB. Other channel counts stay
  `Unsupported`. Ceilings: Multichannel layer color ids are not converted;
  no plate layout or duotone curve is invented on edit.
- **Advanced-blending model** (roadmap G6/G7, change `knko-blend-if-model`):
  `knko`/`clbl`/`infx` consume into `Layer.knockout` / `blend_clipping` /
  `blend_interior` (defaults None/true/true; write only non-defaults) and
  `blending_ranges` parse into a typed `BlendIf` view (composite + per-channel
  black/white ranges, big-endian `u16`; empty/malformed → `None`, raw kept).
  `encode_blend_if` rebuilds the body for an edited view; unmodified open→save
  still re-emits raw ranges. Ceilings: Adobe labels `knko` a boolean while 0/1/2
  is accepted per psd-tools; the render is now in `blend-if-render`. Proven by
  seven codec unit tests (decode, defaults, round-trip,
  ranges, encode). No app UI, no new dependency.
- **Blend If compositing** (roadmap G6, change `blend-if-render`, archived):
  `composite.rs::blend_if_factor` gates a layer's CPU blend weight by its typed
  `BlendIf` view — the composite source range on the source pixel's gray, the
  composite destination range on the running backdrop's gray, and each
  per-channel group (`i < 3`) on that channel. A full `(0, 65535)` range is
  inactive, so a document without Blend If composites byte-identically. The GPU
  path declines a non-default layer (`GpuError::UnsupportedAdvancedBlending`)
  and falls back to the CPU oracle. Ceilings (`ponytail:`): Rec.601 composite
  gray and a hard 0/1 gate (no feather/split); channel groups assumed R,G,B in
  order; **knockout punch-through is not applied**; no Photoshop oracle. Proven
  by seven render unit tests plus the GPU parity suites (a real Vulkan device
  ran them).
- **Image-resource parsing** (roadmap P6/G5, archived
  `2026-09-22-psd-image-resources`): `pictura_codec::decode_image_resources`
  parses the preserved image-resource section into typed
  `ImageResource { id, name, data }` records and exposes the well-known ids
  (`ICC_PROFILE` 1039, `XMP_METADATA` 1060, `EXIF_DATA_1` 1058, `EXIF_DATA_3`
  1059, `IPTC_NAA` 1028). Parsing is tolerant: an unrecognized signature or a
  truncated block returns the records so far and never panics, and `write_psd`
  still re-emits the raw section byte-for-byte. A committed
  `image_resources.psd` (EXIF + XMP) is proven against psd-tools. Ceiling: the
  ICC profile is exposed but not yet applied (assign/convert and a save-side
  resource rewrite are the follow-up). No app change, no new dependency.
- **Embedded ICC profile applied on read** (roadmap P6/G5, archived
  `2026-09-22-psd-icc-convert`): an RGB document whose image resources carry a
  non-sRGB ICC profile (resource 1039) is converted to the sRGB working space on
  load — the composite and every layer's color channels, relative colorimetric —
  via a new `pictura-codec` dependency on `pictura-color` (lcms2). The original
  bytes are recorded in `Document.source_icc` and resource 1039 is dropped from
  the preserved resources so a save is not mis-tagged (`encode_image_resources`
  re-emits the rest byte-for-byte). A Grayscale document, an sRGB/undecodable
  profile, or one lcms2 cannot transform is left untouched. Ceilings
  (`// ponytail:`): "is sRGB" is a `Description`-contains-"srgb" heuristic, the
  intent is fixed at relative-colorimetric, and only RGB is converted. The
  fixture profile is synthesized (`psd_icc_rgb.icc` from `Profile::adobe_rgb()`;
  no Adobe file), proven against an independent PIL/lcms2 conversion, and the app
  shows a status "Converted from ICC profile …" notice. The C++ link now needs
  `-llcms2` (the staticlib does not propagate it). Still open: a File Info
  surface and a user-facing Assign/Convert Profile command.
- **Read-only File Info with decoded EXIF/IPTC/XMP** (roadmap P6/G5, archived
  `2026-09-22-psd-file-info`): `File > File Info…` (previously a disabled menu
  leaf) now opens a read-only dialog with Camera Data (EXIF), IPTC, and Raw Data
  (the raw XMP packet) categories. `pictura-codec` gains dependency-free
  `parse_exif` (TIFF/IFD walker: `Exif\0\0`-prefixed or bare TIFF, IFD0 + Exif
  sub-IFD, ASCII/SHORT/LONG/RATIONAL/UNDEFINED; a multi-value tag decodes as raw
  bytes), `parse_iptc` (IPTC-IIM `0x1C` record stream), and
  `read_metadata(&Document)` gathering resources 1058/1059, 1028, 1060. Defensive
  bounds: a per-IFD entry cap and a total cloned-value budget (a hostile resource
  cannot amplify memory), stop-and-return on malformed input, no panic. Proven by
  the byte-stable `metadata.psd` fixture against the independent `exiftool`
  decoder (self-skipping when absent) and a self-test that opens an inline
  EXIF+IPTC+XMP PSD and checks the dialog rows. Ceilings (`// ponytail:`): IFD0 +
  Exif sub-IFD only (no GPS), XMP is raw text (no field parsing), metadata is
  read-only, and the app decodes the three groups with separate bridge getters.
  Still open: editing/templates, XMP field extraction, sidecars, assign/convert
  profile.
- **IPTC editing and write-back** (roadmap P6/G5, archived
  `2026-09-22-psd-iptc-write`): the File Info dialog's IPTC page now edits the
  six core fields (Object Name, By-line, Copyright Notice, Caption/Abstract,
  Credit, Source); OK applies only the changed fields as one `File Info` undo
  state and marks the document dirty. `pictura-codec` gains
  `frame_image_resource` (the block framer), `Iptc::set`/`remove`,
  `encode_iptc`, and `set_iptc_fields(&mut Document, &[(u8,u8,String)])`, which
  re-frames resource 1028 and reassigns `Document.image_resources`. Safety:
  every other resource keeps its raw bytes, a section that does not decode
  losslessly is left untouched (no unframed bytes dropped), an IIM value longer
  than the `u16` length is truncated so no later record is lost, clearing an
  absent/empty field is a no-op, and unchanged fields record no undo state.
  Proven by `iptc_write_oracle.rs` (edit → `write_psd` → re-read + `exiftool`)
  and self-tests `file_info_metadata` / `file_info_iptc_edit` (456/457).
  Ceilings (`// ponytail:`): XMP is not edited (no IIM↔XMP sync), only the six
  core fields are editable, no EXIF editing or metadata templates, and IIM is
  written as UTF-8.
- **Assign / Convert Profile** (roadmap P6/G5, archived
  `2026-09-22-assign-convert-profile`): `Document.document_icc` holds the ICC
  bytes of the working profile the stored pixels are in (`None` = sRGB, so a
  freshly opened doc is unchanged and read-normalisation is untouched).
  `pictura-codec::assign_document_profile` retags only (rewrites
  `image_resources` with a framed 1039, preserving any unparsed tail), and
  `convert_document` transforms `doc.composite` and every layer's color channels
  (including group children) then retags; `buffer_to_srgb` converts the final
  composite to sRGB for display, so `doc.composite`/layer bytes stay in document
  space and a save emits a consistent 1039. The app wires `Edit > Assign
  Profile…` / `Edit > Convert to Profile…` (`profile_dialog`, bridge
  `assign_profile`/`convert_profile`), each one undo step, via the existing
  whole-document history. Proven by codec unit tests, `icc_oracle` /
  `profile_assignment_oracle` (lcms2 + 1039 bytes), bridge replay tests in
  `tests_impl.rs`, and self-test 458 (`profile_assign_convert`). Ceilings
  (`// ponytail:`): RGB 8-bit only, three built-in profiles only, fixed
  relative-colorimetric intent (no BPC/dither/flatten), and no Color Settings
  policy layer (`WF-011`).
- **XMP parse and edit** (roadmap P6/G5, archived `2026-09-22-xmp-metadata`):
  `pictura-codec/src/xmp.rs` parses resource 1060 into a fixed typed property
  set (title, creator, description, subject, rights, credit, source, headline,
  marked) via a bounded, entity-free, non-recursive scanner, and
  `patch_xmp` edits a property by byte-span, copying every unmanaged byte
  verbatim (unknown namespaces/properties/wrapper); a packet it cannot safely
  rewrite, or a managed property wrapping a comment, fails closed.
  `set_xmp_fields` frames 1060 (creating a minimal packet when absent) through
  the same lossless-prefix + preserved-tail pattern as `set_iptc_fields`, and
  `set_file_info_fields` writes the six IPTC-Core fields to BOTH XMP and IIM so
  the channels agree (scalar = `x-default`/first item governs the no-op; a
  non-primary alternative collapses — ceiling). File Info gains a read-only
  Description category; editing the IPTC page syncs XMP+IIM as one `File Info`
  undo state. Proven by codec unit tests, `metadata_oracle` (exiftool, ran) and
  self-tests 456/457/459. Ceilings (`// ponytail:`): nine managed properties
  only, raw packet read-only, EXIF not editable.
- **Metadata templates** (roadmap P6/G5, archived `2026-09-22-metadata-templates`):
  `pictura-codec/src/xmp/serialize.rs` serializes the nine managed properties to
  a standalone packet and `metadata/template.rs` adds
  `MergeMode { Append, Replace, KeepOriginalReplaceMatching }`, `export_template`
  (managed fields only) and `apply_template` (patches in place; Append fills
  empty only, Replace overwrites and clears template-omitted fields, Keep
  Original overwrites only template-defined fields). Lists merge as full
  `rdf:Seq`/`rdf:Bag` via `patch_xmp_values`; the six shared fields route through
  `set_file_info_fields` so XMP and IIM agree. File Info gains Export/Apply
  Template controls + a mode combo (dialog stays open; callbacks call the bridge
  `export_metadata_template`/`apply_metadata_template`), apply = one
  `Metadata Template` undo state, export = no mutation. Proven by codec unit
  tests, `metadata_oracle` (exiftool, ran, all three modes), self-test 460.
  Ceilings (`// ponytail:`): nine managed properties only, no sidecars/template
  folder/batch, `xmp.rs` is near the 1200-line cap.
- **PSD ZIP write and compression preservation** (roadmap P3/G12, archived
  `2026-09-22-psd-zip-write`): `pictura-core::Compression` (`Rle` default, `Raw`,
  `Zip`, `ZipPrediction`) and `Document.{composite,layer}_compression` record the
  kind read; `write_psd` emits it via `zip_scanlines` (zlib, with a reversible
  per-row delta for ZIP-prediction) and `channel_stream`, for the composite,
  document extra channels, layer color channels, and raster mask. A constructed
  document stays RLE and byte-unchanged; a document with no merged composite now
  writes no image-data section, so a maximize-compatibility-off file round-trips
  equal; only surviving layer records set the layer kind. Proven by codec unit
  tests and the psd-tools write oracle (ran) across raw/RLE/ZIP/ZIP-prediction.
  Ceilings (`// ponytail:`): mixed per-channel kinds within a category normalize
  to the first seen; no user compression choice, no per-channel fidelity.
- **Color Settings / incoming-profile policy** (roadmap P6/G5, archived
  `2026-09-22-color-settings`): `pictura_color::Policy { Preserve (default),
  Convert, Off }` (codes 0/1/2) and `pictura_codec::read_psd_with(bytes, policy)`
  decide how an RGB document's decodable non-sRGB embedded profile is honoured:
  Preserve keeps the pixels + resource 1039 and sets `Document.document_icc` (the
  canvas converts it for display via `buffer_to_srgb`); Convert is the old
  normalisation; Off drops 1039 and stays untagged. An untransformable profile
  (e.g. CMYK on a normalised-RGB doc) is unchanged under every policy.
  `read_psd` stays `Convert` for existing callers (incl. nested smart-object
  reads). The app persists the policy as an application preference (default
  Preserve), pushes it to the view before opening, displays the first frame
  through the profile conversion, and `Edit > Color Settings…` is a real dialog
  (sRGB working space, policy combo). Proven by codec unit tests + `icc_oracle`
  (PIL, ran) and self-tests 461/462. Ceilings (`// ponytail:`): sRGB working
  space only, RGB policy only, no `.csf`, no mismatch/missing dialogs.
- **Agentic control server** (archived `2026-09-22-app-control-server`, plan
  `docs/dev/mcp-agentic-control-plan.md` P0/P1): `./build/pictura --control
  [--control-socket PATH] [--state-home DIR]` starts a `QLocalServer` on a
  per-user Unix socket (owner-only, default
  `$XDG_RUNTIME_DIR/pictura-control.sock`) speaking newline-delimited JSON on
  the GUI thread. Methods `status`, `get_pixel`, `list_layers`, `list_commands`,
  `dispatch_command`, `document`, `edit`, `set_unsaved_policy`; errors
  `bad_request`/`unknown_method`/`invalid_param`/`no_document`/`not_implemented`/
  `refused`/`io_error`/`internal`. `CommandRegistry::describe()` added. Control
  mode is non-interactive by construction: unsaved policy is non-interactive/
  Discard, `document` uses the non-dialog entry points (a pathless `save` →
  `invalid_param`), and `dispatch_command` refuses modal-opening commands (a
  hand-maintained denylist, `// ponytail:`) with `refused`, so no request blocks
  on a modal. Socket is owner-only (Qt mode 0700), no TCP, no eval, oversize
  requests rejected. Proven by the `mcp_control` self-test block (codes 463+).
  **Vision methods** shipped (archived `2026-09-23-agentic-control-vision`):
  `screenshot` (`scope` window/canvas/document, `max_dim` default 1280, PNG as
  base64 with source dimensions), `ui_tree` (capped widget tree with
  window-local rects, `max_depth` default 12, `max_children` 64), and
  `layer_thumbnail` (`index`, `size` default 64). Caller-supplied image sizes are
  capped at the trust boundary (thumbnails 1024, screenshots 4096) because the
  engine allocates `size²` and `max_dim` would otherwise bypass the response-size
  guard; the vision self-test block is codes 483–492. **Engine actions**
  shipped (archived `2026-09-23-agentic-control-actions`): `set_tool` (a
  string→`ToolId` lookup accepting the `status` name or label), `selection`
  (all/deselect/rect/ellipse/lasso/quick/wand → `{has_selection,count,bounds}`),
  `filter`/`adjustment` (kind-only, fixed seed; unknown → `invalid_param`,
  locked/hidden target → `refused`), `layer_op` (name/opacity/visible/blend/
  fill/lock/color/move/translate/delete/duplicate/add → `{ok,layers}`), and
  `set_gpu_compute`; action self-test block codes 493–517. **Input synthesis**
  shipped (archived `2026-09-23-agentic-control-input`): `pointer`
  (click/dblclick/move/drag/scroll in `window` or `image` space; image
  coordinates map as the exact inverse of `ImageView::widgetToImage`,
  `w = image*zoom + offset`) and `key` (a sequence such as `Ctrl+Z`/`Shift+F2`
  parsed to a modifier set and key, sent as a `QKeyEvent` press/release to the
  focused widget else the frame), both via `QApplication::sendEvent` with no
  `Qt6::Test`/private API. Qt 6.11 routes a non-spontaneous key press through the
  shortcut machinery (a `ShortcutOverride` is sent first), so a shortcut fires in
  an active window and a key no shortcut claims reaches the widget handler; a
  window-scoped shortcut matches only while the window is active, so the
  in-process self-test (pre-activation) asserts the widget path with the Move
  tool's arrow nudge and the shortcut path is left to the live `verify-control.sh`
  (P5). Input self-test codes 522–524. **Live e2e** (archived
  `2026-09-23-agentic-control-e2e`): `scripts/verify-control.sh` launches a
  throwaway `pictura --headless --control` on a temp socket, drives
  status→rect-select→`add-noise`→a 64-pixel diff (changed inside the selection,
  none outside)→undo→screenshot over the real newline-JSON protocol, and exits
  non-zero on a mismatch or a missing binary; a trap kills the process and
  removes the socket. It is the only check that exercises
  `QLocalServer`/`QLocalSocket` and a separate process (the in-process block
  calls `dispatch` directly); a subset-of-the-active-layer selection makes a
  mask-ignoring filter fail. Wired into `verify-full.sh` after the build and into
  the CI `qt-headless` job; `verify-fast.sh` stays build-free. Ceilings:
  `pictura-mcp` stdio frontend (P4), the P5 client registration/long-form docs,
  native/global shortcuts, and `filter` caller seed/params remain.
- **Bit-depth preservation** (roadmap P4/G4, archived `2026-09-23-depth-preserve`):
  a 16/32-bit **Grayscale or RGB** document no longer downgrades to 8-bit on
  save. `read_psd` retains the decoded source-depth samples of the composite
  color planes, document extra channels, and every layer channel (stored on
  `Document.source_planes` / `Layer.source_channels`, keyed by channel id in
  canonical order); the 8-bit engine model is unchanged. `write_psd` writes the
  header at the source depth and, per plane, re-encodes the retained samples at
  the recorded compression when the plane is unchanged (its narrowing still
  equals its 8-bit bytes) and widens the 8-bit plane (`v*257` at 16, scaled to
  `[0,1]` at 32) when it changed. Compression is depth-aware: PackBits stays
  byte-wise with a native row stride, ZIP-with-prediction uses the depth-specific
  forward predictor (inverse of the read-side undo). An 8-bit or constructed
  document is byte-identical; a mode the read path converts (CMYK/Lab) still
  saves 8-bit (the app notice reports that case); the retained copy is dropped on
  a scale/rotate/flip (`// ponytail:` an unmodeled channel cannot be resampled)
  and kept on a pure translation. Proven by the `psd-tools` write oracle
  (composite + layer channels, all four compressions, both depths, and a grouped
  document whole-`Document`-equal). Ceilings: editing stays 8-bit (a true
  `u16`/`f32` sample model), no HDR tone map, retained samples cost 2×/4× while
  open.
- **Lab write-back** (roadmap P4/G2, archived `2026-09-23-color-mode-write-back`):
  an 8-bit Lab PSD no longer converts to RGB on save. `read_psd` retains the
  pre-normalization Lab color planes (composite and every layer color channel,
  recursing into groups) in the same store as `depth-preserve`; `write_psd`
  writes header mode Lab, re-emitting the retained planes byte-for-byte when
  unchanged (an 8-bit Lab re-encode would drift saturated colors up to ~19 LSB)
  and converting the working RGB with a profile-free `rgb_to_lab` inverse when
  edited (approximate, `// ponytail:`). RGB/Grayscale/constructed documents are
  unchanged; CMYK/Bitmap/Indexed and a 16/32-bit Lab source still save the
  working mode (the app notice reports Lab only for an 8-bit Lab source). The
  change also fixed a pre-existing read bug: the color-mode normalization now
  recurses into layer groups, so nested layers in a grouped Lab/CMYK/Bitmap/
  Indexed document convert correctly. Proven by the `color_mode_oracle`
  (lcms2/psd-tools, ran) and grouped/16-bit unit tests. Ceilings: CMYK/
  Bitmap/Indexed write-back (CMYK has no profile-free inverse; needs a
  color-management decision), Multichannel/Duotone, and an edited Lab plane's
  approximation.
- **CMYK write-back** (roadmap P4/G2, archived `2026-09-23-cmyk-write-back`): an
  8-bit CMYK PSD now saves back as CMYK (header mode 4, four color planes).
  `read_psd` retains the pre-normalization CMYK color planes (composite and every
  layer, recursing groups) in the same store; `write_psd` re-emits them
  byte-for-byte when unchanged, else converts the working RGB with `rgb_to_cmyk`,
  an **exact** right-inverse of `cmyk_to_rgb` (no-black `K=255`), so an edited
  pixel reads back to exactly the edited RGB. The 4-channel layout adds the extra
  color channel to the header count, the document-extra offset (`4 + i`), and the
  synthesized per-layer `(C,M,Y,K)` channels. Bitmap, Indexed, and a 16/32-bit
  CMYK source still save the working mode; the app notice reports a
  CMYK-preserving save only for an 8-bit source. Proven by the `color_mode_oracle`
  (mode 4, four channels, retained planes byte-identical; lcms2/psd-tools ran) and
  unit tests (edited plane, layer, extra channel). Ceilings: an edited CMYK pixel
  uses a fixed no-black convention, not color management; a 16/32-bit CMYK source
  still writes RGB (now untagged — see the ICC consistency bullet); Bitmap
  write-back and Multichannel/Duotone remain open.
- **Indexed write-back** (roadmap P4/G2/G3, archived
  `2026-09-23-indexed-write-back`): an **unchanged** 8-bit Indexed PSD now saves
  back as Indexed (header mode 2, one index channel, the 768-byte palette).
  `read_psd` retains the palette in a new `Document.source_palette` and each
  index plane (the composite's and every pixel layer's, recursing groups) in the
  existing source store, while `color_mode_data` stays cleared (the "palette is
  consumed" contract is unchanged). `write_psd` emits Indexed only when every
  retained plane still expands through the palette to the current working RGB;
  the header mode is document-wide, so an edited composite or layer, or an added
  color layer, falls back to writing the working RGB. No RGB-to-palette
  quantization is invented (a no-oracle path, like the Lab/CMYK edits). A file
  with no merged composite still writes Indexed (palette + layer index channels,
  no image-data section). The mode notice reports an Indexed-preserving save;
  ceiling: it does not consult the edit, so an edited Indexed document still
  claims "saved as Indexed". Proven by the `color_mode_oracle` (mode 2, palette
  and index planes byte-identical; psd-tools ran) and unit tests (grouped layer,
  raster mask, edited composite/layer fallback, no-merged-composite, added empty
  layer, malformed-input typed error). Ceiling: Bitmap write-back and
  Multichannel/Duotone remain open.
- **Bitmap write-back** (roadmap P4/G2, archived `2026-09-23-bitmap-write-back`):
  a **flat, unchanged** depth-1 Bitmap PSD now saves back as Bitmap (header mode
  0, depth 1, one 1-bit channel) at its source compression (Raw or RLE). `read`
  retains the raw packed composite plane (store depth `BitDepth::One`), and
  `write_psd` borrows it directly (bypassing the `width*height` widening path).
  `depth_of(One)` now yields 1 so the writer finds the packed plane; `depth_bits(1)`
  stays unmapped so a Bitmap read's `source_depth` is unchanged, and
  `output_depth` only honors a 16/32-bit source depth (an inconsistent hand-built
  depth-1 store is a typed error, so `write_psd` never panics). Scope is the flat
  case: an edited Bitmap, a layered Bitmap, or one with an extra channel writes
  the working RGB (a per-channel packed store plus a depth-1 layer path is the
  ceiling; no RGB→1-bit threshold is invented). The mode notice reports a
  Bitmap-preserving save; ceiling: the edit and the flat-only fallback are not
  consulted. Proven by the `color_mode_oracle` (Raw and RLE round trips, packed
  plane byte-identical, psd-tools mode 0) and unit tests (edited/layered/
  extra-channel fallback, typed-error guard). Ceiling: Multichannel/Duotone
  remain open (no natural RGB mapping / spot-ink spec).
- **ICC profile matches the output mode** (roadmap P4/P6, archived
  `2026-09-23-icc-output-mode-consistency`): a 16/32-bit CMYK or Lab source is
  normalized to RGB but keeps its CMYK/Lab resource `1039` (the profile cannot
  build an RGB transform, so both the Convert and Preserve read paths leave it).
  `write_psd` now drops a framable `1039` whose ICC data-space signature (header
  bytes 16..20) does not match the output header color mode (`RGB `/`GRAY`/`CMYK`/
  `Lab `), at the single resource-emit choke point, so the saved file is never
  mis-tagged. A matching profile, a profile too short to carry the signature,
  every other resource, and the unparsed tail re-emit byte-for-byte; the section
  is returned unchanged when nothing mismatches, so existing files write
  identically. The read-time `Preserve` policy is untouched. Tests cover a
  read→write 16-bit CMYK and Lab source (Convert and Preserve), Grayscale keeping
  its `GRAY` profile, and the CMYK/RGB cases. Ceiling: a `1039` the parser cannot
  frame (behind an unrecognized signature) is preserved, not filtered. The later
  `depth-color-mode-write-back` change makes a 16/32-bit CMYK/Lab save in its
  source mode, so that path's profile now matches and is kept; the guard remains
  the general invariant against any mode/profile mismatch.
- **16/32-bit CMYK/Lab write-back** (roadmap P4/G2/G4, archived
  `2026-09-23-depth-color-mode-write-back`): a 16/32-bit CMYK or Lab PSD now saves
  back in its source color mode **and** source depth; before, it saved 8-bit RGB
  (both lost). `read_psd` retains the native Lab/CMYK color planes (composite and
  every layer; the native layer store already existed) and `write_psd` re-emits an
  unchanged plane byte-identically, else re-encodes the working RGB with the
  profile-free 8-bit inverse and widens (Lab approximate, CMYK exact). The
  unchanged test runs in the narrowed-retained domain: two helpers
  (`composite_retained_8`/`layer_retained_8` in `color_mode.rs`, since `write.rs`
  is at the size cap) narrow the retained native plane before comparison, because
  comparing it to `rgb_to_lab`/`rgb_to_cmyk` fails (Lab ±1 LSB, `rgb_to_cmyk`
  forces `K=255`). A short retained plane is a typed error, not a panic. The app
  notice keeps the mode-preserving wording when the depth is retained. Proven by
  the `color_mode_oracle` (16- and 32-bit CMYK/Lab, psd-tools mode 4/9 at source
  depth) and `tests/depth.rs` (composite + layered Lab/CMYK byte-identity, edited
  widening, typed-error guard). Ceilings: editing stays 8-bit (a true `u16`/`f32`
  sample model); a mode with no retained native store (8-bit Lab/CMYK) and
  Indexed/Bitmap save as before; Multichannel/Duotone remain open.
- `vmsk` vector masks (roadmap P3, archived `vector-mask-render`): now decode
  into a derived `Layer.vector_mask` view (raw block preserved and re-emitted)
  and clip the layer through `mask_alpha`, combined with the raster mask by
  multiplication and honoring the invert/disable flags. Fill rule is even-odd by
  default; non-zero is read only from the ag-psd marker. A committed
  `vector_mask.psd` fixture is proven by psd-tools + ag-psd oracles and GPU
  parity; new `vector.rs` in `pictura-core`, `vector_mask.rs` in
  `pictura-codec` and `pictura-render`. Ceilings: open paths / `vscg` / `vsms`
  deferred, non-union operations collapsed, no AA / Feather / Density, layer
  effects gated with content (Photoshop's "Vector Mask Hides Effects" off by
  default is a documented divergence), no resize transform, fixed 16-segment
  flattening, no Photoshop pixel parity.
- `vscg` vector fill content (roadmap P3, archived `vector-fill-content`): now
  decodes and renders as a generative fill clipped by the layer's vector mask (or
  layer bounds), reusing the shipped `SoCo`/`GdFl`/`PtFl` decoders (the fill
  descriptor's classID is `null`; `solidColorLayer`/`gradientFillLayer`/
  `patternFillLayer` are the stroke content, not the fill); the adjustment-block
  fill takes strict precedence (an undecodable block is a no-op regardless of
  `vscg`); a committed `vector_fill.psd` fixture is proven by psd-tools + ag-psd
  oracles; a channel-less vector layer falls back to CPU on the GPU. Ceilings:
  vector stroke (`vstk`) deferred, `vogk`/`vsms` deferred, noise gradients,
  boolean ops beyond union, no AA, no authoring/UI, rasterize stays
  adjustment-block-based.
- **16/32-bit depth read** (roadmap P4/G4, archived `depth-read`): `read_psd`
  now opens 16-bit and 32-bit Grayscale/RGB/CMYK/Lab documents and **normalizes
  them to 8-bit on load** — `source_depth` records the original
  (`Some(Sixteen)` / `Some(ThirtyTwo)`), the written document is uniformly
  8-bit (`write_psd` keeps writing 8-bit, a documented lossy-in-depth save), and
  the app shows a status-bar "Converted from 16-bit"/"…32-bit" notice. The
  narrowings match psd-tools: 16→8 is `sample >> 8`, 32→8 is
  `clamp(trunc(sample * 256), 0, 255)`; the row stride, RLE byte counts, and
  ZIP-with-prediction are depth-aware (16 a per-`u16` running sum mod 2^16; 32 a
  four-byte-plane unshuffle then byte-wise delta). **Every** channel is narrowed,
  including layer color/`-1`/`-2`, unmodeled spot/selection channels (re-wrapped
  as an 8-bit raw stream instead of re-emitting source-depth bytes), and
  document-level extra channels. Bitmap/Indexed at 16/32 stay unsupported. New
  flat fixtures `rgb16.psd`/`rgb32.psd` are proven by a psd-tools oracle (0
  tolerance); `scripts/generate-fixtures.py` is byte-stable and no existing
  fixture/golden changed. Ceilings (all `ponytail:`): no true 16-bit sample model
  (depth is not preserved), the 32-bit path is display-referred so HDR clips at
  1.0 with no tone map, and there is no write-side re-encode to the source depth.

## Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace          # preferred; cargo test --workspace is the fallback
cargo test --workspace --doc           # doctests (nextest does not run them)
cmake -S . -B build -G Ninja -DCMAKE_EXE_LINKER_FLAGS=-fuse-ld=lld && cmake --build build --parallel
./build/pictura --headless --self-test
./build/pictura --headless --self-test crates/pictura-codec/tests/fixtures/two_layers.psd
bash scripts/guard.sh
openspec validate --all --strict
```

## Code health (LOC guardrail)

Every source file is under the **LOC hard cap** (code 1200, tests 1400; target
<800; `AGENTS.md` rule). `scripts/check-file-size.sh` is the guard, `scripts/file-size-allowlist.txt`
the exception list, and `scripts/verify-fast.sh` runs it. A completed
code-splitting pass (`docs/dev/refactor-code-splitting.md`) took the sixteen
over-cap files down to one by pure moves — build, byte-identical self-test
stderr, the Rust suite, and the specs green throughout:

- **C++ app TUs** — `panels/panel_group.cpp` → `panel_group{,_menu,_test}.cpp`;
  `panels/layers_panel.cpp` → `layers_panel{,_actions,_menu,_test}.cpp` +
  `layers_panel_internal.h`; `panels/panel_column.cpp` →
  `panel_column{,_drag,_iconic,_menu,_test}.cpp` + `panel_float.cpp` +
  `panel_column_internal.h`; `frame.cpp` →
  `frame{,_columns,_session,_menus,_build,_test}.cpp`. New TUs are registered in
  `CMakeLists.txt`.
- **Self-test** — the `--self-test` block moved out of `main.cpp` into
  `selftest.cpp` / `selftest.h` (`int runSelfTest(QApplication&, bool, const
  QString&, PicturaMainWindow&, PictureView*, const QImage&, bool, int)`).
  `main.cpp` is now the startup path only (7320 → 180 LOC) that calls
  `runSelfTest`; the self-test lines live in `selftest.cpp`.
- **cxx-qt bridge** — `cxxqt_object.rs` is a Rust-2018 root that keeps the
  cxx-qt bridge and shared private helpers, with concern submodules under
  `src/cxxqt_object/`: `impl_{core,layers,selection,transform,paint,history,filters}.rs`,
  `helpers.rs`, `helpers_composite.rs`, `tests.rs`, `tests_impl.rs`. The root +
  directory layout keeps the generated-header path and the 14 C++ includes
  untouched.
- **Engine crates** — `pictura-filters` (`artistic/filters.rs` →
  `artistic/filters/{effects,brush,common,tests}.rs`; `lib.rs` → `filter.rs`),
  `pictura-render` (`lib.rs` → `composite.rs` + `tests/`; `gpu.rs` →
  `gpu/{mod,backend,shader}.rs`; `gpu_filter.rs` →
  `gpu_filter/{mod,plan,resources}.rs`; `document_ops/layer_ops.rs` →
  `document_ops/layer_ops/{mod,create,paths,properties,tests}.rs`),
  `pictura-adjust` (`lib.rs` → `types/common/apply/tonal/color/auto/tests.rs`),
  `pictura-codec` (`lib.rs` → `error/common/read/write/tests.rs`), and the
  integration tests (`tests/oracle.rs` → `tests/oracle/`;
  `tests/gpu_parity.rs` → `tests/gpu_parity/`).

One file remains allowlisted: `crates/pictura-app/cpp/selftest.cpp` (7212 LOC).
It was extracted whole first to protect the verification oracle; subdividing it
by self-test section is a deliberate later step, out of this pass.

`crates/pictura-codec/tests/oracle.rs` is at **1398 LOC**, just under the 1400
test-LOC cap, and must be split (the `tests/oracle/` submodule directory
already exists) before any further oracle growth.

## Crates

| Crate | Responsibility |
|---|---|
| `pictura-core` | Document/Layer/Channel/Mask/BlendMode(27+pass)/AdjustmentData; no deps |
| `pictura-codec` | PSD/PSB read/write: composite + layer channels raw/RLE/ZIP/ZIP-prediction, layers, masks, adjustment keys, document channels; unknown blend key degrades to Normal; absent composite tolerated |
| `pictura-color` | ICC profiles (sRGB/AdobeRGB/ProPhoto), convert/assign, intents, BPC |
| `pictura-adjust` | 15 destructive adjustments (`apply`) |
| `pictura-filters` | blur/sharpen/noise + stylize/other + pixelate + distort + render filters (`Filter` + `apply`); seeded filters; `artistic` module (15 CS6 Artistic filters with shared `reduce`/`noise`/`texture` helpers) + the four remaining families — Brush Strokes, Sketch, Texture, Oil Paint (29 filters, same shared helpers) |
| `pictura-select` | selection coverage mask, boolean/modify ops, wand, color range; `Selection::{rect,ellipse,polygon}` rasterizers + `CombineMode`/`combine_with` |
| `pictura-ops` | image resize (Nearest/Bilinear/Bicubic), canvas size (9 anchors), rotate/flip + arbitrary rotation; ImageMagick oracle |
| `pictura-render` | CPU compositor (27 blend modes, groups, masks, adjustment layers) + GPU compositor (default backend: 26 GPU modes + 5 adjustment layers, `Backend`/`composite_active`/`composite_region_active` (dirty-rect composite, byte-identical to the sub-rect); GPU-native 8-bit data path; 2-D compute dispatch so large documents like 4000² composite on the GPU instead of falling back to the CPU above the old ~4.19 MP 1-D workgroup ceiling) + GPU filter path (`filter_gpu_available`/`apply_filter_active`, byte-exact CPU-parity kernels — the nine convolution kernels plus the M28 heavy window/effect kernels Surface Blur, Maximum, Minimum, Median, Custom 5×5, Oil Paint; same 2-D dispatch for >4.19 MP) + PSD adjustment encode/decode + `apply_filter` (layer filter gated by mask, GPU-accelerated when `gpu_enabled`) + `document_ops` (document resize/canvas/orientation/crop/layer-translate; `translate_layer_active` composites through the active backend, `translate_layer_rect` shifts a layer rect (and mask) without recompute, CPU `translate_layer`/`recompute` remain the oracle; re-exports `Anchor`/`Resample`) |
| `pictura-testkit` | golden compare/hash + `pictura-diff` CLI |
| `pictura-paint` | dab-splatting brush/pencil stroke engine — tip coverage, spacing, flow/opacity, paint modes; depends on `pictura-core` |
| `pictura-app` | cxx-qt `PictureView` QObject + Qt C++ shell: `commands` (command registry + full documented CS6 menu tree), `frame` (`PicturaMainWindow`: menu bar, tabbed document area with a `PictureView`+`ImageView` per document, file lifecycle New/Open/Save/Save As/Revert/Close/Close All/Exit, status bar, docks, screen modes), `theme` (Fusion dark palette, 4 brightness levels), `session` (XDG state store), real Layers/History/Navigator/Color/Swatches/Info/Histogram panel docks replacing the debug dock (backed by the document and history models, with a frame-owned `ColorState` fed by the Eyedropper), tool layer (`tools`/`toolbox`/`options_bar` — Move/Marquee/Lasso/Quick Selection/Crop/Eyedropper/Hand/Zoom/Brush/Pencil), paint bridge (`begin_paint`/`paint_dab`/`end_paint`/`cancel_paint`/`is_painting`) with a live paint options bar, zoom/pan, GPU demo |

## Milestones done

- **M0/M0.5** — cxx-qt↔Qt6 walking skeleton, PSD codec, harness, CI, GPU spike.
  GPU: offscreen wgpu→readback→QImage works; QRhi imports the wgpu VkDevice/image;
  on-screen present via `QRhiWidget` is blocked (`crates/pictura-app/GPU-INTEROP-NOTES.md`).
- **M1** — layer/channel/mask model + layered PSD round-trip (+psd-tools oracle).
- **M2/M2.5** — CPU compositor (27 modes) + ImageMagick oracle; GPU compositor
  (22 modes, Δ0); app renders the layer stack.
- **M3** — ICC color management (+ ImageMagick oracle).
- **M4 (A/B/C)** — 15 adjustments; adjustment layers (decoded: `nvrt`,`post`,
  `thrs`,`brit`,`levl`,`hue2`; others preserved-only); app adjustment UI.
- **M5 (A/B/C)** — selection math (+ IM morphology oracle); selection↔PSD
  channels; selection-masked adjustments in the app.
- **M6** — `pictura-filters`: blur (Gaussian/Box/Motion/Radial/Average/
  Blur(+More)/Surface), sharpen (Sharpen(+More)/Edges/Unsharp Mask), noise
  (Add Noise seeded/Median/Despeckle). ImageMagick oracle: Gaussian/Box/Median
  exact, USM ±6; Motion classified no-equivalent (IM kernel is one-sided). 47
  filter tests. OpenSpec change `m6-filters`, tasks checked.
- **M6-C** — filter integration: `pictura-render::apply_filter(layer, filter,
  mask)` (destructive, selection/mask-confined, alpha + layer meta preserved) and
  the app `apply_filter(kind)` command + dock control. Headless self-test proves
  confinement (`changed_inside=12 changed_outside=0`). OpenSpec change
  `m6-filter-integration`, tasks checked.
- **M7** — Stylize + Other filters: `Maximum`, `Minimum`, `Offset`, `High Pass`,
  `Custom` (Other); `Emboss`, `Find Edges`, `Solarize` (Stylize). ImageMagick
  oracle exact (Δ0) for Maximum/Minimum/Offset-wrap/Custom/Solarize; Emboss,
  Find Edges, High Pass, Offset-fill classified no-equivalent. App filter kinds
  added. OpenSpec change `m7-stylize-other`, tasks checked.
- **M8** — Pixelate filters: `Mosaic`, `Crystallize`, `Facet`, `Fragment`,
  `Mezzotint`, `Pointillize`, `Color Halftone`. Oracle: Mosaic exact (block
  average when the cell divides the dimensions); the other six no-equivalent
  with recorded deltas; seeded filters reproducible. App filter kinds added.
  OpenSpec change `m8-pixelate`, tasks checked.
- **M9** — Distort warps: `Twirl`, `Pinch`, `Spherize`, `Ripple`, `Wave`
  (single-image inverse-mapping with bilinear resampling; alpha untouched; Wave
  seeded, `repeat_edge` selectable). All five classified no-equivalent against
  the closest ImageMagick operator (Twirl closest at Δ124 vs `-swirl 45`);
  measured deltas recorded. App filter kinds added. OpenSpec change
  `m9-distort`, tasks checked.
- **M10** — `pictura-ops` image operations: `resize` (Nearest/Bilinear/Bicubic),
  `resize_canvas` (9 anchors, grow/shrink), orientation (`rotate90_cw/ccw`,
  `rotate180`, flips, `rotate_arbitrary`). ImageMagick oracle: Nearest and the
  exact right-angle rotations/flips and 3-channel canvas are exact (Δ0); Bicubic
  matches `-filter catrom` (tol 1; `cubic` is a B-spline); Bilinear is exact on
  upscale but no-equivalent downscale; `rotate_arbitrary` measured on the central
  region (max 8 at 30°, 45° no-equivalent). OpenSpec change `m10-image-ops`,
  tasks checked; app/document integration deferred.
- **M11** — Distort filters, part 2: `PolarCoordinates`, `Shear`, `ZigZag`,
  `OceanRipple` (inverse-mapping warps with bilinear resampling; alpha
  untouched; Shear `fill` selectable, Ocean Ripple seeded). All nine Distort
  filters are now implemented, and all four new ones are classified
  **no-equivalent** against their closest ImageMagick operators with measured
  deltas (Polar Δ189/58 and 194/59 vs `-distort Polar`/`DePolar`; Shear Δ255/18
  vs `-shear`; ZigZag Δ170/14 vs `-swirl`; Ocean Ripple Δ227/54 vs `-wave`).
  Shear's curve range check (`x`/`y` in `-1..=1`) added. App filter kinds added.
  OpenSpec change `m11-distort2` MODIFIED the canonical `distort-filters` spec
  (7 MODIFIED + 4 ADDED); tasks checked.
- **M12** — Document operations: document-level resize/canvas/rotate/flip in
  `pictura-render`; reuses `pictura-ops`; structural oracle via psd-tools +
  composite consistency + exactness identities. OpenSpec change
  `m12-document-ops`, archived.
- **M13** — Image ops app UI: `PictureView` commands `resize_image` /
  `resize_canvas` / `rotate_doc` / `flip_doc` (validate → doc op → clear
  selection → recomposite), dock "Image" section (spin boxes + resample and
  9-anchor combos + orientation buttons), and self-test doc-op checks
  (exact remap + rejection + selection-clear assertions, exit codes 19–21).
  `pictura-render` re-exports `Anchor`/`Resample`. OpenSpec change
  `m13-image-ops-ui` (capability `image-ops-app-ui`), archived.
- **M14** — Undo/Redo: snapshot history in `crates/pictura-app/src/history.rs`
  (two-stack, depth 20 = CS6 default, doc+selection clones); capture wired
  into all eleven mutating commands (pre-state clone on success only);
  `open()` resets. `PictureView::undo`/`redo`/`can_undo`/`can_redo`/
  `history_depth`; dock buttons + Ctrl+Z/Ctrl+Y; self-test proves bit-exact
  undo/redo, redo invalidation, open reset (exit codes 22/23).
  OpenSpec change `m14-undo-history` (capability `edit-history`), archived.
- **M15** — Render filters: `Clouds`, `DifferenceClouds`, `Fibers`,
  `LensFlare` in `pictura-filters/src/render.rs` (seeded lattice value noise,
  Difference blend, x-elongated fibers, additive lens flare with
  `LensType`); all four classified no-equivalent (closed Adobe models) and
  verified by 13 property tests. App kinds + combo + self-test confinement
  check (exit 24). Lighting Effects and Scripted Patterns remain future.
  OpenSpec change `m15-render-filters` (capability `render-filters`),
  archived.
- **M16** — App shell foundation: replaced the M0 debug window with a
  CS6-shaped frame. New `pictura-app` C++ units: `commands` (declarative
  registry: stable id, path, label, shortcut, enablement, dispatch),
  `command_tree` (full documented CS6 tree — 523 documented leaves plus 21
  implemented commands and 47 separators; unimplemented leaves disabled), `frame`
  (`PicturaMainWindow`: menu bar, central canvas, status bar with view-options
  popup, dock registration with duplicate `objectName` rejection, screen modes
  `F`/`Shift+F`, canvas colour `Space+F`, `Tab`/`Shift+Tab` hide-all),
  `theme` (Fusion + dark palette, four brightness levels, `Shift+F1`/`F2`),
  `session` (atomic `QSaveFile` XDG state, schema-versioned). Bridge gains
  `has_document()` for enablement. `main.cpp` shrinks to startup + the self-test
  call (`selftest.cpp`);
  new self-test checks (exit codes 25–32) cover menu order, dispatch/inertness,
  no-document enablement, brightness, screen-mode cycle, session round-trip,
  duplicate panel rejection, and hide-all. OpenSpec change `m16-app-shell`
  (capabilities `command-registry`, `workspace-persistence`; MODIFIED
  `application-shell`), archived.
- **M17** — Document lifecycle and multi-document tabs. Rust bridge
  `PictureView` gains `new_document(width,height,mode,depth,background)`
  (8-bit Grayscale/RGB, white/transparent only), `save(path)` (atomic
  temp+rename through `write_psd`), `is_dirty()`, `file_path()`, plus
  `path`/`dirty` state; dirty is set at all 11 mutating-command
  history-capture sites and cleared by open/save. C++ shell: `new_document_dialog`
  (New Document dialog), `dialogs` (`askUnsaved` Save/Discard/Cancel with a
  non-interactive test policy), `frame` rewritten around a `QTabWidget` document
  area (one `PictureView`+`ImageView` per document, active-document targeting
  for all menu handlers/dock/status, tab titles with a `*` modified marker,
  File New/Open/Save/Save As/Revert/Close/Close All/Exit handlers, recent-files
  list persisted in the session store), `session` gains a bounded `recent`
  list, and `commands.h`/`command_tree.cpp` file-lifecycle leaves become
  implemented. `main.cpp` starts empty, opens the CLI path into a document, or
  creates a scratch document for the GPU smoke test; self-test exit codes 33–37
  cover New + white fill, Save As/open pixel round-trip, dirty set/cleared, tab
  switching, and close prompt Cancel/Discard. Verified: `cargo` suite clean
  (401 tests, 0 failed, 1 ignored — unchanged), fixture and no-argument
  self-tests exit 0, `guard.sh` OK. OpenSpec change `m17-document-lifecycle`
  (capabilities `document-lifecycle`, `document-tabs`; MODIFIED
  `application-shell`), archived.
  **Bug fixed (document tab reorder):** the document `QTabWidget` was movable
  but nothing connected `QTabBar::tabMoved`, so dragging a tab left `docs_` in
  the old order while `tabs_->currentIndex()`, `viewAt` and `removeDocument`
  indexed the new one — the wrong document became active, closed, or returned.
  `frame.cpp` now connects `tabMoved(from,to)` to `docs_.move(from,to)` (guarded
  by valid indices); Qt keeps the dragged-to-current tab current, so the active
  document is unchanged. Regression: self-test exit **196**
  `doc_tab_reorder aligned=1` (`PicturaMainWindow::reorderDocumentsForTest`),
  which also exercises `viewAt`/`documentName`/`activeDocumentIndex` after the
  move.
- **M18** — Toolbox and core tools. Engine: `pictura-select` gains
  `Selection::{rect,ellipse,polygon}` coverage rasterizers and
  `CombineMode {New,Add,Subtract,Intersect}` + `combine_with`; `pictura-render`
  document ops gain `crop_document` (reuses the M12 canvas offset; clamps and
  shifts layers/masks/channels) and `translate_layer` (shifts the topmost pixel
  layer's rect). Rust bridge: `select_rect`/`select_ellipse`/`begin_lasso`/
  `lasso_add_point`/`end_lasso`/`quick_select`/`crop`/`translate_layer`/
  `sample_argb`/`selection_bounds`, with dirty/history capture on the new
  mutating ops. C++ shell: `image_view.{h,cpp}` gains pointer signals
  (`mousePressed/moved/released` in image coordinates), `setPanEnabled`, and an
  overlay polygon; new `tools.{h,cpp}` (`ToolId {Move,Marquee,Lasso,
  QuickSelection,Crop,Eyedropper,Hand,Zoom}`, `SelectionMode`, `ToolController`),
  `toolbox.{h,cpp}` (Tools dock, letter shortcuts), `options_bar.{h,cpp}`
  (context-sensitive options bar); `frame.{h,cpp}` hosts them, routes canvas
  events, exposes `activeTool/setActiveTool/foregroundColor/hasPendingCrop/
  commitCrop`. Commands `image.crop`, `view.options`, `window.panels.tools`.
  `selftest.cpp` self-test (exit codes 39–46): tool switching, marquee rect (16 px)
  + ellipse (12 px), combine modes (16/28/12/4), lasso (25 px) + short-lasso
  rejection, quick selection, crop remap, layer move, eyedropper sample
  (`ffff0000`/`00000000`). Verified: build green, fixture and no-arg self-tests
  exit 0, `cargo fmt/clippy` clean, 411 tests (0 failed, 1 ignored; +10 engine
  tests), `openspec validate --all --strict` 40/40 pre-archive (42 after),
  `guard.sh` OK. OpenSpec change `m18-toolbox-tools` (capabilities
  `tool-framework`, `shape-selection-tools`, `canvas-tools`), archived.
- **M19** — SVG icon set and cursors. New `assets/icons/` (40 original
  independent-creation SVGs: `app`, eight `tool.*`, and 31 implemented-command icons named
  by command id, e.g. `file.saveAs.svg`, `view.screenMode.full.svg`) and
  `assets/cursors/` (eight `tool.*` SVG cursors; eye-dropper hotspot (2,22),
  others (12,12)); `assets/pictura.qrc` bundles all 48. `icons.{h,cpp}` provides
  `QIcon pictura::icon(id)` and `QCursor pictura::cursor(id)` (QSvgRenderer,
  DPR-scaled render, existence-guarded so unknown ids are silent). Build gains
  `Qt6::Svg`, `CMAKE_AUTORCC`, the `.qrc`, and the `Qt6::Svg` link. Wiring:
  application/window icon, Tools-panel action icons, the 31 implemented
  menu-action icons, and the active tool's SVG cursor. `selftest.cpp` self-test exit
  codes 47–49: all 40 icons resolve + unknown is null, all 8 cursors resolve,
  window icon set. Verified: build green, fixture and no-arg self-tests exit 0
  with no Qt warnings, `cargo fmt/clippy` clean, 411 tests (0 failed, 1 ignored;
  unchanged), `openspec validate --all --strict` 43/43 pre-archive (44 after),
  `guard.sh` OK. OpenSpec change `m19-svg-icons` (capabilities `icon-assets`,
  `svg-cursors`), archived.
- **M20** — Panel parity. `history.rs` becomes a labeled linear model (one
  labeled state per undoable step, depth 20, plus up to 10 named snapshots);
  `edit-history` undo/redo semantics are unchanged. Bridge gains
  `layer_blend`/`set_layer_blend`, `layer_opacity`/`set_layer_opacity`,
  `set_layer_name`, `move_layer`, and `layer_thumbnail`. Seven `QDockWidget`
  panels in `crates/pictura-app/cpp/panels/` (`layers_panel`, `history_panel`,
  `navigator_panel`, `color_panel` + `ColorState`, `swatches_panel`,
  `info_panel`, `histogram_panel`; objectNames `layersPanel`, `historyPanel`,
  `navigatorPanel`, `colorPanel`, `swatchesPanel`, `infoPanel`,
  `histogramPanel`). `frame` registers the docks (replacing the debug dock),
  rebinds them on active-document change, owns a `ColorState` fed by the
  Eyedropper (`ToolController::foregroundSampled`), routes
  `ImageView::mouseMoved` into the Info panel, and implements checkable
  `Window > Panels` toggles for Navigator/History/Color/Swatches/Info/Histogram
  in addition to Layers/Tools. `CMakeLists.txt` gains the panel sources.
  `selftest.cpp` self-test exit codes 50–52 (`m20_layer count=2 name=1 blend=1
  badblend=1 opacity=128 dirty=1`; `m20_history states=5 open_label=1 grew=1
  jump=1 snapshot=1`; `m20_panels registered=7 toggled=7`), and the
  headless-shutdown hang is fixed by disabling the interactive
  unsaved-document prompt before the self-test quit timer. Deferred non-goals:
  group-tree expansion, drag-reorder, layer lock flags, clipping/link/color
  labels, filter/search row, swatch library file I/O, Info color samplers,
  Histogram source/cache states, and history branching beyond the bounded
  stack. Verified: build green, fixture and no-arg self-tests exit 0, `cargo
  fmt/clippy` clean, 414 tests (0 failed, 1 ignored), `openspec validate --all
  --strict` 45/45 pre-archive. OpenSpec change m20-panels (capabilities
  layers-panel, history-panel, navigator-panel, color-swatches-panel,
  info-histogram-panel), archived.
- **M21** — Paint engine. New `pictura-paint` crate (depends on `pictura-core`
  only): dab-splatting stroke engine with a procedural round/elliptical tip
  (size 1..5000, hardness, roundness, angle, flip), anti-aliased Brush vs
  aliased Pencil, fixed and velocity-driven spacing with residue carry-over,
  per-pixel coverage accumulation with a flow/opacity model
  (`acc = 1-(1-acc)(1-flow*tip)`, composited alpha `= opacity*acc`), and paint
  modes Normal/Dissolve/Behind/Clear; 25 in-crate tests. Bridge `PictureView`
  gains `begin_paint`/`paint_dab`/`end_paint`/`cancel_paint`/`is_painting`, a
  pre-stroke base plus working document, live image refresh per dab, and one
  labelled history state ("Brush"/"Pencil") per completed stroke. Qt:
  `ToolId::Brush`/`Pencil` (values 8/9), `B`/`Shift+B` cycling, size and hardness
  shortcuts (`[`/`]`, `Shift+[`/`Shift+]`), a paint options bar (size, hardness,
  opacity, flow, mode, Pencil Auto Erase), and `ToolController` stroke routing.
  Bug fixed: `PictureView::new_document` now creates one raster layer (opaque for
  "white", transparent for "transparent"), matching
  `docs/10-workflow-io/open-and-new.md`; previously a fresh document had no layer
  and could not be painted. `selftest.cpp` self-test exit codes 53–56
  (`m21_stroke ended=1 painted=196 dirty=1 hist=2`; `m21_opacity a1=84 a2=140`;
  `m21_aliased pencil_ok=1 brush_aa=1`; `m21_undo changed=1 restored=1`), with
  identical output on fixture and no-argument runs. Deferred non-goals:
  sampled/bristle/erodible/airbrush tips, Shape Dynamics, Scattering, Texture,
  Dual Brush, Color Dynamics, Transfer, Brush Pose, airbrush time build-up,
  tablet pressure mapping, `.abr` presets, Brush/Brush Presets panels, HUD, the
  remaining 23 paint modes, 16/32-bit and non-RGB painting, lock transparency,
  and sparse tile scratch storage. Verified: `cmake --build build` OK, both
  self-tests exit 0 with no FAILs, `cargo fmt/clippy` clean, 439 tests (0 failed,
  1 ignored; +25 `pictura-paint` tests), `openspec validate --all --strict`
  50/50 pre-archive. OpenSpec change m21-paint-engine (capabilities paint-engine,
  brush-tools; MODIFIED tool-framework), archived.
- **M22** — Artistic filters. New `pictura-filters::artistic` module implements
  the 15 CS6 Artistic filters — Colored Pencil, Cutout, Dry Brush, Film Grain,
  Fresco, Neon Glow, Paint Daubs, Palette Knife, Plastic Wrap, Poster Edges,
  Rough Pastels, Smudge Stick, Sponge, Underpainting, Watercolor — as
  behavioural-parity models (Adobe's kernels are closed), each carrying a
  `// ponytail:` ceiling note. Shared helpers: `artistic/reduce.rs`
  (`posterize`, `edge_magnitude`), `artistic/noise.rs` (seeded value noise via
  `ChaCha8Rng`), and `artistic/texture.rs` (`TextureSurface`
  Brick/Burlap/Canvas/Sandstone, `emboss`, `TextureOptions { surface, scaling,
  relief, light_direction, invert }`). All 15 are `Filter` variants with typed
  parameters, in-range validation, alpha preservation, and seeded determinism for
  the stochastic ones; new enums `BrushType`
  (Simple/LightRough/DarkRough/WideSharp/WideBlurry/Sparkle) and
  `TextureSurface`. App `filter_from_kind` maps the 15 kebab-case kinds with fixed
  in-range defaults and `seed: 1`. Self-test exit codes 57–58
  (`m22_applied applied=15/15`; `m22_deterministic=1`), measured on fixture and
  no-arg runs. Deferred non-goals: Filter Gallery dialog and cumulative stack,
  Smart Filter entries, `Edit > Fade`, 16/32-bit and CMYK/Lab gating, `Load
  Texture` file I/O, and the remaining families (Brush Strokes, Sketch, Texture,
  Oil Paint). Verified: `cmake --build build` OK; both self-tests exit 0 with no
  FAILs; `cargo fmt --all --check` OK; `cargo clippy --workspace --all-targets --
  -D warnings` OK; 465 tests (0 failed, 1 ignored; +26 filter tests); `openspec
  validate --all --strict` 52/52 pre-archive. OpenSpec change m22-artistic-filters
  (capability artistic-filters), archived.
- **M23** — CS6 UI chrome. `theme.{h,cpp}` gains `Theme::styleSheet(int level)`,
  a CS6-style QSS built from the four-level dark ramp (menu bar, options bar,
  dock tabs/title bars, tool buttons, status bar, scrollbars, menus, tooltips,
  panel content, push buttons); `Theme::apply` now sets the palette then the
  stylesheet. `toolbox.{h,cpp}` is rebuilt as a two-column icon-button grid
  (10 tools) replacing the single-column toolbar, plus a
  `ForegroundBackgroundWidget` (overlapping fg/bg swatches, active target,
  reset) and a screen-mode button emitting `screenModeRequested()`. `ColorState`
  gains an active target (`foregroundActive()`/`setForegroundActive()`/
  `activeChanged`) shared by the Color panel and the toolbox control.
  `frame.cpp` tabifies the default docks into CS6 groups — Color+Swatches,
  Layers+History, Navigator+Info+Histogram — and sets the canvas default colour
  to the CS6 dark grey `QColor(37,37,37)` (the four-entry cycle is unchanged).
  Reference screenshot: `docs/02-ui-ux/reference/cs6-workspace.png`. Self-test
  hygiene: an isolated `XDG_STATE_HOME` (`QTemporaryDir`) is set before the frame
  is constructed, so a user-saved layout cannot affect the checks. Self-test
  exit codes 59–61, measured identically on fixture and no-argument runs:
  `m23_stylesheet applied=1 levels=4 distinct=1`;
  `m23_toolbox dock=1 buttons=11 fgbg=1`; `m23_groups grouped=4/4`. Deferred
  non-goals: pixel-exact CS6 metrics and icon art, HUD/on-image displays, new
  panels (Gradients/Patterns/Properties/Adjustments/Libraries/Channels/Paths/
  Brush), workspace presets/switcher, icon-collapse docks, floating-panel drop
  zones, and deeper Layers-panel internals beyond M20. Verified:
  `cmake --build build` OK; both self-tests exit 0 with no FAILs; `cargo fmt
  --all --check` OK; `cargo clippy --workspace --all-targets -- -D warnings`
  OK; 465 tests (0 failed, 1 ignored; unchanged — no Rust changes); `openspec
  validate --all --strict` 53/53 pre-archive. OpenSpec change
  m23-cs6-ui-chrome (MODIFIED application-shell, tool-framework), archived.
- **M24** — Panel rail and right-side placeholders. `panels/placeholder_panel.{h,cpp}`
  adds one shared `pictura::PlaceholderPanel(title, message, parent)` dock with a
  centred CS6 empty-state label; `panels/panel_rail.{h,cpp}` adds
  `pictura::PanelRail`, a vertical icon-only `QToolBar` (objectName `panelRail`).
  `frame` creates the eight new placeholder docks — objectNames `gradientsPanel`,
  `patternsPanel`, `propertiesPanel`, `adjustmentsPanel`, `librariesPanel`,
  `channelsPanel`, `pathsPanel`, `actionsPanel` (Properties shows "No Properties"),
  structural only, no real functionality yet — and tabifies the right docks into
  the three CS6 groups Color+Swatches+Gradients+Patterns,
  Properties+Adjustments+Libraries, and Layers+Channels+Paths (tabify + raise the
  first). The rail carries five glyph buttons for History, Actions, Info,
  Navigator, Histogram; each button and its matching `Window > Panels` command
  share one toggle path, and the button's checked state tracks the dock's
  visibility. `commands.h`/`command_tree.cpp` gain ids and implemented, checkable
  `Window > Panels` entries for the eight new panels; `CMakeLists.txt` gains the
  two new sources. Self-test exit codes 61 (grouping, revised), 62 (panels), and
  63 (rail), measured identically on fixture and no-argument runs:
  `m24_groups grouped=8/8`; `m24_panels found=8/8 properties_empty=1`;
  `m24_rail actions=5 toggled=1 synced=1`. Deferred non-goals: real content for the
  placeholder panels (gradient/pattern presets, Properties binding, adjustment
  presets, libraries, channel/path lists, actions), icon-collapse auto-collapse,
  floating-panel drop zones, workspace presets/switcher, and panel-title-bar menus.
  Verified: `cmake --build build` OK; both self-tests exit 0 with no FAILs;
  `cargo fmt --all --check` OK; `cargo clippy --workspace --all-targets --
  -D warnings` OK; 465 tests (0 failed, 1 ignored; unchanged — no Rust changes);
  `openspec validate --all --strict` 53/53 pre-archive. OpenSpec change
  m24-panel-rail (capability panel-rail; MODIFIED application-shell), archived.
- **M25** — Remaining filter families. `pictura-filters` gains the four remaining
  CS6 families, 29 filters total: **Brush Strokes** (`brush_strokes.rs`: Accented
  Edges, Angled Strokes, Crosshatch, Dark Strokes, Ink Outlines, Spatter, Sprayed
  Strokes, Sumi-e), **Sketch** (`sketch/{mod,relief,paper}.rs`: Bas Relief, Chalk
  & Charcoal, Charcoal, Chrome, Conté Crayon, Graphic Pen, Halftone Pattern, Note
  Paper, Photocopy, Plaster, Reticulation, Stamp, Torn Edges, Water Paper),
  **Texture** (`texture.rs`: Craquelure, Grain, Mosaic Tiles, Patchwork, Stained
  Glass, Texturizer), and **Oil Paint** (`oil_paint.rs`). All are
  behavioural-parity models (Adobe kernels closed), reusing the M22 shared helpers
  `artistic::{reduce,noise,texture}` (posterize, edge_magnitude, clamp_u8,
  value_noise, surface_height, emboss, `TextureOptions`). New `Filter` enums
  `StrokeDirection`, `LightDirection`, `HalftoneType`, `GrainType` and 29 variants
  in `lib.rs`; `apply` dispatches to the family functions. Colour-dependent Sketch
  filters carry explicit `foreground`/`background` RGB (Bas Relief included —
  dark/recessed→foreground, light/raised→background); Conté Crayon and Texturizer
  take shared `TextureOptions`; seeded filters carry `seed: u64` and are
  bit-reproducible. **Oil Paint is a deliberate non-parity divergence:** CS6
  hard-requires a supported GPU (closed OpenCL kernel, no CPU fallback), so this
  is a CPU behavioural model (gradient-orientation directional edge-stopping
  smoothing + a luma/scale/bristle height field shaded Lambert/Blinn-Phong from
  `angular_direction`/`shine`); it is deterministic (no seed) and a GPU compute
  path is the deferred upgrade. **Accented Edges** is neutral (no-op) at Edge
  Brightness 25 by contract; the app default is set to 38 (CS6's dialog default)
  so the menu item visibly acts. App: `filter_from_kind` maps all 29 kebab-case
  kinds with fixed in-range defaults (`seed: 1` for seeded ones). Self-test exit
  codes **68/69** (`m25_applied applied=29/29`; `m25_deterministic=1`), identical
  on fixture and no-argument runs. Self-test label disambiguation: the earlier
  canvas-perf checks were renamed `m25_*` → `canvas_*` (`canvas_centre`,
  `canvas_middle_pan`, `canvas_move`, `canvas_preview_cache`) so the `m25_` prefix
  belongs to the actual M25 milestone; exit codes 64–67 unchanged. Verified:
  `cmake --build build` OK; both self-tests exit 0; `cargo fmt/clippy` clean;
  **508 tests (0 failed, 1 ignored)** — up from 465; `openspec validate --all
  --strict` 54/54 pre-archive. OpenSpec change `m25-filter-families` adds
  capabilities `brush-stroke-filters`, `sketch-filters`, `texture-filters`,
  `oil-paint-filter` (4 new → **57** capabilities after archive). Deferred
  non-goals: Filter Gallery dialog and cumulative/reorder stack, Smart Filters,
  `Edit > Fade`, 16/32-bit and CMYK/Lab gating, `Load Texture` file I/O, and an
  Oil Paint GPU compute pass.

- **M26** — GPU compute backend (GPU default). `pictura-render::gpu` gains
  `Backend { Gpu, Cpu }`, `gpu_available()` (cached probe), and
  `composite_active(doc, gpu_enabled) -> (PixelBuffer, Backend)`; re-exported from
  the crate root. The compute pipeline/bind-group layout/params are cached once per
  device (was rebuilt per composite). The params uniform was moved from
  process-global into the per-composite `Gpu` — a real race fix for parallel
  composites. Compositor completeness: the four **non-separable** modes
  Hue/Saturation/Color/Luminosity now run on the GPU (mode ids 23–26; separable
  ids 1–22 unchanged); the app's five **adjustment layers** (invert, posterize,
  threshold, brightness/contrast, hue/saturation) are applied on the GPU at the
  layer's stack position, matching the CPU oracle; **Dissolve** is the only
  remaining CPU-only blend mode (random, not bit-reproducible). Layer source/mask
  uploads are now rect-only. Parity: separable 26 modes max delta 0 LSB,
  non-separable 4 modes max delta 0 LSB, adjustments 0 LSB except hue/saturation
  1 LSB (all within ±1). App: an app-wide `gpuCompute` preference (default `true`)
  is persisted in the XDG session store, schema **v2** (a missing field or
  schema-1 store loads `true`). A checkable implemented command
  **`view.gpuCompute`** ("Use GPU Compute", View menu after View > Options)
  toggles it, enabled only when an adapter is present (greyed when none). The
  status bar shows `GPU` / `CPU` / `CPU (no GPU)`. Bridge `PictureView` gains
  `set_gpu_compute`, `gpu_compute`, `gpu_available`, `active_backend`;
  `current_buffer`/`document_to_image` route through `composite_active`, so the
  canvas, panels, painting refresh, and every recomposite use the GPU by default,
  falling back to CPU (per call) for `Dissolve`, unsupported adjustments, or any
  `GpuError`. Self-test exit codes **70/71**: `m26_gpu available=1 default_on=1
  off_cpu=1 on_back=1`; `m26_session gpu_off=1 gpu_on=1 default=1` — identical on
  fixture and no-argument runs. Timing evidence (1024×1024, 4 pixel layers, debug
  test build): `m26 timing 1024x1024x4layers: cpu 485 ms, gpu 744 ms` (the GPU
  path is readback-bound). Verified: `cmake --build build` OK; both self-tests
  exit 0; `cargo fmt/clippy` clean; **515 tests (0 failed, 1 ignored)** — up from
  508 (514 before the close-out timing test); `openspec validate --all --strict`
  58/58. OpenSpec change `m26-gpu-compute` adds capability `gpu-compute-backend`
  and MODIFIES `gpu-compositing` (→ **58** capabilities after archive). Deferred:
  on-screen zero-copy present (manual QRhi + QWindow swapchain; `QRhiWidget`
  blocks device adoption — see `crates/pictura-app/GPU-INTEROP-NOTES.md`),
  off-GUI-thread compositing, GPU filters and GPU painting (next change
  `m27-gpu-filter-acceleration`), and Dissolve on GPU.
- **M27** — GPU-native acceleration. The compositor data path is now 8-bit end
  to end: layer sources upload as raw planar 8-bit channel samples over the layer
  rect, the mask uploads as an 8-bit plane, the working canvas is packed 8-bit
  RGBA in a storage buffer, and output is read back as packed 8-bit — no
  host-side full-canvas `f32` buffer and no `f32` readback (was the M26
  bottleneck). Public API and ±1 LSB parity unchanged. Release timing,
  1024×1024×4 pixel layers: **CPU 72 ms → GPU 19 ms (~3.8×; pre-M27 was ~1.2×)**.
  Parity: separable 26 modes 0 LSB, non-separable 4 modes 0 LSB, five adjustment
  layers 0 LSB except hue/saturation 1 LSB, group/mask scenes 0 LSB. **GPU filter
  acceleration:** new `pictura-render` API `filter_gpu_available()` and
  `apply_filter_active(filter, buf, gpu_enabled) -> Result<Backend, FilterError>`
  (re-exported from the crate root); a wgpu compute shader (kernel / separable-H /
  separable-V / motion / combine modes, weights from the CPU kernel builders)
  accelerates nine filters **byte-exact (0 LSB vs the CPU oracle, alpha
  bit-identical)** — Gaussian Blur, Box Blur, Motion Blur, Blur, BlurMore,
  Sharpen, SharpenMore, Unsharp Mask, High Pass — while every other filter falls
  back to the CPU `pictura_filters::apply` byte-identically (including the M22
  Artistic and M25 Brush Strokes/Sketch/Texture families and Oil Paint).
  `pictura_render::apply_filter` gained a `gpu_enabled` parameter and routes
  through `apply_filter_active`; the app passes the M26 `gpuCompute` flag, so
  filters are GPU-accelerated by default and CPU when disabled or no adapter
  exists. `pictura_render::gpu::shared_device()` (crate-private) now serves both
  the compositor and the filter path; the filter path no longer opens a second
  Vulkan device. Self-test exit code **72**: `m27_filter byte_identical=1` — a
  Gaussian Blur through the GPU path and the CPU path produces byte-identical
  images, identical on fixture and no-argument runs. Verified: `cmake --build
  build` OK; both self-tests exit 0; `cargo fmt/clippy` clean; **519 tests (0
  failed, 1 ignored)** — up from 515; `openspec validate --all --strict` 59/59.
  OpenSpec change `m27-gpu-acceleration` adds capability
  `gpu-filter-acceleration` and MODIFIES `gpu-compositing` (→ **59** capabilities
  after archive). Deferred: GPU kernels for the painterly/stochastic filter
  families and Oil Paint; GPU painting/brush; on-screen zero-copy present;
  off-GUI-thread compute; Dissolve on GPU.
- **M28** — GPU heavy filters. `crates/pictura-filters/tests/profile.rs` profiles the
  CPU filters at 1024² (release) and ranks them by cost; the heavy **deterministic**
  kernels were then ported onto the M27 GPU filter path. Six new kernels —
  **Surface Blur, Maximum, Minimum, Median, Custom (5×5), Oil Paint** — match the
  CPU oracle within ±1 LSB (mostly 0) with alpha bit-identical. Surface Blur is a
  bilateral `SURFACE` window (spatial Gaussian × luma-range Gaussian, `f32`
  accumulation, `round_away` quantize); Median is an **exact order statistic**
  (binary search over the byte value), not a separable approximation;
  Maximum/Minimum use the morphology window; Custom adds the 5×5 `KERNEL` path.
  **Oil Paint now runs on the GPU** (four passes: luma → edge-stopping directional
  aggregation → height/bristle → Lambert/Blinn-Phong shading), resolving the M25
  CPU-only deferral. The GPU filter path therefore accelerates the previously-named
  nine kernels plus these six. Measured GPU speedups at 1024² (release, this box):
  Surface Blur **7161 → 83 ms (~86×)**, Median **1273 → 6.2 ms (~205×)**, Oil Paint
  **10063 → 15.3 ms (~658×; the brief's baseline was ~1446 ms)**, Custom
  byte-exact. **Selection is profile-driven:** a filter is GPU-accelerated only if
  it is high-cost (≥ ~150 ms at 1024²) *and* reaches ±1 LSB. The stochastic/seeded
  filters stay on the CPU with a byte-identical fallback — `Crystallize`
  (3114 ms), `Watercolor`, `Conté Crayon`, `Paint Daubs`, `Dry Brush`,
  `Ocean Ripple`, `Spatter`, `Sponge`, `Palette Knife`, `Add Noise`,
  `Colored Pencil` — because their RNG stream cannot be reproduced bit-exactly on
  the GPU; the warps/distort and render filters are likewise deferred. Self-test
  exit code **73**: `m28_heavy byte_identical=1` (Surface Blur and Median produce
  identical images through the GPU and CPU paths), identical on fixture and
  no-argument runs. Verified: `cmake --build build` OK; both self-tests exit 0;
  `cargo fmt/clippy` clean; **523 tests (0 failed, 1 ignored)** — up from 519;
  `openspec validate --all --strict` 60/60 pre-archive. OpenSpec change
  `m28-gpu-heavy-filters` MODIFIES `gpu-filter-acceleration` (no new capability →
  **59** capabilities after archive). Deferred: stochastic-family GPU kernels via
  CPU-pre-generated RNG fields, GPU painting/brush, on-screen zero-copy present,
  off-GUI-thread compute, Dissolve on GPU.
- **M29** — large-document performance. A 4000×4000 layer move was 3–5 s per
  update; two root causes were fixed. **GPU dispatch cliff:** the compositor and
  filter paths dispatched a 1-D grid (`ceil(n/64)` workgroups, limit 65535 ≈
  4.19 MP), so 4000² (16.7 MP, 250 000 workgroups) silently fell back to the CPU.
  Both now use a **2-D dispatch** (`x = min(ceil(n/64), 65535)`,
  `y = ceil(ceil(n/64)/x)`, shader index `gid.x + gid.y * (grid_x*64)`);
  rejection only when the 2-D workgroup product is exceeded (~2.8×10¹⁴ px).
  4000² now composites on the GPU (`Backend::Gpu`): **CPU 887 ms → GPU 332 ms**
  (release, 3 layers, 1 LSB). Accelerated filters (Surface Blur, Median) also run
  on the GPU at >4.19 MP. **App-side per-update costs:** `sample_argb`
  re-composited the whole document (~286 ms) — it now reads the cached `image`
  QImage (**0 ms**); `layer_thumbnail(24)` built a full-size RGBA image — it now
  gathers the planar channels straight into a ≤24 px buffer (**90 ms → 0.02 ms**);
  the histogram scans all pixels — it now downsamples to ≤512² first
  (~145 ms → <10 ms); `begin_move_preview` cloned the whole document — it now
  hides the layer in place (**344 → ~306 ms**); `commit_move` composites through
  the new GPU-aware `translate_layer_active` (**739 → ~403 ms**). New
  render-crate API `translate_layer_active(doc, dx, dy, gpu_enabled)` (the CPU
  `translate_layer`/`recompute` remain the oracle). Self-test unchanged codes; the
  temporary 4000² timing block was removed (self-test stays fast) and the
  `move_profile_4000`/`move_profile_1024` tests are `#[ignore]`d (run with
  `cargo test -p pictura_app --release -- --ignored --nocapture move_profile`).
  Verified: `cmake --build build` OK; both self-tests exit 0; `cargo fmt/clippy`
  clean; **530 tests (0 failed, 3 ignored)** — up from 523; `openspec validate
  --all --strict` 60/60. The M29 change MODIFIES `gpu-compositing`,
  `gpu-filter-acceleration`, `info-histogram-panel`, `layers-panel`,
  `document-canvas`; no new capability (→ **59** capabilities after archive).
  Remaining bottlenecks: the history capture still clones the whole document
  (~60 ms/state, and up to 20 states of memory) — **copy-on-write or tile diffs**
  is the deferred fix. (The initial "readback-bound" guess was measured wrong in
  M32: the 4000² composite is dominated by host-side CPU per-pixel assembly, not
  the 64 MB readback — see below.)
- **M30** — canvas transparency and clipping. `ImageView::paintEvent` now draws a
  **transparency checkerboard** behind the document so pixels with alpha < 255
  reveal it (a fully transparent document shows the checkerboard; opaque pixels
  cover it). The checkerboard is **screen-space** (constant 8 px cells,
  independent of zoom), **anchored to the document origin** (stable while
  panning), rendered with a cached 2×2-cell `QPixmap` tile via a brush origin and
  clipped to the document rect ∩ viewport (O(1), never allocated beyond the
  screen). Two light tones `#FFFFFF` / `#CCCCCC` (Photoshop "Light" grid). All
  canvas content — the composited image, the Move-tool live preview layer, and the
  selection overlay — is now **clipped to the document rect**, so a layer dragged
  outside the canvas is cropped instead of drawn over the surrounding area. Test
  hooks `ImageView::transparencyCellSize()/transparencyColorA()/transparencyColorB()`
  were added. Self-test exit code **74**: `m30_canvas checker=1 clipped=1` (a
  transparent document shows both checker tones inside the document rect and the
  canvas colour outside; a preview layer dragged past the canvas edge is clipped),
  identical on fixture and no-argument runs. Verified: `cmake --build build` OK;
  both self-tests exit 0; `cargo fmt/clippy` clean; **530 tests (0 failed, 3
  ignored)** (no Rust change; the check is in the C++ self-test); `openspec
  validate --all --strict` 60/60. The M30 change MODIFIES `application-shell`
  (no new capability → **59** capabilities after archive). Deferred: the
  `Transparency & Gamut` preferences pane (grid size None/Small/Medium/Large and
  colour sets Light/Medium/Dark/Red/Custom), the `View > Show > Transparency Grid`
  toggle, gamut warning, and the GPU/RHI-backed canvas.
- **M31** — region (dirty-rect) compositing. New render API
  `pictura_render::composite_region_active(doc, rect: PsdRect, gpu_enabled) ->
  (PixelBuffer, Backend)` composites only the (clamped) rect and returns a
  rect-sized buffer **byte-identical (0 LSB)** to the corresponding
  sub-rectangle of `composite_active`, verified across separable, non-separable
  (Hue), adjustment-layer, masked and isolated-group scenes. The GPU path threads
  a region origin/size/stride so the canvas, per-layer sources, mask and group
  inner canvases are region-sized and only the region is dispatched and read
  back; the CPU fallback is a full composite + slice (`ponytail:`).
  `composite_gpu`/`composite_active` are unchanged (the region kernel over the
  full rect). Measured: 4000² with a 512² region **52 ms vs 3082 ms full (59×)**.
  New `pictura_render::translate_layer_rect(doc, dx, dy)` shifts a layer's rect
  (and mask) **without** recomputing (the CPU `translate_layer`/`recompute`
  remain the oracle). App: `PictureView` caches the composited document `QImage`;
  `refresh_region(rect)` composites the rect through the active backend and
  patches the cached image (and `doc.composite`) so pixels outside the rect are
  untouched. `commit_move` now invalidates `old ∪ new` layer bounds and refreshes
  only that region; `paint_dab` invalidates the stroke's `dirty()` rect. All
  other mutations keep the full recomposite path (the incremental-extension
  point). Measured region-move: a 64² region **0.497 ms vs 585 ms full
  (~1170×)**. A guard (`REGION_REFRESH_BUDGET = 1_000_000` px) falls back to the
  full recomposite for large dirty unions, because the cached image is patched
  with per-pixel `QImage::set_pixel_color` (FFI per pixel) — the `ponytail:`
  upgrade is a C++ `ImageView::blitRegion` via
  `QPainter::CompositionMode_Source`, or M34's GPU-resident present. Self-test
  exit code **75**: `m31_region moved=1 outside_unchanged=1 undo=1` and
  `m31_region_large moved=1 vacated=1 undo=1` (the latter drives the
  full-recomposite fallback), identical on fixture and no-argument runs.
  Verified: `cmake --build build` OK; both self-tests exit 0; `cargo fmt/clippy`
  clean; **536 tests (0 failed, 3 ignored)** — up from 530; `openspec validate
  --all --strict` 60/60. The M31 change MODIFIES `gpu-compositing` and
  `document-canvas` (no new capability → **59** capabilities after archive).
- **M32** — interactive-path region completion and present caching.
  `begin_move_preview` no longer composites the whole document: it
  region-composites only the moved layer's clamped rectangle with that layer
  hidden (`composite_region_active`) and blits it into a clone of the cached
  canvas, **byte-identical** to a full composite with the layer hidden; it falls
  back to the full path when the layer has no image, the clamped rect is empty or
  over `REGION_REFRESH_BUDGET`, or the cache is null/stale. `set_layer_visible`
  refreshes only the toggled layer's influence rectangle — a raster layer's
  `rect`, or a bounded adjustment layer's mask rect (enabled + data + zero
  `default_color`) — and falls back to a full recomposite for groups, unbounded
  adjustments and masks with a non-zero default. `ImageView` now presents from a
  zoom cache keyed on the source `cacheKey()` and the zoom, built **through a
  `QPainter`** so it is pixel-identical to the previous transform draw (a
  `QImage::scaled` smooth build diverged from the painter's non-smooth filter and
  fractional-offset phase — that was found and fixed); the cache is invalidated on
  image or zoom change and falls back to the transform draw above a 64 MP bound.
  Test hooks: `setPresentCacheEnabledForTest`,
  `presentCacheRebuiltOnLastPaint/RebuildCount/ImageSize/ImageKey`; a
  `topmost_pixel_layer_index` bridge accessor was added for the self-test. The
  measurement that scoped M32: a full 4000² GPU composite costs ~254 ms, split
  `build_source` 146 ms (58%), `to_pixel_buffer` 35 ms, `mapped.to_vec` 22 ms,
  `zero_canvas` 18 ms, `build_mask` 16 ms, readback submit/poll/map 7 ms, GPU
  dispatch 0.2 ms — **host-side CPU assembly dominates, not the readback**, which
  is why M32 deferred the Qt Quick zero-copy present (M34) and why M33 targets the
  assembly. Measured move preview (4000², release, RTX 3090): the old body (hide
  layer → full composite) **276 ms**; the new region body with a canvas-sized layer
  is over the per-pixel blit budget and falls back (**280–290 ms**, unchanged),
  while a moved **512²** layer takes the region path at **~30 ms (~10×)**. Self-test
  exit codes **76/77/78**: `present_cache reused=1 zoom_rebuild=1 src=32x32 z0=1
  z1=2 size=64x64 stable=1 identical=1` and `m32_region preview_base=1
  visibility=1`, identical on fixture and no-argument runs. Verified: `cmake --build
  build` OK; both self-tests exit 0; `cargo fmt/clippy` clean; **539 tests (0
  failed, 3 ignored)** — up from 536; `openspec validate --all --strict` 60/60. The
  M32 change MODIFIES `document-canvas` (no new capability → **59** capabilities
  after archive).
- **M33** — full-composite throughput. `build_source` assembles layer sources with
  whole-row `copy_from_slice` per plane (row-wise fast path; the per-pixel scan
  remains the fallback when a channel does not cover the clamped row intersection,
  preserving grayscale 2-plane, missing-G/B-aliases-channel-0, and
  missing-alpha-fills-255 semantics); `build_mask` fills the influence rect
  row-wise when there is no enabled data-carrying mask (per-pixel `mask_alpha`
  retained otherwise); the readback de-interleaves directly from the mapped
  staging slice into the planar `PixelBuffer` with no host packed RGBA
  intermediate (`to_pixel_buffer` walks 4-byte chunks via `as_chunks::<4>`); the
  canvas is cleared with a GPU command (`clear_buffer`) instead of a host
  full-canvas zero write. Output stays byte-identical (GPU 0 LSB, ±1 LSB vs the
  CPU oracle, alpha unchanged) — `cargo test --workspace` includes the new
  equivalence tests `m33_rowwise_source_matches_per_pixel` and
  `m33_rowwise_mask_matches_per_pixel`, and `gpu_parity` (18 tests) passes.
  Measured 4000² (2 RGB layers, release, RTX 3090): total **~123 ms (from
  ~254 ms, ~2×)**; `zero_canvas` **0.05 ms (from 18)**, `build_source` **~69 ms
  (from 146)**, `build_mask` **~15 ms (from 16)**, readback (transfer +
  de-interleave) **~38 ms (from ~57 = 22 `to_vec` + 35 de-interleave)**,
  dispatch 0.2 ms. 1024² total **~4.3 ms**. The remainder is dominated by the
  per-composite source/mask **upload** (~128 MB + 16 MB per composite), which only
  resident per-layer GPU buffers would remove (deferred). M33 stopped here
  because the measured bottleneck after the change is the upload, and residency
  needs content versioning (deferred); `build_mask` gained least because its
  upload dominates the removed `mask_alpha` calls. Self-test: unchanged codes;
  **541 tests, 0 failed, 5 ignored** (was 539, 3 ignored); `openspec validate
  --all --strict` 60/60; the M33 change MODIFIES `gpu-compositing` only (no new
  capability → **59** after archive).
- **M33 follow-up — A2: shader-side planar output** (direct continuation of the
  M33 deferred item; no OpenSpec change, no new capability). A second compute
  shader (`crates/pictura-render/src/gpu/shader.rs` `PLANAR_SHADER`, entry
  `cs_planar`) de-interleaves the packed RGBA canvas into four byte planes in one
  storage buffer (`backend.rs` `PlanarResources`, cached in `Devices` via
  `planar_resources()`), and `Gpu::read_canvas` copies each plane's `n` bytes
  into the planar `PixelBuffer` instead of gathering a channel per pixel.
  `to_pixel_buffer` stays (`#[cfg(test)]`) for the profile micro-benchmark.
  Byte-identical: the 20 `gpu_parity` tests pass. Measured 4000² (2 RGB layers,
  release, RTX 3090): `readback` **~41 ms → ~27–32 ms**, `total` **~130 ms →
  ~117–131 ms** (the GPU planarize dispatch and the 64 MB plane stream are
  bandwidth-bound; the host gather is gone). `cargo test --workspace` **592
  tests, 0 failed, 9 ignored** (unchanged); self-test stderr byte-identical.
- **M36 — layer attributes end-to-end** (the first milestone of the Layers-panel
  program; see `docs/dev/layers-panel-program.md`). `pictura_core::Layer` gained
  `fill: u8` (default 255), `lock: LockFlags` (newtype, `TRANSPARENCY|PIXELS|
  POSITION`, `all()` = 0x07) and `color: ColorLabel` (None/Red/Orange/Yellow/
  Green/Blue/Violet/Gray); 37 explicit `Layer { .. }` literals across 14 files
  were updated with the defaults (plus the `..bare` update-syntax layer in
  `pictura-core`'s `masked` test). The compositor now uses effective layer alpha
  = `opacity/255 × fill/255` in both the CPU oracle and the GPU shader (a new
  `fill` uniform word; groups use 1.0), byte-identical to the previous composite
  at `fill == 255` and within ±1 LSB on the GPU. The codec reads/writes the
  `lspf` (lock), `lclr` (color) and `iOpa` (fill) additional-layer blocks,
  omitted at defaults — a default document's `write_psd` output is byte-identical
  (proven against `crates/pictura-codec/tests/fixtures/default_before.psd`),
  and psd-tools reads the new attributes back. The bridge gained
  `layer_fill`/`set_layer_fill`, `layer_lock`/`set_layer_lock`,
  `layer_color`/`set_layer_color` with the frozen refusal rules (fill refused for
  group/Background/fully-locked; opacity refused for Background/fully-locked;
  lock/color refused for Background) and history labels `Fill Opacity`/`Lock`/
  `Layer Color`. The panel gained a Fill spinbox, a four-button lock strip and an
  eight-entry color-label context menu. Honest limits: CS6's `lspf` "Lock All"
  high-bit `0x80000000` encoding is not handled (writes `0x07`); `layer_kind`'s
  `"background"` is a name+index heuristic (M37 replaces it); the color context
  menu is not greyed on the Background row (the bridge refuses). The `lspf`/
  `iOpa`/`lclr` source disagreements are recorded in
  `docs/dev/layers-panel-program.md` §5. Self-test exit codes 86–89:
  `m36_attrs fill=1 lock=1 color=1 undo=1`, identical on the fixture and
  no-argument runs. Verified: `cmake --build build` OK; both self-tests exit 0;
  `cargo fmt --all --check`/`cargo clippy --workspace --all-targets --
  -D warnings` clean; **556 tests, 0 failed, 7 ignored** (up from 546/7; the
  ignored set is unchanged, so the raw `cargo test` ignored count is 8 with the
  `pictura-render` doctest); `openspec validate --all --strict` 60/60. The M36
  change MODIFIES `layers-panel` and `psd-layer-io` and ADDs to
  `layer-compositing` (no new capability → **59** capabilities after archive).
  Deferred: group Fill in compositing (CS6 has no group Fill; the value
  round-trips and is ignored), the forced type/shape locks, and a first-class
  Background flag (M37).

- **M37 — layer creation and grouping** (the second milestone of the Layers-panel
  program; see `docs/dev/layers-panel-program.md`). New
  `crates/pictura-render/src/document_ops/layer_ops.rs` (418 lines) adds five pure
  functions over `&mut Document` — `add_layer(doc, above, name) -> i32`,
  `add_group(doc, above, name) -> i32`, `duplicate_layer(doc, index) -> i32`,
  `group_layer(doc, index) -> i32`, `ungroup_layer(doc, index) -> bool` — plus
  `next_layer_name(doc, prefix)`, re-exported from `document_ops` and the crate
  root. Insertion is "directly above `above`" = `above + 1` in the bottom-first
  stack, clamped to the top; a negative sentinel (no selection) or an out-of-range
  value also lands on top. `duplicate_layer` deep-clones the node (children,
  channels, mask, adjustment and all attributes) directly above the source and
  names the copy `"<name> copy"`; `group_layer` wraps the target in place (the
  group takes the layer's slot, the layer becomes its only child) and names it
  `"Group N"` via `next_layer_name`; `ungroup_layer` splices the children back in
  order and refuses a non-group (state unchanged). A new layer is a
  document-sized transparent raster layer (channels `0/1/2/-1` of `w*h` zero
  bytes, `Normal`/opacity 255/fill 255/visible) and leaves `composite_rgba`
  unchanged; a new group is an empty `is_group` node with a `Normal` blend (not
  `PassThrough`) and an empty `{0,0,0,0}` rectangle. Ceiling: the empty layer is
  stored document-sized and costs `w*h*4` bytes before it is painted — Photoshop
  stores nothing until a dab — marked with a `// ponytail:` empty-rect upgrade in
  the module. The bridge `PictureView` gains `add_layer`/`add_group`/
  `duplicate_layer`/`group_layer`/`ungroup_layer` `#[qinvokable]`, each
  `recomposite()`-then-`record()` with labels `New Layer` / `New Group` /
  `Duplicate Layer` / `Group Layers` / `Ungroup Layers`; a failed op records
  nothing. The panel adds **New Group** then **New Layer** buttons before Delete
  (Add Adjustment / New Group / New Layer / Delete / Move Up / Move Down) and
  `LayersPanel::currentLayer()`/`selectLayer(int)`; the five `Layer` leaves are
  frozen to `command_ids` (`layer.new.layer`, `layer.new.group`,
  `layer.duplicate.layer`, `layer.group.layers`, `layer.ungroup.layers`) and
  handled in `frame.cpp` against the active view's current layer. Honest limit:
  `Group Layers`/`Ungroup Layers` (and `Duplicate Layer`) act on the **single**
  selected layer — CS6 groups a multi-selection; M39's selection work upgrades
  these to per-selection operations (the handler carries the `// ponytail:` note).
  Self-test exit codes 90–94: `m37_create new=1 group=1 duplicate=1 ungroup=1
  undo=1` (count growth, an inert transparent layer, a `" copy"` duplicate name,
  wrap/unwrap ordering, one history step per op, and five undos restoring the
  start), identical on the no-argument and `two_layers.psd` runs. Verified:
  `cmake --build build` OK; both self-tests exit 0 with no FAILs; `cargo fmt --all
  --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean;
  **568 tests, 0 failed, 7 ignored** (up from 556/7; the ignored set is unchanged,
  so the raw `cargo test --workspace` ignored count is 8 with the pre-existing
  `pictura-render` doctest); `openspec validate m37-layer-creation --strict` valid
  and `openspec validate --all --strict` 60/60. The M37 change MODIFIES
  `layers-panel` (no new capability → **59** capabilities after archive).
  Deferred: a New Layer / New Group **dialog** (neutral-color fill, blend/opacity,
  use-previous-as-clipping), multi-selection grouping, and empty-rect layer
  storage.

- **M38 — icon and cursor library** (a user-requested interruption to the
  Layers-panel program; see `docs/dev/layers-panel-program.md` and the contract
  `docs/dev/m38-icon-cursor-library.md`; OpenSpec change
  `m38-icon-cursor-library`). The full CS6 toolbox catalogue and icon/cursor set
  landed in the Qt shell only — no Rust, bridge, document, compositor, or PSD
  change. **Assets:** `assets/icons/` now holds 137 SVGs and `assets/cursors/`
  71; `assets/pictura.qrc` was regenerated to 208 entries with the on-disk set
  equal to the listed set (all parse under `xmllint`, none use `<text>`, and a
  48 px render check found no blank and no solid-fill assets). **Tool
  catalogue:** `ToolId`/`ToolInfo` in `tools.{h,cpp}` expanded from the M19/M23
  ten to all **71** CS6 tools in catalogue order, each carrying `name`, `label`,
  `shortcut`, `Qt::CursorShape`, `hint`, `group` 1..23, `implemented`, and
  `hotspotX/Y`; `allToolIds()` returns 71, `implementedToolIds()` 10,
  `toolImplemented()` answers membership, and `static_assert(kToolCount == 71)`
  guards the table (the separate `ToolCatalogueEntry` layer in the design was
  collapsed into the existing `ToolInfo` table). **Toolbox:** rebuilt as one
  button per group (23 slots, single column); a slot shows its current member
  (first implemented member, else first) and gets a flyout triangle plus a
  hold/right-click menu only when the group has more than one member; Alt-click
  cycles the implemented members; an all-unimplemented slot is disabled with the
  exact `<label> — not implemented yet` tooltip; `setActiveTool` refuses an
  unimplemented id. The fg/bg swatch widget and the Screen Mode button are
  unchanged below the slots; `slotButtons()` exposes the slots for the
  self-test. **Cursors:** `cursor(id, hotX, hotY)` was added (the 24×24 centre
  default is preserved); the hardcoded eyedropper hotspot is gone and per-tool
  hotspots come from the catalogue; a null or failed SVG render falls back to
  the tool's `Qt::CursorShape`, so a missing asset can never yield an invisible
  cursor. **Panels:** rail buttons carry their `window.panels.<panel>` icons
  (history/actions/info/navigator/histogram — none null); the Layers strip is
  the CS6-order icon set `link, fx, mask, fillAdjustment, group, newLayer,
  delete` (link/fx/mask disabled "not implemented yet";
  fillAdjustment/group/newLayer/delete wired to the existing behaviour); the
  History snapshot button uses `history.snapshot`. Existing objectNames and
  `currentLayer()`/`selectLayer()`/`setView` are unchanged. **Self-tests:**
  `m38_tools icons=1 cursors=1 slots=1 guard=1` (exit codes 95–98) and
  `m38_panels rail=1 strip=1 history=1` (codes 99–101), both identical on the
  no-argument and `two_layers.psd` runs; all earlier lines/codes are unchanged
  (`m23_toolbox dock=1 buttons=24 fgbg=1` still passes). Two fixes during the
  milestone: `assets/cursors/tool.move.svg` was clobbered by a parallel asset
  agent and restored from git before the required compound rewrite, and
  `tool.brush`/`tool.pencil` had no cursor assets at all (the original defect)
  and were added. **Honest limits:** dock-tab window icons were not wired (only
  the rail, Layers strip, and History snapshot get icons); the 3D-object,
  3D-camera, and Count slots are Extended-only and every all-unimplemented slot
  is disabled; the blur/sharpen/smudge slot has no default letter (flyout or
  Alt-click only); the toolbox 1-/2-column toggle is not present (single column
  only); and the **functionality** of the 61 new catalogue entries is not
  implemented — assets and disabled UI only. Verified: `cargo fmt --all
  --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean;
  **568 tests, 0 failed, 7 ignored** (unchanged — no Rust change; the raw
  `cargo test --workspace` ignored count is 8 with the pre-existing
  `pictura-render` doctest); `openspec validate m38-icon-cursor-library
  --strict` valid and `openspec validate --all --strict` 60/60; both self-tests
  exit 0. The M38 change ADDs to `application-shell` and `layers-panel` and
  MODIFIES `tool-framework`, `panel-rail`, `svg-cursors` (no new capability →
  **59** capabilities after archive). Because it claimed the M38 number, the
  Layers-panel program's stages shift by one: **M39** panel anatomy, **M40**
  filtering/search, **M41** remaining management ops, **M42** styles/effects,
  **M43** smart objects / vector masks / artboards / layer comps; the deferred
  canvas-performance tracks stay by name.

- **M39 — panel anatomy** (the Layers-panel program's panel-anatomy milestone
  after the M38 interruption; see
  `docs/dev/layers-panel-program.md`; OpenSpec change `m39-panel-anatomy`, brief
  `docs/dev/m39-panel-anatomy.md`). The Layers panel becomes a full expandable
  tree over the document's existing `Layer.children` hierarchy, and edits become
  path-addressed and selection-based. **Path/tree core**
  (`crates/pictura-render/src/document_ops/layer_ops.rs`): a frozen path grammar
  (`segment *("/" segment)`, digits with no leading zeros, bottom-first),
  `resolve_path`/`resolve_path_mut`/`parent_path`, `flatten_rows ->
  Vec<(String,u32)>` (depth-first, topmost-first `(path, depth)` pairs), and
  `is_background` (the single source of truth; the app's `is_background_layer`
  delegates). Batch ops `set_visible_paths`/`apply_visibility`/`set_blend_paths`/
  `set_opacity_paths`/`set_fill_paths`/`set_lock_paths`/`set_color_paths`/
  `delete_paths`/`duplicate_paths`/`group_paths`/`ungroup_paths`/`rename_path`/
  `move_path`/`add_layer_in`/`add_group_in` apply the frozen per-node skip vs
  whole-op refuse table (only `group_paths` refuses the whole op); structural ops
  resolve-then-apply deepest-first with ancestor-descendant dropping so indices
  never invalidate. `move_path` refuses the Background and fully-locked layers
  (the LAY-002 ruling). **Bridge** (`cxxqt_object/impl_layers.rs`): `layer_row_count` + 17
  `layer_row_*` getters over the projection (path, depth, name, kind, visible,
  blend, opacity, fill, lock, color, clipping, has-mask, has-adjustment,
  expandable, child-count, thumbnail, mask-thumbnail); `set_layer_name_path`/
  `move_layer_path`; `QStringList` batch mutators for visible/blend/opacity/fill/
  lock/color plus `apply_visibility(paths,label)`, `delete_layers`,
  `duplicate_layers`, `group_layers`, `ungroup_layers`; `add_layer_in`/
  `add_group_in`. Undo contract: `recomposite()` then `record(label)` only when
  the changed count is non-zero; a zero-change call records nothing and emits no
  `changed`. Labels per the design (`Set Visibility`, `Solo Visibility`,
  `Restore Visibility`, `Group Layers`, …). Every legacy top-level
  `layer_*(i)`/`set_layer_*(i)`/M37 op is left **byte-identical** — the new path
  surface is the tree implementation and nothing calls legacy after the panel
  migration — with the deliberate asymmetry that legacy `move_layer` still swaps
  unconditionally while `move_layer_path` refuses. Thumbnails:
  `layer_row_thumbnail(i,size,entire_document)` (layer bounds vs
  document-positioned) and `layer_row_mask_thumbnail`; groups and adjustments
  return a null image (the delegate draws a folder glyph). **Panel**:
  `LayersModel` becomes a `QAbstractItemModel` over the flat projection (roles,
  `internalPointer` path, `CheckStateRole`→visibility, `EditRole`→rename);
  `LayerRowDelegate` paints the eye (with a press hit-test), thumbnail/folder
  glyph, name, color swatch, clip indent + base underline, mask thumbnail, and
  `fx` badge (null-asset safe); a `QTreeView` with `ExtendedSelection`,
  `SelectRows`, `uniformRowHeights`, `expandsOnDoubleClick(false)`,
  `dragEnabled(false)`; panel-side expansion (`QSet<QString>`, default collapsed,
  new groups expanded) and path-based re-selection; multi-selection routing for
  the header controls and Delete/Duplicate/Group/Ungroup (one undo step each);
  solo (`Alt`-click) snapshots visibility and uses `apply_visibility` for a
  one-step exact restore; rename `Tab`/`Shift+Tab` moves to the next/previous
  visible row with no wrap; tooltips `"<name> (<kind>)"`; Panel Options (Medium /
  Entire Document / Expand New Effects on) persisted in session **schema v3**
  (`layersThumbSize`/`layersThumbContents`/`layersExpandNewEffects`); the panel
  menu (`layersPanelMenu`) + row context menu with the wired commands only and a
  `Color Label` submenu; eye right-click show-only/show-all. The seven-button
  strip is unchanged and reordering is menu-only. **Fixes/decisions:**
  `PicturaMainWindow::saveSession()` now loads before writing so the v3 fields
  are not clobbered; `layerTooltip` now always returns `"<name> (<kind>)"` (the
  old helper predated M39 and would have failed the new tooltip check).
  **Self-tests:** `m39_tree` (102/103), `m39_multi` (104/105), `m39_solo` (106),
  `m39_rename` (107), `m39_options` (108), `m39_badges` (109), `m39_menus` (110),
  `m39_tooltip` (111), `m39_strip` (112); all earlier lines/codes unchanged.
  **Honest limits:** the clip-indent rendering is not positively proven (no
  bridge operation can set `Layer.clipping` and no fixture has a clipped layer —
  `m39_badges clip=1` only proves the role is plumbed and `clipBase` is not
  spuriously set); group thumbnails are folder glyphs (no group composite);
  drag-reorder is explicitly deferred to M41; multi-row move is deferred;
  expansion is session-only (not persisted); solo is one undo step per direction.
  Verified: `cargo fmt --all --check` and `cargo clippy --workspace --all-targets
  -- -D warnings` clean; **588 tests, 0 failed, 7 ignored** (up from 568/7; the
  ignored set is unchanged, so the raw `cargo test --workspace` ignored count is
  8 with the pre-existing `pictura-render` doctest); `openspec validate
  m39-panel-anatomy --strict` valid and `openspec validate --all --strict`
  60/60; both self-tests exit 0. The M39 change MODIFIES `layers-panel` (no new
  capability → **59** capabilities after archive). Deferred within the program:
  filtering/search (M40), the remaining management ops and drag-reorder (M41),
  styles/effects (M42), smart objects / vector masks / artboards / layer comps
  (M43).

- **M40 — CS6 Tools panel** (a second user-requested interruption to the
  Layers-panel program; OpenSpec change `m40-tools-panel`, brief
  `docs/dev/m40-tools-panel.md`). The Tools panel becomes CS6-shaped. **Flyout:**
  `ToolSlotButton` paints a 5 px filled triangle at the **bottom-right** when the
  slot's group has ≥2 members (counting unimplemented members); `MenuButtonPopup`
  and `setMenu` are gone so the icon is centred with no stock arrow. A **300 ms**
  hold timer opens the group `QMenu` below the button (screen-clamped);
  **right-click opens immediately**; a release before the timeout selects
  normally; `Alt`+click still cycles. Menu items carry the group's key via
  `QAction::setShortcut` + `setShortcutVisibleInContextMenu(true)` +
  `setShortcutContext(Qt::WidgetWithChildrenShortcut)` (no window-global shortcut,
  so a disabled item cannot steal the key); unimplemented members stay disabled
  with the "not implemented yet" tooltip. `refreshSlot()` no longer calls
  `setShortcut`. **Columns:** a custom `QDockWidget` title bar (`Tools` label + a
  flat double-arrow button, `objectName` `toolsColumnToggle`) toggles one/two
  columns; the icon shows the **target** layout
  (`assets/icons/panel.columnsTwo.svg` in one column, `panel.columnsOne.svg` in
  two — both new, 24×24 stroke-`#c8c8c8`, added to the regenerated qrc); reflow is
  row-major `(i/2, i%2)` with `minimumWidth` 66 → 104; the fg/bg widget and Screen
  Mode button stay pinned below the slots. **Standalone dock:**
  `setAllowedAreas(Left|Right)` and `setFeatures(Movable|Floatable|Closable)`;
  tabification refused by an event filter plus a reactive
  `PicturaMainWindow::ensureToolsNotTabified()` (float → re-add → show) driven from
  `dockLocationChanged`/`topLevelChanged`. **Shift cycling:**
  `toolShortcutKeys()`/`toolGroupForKey()` in `tools.{h,cpp}` and
  `Toolbox::handleToolKey(key, shift)`; `frame.cpp` registers one plain and one
  `Shift`+letter `QShortcut` per distinct key routed to it, replacing the
  hard-coded B / Shift+B `cyclePaintTool`. A plain letter activates the slot's
  current member; `Shift` cycles to the next **implemented** member (skipping
  unimplemented, wrapping); an all-unimplemented group is a no-op;
  `setShiftKeyForToolSwitch(false)` makes the plain letter cycle. Gated on a new
  session preference `useShiftKeyForToolSwitch` (default **true**) — **no UI
  yet**; M41 adds the Preferences dialog (General + Interface) and gives it a home
  alongside `Auto-Collapse Iconic Panels`. **Session v4:** `toolsColumns` (1|2)
  and `useShiftKeyForToolSwitch` with per-key defaults;
  `PicturaMainWindow::saveSession()` still loads-then-writes so unknown keys
  survive; `columnsChanged` persists. **Self-tests:** `m40_columns` (113/114),
  `m40_flyout` (115), `m40_keys` (116), `m40_shift` (117, driven through the real
  `QShortcut`/`QKeyEvent` path), `m40_dock` (118), `m40_session` (119);
  `m23_toolbox` tightened from a loose `buttons < 10` to `buttons != 25` plus
  `toggle=1` (the new title-bar toggle is found by `objectName` and is not one of
  the 23 slots). All earlier lines/codes unchanged. **Honest limits:** Qt has no
  clean per-dock tabify veto — the event filter only covers the toolbox body and a
  drop can briefly tabify before the reactive re-dock; the
  **right-click-immediate and quick-release-select timing are not self-tested**
  (the paths exist but only triangle/member-split/popup-placement are asserted);
  the preference has no UI until M41; the two-column width is 66/104 rather than
  the sketched 34/64 (the swatch + dock chrome need the width); `m40_columns`
  asserts widening via `minimumWidth` (the actual mechanism) with actual width
  only non-decreasing. No Rust changes: **588 tests, 0 failed, 7 ignored**
  (unchanged; the raw ignored count is 8 with the pre-existing
  `pictura-render` doctest). Verified: `cargo fmt --all --check` and
  `cargo clippy --workspace --all-targets -- -D warnings` clean;
  `TASK_ALLOWS_DOCS=1 bash scripts/verify-fast.sh` → `verify-fast: OK`
  (588 tests, 0 failed); both self-tests exit 0 with all m20–m40 lines `=1`;
  `openspec validate m40-tools-panel --strict` valid and
  `openspec validate --all --strict` 60/60. The M40 change MODIFIES/ADDs to
  `tool-framework` and ADDs to `application-shell` (no new capability → **59**
  capabilities after archive).

- **M41 — CS6 panel column** (a third user-requested interruption to the
  Layers-panel program; OpenSpec change `m41-panel-column`, brief
  `docs/dev/m41-panel-column.md`). The fifteen right-hand `QDockWidget`s are
  replaced by a custom column of plain content widgets. **Hosts:** `PanelGroup`
  is a `QTabWidget` with `setTabPosition(QTabWidget::North)` forced; the tab
  text is the panel title and there is **no separate group label** (a
  single-panel group still shows its tab). `PanelColumn` is a `QScrollArea` over
  a vertical `QSplitter` of groups with no hard panel minimums, so the window
  shrinks freely. **Groups (CS6 Essentials, fixed):** `Color | Swatches |
  Styles`; `Adjustments` (plus a hidden `Properties` tab); `Layers | Channels |
  Paths`; `Navigator | Histogram | Info`; iconic `History`, `Actions`; and a
  hidden overflow group `Gradients | Patterns | Libraries` (kept reachable from
  `Window → Panels`). `Styles` is a new placeholder; Properties is folded into
  Adjustments. **Width toggle:** a `panelColumnToggle` double-arrow in the column
  header switches **normal ⇄ iconic**. **Iconic mode** is a vertical icon strip
  with group dividers, labels-on-widen (threshold 120 px), and a `Qt::Popup`
  flyout per panel that reparents the panel in and restores it on close.
  Per-group **Collapse to Icons** reuses the same component; **Minimize** rolls a
  group up to its tab bar (distinct from iconic). **Tab context menu** (right-click
  a group's tab bar), exactly: `Close`, `Close Panel Group`, `Minimize`,
  `Collapse to Icons`, sep, `Auto-Collapse Iconic Panels` (checkable),
  `Auto-Show Hidden Panels` (checkable), sep, `Interface Options…`. **Drag &
  drop:** in-group reorder, cross-group regroup, between-groups insert (new
  group), with a **3 px `#2a7fff` drop indicator** (`panelDropIndicator`) — a
  vertical marker at the tab index or a full-width horizontal bar at the group
  boundary; hidden on commit/cancel/out. **Tear-off:** leaving the column floats
  the source group in a `Qt::Tool` `panelFloat`; dragging it back re-docks at the
  index; the float is hidden/`deleteLater`-ed when emptied. Drops are
  remove-then-insert, so a panel is never double-parented. **M24 `PanelRail`
  deleted** (`panel_rail.{h,cpp}` removed from disk and CMake, creation + five
  `setPanelChecked` connections gone). `Window → Panels` is the single visibility
  path; the `m24_rail` self-test (code 63) was repointed to it (`actions=5
  toggled=1 norail=1`). **Tools panel fixes:** the `"Tools"` title label is gone
  (the `toolsColumnToggle` stays); the hard-coded 66/104 widths became content-fit
  **34 / 65** px; `ForegroundBackgroundWidget` scales to the column (30/40) and
  no longer widens the dock. M40 flyout/hold/right-click/shortcuts/Shift-cycling/
  left-right dock all intact. **Session v5:** `panelRailMode` (`normal`|`iconic`,
  default normal), `railWidth` (0 = derive), `autoCollapseIconic` (default
  **false**), `autoShowHidden` (default **false**), `panelGroups` (per-group
  `order`/`visible`/`minimized`/`collapsed`), `schemaVersion` 5.
  `saveSession()` starts from the parsed on-disk object so unknown keys survive;
  a v4 store loads with v5 defaults. **Preferences dialog** (new
  `preferences_dialog.{h,cpp}`, `objectName` `preferencesDialog`, modeless, page
  list + `QStackedWidget`) with exactly two real pages, **General** (the real
  brightness setting) and **Interface** (`Use Shift Key For Tool Switch`,
  `Auto-Collapse Iconic Panels`, `Auto-Show Hidden Panels`). Wired to
  `Edit → Preferences → General` (`edit.preferences.general`, implemented) and
  `→ Interface` (`edit.preferences.interface`, implemented); the other nine
  Preferences leaves stay disabled no-ops (enablement unchanged).
  `PanelColumn::interfaceOptionsRequested()` opens Interface. **M40's UI-less
  preference now has a home:** `Use Shift Key For Tool Switch` →
  `Toolbox::setShiftKeyForToolSwitch` → `handleToolKey`, persisted.
  **`Auto-Collapse Iconic Panels`** (default off) — when a flyout closes and the
  column is in normal mode it returns to iconic; inert when already iconic; no
  per-panel expand-in-place state machine. **`Auto-Show Hidden Panels`** (default
  off) — the iconic strip includes hidden panels' icons and opening one reveals
  it; inert: no hover-at-edge gesture. **Self-tests**, new codes **120–130**:
  `m41_tabs`(120), `m41_width`(121), `m41_iconic`(122), `m41_menu`(123),
  `m41_minimize`(124), `m41_prefs`(125), `m41_drag`(126), `m41_tearoff`(127),
  `m41_session`(128), `m41_tools`(130). Code **129** (`m41_rail`) was folded into
  the repurposed `m24_rail` rather than allocated. Earlier codes all still pass
  with adapted internals (`m24_groups`/`m24_panels` now assert `PanelColumn`
  membership via `groupOfForTest`, `m38`/`m39`/`m40_dock` use
  `QWidget`/`PanelColumn`). **Harness fix (not product):** `main.cpp` now forces
  `QT_QPA_PLATFORM=xcb` for `--self-test` when both `DISPLAY` and
  `WAYLAND_DISPLAY` are set and no platform is pinned. Without it, a live Wayland
  session leaked through `xvfb-run`, Qt picked Wayland, and a
  programmatically-opened `QMenu` popup could not grab and was dismissed — making
  `m40_flyout` (code 115) flaky (**3/8** pass under Phase A; clean HEAD measured
  **8/8** only by luck, **3/12** in the agent's sample). Deterministic **5/5**
  no-arg after the guard. **Honest limits:** multi-monitor tear-off untested and
  the float is not screen-clamped; no cross-process drag; no translucent drag
  ghost (the float is the feedback); float chrome is a plain WM-framed
  `Qt::Tool`; Escape-cancel is not key-bound (`cancelDrag()` is programmatic
  only); drop-on-gap needs the ~4 px splitter or a group's top/bottom half;
  torn-off floats are not serialized; `panelGroups` keys groups by first-ever
  panel objectName and skips a stale name; iconic label threshold (120 px) and
  popup size are unsourced constants; the frame's own minimum width is ~776 px
  from the M40 options bar (column min is 64 px) so shrink-to-nothing is
  untestable; `Styles` has no dedicated SVG asset; the group collapse-icon row is
  horizontal while the column iconic strip is vertical (shared flyout/button
  code, not the identical widget). No Rust/bridge/codec/compositor/PSD change.
  **Capability:** ADD `panel-column`; the M24 `panel-rail` requirement is
  REMOVED (its spec persists without the rail requirement); MODIFIED
  `application-shell`, `tool-framework`, `workspace-persistence`. 59 canonical
  specs → **60 after archive**. Verified: `cargo fmt --all --check` and
  `cargo clippy --workspace --all-targets -- -D warnings` clean;
  `TASK_ALLOWS_DOCS=1 bash scripts/verify-fast.sh` → `verify-fast: OK`;
  **588 tests, 0 failed, 7 ignored** (unchanged — no Rust change; the raw
  ignored count is 8 with the pre-existing `pictura-render` doctest); both
  self-tests exit 0 with the new `m41_*` lines and every earlier m20–m40 line
  unchanged; `openspec validate m41-panel-column --strict` valid and
  `openspec validate --all --strict` 60/60 pre-archive. The M41 change MODIFIES
  `application-shell`, `tool-framework`, `workspace-persistence` and REMOVES the
  M24 `panel-rail` requirement, ADDing the `panel-column` capability (→ **60**
  capabilities after archive).

- **M42 — panel refinements** (a fourth user-requested interruption to the
  Layers-panel program; OpenSpec change `m42-panel-refinements`, brief
  `docs/dev/m42-panel-refinements.md`, research `docs/dev/m42-panel-menus.md`).
  **Phase A — chrome fixes.** The normal-mode `PanelColumn` minimum width is the
  widest visible group's `sizeHint` clamped to `[180,320]` (iconic-strip floor
  40); entering iconic now starts at the smallest possible width instead of
  keeping the prior splitter width. Iconic-strip buttons grew `24→30` (pixmap
  `16→20`) and Tools slots `30→32` (icons `20→22`). The floated Tools dock hugs
  its content height (the trailing stretch is zeroed on `topLevelChanged(true)`
  and the body layout invalidated so the two-column floor is not cached) and
  stays width-tight (1-col min = content = 36, 2-col 69).
  `ForegroundBackgroundWidget` gains the CS6 double-arrow swap control top-right
  (the default-colors X is kept) wired to the **`X`** key (was free; not in the
  tool catalogue or the frame's shortcut list). **Menu-bar overlay root cause
  (item 8):** `frame.cpp` constructed `actionsPanel_` **twice**; the first
  `PlaceholderPanel("Actions")` was never added to a group, so it remained a
  direct child of the main window and painted its dim "Actions" label over
  `File`/`Edit` — that was the "File Act…" artifact (confirmed against
  `/tmp/opencode/clean_wide.png`; a fresh `XDG_STATE_HOME` reproduced it, so it
  was not the persisted layout). Fixed by deleting the duplicate.
  Defence-in-depth: `saveState()` records `layoutRevision` and
  `restoreStoredLayout` discards a stored layout whose revision mismatches
  (`kLayoutRevision=2`), so the pre-M41 dock blob is no longer restored (**the
  session gains `layoutRevision`; no schema bump**). **Phase B — compact-strip
  drag** reuses the M41 `beginPanelDrag`/`updateDrag`/`commitDrop`/`resolveDrop`
  path (no second drag system): `resolveDrop` gained an `onStrip` branch with
  `stripInsertionIndexAt`, `applyStripDrop` rewrites order via
  `PanelGroup::setPanelOrder` or reuses `takePanel`/`insertPanel`/
  `cleanupEmptyGroup` across groups, and a dedicated `stripIndicator_` draws the
  blue line (the normal indicator lives in the hidden scroll viewport). Commit is
  queued (`Qt::QueuedConnection`) so the strip can rebuild after the button's
  event returns. **Phase C — compact flyout.** Opens on the **inner** side,
  derived from the column's geometry vs its window (right-edge column ⇒ popup to
  the left), clamped to `QScreen::availableGeometry`. The open icon is a
  checkable/pressed strip button cleared on restore. The popup is now
  **group-styled**: a `panelFlyoutHeader` (`panelFlyoutTitle` + stretch +
  `panelFlyoutClose`) above the detached panel, with a new
  `assets/icons/panel.closeChevron.svg` (double right chevron, 24×24 `#c8c8c8`,
  qrc regenerated) for the close button. Still `Qt::Popup` (click-away), still
  reparents the panel back exactly once. **Phase D — per-widget header menu.** A
  `▾` `QToolButton` (`panelWidgetMenu_<panelName>`) is installed as
  `QTabWidget::setCornerWidget(..., Qt::TopRightCorner)` on each `PanelGroup`, and
  follows the **current tab** (menu + tooltip switch on `currentChanged`); hidden
  when the current panel has no menu table. Menu contents are transcribed from
  `docs/dev/m42-panel-menus.md` (CS6 panel fly-outs, order best-effort;
  `[toggle]`/radio entries checkable). Unimplemented entries ship **disabled**
  with `"<label> — not implemented yet"`. `Close`/`Close Panel Group` are
  excluded (they stay on the M41 tab menu). **Wired:** Layers — New Layer…,
  Duplicate Layer/Group…, Delete Layer/Group, New Group…, Group Layers, Ungroup
  Layers, Hide Layers, Arrange ▸ Move Layer Up/Down, Panel Options…; History —
  Step Forward/Backward, New Snapshot…; Adjustments — Invert, Posterize,
  Threshold, Brightness/Contrast, Hue/Saturation. Everything else disabled (all
  of Channels, Paths, Color, Swatches, Styles, Navigator, Histogram, Info,
  Actions, Properties; Channels/Paths `Panel Options…` stay disabled — no options
  dialog exists for placeholder panels). Gradients/Patterns/Libraries get no
  header button (not CS6 panels: picker pop-ups / CC-only Libraries). **Phase E —
  in-window float overlay.** `PanelFloat` is no longer a `Qt::Tool` top-level: it
  is a plain child of the main window (`Qt::Widget`, `WA_StyledBackground`,
  `#panelFloat{background:#3a3a3a;border:1px solid #555}`), raised, clipped to
  `centralWidget()`'s rect and clamped there on every header drag (`moveFloat`).
  It is parented to `window()` **not** `centerSplitter`, because
  `QSplitter::childEvent` auto-inserts non-window children as panes. Re-dock is
  the existing `applyGroupDrop` (remove-then-insert; overlay destroyed once
  emptied). `m41_tearoff` is unchanged and proves the same claim (group left the
  column, contains its panels, re-docks, no float remains); windowness is now
  proven separately by `m42_float_overlay` (`!isWindow()`). **Self-tests**
  131–139: `m42_minwidth`(131), `m42_iconic`(132), `m42_dragstrip`(133),
  `m42_flyout`(134), `m42_widgetmenu`(135), `m42_float_overlay`(136),
  `m42_fgbg`(137), `m42_menubar`(138), `m42_tools`(139); `m41_width`(121) was
  amended (output format unchanged) and all 120–140 lines are `=1`. **Honest
  limits:** the per-panel menu order/separators are the research doc's
  best-effort reconstruction; the disabled entries are stubs (no invented
  dialogs); Layers `Panel Options…`/History `New Snapshot…` open modal dialogs so
  the test asserts structural wiring, not execution; `New Snapshot…` is marked
  disabled in the research doc but was wired to the existing snapshot behavior;
  Adjustments route through the Layers panel's view (the Adjustments placeholder
  has no view handle); the float overlay does not persist position, is not
  re-clamped on window resize, and cannot cover the docked Tools panel (clamped
  to the central-widget rect); min widths/icon sizes are chosen constants, not
  CS6 metrics; `m42_tools` tolerates ±8 px on the float height under xvfb; the
  stale-layout revision guard is defence-in-depth (proven not to be the overlay's
  cause); drop-on-stack from the strip requires the column to be expanded
  mid-drag. No Rust change: **588 tests, 0 failed, 7 ignored** (unchanged; the
  raw ignored count is 8 with the pre-existing `pictura-render` doctest).
  **Capability:** MODIFIES `panel-column`, `tool-framework`, `application-shell`;
  **no new capability** → **60** canonical specs after archive. Verified:
  `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --
  -D warnings` clean; `TASK_ALLOWS_DOCS=1 bash scripts/verify-fast.sh` →
  `verify-fast: OK`; both self-tests exit 0 with all m42 lines `=1` and every
  earlier m20–m41 line unchanged; `openspec validate m42-panel-refinements
  --strict` valid and `openspec validate --all --strict` 61/61.

- **M43 — panel multicolumn** (a fifth user-requested interruption to the
  Layers-panel program; OpenSpec change `m43-panel-multicolumn`, brief
  `docs/dev/m43-panel-multicolumn.md`). **Phase A — drag/chrome/float.** A tab
  press drags/floats **only that panel** (a one-panel float built via `takePanel`
  + a fresh wired `PanelGroup`), an empty-header press drags/floats the **whole
  group**; both work docked and for a group already floating
  (`PanelColumn::beginPanelDrag`/`createFloat` payload-aware). The panel tab bar
  carries `objectName` `panelTabBar` and scoped QSS — selected tab `${base}`
  (identical to the `QTabWidget::pane`/widget background), unselected
  `${window}` (hover `${hover}`); the document bar is `documentTabBar`,
  unaffected (pixel sample: active `srgb(35,35,35)` = base, inactive
  `srgb(43,43,43)` = window). The corner `▾` is fixed with `ElideRight` +
  `setExpanding(false)` + the corner width added to `updateMinimumWidth`, so it
  is fully visible at the column minimum. `placeFlyout` uses the **actual button
  geometry** and the column's side, clamping only the inner coordinate so it can
  never flip outward or overlap the button. The Tools dock gets
  `setFixedWidth(content)` + `QSizePolicy::Fixed` horizontal + a `Resize`
  event-filter clamp (belt-and-braces for QMainWindow's internal splitter); the
  separator no longer resizes it, min == max == content width in 1- and 2-column
  modes and while floating, with the M42 float-height and the M40 standalone-dock
  contract intact. **`D`** resets fg/bg to default (black/white) through the
  existing `resetColors()`; the M42 `X` swap is kept (`D` was free — not in the
  tool catalogue). Compact icon buttons grew again (30 → 34 px, pixmap 24).
  **Phase B — multi-column host.** `centerSplitter_` now hosts an ordered set:
  left `PanelColumn`s, the document tabs (`objectName` `documentTabs`), right
  `PanelColumn`s; stretch stays on the tabs. `PanelColumn::side()` is derived
  from the splitter index vs `documentTabs` (not geometry). Columns are
  **created on drop** (`createPanelColumn(side)`) and **removed when empty**
  (`removeColumnIfEmpty`, dynamic-only), with the frame owning the wiring
  (`stateChanged`, `interfaceOptionsRequested`). `resolveDrop` gained a
  `DropKind` enum — `Reorder`, `IntoGroup`, `AboveGroup`, `BelowGroup`,
  `OnStrip`, `NewColumnLeft`, `NewColumnRight`, `Outside` — with a **28 px outer
  band / over-the-Tools-dock** rule for new columns, group top/bottom halves for
  above/below, tab-bar hits for into-group, cross-column `IntoGroup` delegation,
  and compact mode handled first so strip drags still reorder. One `#2a7fff`
  indicator marks every candidate. `flyoutSide()` is now `side() == Right ?
  "left" : "right"`. `Window → Panels` targets the owning column;
  `setPanelsHidden`/screen modes iterate all columns. **Phase C — session v6.**
  `panelColumns: [{side, order, groups:[{name, order, visible, minimized,
  collapsed}]}]` with `schemaVersion` **6**; the legacy flat `panelGroups` is
  still written as a mirror and a **v5 store loads as a single right-hand
  column** (synthesised when `panelColumns` is absent); load-then-write keeps
  unknown keys; the M42 `layoutRevision` guard and all v4/v5 keys survive.
  `applyPanelSession` rebuilds N columns and re-applies rail mode to all;
  `clearDynamicColumns` for re-apply. `m41_session`'s schema assertion changed
  from `==5` to `>=5` (the schema advanced — the only earlier-check change).
  **Self-tests** 140–151: `m43_tabdrag`(140), `m43_tabcolors`(141),
  `m43_corner`(142), `m43_newcolumn`(143), `m43_intogroup`(144),
  `m43_boundary`(145), `m43_singlefloat`(146), `m43_tools`(147),
  `m43_icon`(148), `m43_flyout`(149), `m43_dreset`(150), `m43_session`(151);
  all older codes green. **Honest limits:** cross-column drops delegate only
  `into-group` — a whole group dropped onto another existing column (not the
  outer edge) resolves as tear-off rather than a cross-column move; new columns
  are only allocated at the workspace outer edges or over the Tools dock;
  compact "above the first group" boundary is unreachable (the gap resolves as
  on-strip); the vertical group order **inside** a column is written but not
  re-applied on restore (pre-existing M41 behaviour); one workspace-wide
  `panelRailMode`/`railWidth`; float existence/position not persisted; only the
  left+right pair is self-tested (not multiple columns on the same side); the
  primary right column is never removable (an all-left layout leaves it present
  but empty); the Tools `setFixedWidth` clamp is verified under xcb only;
  `headerCornerWidthForTest` over-reserves when the `▾` is hidden. No Rust
  change: **588 tests, 0 failed, 7 ignored** (unchanged — no Rust change; the raw
  ignored count is 8 with the pre-existing `pictura-render` doctest).
  **Capability:** MODIFIES `panel-column`, `application-shell`, `tool-framework`,
  `workspace-persistence`; **no new capability** → **60** canonical specs after
  archive. Verified: `cmake --build` clean; both self-tests exit 0 (no-arg 5/5)
  with all m43 lines `=1` and every earlier m20–m42 line unchanged except the
  `m41_session` `>=5` schema assertion; `cargo fmt --all --check` and `cargo
  clippy --workspace --all-targets -- -D warnings` clean; `TASK_ALLOWS_DOCS=1
  bash scripts/verify-fast.sh` → `verify-fast: OK`; `openspec validate
  m43-panel-multicolumn --strict` valid and `openspec validate --all --strict`
  61/61.

- **M44 — panel/theme polish** (a sixth user-requested interruption to the
  Layers-panel program; OpenSpec change `m44-panel-theme-polish`, brief
  `docs/dev/m44-panel-theme-polish.md`). **Phase A — new-document canvas bug
  (E1).** On startup with no PSD the canvas showed the M0.5 GPU demo's repeating
  black/white/green/red banding instead of the white scratch document. Root
  cause: `PictureView::render_gpu` (the M0.5 GPU spike now in
  `cxxqt_object/impl_core.rs`)
  offscreen-rendered `crate::gpu::render_gradient` and **assigned it directly to
  `rust.image` without touching `rust.doc` or setting `display_dirty`**; startup
  creates a white scratch document, calls `render_gpu()`, then presents
  `view->image()`, so the canvas showed the gradient while the document composite
  stayed white — any later recomposite derives the display from `doc.composite`,
  which is why moving a layer turned it white. Fix: removed the image
  assignment; `render_gpu` is now a pure smoke probe (`Rendered { distinct, .. }`)
  that never mutates the display image (a pre-fix capture had 8634 distinct
  canvas colours, 1 after). New Rust regression
  `crates/pictura-render/tests/gpu_parity.rs::fresh_white_document_composites_uniform_white_twice`
  (37×23, CPU oracle + GPU first and second composite byte-equal white; self-skips
  without an adapter, **runs** on the RTX 3090). The startup check that previously
  asserted the gradient was non-blank became `fresh_white=1`; self-test **153**
  `m44_newdoc white=1 uniform=1 immediate=1`. **Phase B — theme/borders/style.**
  **W1** the collapse chevrons were inverted; swapped the `panel.columnsOne`/
  `columnsTwo` mapping so the toggle shows the correct target state (**154**
  `m44_chevrons`). **W2** the "last item active" default was
  **`PanelColumn::restorePanelState`**: replaying `PanelGroup::setPanelVisible`
  left the **last visible** tab current after a session restore (every normal
  launch after the first), fixed with `setCurrentToFirstVisible()` at the end of
  each group's restore (**155** `m44_defaultactive`). **W3** tab colours: the
  measured visible widget surface is `${window}` (`#2b2b2b`), not the QSS pane;
  active tab = `${window}`, inactive = `${base}` (`#232323`), panel groups get
  `QTabWidget#panelGroupTabs::pane { background: ${window} }`, document tabs
  unchanged (**156** `m44_tabswap`; the M43 `m43_tabcolors` direction corrected).
  **W6** group divider: splitter `setHandleWidth(kGroupDividerWidth = 6)` +
  `QSplitter#panelColumnSplitter::handle { background: ${border} }` (**159**).
  **C2** the compact-strip divider is now 2 px `${border}` (dark grey) instead of
  white (**161**). **C4** strip labels now **elide as soon as there is any room**
  (`QFontMetrics::elidedText`), replacing the fixed `>= 120 px` show/hide
  threshold (**164**). **F1/F2** document tab bar: `QTabBar#documentTabBar {
  border-right: 1px solid ${border}; border-top: 0 }` plus
  `QTabWidget#documentTabs::pane { border-top: 0 }` (**165**; F2 is a no-op at the
  sampled pixels — the only top line was the options bar's own bottom border).
  **S1** darker grey 1 px `${border}` on `#toolsPanel`, `#panelGroupTabs`,
  `#panelColumnIconStrip`, `#panelColumnContainer` (**166**). **S2** removed the
  inline `#panelFloat`/`#panelIconFlyout`/`#panelFlyoutHeader` stylesheets;
  docked, popup and floating now all take the `${window}` surface + `${border}`
  border from theme. Shared constants `Theme::kPanelBorderWidth = 1`,
  `Theme::kGroupDividerWidth = 6`. **Phase C — drag/dock.** **T1** the floating
  Tools dock height is now locked (both axes fixed while floating; the M40
  four-area dock contract is intact — `m40_dock` now asserts all four areas).
  **T2/W5** `CreatePanelColumn(side, anchor)` inserts immediately before/after an
  **anchor** column (not only the splitter ends); `columnEdgeAnchorAt()` yields a
  new-column target beside any `PanelColumn`; `newColumnSideAt` maps the Tools
  dock's side; the Tools dock uses `setAllowedAreas(AllDockWidgetAreas)` with a
  width-or-height lock per dock side. Same M43 `DropKind` resolver, one `#2a7fff`
  indicator (**158** `m44_docksides toolbar=1 column=1 workspace=1 float=1`).
  **W4** `createFloat` no longer cleaned the source group on a tab drag (the
  source tab bar owns the implicit mouse grab), so the float now **tracks the
  cursor until release**; the source is cleaned on commit/cancel (**157**
  `m44_floatdrag`). **C3** `resolveIconicDrop` proximity rule: icon hit → into
  that group; divider → new group at that boundary; inside a group container →
  on-strip/boundary; otherwise new group at the top/bottom end. Each group is a
  `panelIconGroup` container with a `panelIconGroupGrip` (`•••`) drag handle above
  its icons, and the container gets a `${base}` background/border so the icons
  read as one group (**162** `m44_compactdrop`, **163** `m44_draghandle`).
  **160** `m44_popupstyle parity=1` — the flyout shares the docked group's styling.
  **Self-tests** 153–166: `m44_newdoc`(153), `m44_chevrons`(154),
  `m44_defaultactive`(155), `m44_tabswap`(156), `m44_floatdrag`(157),
  `m44_docksides`(158), `m44_divider`(159), `m44_popupstyle`(160),
  `m44_compactdivider`(161), `m44_compactdrop`(162), `m44_draghandle`(163),
  `m44_elide`(164), `m44_filebar`(165), `m44_panelborder`(166); exit **152**
  remains the user's headless-platform check; all earlier m20–m43 lines unchanged
  except the corrected `m43_tabcolors` direction and the strengthened `m40_dock`.
  **Honest limits:** widget columns dock only left/right of another
  column/workspace (the host is a horizontal splitter) — top/bottom is supported
  for the Tools **dock** only; the compact "very close above/below ⇒ into" zone is
  the group container including the grip; multiple columns on one side with a
  column drag in flight remain best-effort; the M44 spec's parenthetical describes
  the pane as `${base}` while the measured surface/implementation uses `${window}`
  (the behavioural scenario holds); F2 is a no-op at sampled pixels; elide width
  is an approximate row allowance; the Rust regression pins the composite
  invariant but not the demo overlay itself (that is pinned by `m44_newdoc` + the
  startup `fresh_white` assertion). **Capability:** MODIFIES `panel-column`,
  `application-shell`, `tool-framework`, `document-canvas`; **no new capability**
  → **60** canonical specs after archive. Verified: `cmake --build` clean;
  `./build/pictura --headless --self-test` exit 0 with all `m44_*` `=1`; the PSD
  headless self-test exit 0; `cargo fmt --all --check` and `cargo clippy
  --workspace --all-targets -- -D warnings` clean; `cargo test --workspace`
  **589 tests, 0 failed, 7 ignored** (up from 588/7; the raw ignored count is 8
  with the pre-existing `pictura-render` doctest); `openspec validate
  m44-panel-theme-polish --strict` valid and `openspec validate --all --strict`
  61/61.

- **M45 — panel fixes** (a seventh user-requested interruption to the
  Layers-panel program; OpenSpec change `m45-panel-fixes`, brief
  `docs/dev/m45-panel-fixes.md`). **Phase A — Tools toolbar.** **T1 sizing:**
  `updateContentMetrics` previously fixed only one axis and read `sizeHint()`
  before the reflowed grid was active, while `topLevelChanged` latched a
  `floatHeight_`; the M43 width lock and M44 height lock could each keep a size
  from the previous column count, so 1↔2 read as 1-col ≈507 px and 2-col ≈862 px.
  Both axes now come from one content formula (`contentWidth` + a new
  `contentHeight` = title bar + margins + `rows*slot + gaps` + fg/bg + screen
  mode + body spacing), released and re-fixed after `layout()->activate()`;
  measured `w1=36 h1=862 w2=69 h2=507`, stable across the toggle. **T2:** the dock
  allowed areas revert to **left/right only** (M44 had `AllDockWidgetAreas`),
  keeping the M40 contract and the per-side lock. **T3:** the floating toolbar now
  docks **beside any widget column** via the existing `columnEdgeAnchorAt` + a new
  `PanelColumn::showEdgeDropIndicator(side)` (reusing the single `#2a7fff`
  indicator) and `commitToolboxDrop` (inserts at the anchor's splitter index); the
  toolbox title-bar drag emits `toolbarDragMoved`/`toolbarDragFinished`. It is
  hosted as a **central-splitter pane** (a `QDockWidget` cannot sit *between*
  columns); its title-bar re-float is not re-wired yet. **Phase B — widget
  panel.** **W1/W2/W3/W6 indicator correctness:** `DropTarget` gained
  `PanelColumn* owner`; `resolveDrop` sets it (`this` for local/workspace-edge/
  iconic, the **anchor** for beside-column, and the **destination column** for a
  delegated cross-column `IntoGroup`); `updateDrag` renders through the owner and
  clears the previous owner's line on change. Root cause: `resolveDrop` delegated
  the target but `showIndicatorFor` still ran on the **source column**, mapping
  the target's tab-bar x through the wrong `scroll_->viewport()` — so a
  right-hand target mapped off to the left (W1) or outside/clipped (W2). Tab
  inserts draw at `target.group->tabInsertionX(target.tabIndex)` (W3) and bottom
  boundaries at the last visible group's bottom edge (W6), in the owning column.
  `kEdgeInside` reduced 6→0 so an inside-edge tab insert is not mistaken for a
  new-column anchor. **W4 emptied column:** one `PanelColumn::maybeRemoveSelf()`
  (calls `frame->removeColumnIfEmpty(this)`) runs from `commitDrop`, `cancelDrag`,
  `closeGroup`, `showPanel(name,false)`, `restoreFlyoutPanel`;
  `removeColumnIfEmpty` **rehomes still-live groups into the primary column** via
  the new `PanelColumn::adoptGroup` (so panels survive for a later Window-menu
  show) and keeps a column that still owns a float. **W5 minimize actually
  collapses:** `PanelGroup::applyMinimize` clamps the **group's** `maximumHeight`
  (and size policy) to the tab-bar height, saving/restoring
  `savedGroupMaxHeight_`; the tab menu entry is state-derived — **"Expand Panel"**
  while minimized, "Minimize" otherwise (`tabMenuActionsForTest` applies the same
  substitution). **W7 never clip:** the scroll area's horizontal policy changed
  `AlwaysOff → ScrollBarAsNeeded`, tab text elides, the corner button keeps its
  reserved width. **W8 shared floor:** one `constexpr int kPanelMinWidth = 180`
  for every normal-mode column (the compact strip keeps `kIconStripMinWidth = 40`),
  replacing the per-column widest-derived floor, so columns share the floor and
  none can vanish. **Phase C — compact parity.** **C1 the popup is a real
  group:** `ensureFlyout` no longer builds a bespoke one-tab header;
  `openIconFlyout` finds the group, `PanelGroup::setCurrentPanel(clicked)`,
  records its index, and reparents the **whole `PanelGroup`** into the popup (it
  stays in `groups_`); `restoreFlyoutGroup` inserts it back at the recorded
  splitter index exactly once, guarded by `restoringFlyout_`/`flyoutGroup_`.
  Docked, popup and floating are now the **same widget instance** (same tabs, `▾`
  menu, minimize, drag, styling). **C2 group-drag line:** in compact mode a
  whole-group drag (`!dragIsPanel_`) anchors `stripIndicator_` to the target
  group's **container top −1** (above the `•••` grip dots), not the first icon
  button; panel drags keep the M42/M44 anchoring. **Self-tests** 167–179
  (`m45_tools_sizing` 167, `m45_tools_sides` 168, `m45_tools_beside_column` 169,
  `m45_indicator_side` 170, `m45_indicator_cross_column` 171,
  `m45_indicator_rightmost_tab` 172, `m45_empty_column_removed` 173,
  `m45_minimize_collapse` 174, `m45_indicator_bottom` 175, `m45_no_clip` 176,
  `m45_min_width_floor` 177, `m45_popup_group` 178, `m45_compact_group_line`
  179); exit **152** remains the headless-platform check. **Honest limits:** the
  toolbar is hosted as a central-splitter pane, so it cannot sit *between* columns
  as a dock and its title-bar re-float is not re-wired yet; a narrow column's
  group content can still scroll (W7's stated trade-off); `removeColumnIfEmpty`
  deliberately does not tear down a column that owns a live float; drag *from
  inside* the popup is not automatically tested; `PanelGroup::detachPanel`/
  `attachPanel`/`detached_` are now unused (left in place) and `theme.cpp`'s
  `panelFlyoutHeader` selector is dead (untouched); C2 falls back to the last strip
  box when the boundary maps to a group hidden from the strip; chosen constants
  remain unsourced CS6 metrics. No Rust change: **589 tests, 0 failed, 7 ignored**
  (unchanged; the raw ignored count is 8 with the pre-existing `pictura-render`
  doctest). **Capability:** MODIFIES `panel-column`, `application-shell`,
  `tool-framework`; **no new capability** → **60** canonical specs after archive.
  Verified: `cmake --build` clean; `./build/pictura --headless --self-test` exit 0
  with all `m45_*` `=1`; the PSD headless self-test exit 0; `cargo fmt --all
  --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean;
  `cargo test --workspace` **589 tests, 0 failed, 7 ignored**; `openspec validate
  m45-panel-fixes --strict` valid and `openspec validate --all --strict` 61/61.

- **CI headless and build speed** (infrastructure, not a milestone; archived
  OpenSpec change `ci-headless-and-speedup`). `main.cpp` gains an explicit **`--headless`**
  flag: it selects the offscreen QPA plugin before `QApplication` (when
  `QT_QPA_PLATFORM` is unset), wins over the `--self-test` xcb override, and
  implies `--self-test` when no document is given so it never blocks in
  `app.exec()`. The self-test asserts `QApplication::platformName() ==
  "offscreen"` and exits `152` otherwise. `--interop-probe` is excluded (it
  needs a real platform Vulkan instance); explicit `QT_QPA_PLATFORM=offscreen`
  and `xvfb-run` still work. Build speed: a new `.cargo/config.toml` links with
  `lld`, `[profile.test]` compiles dependencies at `opt-level = 0` with the
  workspace crates pinned at `2`, and CMake uses Ninja + `--parallel` with the
  self-test run headless. CI is rebuilt into three jobs — `rust` (fmt, clippy,
  nextest, doctests), `qt-headless` (pinned Qt 6.11.1 + CMake/Ninja +
  `--headless --self-test`), and `oracles` (ImageMagick + `psd-tools`, full
  `cargo test --workspace`) — with registry/sccache caching. No Rust API,
  document, codec, compositor, or dependency change: **588 tests, 0 failed, 7
  ignored** (unchanged).

- **Post-M47 cleanup pass** (review follow-up; no OpenSpec change, no new
  capability, canonical specs unchanged). **Bug fix — document tab reorder:** the
  document `QTabWidget` was movable but nothing connected `QTabBar::tabMoved`, so
  dragging a tab left `docs_` in the old order while `tabs_->currentIndex()`,
  `viewAt` and `removeDocument` indexed the new one — the wrong document became
  active, closed, or returned. `frame.cpp` now connects `tabMoved(from,to)` to
  `docs_.move(from,to)` (valid-index guarded); Qt keeps the dragged tab current,
  so the active document is unchanged. Regression: self-test exit **196**
  `doc_tab_reorder aligned=1` (`PicturaMainWindow::reorderDocumentsForTest`,
  which also exercises `viewAt`/`documentName`/`activeDocumentIndex` after the
  move). **Dead-code deletions:** `PanelGroup::detachPanel`/`attachPanel`/
  `removePanel`, `PanelColumn`'s `removePanel`, the unused `blendName`,
  `activeForeground_`, a no-op provider, the dead `panelFlyoutHeader` QSS
  selector, and three unused panel test hooks. **Dedupe (pure moves to one
  home):** the duplicated filter math helpers hoisted into
  `kernel`/`luma`/`texture` (bit-identical, oracle-verified), the frame TU
  include block shared via `frame_includes.h`, the shared `storage_entry` in
  `pictura-render::gpu`, and the bridge `mutate_layer`/`reset_edit_state`
  helpers. **Hardening:** the panel-drop path no longer orphans/leaks a lifted
  panel when no column can be created (`applyNewColumnDrop` re-inserts it into
  its home group), session reads are capped at 1 MiB, thumbnail size math
  widened, temp files cleaned up on save failure, over-cap layer channel counts
  rejected by the codec writer, filter coverage honours disabled masks, and
  NaN/overflow guards added in `pictura-adjust`/`pictura-filters`/
  `pictura-select`. **Kept despite review flags** because they are spec'd or
  planned: `history_depth`, `move_layer`, `layer_thumbnail`, `file_path`,
  `resize_canvas`, `rotate_arbitrary`, `composite_gpu_or_cpu`, `parent_path`,
  `hash_bytes`, `Layer::is_group`, `pictura_color::assign`, the whole
  `pictura-color` crate, and `move_preview`. Verified: `cmake --build` clean;
  `./build/pictura --headless --self-test` exit 0 with `doc_tab_reorder=1`;
  `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D
  warnings` clean; `cargo test --workspace` **592 tests, 0 failed, 9 ignored**
  (up from 589/7; the raw ignored count rose 8→9 with the newly-ignored M25
  `filter_profile_1024`).
- **Post-M47 cleanup pass, part 2** (follow-up to the above; commits
  `f0b3205`, `3dabab9`). `pictura-filters/src/filter.rs` (919 LOC) split into a
  facade plus `filter/types.rs` (`Filter` enum) and `filter/apply.rs`
  (`apply()`); `Selection::combine_with` now mutates in place and returns `()`
  (the clone-return was unused by its sole caller); the panel tab-menu labels
  come from one `tabMenuTexts(bool)` source instead of a duplicated array;
  `gSharedFloor` is recomputed from live columns instead of only growing (M45
  shared-floor semantics preserved); the layers row build is two-pass so a
  forward-referenced parent resolves instead of silently rooting; and the
  write-only `PanelGroup::defaultIconic` / `PanelColumn::isDynamic` flags are
  gone. All self-test stderr unchanged (code 196 unaffected).

- **layers-panel-controls** (the Layers-panel program's controls stage, named by
  content; OpenSpec change `layers-panel-controls`, archived). Opacity and Fill
  became percentage controls: a reusable `PercentField`
  (`crates/pictura-app/cpp/panels/percent_field.{h,cpp}`) with a text box, a
  popup slider, and drag-on-label scrubbing, converted at the view boundary with
  `round(pct*255/100)`; the bridge keeps reporting and editing `0..=255`, so no
  stored-format change. The lock strip became five icon toggles — alpha, paint,
  position, nesting, full — and `LockFlags` gained `NESTING = 0x08`, with
  `all()`/`is_all()` moving to the four-bit `0x0F`. The `lspf` reader mask
  widened to `& 0x0F` and `lock_from_bits` sets the fourth flag, so the PSD
  round-trips; `lock_bit("nesting")` reaches the bridge. A nesting-locked layer
  keeps its structural parent: `group_paths` refuses and `ungroup_paths` skips
  it, while within-container `move_path` still reorders. The row delegate paints
  a clipping-mask glyph (`layers.clipMask`) for a clipped layer. Six independent-creation
  SVGs were added and the qrc regenerated. Ceiling: a PSD whose `lspf` is the old
  three-bit `0x07` is no longer treated as fully locked (`all` now requires
  `0x0F`); `Lock All` sets all four bits. Self-tests `lpc_percent` (199) and
  `lpc_nesting` (200) live in a new `selftest_layers_controls.cpp` so
  `selftest.cpp` stays at its 6730 allowance. Verified: `cargo nextest run
  --workspace` **594 passed, 8 skipped**; `TASK_ALLOWS_DOCS=1 bash
  scripts/verify-full.sh` → `verify-full: OK` (TOTAL 768 passed · 9 skipped · 0
  failed, file-size OK, guard OK, `openspec validate --all --strict` 61/61).
  Capability: MODIFIED `layers-panel`; no new capability.

- **layers-filtering-search** (the Layers-panel program's filter/search stage,
  named by content; OpenSpec change `layers-filtering-search`, archived). The
  panel gained the CS6 filter/search row above the blend/opacity header: a
  dimension popup (Name, Kind, Effect, Mode, Attribute, Color; default Kind), a
  criteria stack, and an on/off switch. A new `LayersFilterProxyModel`
  (`layers_filter_proxy.{h,cpp}`) wraps the tree model and accepts a row when it
  matches or when any descendant matches (ancestor promotion); Name is a
  case-insensitive substring, Kind a multi-select over the model's kinds, Mode a
  blend key, Color a label index, and Attribute one of Visible/Hidden/Locked/Has
  Mask/Clipped; active criteria AND and Kind values OR. The Effect dimension is
  present but disabled until layer styles exist. The panel now uses the proxy as
  its view model, with `proxyIndexForPath`/`pathForProxyIndex` mapping selection,
  expansion, rename, and eye hit-tests; activation auto-expands promoted groups
  and toggling off restores the prior expansion. Filtering is view-only — no
  history state, never serialized — and resets to Kind/off on a document switch.
  New `layers_filter_bar.{h,cpp}`; the shared blend table moved to
  `layers_panel_internal.h`; theme QSS added. Self-tests `lfs_name` (201),
  `lfs_kind` (202), `lfs_mode` (203), `lfs_color` (204), `lfs_none` (205),
  `lfs_ancestor` (206), `lfs_toggle` (207), `lfs_live` (208), `lfs_reset` (209)
  live in a new `selftest_layers_filter.cpp`, keeping `selftest.cpp` at 6730.
  Ceiling: Kind offers only the kinds the model has (pixel, adjustment, group,
  background) and Effect is inert until the styles stage; the ancestry scan is
  O(subtree) per row (`// ponytail:` in the proxy). Verified: `TASK_ALLOWS_DOCS=1
  bash scripts/verify-full.sh` → `verify-full: OK` (TOTAL 777 passed · 9 skipped
  · 0 failed, file-size OK, guard OK, `openspec validate --all --strict` 62/62).
  Capability: ADD `layers-filtering-search`; no Rust change.

- **layers-panel-chrome-fixes** (a UI-correction follow-up to the controls and
  filter stages, named by content; OpenSpec change `layers-panel-chrome-fixes`,
  archived). **Header order/labels:** the header now stacks the filter row, then
  `[blend | Opacity label + field]`, then `[five locks | Fill label + field]`,
  then the layer list, then the action strip, and the `PercentField` popup opens
  centred under its field. **One menu entry point:** the panel's own Qt menu
  button (`layersPanelMenu`) was removed; the panel-group widget menu (`▾`) is
  the single path for the wired Layer commands, and the M39 menu self-test now
  asserts only the row and colour menus. **Left-anchored rows:** a new
  `LayersTreeView` suppresses the stock branch indicators; the tree runs at zero
  indentation and the delegate anchors the eye at the panel's left edge for every
  row, indenting the thumbnail/name by depth and drawing an expand/collapse
  chevron for a group (the panel's event filter toggles expansion on a chevron
  click, like the eye). **Filter lightswitch:** two independent-creation SVGs
  (`layers.filterOn/Off`) back an icon toggle that starts **on**; with no
  criterion the filter is inert, and the auto-expand-on-filter only runs when a
  criterion is actually active (`hasActiveCriteria`), so an enabled empty filter
  never expands every group. Self-test `lpc_chrome` (210) covers the order,
  labels, removed menu button, on/with-icon toggle, left-anchored eye, and the
  chevron click; the filter-reset test still passes with the reset now Kind/on.
  Ceiling: the kind toggle labels elide at the narrow default width. Verified:
  `TASK_ALLOWS_DOCS=1 bash scripts/verify-full.sh` → `verify-full: OK` (TOTAL 778
  passed · 9 skipped · 0 failed, file-size OK, guard OK, `openspec validate --all
  --strict` 62/62). Capability: MODIFIED `layers-panel` (+1 requirement, ~1) and
  `layers-filtering-search` (~2); no Rust change.

- **layers-panel-row-interactions** (a third UI pass on the Layers panel, named
  by content; OpenSpec change `layers-panel-row-interactions`, archived).
  **Percent fields:** `PercentField` now owns its text label and a `%` suffix,
  and pressing/dragging the label or the `%` scrubs the value like the field.
  **Semantic lock icons:** only `layers.lockAll` is a padlock; alpha is a
  transparency checkerboard, paint a brush, position a move cross, nesting
  nested squares. **Kind filter icons:** the Kind toggles are icon-only
  (`layers.kindPixel/Adjustment/Group/Background`) with the kind name as
  tooltip. **Eye and disclosure:** the visibility toggle is an eye SVG
  (`layers.eyeOn/Off`) drawn slightly inset (`kEyeInset = 6`), and a group shows
  `layers.disclosureRight`/`Down` at its indented position. **Drag and drop:**
  `LayersTreeView` enables drag/drop, overrides `startDrag` to carry the current
  row's path in `application/x-pictura-layer`, and resolves a drop to a target
  path plus a mode (above/below/into); the panel's handler calls the new
  `move_layer_to`. The path math lives in `pictura_render::move_path_to` (mode
  0/1/2) with refusal rules for the Background, fully/nesting-locked sources, a
  self or descendant drop, and an `Into` target that is not a group; the bridge
  recomposites and records one `"Move Layer"` state. Dragging no longer starts a
  rubber-band multi-selection, and dropping a row on the Delete / New Layer /
  New Group strip buttons runs delete / duplicate / group on the dragged paths
  (the buttons are drop targets with a `layerDropAction` property); mask/link/fx
  stay inert. Self-tests `lpr_rows` (211), `lpr_drag` (212), `lpr_drop` (213)
  plus the engine unit test `move_path_to_reparents_and_refuses`. Ceiling: only
  the current row is dragged (a multi-selection drag moves one row; marked
  `// ponytail:`). Verified: `TASK_ALLOWS_DOCS=1 bash scripts/verify-full.sh` →
  `verify-full: OK` (TOTAL 779 passed · 9 skipped · 0 failed, file-size OK, guard
  OK, `openspec validate --all --strict` 62/62); `cargo nextest run --workspace`
  **595 passed, 8 skipped**. Capability: MODIFIED `layers-panel` (+2, ~1) and
  `layers-filtering-search` (~1); no new dependency.

- **layers-panel-control-polish** (a fourth UI pass; OpenSpec change
  `layers-panel-control-polish`, archived). Four fixes. **Eye only:** the model
  stops reporting `Qt::CheckStateRole`, so a row paints no native checkbox next
  to the eye; the eye stays the sole visibility control. **`%` inside:** the
  PercentField suffix is reparented onto the value `QLineEdit`, a right text
  margin is reserved, and it is repositioned on resize, so the sign sits in the
  box. **Lock badge:** `LayerRowDelegate::lockRect` plus a `layers.lockAll` badge
  painted at the row's right edge (before fx/mask) whenever `LockRole != 0`.
  **Preview vs commit:** new bridge `preview_layers_opacity`/`commit_layers_opacity`
  and `preview_layers_fill`/`commit_layers_fill` (new `opacity_preview_changed`/
  `fill_preview_changed` flags) mirror the move-tool split; `PercentField` emits
  `valueChanged` during a scrub/slider drag and one `valueCommitted` on release
  (or text `editingFinished`, or popup hide), so a drag previews live and adds a
  single undo state. `set_layers_*`/`set_layer_*` are untouched. Ceiling: the
  preview flag is sticky, so a drag that returns exactly to its start value still
  records one redundant state (marked `// ponytail:`). Self-tests `lpc_preview`
  (214), `lpr_percent` (215), `lpc_lockbadge` (216), `lpr_eye` (217) in
  `selftest_layers_controls.cpp`. Verified independently: `TASK_ALLOWS_DOCS=1 bash
  scripts/verify-full.sh` → `verify-full: OK` (TOTAL 786 passed · 9 skipped · 0
  failed, file-size OK, guard OK, `openspec validate --all --strict` 63 items
  while the change was open); `cargo nextest run --workspace` **595 passed, 8
  skipped**; app self-test **153 passed, 0 failed**. Capability: MODIFIED
  `layers-panel` (4 requirements). Follow-up fix: the inside-`%` field was too
  narrow (`setFixedWidth(34)`) and clipped `100` to `0`; the edit is now sized
  from the font metrics for `100` + `%`, and `lpr_percent` asserts the fit. The
  `layers.eyeOn/Off` art was redrawn as a bolder Lucide-style eye for 14–20 px
  and `LayerRowDelegate::paintAsset` now renders a square, device-pixel-ratio-
  aware pixmap so the eye and chevron stay crisp. The popup slider is now a
  `JumpSlider` that jumps to the clicked point and tracks the held cursor, since
  the stock `QSlider` only page-steps; `lpr_slider` (218) covers it.

## Canvas viewport & performance (post-M24 pass)

Not an OpenSpec capability — a correctness/performance pass; the intended
behaviour (move-tool behaviour, budget, suspected bottlenecks, acceptance
checks) is written up in `docs/dev/canvas-view-spec.md`.

- `image_view.{h,cpp}`: `setImage` now fits-and-centres (fit when the image
  exceeds the viewport, else 100 % centred) and re-applies that initial view on
  resize until the user pans/zooms; middle-button drag pans the canvas
  regardless of the active tool; `fitOnScreen`/`actualPixels` re-arm the
  initial view.
- `frame.{h,cpp}`: `refresh()` keeps the canvas/status/menu updates synchronous
  but defers the expensive panel refresh (`retargetDock`) behind a single-shot
  120 ms `QTimer`, so a burst of `changed` signals no longer blocks the canvas
  repaint; tab add/remove/switch force an immediate panel refresh.
- `cxxqt_object/impl_transform.rs` + `tools.cpp`: the Move tool previews live **without
  compositing during the drag**. `begin_move_preview()` caches a base image (the
  document composited with the moved topmost raster layer hidden), the layer's
  own image, its document-space top-left, and its opacity; `move_preview_base/
  layer/x/y/opacity` and `end_move_preview()` expose/clear it.
  `ImageView::beginMovePreview(base, layer, layerPos, opacity)` +
  `setMovePreviewDelta(delta)` + `endMovePreview()` draw the cached base then the
  moved layer at the live Qt delta (source-over with `setOpacity`) in
  `paintEvent`. `ToolController` Move press seeds the preview, move updates only
  the delta, and release calls `end_move_preview` → `commit_move(dx, dy)` once
  (a single composite, exactly one "Move Layer" history state) →
  `endMovePreview`; switching tools or rebinding the canvas cancels it. The old
  per-event `move_preview(dx, dy)` is retained only for the self-test
  (`// ponytail: slow path`). One drag = one history state; undo restores the
  pre-drag pixels.
- Move-tool root cause, measured: each mouse-move previously ran `move_preview`
  → `pictura_render::translate_layer` (full composite) plus `document_to_image`
  (a **second** full composite) and a full planar→RGBA conversion. A single
  1024×1024 two-layer `composite_rgba` measures ~**257 ms in the debug build**
  (the old CMake default) and ~**39 ms in the optimized build** (~6.6×). The one
  remaining cost is the single commit composite on mouse-up; the live preview is
  source-over only (non-Normal blend modes, masks, and clipping are not
  reproduced mid-drag, but the committed image is exact).
- `CMakeLists.txt` now defaults `CMAKE_BUILD_TYPE` to `RelWithDebInfo` when
  unset; Corrosion maps any non-Debug config to cargo `--release`, so the Rust
  crate is built optimized too (verified: `build/libpictura_app.a` is the release
  artifact).

Self-test exit codes 64–67, measured identically on fixture and no-argument
runs: `canvas_centre offset=(270.691, 5) zoom=1`; `canvas_middle_pan
delta=(30,15)`; `canvas_move preview=1 hist=1 undo=1`; `canvas_preview_cache
began=1 base=1 layer=1 hist_unchanged=1`. These labels were renamed `m25_*` →
`canvas_*` during M25 so the `m25_` prefix belongs to the actual milestone; the
exit codes are unchanged. Gates green:
`cmake --build build`; both self-tests exit 0; `cargo fmt/clippy/test` clean
(no new Rust tests; the app crate stays green). The headless self-test cannot
measure frame timing, so the interactive feel (pan/zoom/move latency on large
documents) still needs a real GUI check.

## Spec workflow (OpenSpec)

OpenSpec is the per-change requirements layer over `docs/`. See `AGENTS.md`
"Spec workflow (OpenSpec)". M0–M34 are archived; `openspec/specs/` is now the
canonical contract, with the per-change history under
`openspec/changes/archive/`. New work starts as a new change under
`openspec/changes/` (not as code), with `proposal.md`, `design.md`, `tasks.md`,
and `specs/<capability>/spec.md` deltas, archived into `openspec/specs/` when
complete.

## Conventions (keep doing)

- Task briefs live in `docs/dev/m*-*.md`; docs changes need a commit message
  containing `TASK-ALLOWS-DOCS` or `TASK_ALLOWS_DOCS=1` for `guard.sh`.
- Each milestone: freeze interfaces → dispatch 2–3 `general` sub-agents on
  **disjoint files/crates** → orchestrator integrates, un-ignores oracle tests,
  verifies, commits. Never let an implementer verify its own work without an
  independent oracle (psd-tools / ImageMagick / the app self-test).
- Oracles: don't fake tolerances. Where ImageMagick/Photoshop semantics diverge,
  reclassify as "no faithful equivalent" and use property/known-value tests.

## Next: panels program (remaining stages), canvas perf series deferred

### Panels program — panel/column/toolbar work done; layer filtering/search next

The CS6 Layers panel program's research, gap analysis, and staged plan live in
`docs/dev/layers-panel-program.md`. **M36 — layer attributes end-to-end** (change
`openspec/changes/m36-layer-attributes`) and **M37 — layer creation and grouping**
(change `openspec/changes/m37-layer-creation`) are implemented and verified (see
the milestone entries above): M36 added `Layer.fill`/`lock`/`color`, `opacity ×
fill` compositing on CPU and GPU, `lspf`/`lclr`/`iOpa` PSD I/O, the bridge
getters/setters, and the Fill/lock/color panel controls; M37 added
`document_ops::layer_ops` New Layer / New Group / Duplicate / Group / Ungroup, the
bridge methods, the panel buttons and the five `Layer` menu commands.

**M39 — panel anatomy** (change `openspec/changes/m39-panel-anatomy`, brief
`docs/dev/m39-panel-anatomy.md`) is implemented and independently verified (see
the milestone entry above): the full expandable layer tree, the frozen layer-path
grammar and depth-first topmost-first projection, the path/batch bridge API,
the `QAbstractItemModel` tree + delegate row anatomy, multi-selection with the
per-node refusal table, solo visibility, `Tab` rename, Panel Options (session
schema v3), the panel/row menus, tooltips, and the explicit drag-reorder
deferral to M47.

**M38 was a user-requested interruption: the full CS6 toolbox icon/cursor
library and the panel icons** (`openspec/changes/m38-icon-cursor-library`,
contract `docs/dev/m38-icon-cursor-library.md`) — the frozen 71-tool catalogue,
the full icon/cursor asset set, the single-column flyout toolbox, and the
panel/Layers/History icons. It is implemented and verified (see the milestone
entry above); it took the M38 number, so the panel stages shifted by one:
**M39 — panel anatomy** is done (see the milestone entry above).

**M40 was a second user-requested interruption — the CS6 Tools panel**
(`openspec/changes/m40-tools-panel`, brief `docs/dev/m40-tools-panel.md`): the
custom lower-right flyout triangle, hold/right-click flyout with shortcut keys,
the one/two-column double-arrow toggle, the standalone dock (left/right only, no
tab groups), and the generic `Shift`+letter group cycling gated by a session
`Use Shift Key For Tool Switch` preference (session schema v4). It is
**implemented and independently verified** (see the milestone entry above).

**M41 was a third user-requested interruption — the CS6 panel column**
(`openspec/changes/m41-panel-column`, brief `docs/dev/m41-panel-column.md`):
content widgets moved under a `PanelColumn`: in normal mode vertical tab groups
with the tabs explicitly on top, in compact mode an icon strip with group
dividers, labels-on-widen, and `Qt::Popup` flyouts that close on click-away; the
`Auto-Collapse Iconic Panels` preference defaults **off**, the double-chevron
toggle, fixed CS6 Essentials groups, and the hard panel minimums removed with a
scroll so the window resizes freely; session **v5** adds
`panelRailMode`/`railWidth`/`autoCollapseIconic`, and the Preferences dialog
(General + Interface panes) gives `Use Shift Key For Tool Switch` and
`Auto-Collapse Iconic Panels` a UI — M40's previously UI-less `Use Shift Key For
Tool Switch` now has its home there. It is **implemented and independently
verified** (see the milestone entry above). **M42 was a fourth user-requested
interruption — panel refinements** (`openspec/changes/m42-panel-refinements`,
brief `docs/dev/m42-panel-refinements.md`, research
`docs/dev/m42-panel-menus.md`): the compact-strip drag/reorder, the group-styled
compact flyout, the per-widget header menus, the in-window float overlay, the
bounded normal-mode width / larger icons, and the menu-bar overlay root cause (a
duplicate `actionsPanel_`). It is **implemented and independently verified** (see
the milestone entry above). **M43 was a fifth user-requested interruption — panel
multicolumn** (`openspec/changes/m43-panel-multicolumn`, brief
`docs/dev/m43-panel-multicolumn.md`): the multi-column host of
create-on-drop/remove-when-empty columns, tab-vs-group drag and single-panel
floats, the unified `DropKind`/`resolveDrop` drop targets, the `panelTabBar` tab
colours, the fixed corner button and actual-geometry inner-side flyout, the
fixed-width Tools dock, the `D` colour reset, and session **v6**
  (`panelColumns`). It is **implemented and independently verified** (see the
  milestone entry above). **M44 was a sixth user-requested interruption — panel/
  theme polish** (`openspec/changes/m44-panel-theme-polish`, brief
  `docs/dev/m44-panel-theme-polish.md`): the new-document canvas E1 root cause
  (the GPU demo image assigned straight to `rust.image`) and its fix, the
  chevron/tab-colour/default-active corrections, the theme border/tab/flyout/elide
  unification, the float-drag continuation and any-side docking, and the compact
  group-relative drop with drag handles. It is **implemented and independently
  verified** (see the milestone entry above). **M45 was a seventh user-requested
  interruption — panel fixes** (`openspec/changes/m45-panel-fixes`, brief
  `docs/dev/m45-panel-fixes.md`): the resolver-owner drop indicator, the single
  emptied-column cleanup, the group-height minimize with its state-derived label,
  the one-formula Tools sizing and left/right-only beside-column pane, the shared
  minimum-width floor with no clipping, and the whole-group compact popup. It is
  **implemented and independently verified** (see the milestone entry above).
  The M40/M41/M42/M43/M44/M45 interruptions claimed six numbers the Layers-panel
  program had reserved. Two further user-requested panel-fix changes then took
  **M46** (`m46-panel-toolbar-fixes`) and **M47**
  (`m47-panel-interaction-fixes`): drop-indicator geometry, cross-column commit
  routing, the floating-Tools drag across Qt's dock mouse grab, primary-column
  removal, minimize min-height clamping, empty/ghost-column cleanup, compact
  float drag and group creation, the shared content floor with no horizontal
  scroll, the Tools central-splitter pane, and the floating close button. Both
  are implemented, independently verified, and **archived**
  (`openspec/changes/archive/2026-09-18-m46-panel-toolbar-fixes/` and
  `…-m47-panel-interaction-fixes/`). The **controls stage**
  (`layers-panel-controls`: percent Opacity/Fill, the five-lock strip with the
  nesting lock, and the clipping row indicator) and the **filter/search stage**
  (`layers-filtering-search`: the six-dimension filter row with ancestor
  promotion) have landed and archived, followed by a **chrome pass**
  (`layers-panel-chrome-fixes`: header order and labels, left-anchored eye with
  a group chevron, and the filter lightswitch on by default), a **row pass**
  (`layers-panel-row-interactions`: label scrub + `%`, semantic lock/eye/Kind
  icons, and row drag-and-drop reorder plus drop-on-strip), and a **control
  polish pass** (`layers-panel-control-polish`: eye-only rows, `%` inside the
  box, a right-side lock badge, and one-undo-state live Opacity/Fill preview),
  and the **management stage** (`layers-panel-management`: Merge
  Down/Layers/Visible/Clipping Mask and Flatten, the New Layer/Group dialog and
  Group from Layers, a first-class `Layer.background` flag with both
  conversions, Layer via Copy/Cut, Select Similar/Linked and transient link
  sets, Delete Hidden/Hide Layers, a minimal solid-fill layer kind with the
  Rasterize subset, and the deferred drag-reorder drop rules with a dry-run
  predicate).
  The Layers-panel program therefore continues **by content, not by number**:
  next is **layer styles / effects** (fx menu and badge, effect child rows, the
  Layer Style dialog, effect rendering passes, and the `Fill`-after-effects
  compositor change), then **smart objects / vector masks /
  artboards-as-non-goal / layer comps**. The management stage's deliberate
  inferred behavior and documented ceilings (rename↔background, the
  clipping-coverage compositor gap, zero-layer Flatten, `select_similar`
  excluding the active layer, fixed background/solid-fill colors, the `SoCo`
  descriptor ceiling) are listed in
  `openspec/changes/archive/…-layers-panel-management/design.md` §Residual. The
  pre-shift numbers still stand in `docs/dev/layers-panel-program.md`; this file
  is the up-to-date anchor.

The **selection tools** `selection-tools-and-menu` change is also landed and
archived: Elliptical Marquee (plus marquee Anti-alias/Feather/Style options),
Polygonal Lasso (click-vertex, close on first vertex/double-click/Enter, Esc
cancels, Feather), Magic Wand (Tolerance, Contiguous global/contiguous, combine
modes), and the whole Select menu — Reselect, Inverse, Modify
(Border/Smooth/Expand/Contract/Feather), Grow, Similar, Save/Load Selection to
an alpha channel, and All/Deselect/Similar Layers — mostly wiring the existing
`pictura-select` engine. Magnetic Lasso, Color Range, Refine Edge, and Transform
Selection are deliberately deferred and left visible-but-disabled; the anti-alias
and Sample-All-Layers options are honest disabled controls. The residual ceilings
are recorded in `openspec/changes/archive/…-selection-tools-and-menu/design.md`
§Residual.

The committed selection edge is drawn as **marching ants**: `pictura-select::contour`
extracts the 50 %-coverage boundary as chained lattice polylines (bbox-scoped, with
a Select-All short-circuit), `PictureView::selection_contour()` serializes them, and
`ImageView` paints them as an animated black dash over a white line in the document
transform. A tool drag shows the same marching-ants outline live as a selection
preview (`ImageView::setSelectionPreview`, the ellipse tool previewing the actual
ellipse, not its bounding box), replaced by the committed contour on release. Tool
commits reach the canvas through the now-connected
`ToolController::selectionCommitted` → `refreshSelectionOverlay()`; menu/`changed`
paths go through `refresh()`. `View > Show > Selection Edges` is wired as a
per-canvas toggle (default on). `apply_selection` deliberately does not emit
`changed()` (Quick Selection would repaint per mouse-move); only the released
commit refreshes the overlay.

A selection UX pass followed: the options bar packs left (`addStretch` per page)
with icon selection-mode buttons and a leading tool-icon + presets-chevron button;
the marquee/elliptical tools use a crosshair reticle cursor with Shift `+` / Alt
`-` variants; the lasso tools gained a top-left arrowhead (hotspot `(2,2)`); the
polygonal lasso previews an open polyline through clicked vertices only (no
rubber band to the cursor, no phantom closing edge) and commits on close; and
dragging inside an existing selection with a marquee/lasso tool now moves the
selection outline (arrow + small-marquee cursor) instead of starting a new
selection, recorded as one `Move Selection` state (`Selection::translate`).
Presets, pixel-content moves, and the deferred tools remain future work.

A modifiers-and-content-move pass then closed the Photoshop gap. Combine quick
keys are decided at the first press and locked for the gesture: Shift = Add, Alt
= Subtract, Shift+Alt = Intersect, but only when a selection already exists
(otherwise the options-bar mode applies); `ToolController::selectionModeForModifiers`
maps them and `dragMode_` carries the result through the commit. The Rectangular
and Elliptical Marquee constrain the Normal drag — Shift squares/circles it, Alt
treats the press point as the centre — and a floating `W x H` readout follows the
cursor (`ImageView::setDragSizeHint`). The Polygonal Lasso preview is now a solid,
open rubber band from the first vertex through the clicks to the live cursor (the
`solid` flag on `setSelectionPreview`); the cursor segment is preview-only. The
canvas enables mouse tracking so the move-selection cursor appears on hover, and
Ctrl over a selection shows it too. Moving selected pixels is real:
`pictura_render::move_selection_content` reuses `layer_via_copy`/`layer_via_cut`,
translates the new layer, and merges it back down (or leaves it as a new layer
when duplicating); the bridge records one `Move Selection` state. The Move tool
moves the selected pixels (Alt duplicates to a new layer) whenever a selection
exists, and a selection tool with Ctrl does the same (Ctrl+Alt duplicates) while
a plain drag keeps moving only the outline. Content-move drags preview the moving
selection outline, not the pixels (`ponytail:` the masked base composite is the
upgrade), and the target layer is still the topmost pixel layer, matching the
existing Move tool. Self-tests `tsc_quick_modes` (270) through
`tsc_quick_mode_drag` (276) cover the mapping, geometry, view hooks, polygon band,
and the move/duplicate wiring.

PSD interop phase P1 (`docs/dev/psd-support-roadmap.md`) landed in
`pictura-codec`: the composite and layer channel readers now accept compression
`2` (ZIP/zlib deflate, with a raw-deflate fallback) and `3`
(ZIP-with-prediction, inverting the byte-wise per-row delta after inflating), an
unrecognized blend key degrades to `BlendMode::Normal` instead of aborting the
file, and a layered document whose merged composite is absent parses with a
zero-filled composite. `flate2` (pure-Rust miniz_oxide backend) is the only new
dependency. Write behaviour is unchanged (still raw) so the byte-layout golden
holds. psd-tools' writer only emits raw/RLE, so the differential oracle
(`tests/oracle.rs`) builds a ZIP/ZIP-with-prediction PSD by hand and requires
psd-tools' *decoder* to agree byte-for-byte with `read_psd`. Remaining PSD gaps
(color modes, 16/32-bit, image resources/ICC/metadata, effects/smart
objects/text, PSB write, write-side RLE/ZIP) are tracked in the roadmap.

PSD interop phase P2 (`psd-opaque-preservation`) made an open→save round trip
lossless for everything the engine does not interpret. `Document` now carries
the raw color-mode-data and image-resource sections, the global layer mask, and
the trailing layer-section bytes; `Layer` carries its original blend key (only
when unrecognized), blending ranges, unknown additional-layer-info tagged blocks,
and unmodeled channels (the `-3` real user mask and any other id outside
`-2..=2`); `LayerMask` carries the mask block bytes past the fixed fields. The
codec captures them on read and re-emits them verbatim on write, so ICC/EXIF/XMP
resources, resolution, effects/smart-object/text blocks, and vector-mask channels
are preserved even though the engine cannot render them yet. A recognized blend
key is not stored, so a constructed document still equals one read from disk, and
engine-created documents keep empty storage — `write_psd` output for them is
byte-identical (the `default_before.psd` golden holds). The model change touched
~160 struct literals across the workspace via `..Default::default()`, and
`layer_ops/tests.rs` was split (`tests_via.rs`) to stay under the size cap; the
independent oracle proves a psd-tools-authored fixture round-trips
whole-`Document`-equal and that psd-tools still opens our re-emitted resources.

> These numbers reuse M36–M38 previously sketched for canvas performance below.
> `docs/dev/canvas-compositing-plan.md` is frozen and still uses them, so read
> those tracks by name (history COW, resident GPU sources, 256² tiles), not by
> number; they are deferred until after the panel program.

M31 removed the full composite and readback from every move and paint
(dirty-rect compositing), M32 removed it from the move-preview base and the
visibility toggle and cached the present-scale, and M33 removed the host-side
per-pixel assembly from the remaining full composites; every other mutation still
composites and reads back the **whole** document, and the CPU compositor and
`pictura_filters::apply` are still the oracles. The research and the M31–M35 plan
are written up in `docs/dev/canvas-compositing-plan.md`; the M32 brief with the
measured phase table is `docs/dev/m32-interactive-canvas.md`, and the M33 brief is
`docs/dev/m33-composite-throughput.md`. The 4000² full composite is now ~123 ms,
dominated by the per-composite source/mask **upload** (~128 + 16 MB), not the GPU
dispatch (~0.2 ms) or the ~38 ms readback, so a zero-copy present still saves
little while the upload stays.

The perf series was **paused** for **M34 — composite coherence and cheap
undo/redo** (OpenSpec change `m34-composite-coherence`, **implemented**; brief
`docs/dev/m34-composite-coherence.md`): the canvas rebuild persists its rendered
result into `doc.composite` (RGBA for an RGB document, plane-count-preserving
for a non-RGB mode), Save serializes that current composite, `undo`/`redo`
restore the display from the snapshot composite instead of a full composite
(unconditionally), and the `record` call moves after the composite step so the
snapshot carries it. It is app-local and bounded: no `write_psd` format change,
no new capability. Next in order:

- **M35 — region blit in C++ (implemented; archive pending).** OpenSpec change
  `m35-cpp-region-blit` (MODIFIED `document-canvas`; no new capability → **59**
  after archive), brief `docs/dev/m35-cpp-region-blit.md`. `PictureView` no longer
  maintains a region-patched full-resolution `QImage`: `refresh_region` composites
  the region, patches the planar `doc.composite` with `copy_from_slice`, converts
  only the region-sized buffer to a `QImage`, sets a `display_dirty` marker and
  emits a new `region_blitted(QImage, x, y)` signal; `ImageView::blitRegion` paints
  it with `QPainter` + `CompositionMode_Source` (invalidating the present zoom
  cache so the next paint rebuilds it identically). The per-pixel
  `QImage::set_pixel_color` loop and `REGION_REFRESH_BUDGET` are deleted, so a large
  dirty region is blitted in C++ instead of forcing a full document composite.
  `image()` rebuilds from `doc.composite` when `display_dirty` (with an explicit
  in-stroke guard), `sample_argb` reads the planar composite directly, and
  `move_preview_base` builds from the current composite. Measured 4000² (release,
  GPU): a 1024² region refresh **~33 ms → ~8.8 ms** (composite 4.95 + composite
  patch 1.81 + region convert 2.04) plus one `QPainter::drawImage` blit in C++; a
  512² region refresh **3.29 ms**. The old path was a 27.3 ms per-pixel FFI blit
  plus a ~5.6 ms composite. Honest notes: `refresh_region` no longer emits
  `changed`, so panel refresh on the region path is debounced in `frame.cpp`; a
  region blit invalidates the present zoom cache (one rescale on the next paint,
  the same cost as the old `replaceImage`); `begin_move_preview` now pays one planar
  clone + conversion per drag start; mid-stroke `sample_argb` reads the pre-stroke
  composite (unreachable while painting). `m31_region_large` now asserts the region
  path ran *and* the canvas equals a full recomposite (strictly stronger than
  before), and `move_preview_region` no longer has an oversized-rect fallback
  because the budget is gone. Verified: `cargo test --workspace` **546 tests, 0
  failed, 7 ignored** (up from 544, 6 ignored; the new
  `m35_region_refresh_profile_4000`), `cargo fmt`/`clippy` clean, both self-tests
  exit 0 with `m35_region_blit region=1 changed=0 canvas=1 rebuilt=1 cache=1` and
  `m35_region_large region=1 recomposite=0 canvas=1`, `openspec validate --all
  --strict` 60/60.

- **M34 — composite coherence and cheap undo/redo (implemented; archived).**
  OpenSpec change `m34-composite-coherence` (MODIFIED
  `edit-history`, `document-lifecycle`; no new capability → **59** after archive).
  The canvas rebuild now persists its rendered frame into `doc.composite`
  (`store_composite`: RGBA for an RGB document, colour-plane-count-preserving
  otherwise), `recomposite` renders → stores → builds the `QImage` from the
  rendered frame, `refresh_region` patches the composite on its region path, and
  `record` runs after the composite step so every snapshot carries a current
  composite. `undo`/`redo` therefore rebuild the display with
  `buffer_to_image(&snapshot.doc.composite)` instead of a full composite, and
  Save serializes the current composite. Measured 4000² (2 RGB layers, release,
  GPU): the old undo display path `document_to_image(gpu=true)` **163.16 ms** vs
  the new `buffer_to_image(snapshot.composite)` **40.94 ms** (~4×); the remaining
  per-snapshot cost is the whole-document `History::capture` clone at **59.49 ms**
  (COW/tile-diff history remains deferred). Honest limits: a layered **grayscale**
  document with transparent coverage restores opaque because its composite stays
  1-plane, and a dimension-changing op on grayscale keeps the pre-existing
  4-plane outcome. Verified: `cargo test --workspace` **544 tests, 0 failed, 6
  ignored** (up from 541, 5 ignored; the new `m34_undo_profile_4000`), `cargo
  fmt`/`clippy` clean, both self-tests exit 0 with
  `m34_coherent composite=1 undo=1 save=1`, `openspec validate --all --strict`
  60/60.
- **M33 — full-composite throughput (done; archived).** OpenSpec change
  `m33-composite-throughput` (MODIFIED `gpu-compositing`; no new capability),
  implemented and verified. Row-wise source/mask assembly, a fused planar readback
  that skips the packed `Vec`, and a GPU command-buffer canvas clear took the
  4000² two-layer composite ~254 ms → ~123 ms (~2×), byte-identical. The remaining
  bottleneck is the per-composite upload; resident per-layer GPU source buffers are
  deferred (they need content versioning).
- **GPU-resident zero-copy present (deferred; was planned as M34)** via Qt Quick
  (`QQuickRhiItem` sharing the window's `QRhi` +
  `QQuickWindow::createTextureFromRhiTexture()`, or one shared Vulkan device via
  `QQuickGraphicsDevice::fromDeviceObjects(...)`); `QRhiWidget` cannot adopt the
  wgpu device. Deferred because it removes only the ~38 ms readback of a ~123 ms
  composite, not the upload.
- **256² GPU tiles (deferred) + LRU + seam gutters + mipmaps**
  (Graphite-style), only if pan/zoom over documents larger than VRAM demands it;
  includes display-time LoD so a zoomed-out view composites a proxy.

Deferred canvas-performance tracks (previously sketched as M36–M38; those
numbers are now claimed by the panels program above, so these are deferred
until after M49). The remaining canvas-performance tracks — history
copy-on-write / tile diffs, resident per-layer GPU source buffers, 256² tiles +
LoD, plus the GPU-resident zero-copy present — each need their own design (the
small, app-local region-blit slice landed as M35 above):

- **Cheap undo/redo + composite coherence/save** — landed as M34
  (`m34-composite-coherence`); see the brief `docs/dev/m34-composite-coherence.md`.
  The original concern — persisting the rendered composite into `doc.composite`
  changes what `write_psd` serializes — is handled by storing the rendered RGBA
  frame for RGB and preserving the composite's colour-plane count for a non-RGB
  mode, so the byte layout is unchanged.

- **M35 — region blit in C++ / `REGION_REFRESH_BUDGET` removal — landed.** See
  the milestone entry above; OpenSpec change `m35-cpp-region-blit`, brief
  `docs/dev/m35-cpp-region-blit.md`. The per-pixel `QImage::set_pixel_color` loop
  and the budget fallback are gone; a dirty region of any size takes the region
  path.
- **History copy-on-write / tile diffs** — the history capture still clones
  the whole document (~60 ms per state at 4000², and holds up to 20 states).
- **Resident per-layer GPU source buffers** — deferred from M33; the
  shader-side planar output landed as the M33 A2 follow-up above. Keeping a
  layer's source plane resident on the GPU across a composite session needs
  content versioning to detect a changed layer; the remaining composite cost is
  the per-composite upload (~128 MB + 16 MB at 4000²), which residency would
  remove.
- **Transparency grid preferences** — M30's checkerboard is fixed at an 8 px
  Light (`#FFFFFF`/`#CCCCCC`) grid; the `Transparency & Gamut` preferences pane
  (grid size None/Small/Medium/Large, colour sets Light/Medium/Dark/Red/Custom),
  the `View > Show > Transparency Grid` toggle, and gamut warning are deferred.
- **GPU painterly/stochastic filters and GPU painting** — the remaining M22
  Artistic and M25 Brush Strokes/Sketch/Texture families (`Watercolor`,
  `Conté Crayon`, `Paint Daubs`, `Dry Brush`, `Ocean Ripple`, `Spatter`,
  `Sponge`, `Palette Knife`, `Add Noise`, `Colored Pencil`, `Crystallize`) on the
  GPU by **pre-generating their seeded RNG fields on the CPU and running only the
  spatial work on the GPU**, since the RNG stream cannot be reproduced
  bit-exactly on the GPU, and the paint dab loop (GPU painting).
- Real content for the M24 placeholder panels (gradient/pattern presets,
  Properties binding, adjustment presets, libraries, channel/path lists, actions)
  and image modes / bit-depth (16/32-bit, CMYK/Lab gating for filters and
  adjustments).

Process: every new milestone is proposed through OpenSpec first
(`openspec/changes/<name>`, new capabilities), validated, then implemented.
M6 through M34 are archived; their deltas now live in `openspec/specs/`.

## Known risks / open items

- Core API is frozen only where noted; adding fields breaks struct literals.
- PSD descriptor coverage is partial (adjustment layers, layer styles not yet).
- GPU is the default compositor and the default filter path, but the CPU
  compositor and `pictura_filters::apply` remain the oracles.
- A `Dissolve` layer or an unsupported adjustment forces a whole-document CPU
  fallback for that composite.
- Fifteen filter kernels are GPU-accelerated (the blur/sharpen/High Pass family
  plus the M28 heavy window/effect set); the stochastic/seeded filters and the
  warps/distort and render filters fall back to the CPU oracle byte-for-byte.
- The GPU compositor and filter path are host-side bound, not readback-bound: at
  4000² the composite is now dominated by the per-composite source/mask upload
  (~128 + 16 MB) after M33 removed the per-pixel assembly, while the 64 MB
  readback is ~6 ms (see `docs/dev/canvas-compositing-plan.md` §2.1); resident
  per-layer buffers (deferred) and zero-copy present (M34) both aim at this.
- M35 (`m35-cpp-region-blit`) removed both the per-pixel `QImage::set_pixel_color`
  blit and the 1 MP `REGION_REFRESH_BUDGET` fallback: the planar composite is the
  authoritative canvas, `refresh_region` signals the region, and C++
  `ImageView::blitRegion` (`QPainter`, `CompositionMode_Source`) paints it. A
  Display-resolution proxy (LoD) is still absent, so a zoomed-out composite still
  covers the whole document.
- History capture clones the whole document for each state, so large documents
  pay both RAM (up to 20 states) and latency (~60 ms/state at 4000²); copy-on-write
  or tile diffs are the deferred fix.
- The GPU path still silently falls back to the CPU on any `GpuError`.
- `pictura-app` has one `#[ignore]`d interop test.
- The recent-files menu is rebuilt at startup, so a file opened in-session
  appears there only after restart.
- Quick Selection is a wand-union approximation, not a true Photoshop quick
  selection; crop is destructive (no crop region / no non-destructive re-crop);
  selection marching ants are not implemented — only a rubber band during drag
  and the committed bounds are shown.
- Icon art is a first functional pass; a visual refinement pass can change SVG
  paths without any code change.
- Cursors render at a single DPR (no per-screen 2×/3× cursor variants yet).
- The M20 Layers panel shows top-level rows only: group-tree expansion,
  drag-reorder, layer lock flags, clipping/link/color labels, and a
  filter/search row are not implemented.
- Swatch library file I/O, Info color samplers, and Histogram source/cache
  states are not implemented.
- Painting is limited to 8-bit RGB single raster layers; the coverage scratch is
  a layer-sized buffer (sparse tiles deferred).
- Only the Normal/Dissolve/Behind/Clear paint modes exist; there is no tablet
  pressure mapping or brush presets yet.
- Artistic filters are behavioural-parity models without an Adobe oracle; the
  Filter Gallery UI, Smart Filters, and depth/mode gating are not implemented.
- The CS6 chrome is a defensible dark look, not a pixel-exact match (exact CS6
  colours/metrics are unsourced).
- Panel contents beyond M20 and workspace presets/icon-collapse docks are not
  implemented.
- The M24 panels are structural placeholders with empty states, not features;
  icon-collapse, workspace presets, and panel-title-bar menus are not
  implemented.
- Oil Paint is a CPU behavioural model: CS6 requires a supported GPU (closed
  OpenCL kernel, no CPU fallback), so the result is a deliberate non-parity
  divergence rather than verified parity.
- The OS font/filter gallery UI is still absent.
