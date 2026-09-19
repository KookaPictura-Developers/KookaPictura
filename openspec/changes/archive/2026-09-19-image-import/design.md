## Context

`File > Open` and `File > Place…` accept only `*.psd *.psb`
(`crates/pictura-app/cpp/frame.cpp:497`,
`crates/pictura-app/cpp/frame_menus.cpp:69`). `pictura_codec::read_psd` is the
only decoder in the tree, and the engine is deliberately Qt-free
(`AGENTS.md`: `core|codec|color|adjust|filters|ops|paint|select` carry spec
math, no Qt). The app already owns a seam between Qt images and engine pixels:
`helpers_composite.rs::buffer_to_image` renders an engine `PixelBuffer` to a
`QImage` in `Format_RGBA8888`, and `rgba_image` wraps packed RGBA back into a
`QImage`. There is no path in the other direction yet, and no way to build a
`Document` or `Layer` from decoded pixels.

Qt already decodes every raster format the platform supports. `cxx-qt-lib`'s
`QImage::from_data` calls `QImage::fromData`, which auto-detects the format from
the header. The engine therefore needs no new decoder: it needs (a) a decode
edge at the app boundary, (b) construction functions that turn packed RGBA8888
into `Document`/`Layer`, and (c) a cheap, Qt-free header probe so an
attacker-supplied file cannot make the process allocate before Qt touches it.

The smart-object machinery is complete: `convert_to_smart_object` authors an
embedded source from a raster layer while keeping the raster proxy, and
`place_smart_object`/`open_as_smart_object` already exist for PSD. Place for a
raster image can reuse `convert_to_smart_object` with no new smart-object code.

## Goals / Non-Goals

**Goals:**

- Open and Place common raster images (PNG/JPEG/GIF/BMP/TIFF/WebP/…) alongside
  PSD/PSB.
- Keep the engine Qt-free and dependency-free: decode lives at the app
  boundary; the engine receives `(width, height, rgba_bytes)`.
- Reject over-budget imports before Qt allocates, and cap the actual decoded
  allocation.
- Preserve the existing PSD/PSB native path and the existing smart-object
  behavior; add no new smart-object machinery.
- One runnable check per new non-trivial path (Rust unit tests + one C++
  self-test).

**Non-Goals:**

- OS drag-and-drop (Phase 2) and the Free Transform placement session (Phase 3).
- Original-file-bytes fidelity, Replace Contents for images, multi-frame beyond
  frame 0, ICC/EXIF/XMP preservation, lazy/gigapixel tabs.
- Any new decoder crate. No `image` crate.
- Changing `docs/`.

## Decisions

### D1. Decode at the app boundary, construct in the engine

`QImageReader`/`QImage` decode any supported file to `Format_RGBA8888` in the
app crate. The engine gains no Qt type: `Document::from_rgba` and
`add_raster_layer_from_rgba` take `rgba: &[u8]` in the same packed RGBA8888 byte
order `buffer_to_image`/`rgba_image` already use, so no new conversion exists.

**Alternative considered:** put a decoder in `pictura-codec`. Rejected: the
engine would take a format dependency and duplicate what Qt already has.

### D2. The Qt decode helper is C++, reached from Rust through the cxx bridge

`cxx-qt-lib` 0.10 exposes `QImage::from_data`, `width`, `height`, and `format`,
but **not** `convertToFormat` or a raw-bit accessor, so Rust cannot extract the
RGBA buffer from a decoded `QImage`. The decode helper is therefore C++:
`decode_image_rgba(const uint8_t* data, size_t len, int* width, int* height) ->
rust::Vec<uint8_t>`, implemented with `QImage::fromData(...).convertToFormat(
Format_RGBA8888)` (or `QImageReader` on a `QBuffer`). It is declared in the
existing `#[cxx::bridge]` in `crates/pictura-app/src/cxxqt_object.rs` and
implemented in a small new translation unit under `crates/pictura-app/cpp/`,
registered explicitly in `CMakeLists.txt` (no globbing). Rust keeps file I/O: it
reads the bytes, probes them, then passes the bytes to the decode helper, so the
file is read once and the probe runs before any Qt decode.

### D3. The probe is a pure Qt-free parse; the budget is advisory and paired

`pictura_codec::probe_image(bytes: &[u8], budget: ImageBudget) ->
Result<ImageProbe, ImportError>` sniffs the container and parses the declared
format, width, height, and bit depth from the header alone — PNG (IHDR), JPEG
(JFIF/SOF), GIF, BMP, TIFF (II/MM), WebP (RIFF) — without decoding or allocating
from the declared size. `ImageBudget { max_dimension: u32, max_alloc_bytes: u64 }`
holds tunable, documented defaults (dimension 30 000, allocation 512 MiB).
`ImportError` carries the source description, declared dimensions, bytes read,
and which limit was exceeded.

The header is attacker-controlled, so the probe is **advisory only**: a
too-small declared size proves nothing about the decoded size. The decode edge
must therefore also check the **actual** decoded `width * height * 4` against the
same budget before converting to engine structures (D5). Both checks exist; the
probe is the cheap early refusal, the post-decode check is the safety cap.

### D4. Engine construction mirrors the existing `create.rs` builders

- `pictura_core::Document::from_rgba(name, width, height, rgba) -> Document`
  builds `Document::new(width, height, ColorMode::Rgb, BitDepth::Eight)`, seeds
  `composite` as a 4-plane RGBA `PixelBuffer` (the layout `store_composite`
  produces for RGB after any rebuild), and pushes exactly one pixel layer named
  `name` with `rect = (0, 0, w, h)` and planar channels `0`, `1`, `2`, `-1`
  (the `Layer.channels` layout `transparent_layer` and `rasterize_smart_object`
  use). No smart object, no adjustment.
- `pictura_render::add_raster_layer_from_rgba(doc, name, width, height, rgba) ->
  String` mirrors `create.rs`'s builders (`add_solid_fill`): construct the same
  planar layer and **append it at the top** (`doc.layers.push`, as
  `place_smart_object` does), returning the path via `format_segments`. It is
  sized to the image `(0, 0, w, h)`, not to the document, so Place can import an
  image of any size.

**Alternative considered:** reuse `add_layer_in` + fill channels. Rejected: it
produces a document-sized layer at the insertion point, not a native-size
topmost raster, and it is not "append at top".

### D5. The Open and Place bridges replace/append, and record one undo state

- `PictureView::open_image(path) -> bool`: read bytes → probe → decode to RGBA →
  check actual allocation → `Document::from_rgba` with the file's base name →
  `store_composite(current_buffer(...))` → set `image`/`doc` → `reset_edit_state`
  → capture exactly one `"Open"` snapshot → clear `path` (`None`) → `dirty =
  false`. Returning `false` on any refusal leaves the view untouched. `path` is
  `None` deliberately: imported pixels become a PSD document, and keeping the
  image path would let `Ctrl+S` write PSD bytes over the source image. The tab
  is therefore untitled, matching `open_as_smart_object`. (The task allows
  "untitled-or-named"; untitled is the data-loss-safe choice.)
- `PictureView::place_image(path) -> QString`: read bytes → probe → decode →
  actual-allocation check → `add_raster_layer_from_rgba` on the current document
  → `convert_to_smart_object(doc, path)` → `clear_link_sets` + `recomposite` +
  `record("Place")` → return the new layer path. Any refusal (no document,
  missing/unreadable file, unsupported format, over budget, failed conversion)
  returns an empty `QString` and records nothing. The layer keeps a raster proxy
  (the decoded pixels), so it renders from the proxy; `convert_to_smart_object`
  authors the embedded payload as a PSD of that proxy, **not** the original
  image bytes. No new smart-object engine code.

### D6. Qt's runtime support is the allow-list; v1 refuses, it does not placeholder

The engine probe may name a format Qt cannot actually decode (a missing image
plugin). The authoritative runtime set is `QImageReader::supportedImageFormats()`.
v1 policy: attempt the Qt decode; if Qt cannot read the file, **refuse** (return
`false`/empty, record nothing). A labeled placeholder is deliberately not built —
it is extra UI for a case the user can resolve by converting the file. Only
frame 0 of a multi-frame GIF/APNG is imported.

### D7. PSD/PSB stays native

The Open and Place handlers inspect the chosen path's suffix. `*.psd`/`*.psb`
use the existing `open`/`place_smart_object` native `read_psd` path unchanged;
every other supported suffix routes through `open_image`/`place_image`. PSD is
never handed to Qt. The dialogs gain an `Images (…)` filter beside the existing
`Photoshop files (*.psd *.psb)` filter.

### D8. Verification

- **Rust unit tests**: `pictura-codec` probe tests for each recognized header,
  an over-budget refusal, an unknown-format refusal, and a truncation
  non-panic; `pictura-core` `from_rgba` tests for size/mode/depth, the single
  layer, and RGBA→planar fidelity; `pictura-render` tests for
  `add_raster_layer_from_rgba` (topmost, native-size, planar pixels) and that
  `convert_to_smart_object` succeeds on it.
- **C++ self-test** in an existing suite (no new file, `CMakeLists.txt` app
  sources unchanged): generate a small PNG with `QImage::save` to a temp path,
  open it and assert a one-layer document of the right size with the expected
  pixel; place it into an open document and assert a new smart-object layer and
  one `"Place"` history state; assert an unknown-format file and an over-budget
  file refuse without a history state. Exit code **290** (the max in
  `crates/pictura-app/cpp/selftest*.cpp` is 289).

## Risks / Trade-offs

- **Header-vs-reality budget bypass.** The probe trusts declared dimensions. →
  Paired with the post-decode actual-allocation cap (D3/D5); the probe is only
  an early refusal.
- **`QImage::fromData` memory spike.** Qt allocates the full decoded image in
  C++ before Rust sees the dimensions. → The probe refuses the obvious cases
  first; the post-decode cap refuses the rest and drops the buffer. A hard
  process-level cap would need `QImageReader::setScaledSize`/`size()`, noted as
  a future tightening rather than built now.
- **Placed image payload is a re-encoded PSD.** `Export Contents` yields the
  proxy, not the original PNG/JPEG. → Named in the proposal's non-goals;
  original-bytes fidelity is a follow-up.
- **Open is untitled, so `Ctrl+S` prompts Save As.** → Intended: an imported
  raster is an editable PSD document, and the source image must not be
  overwritten.
- **New C++ translation unit must be registered.** CMake has no globbing
  (`AGENTS.md`). → The tasks list the explicit `CMakeLists.txt` edit; the
  self-test stays in an existing suite so only the helper is new.
- **Exit-code discipline.** Codes are append-only and identify failures. → Take
  290 (289 is the current max), keep the check descriptive and within the
  suite's `scripts/file-size-allowlist.txt` ceiling.
