# Rust Core Design

- **Spec ID:** `ARCH-002`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `No` — this is the proposed implementation substrate for the CS6 feature set; no CS6 user-visible feature is defined here.
- **Depends on:** `ARCH-001` (layering, process/FFI boundaries). Feeds all feature specs.

This document proposes the Rust workspace layout, the core document types, the
error model, and the `unsafe` policy for Kooka Pictura's Rust core. Every crate
and module name is a **design proposal**. Every third-party crate named here was
verified to exist on crates.io at time of writing; the documentation URL for each
is listed under Sources.

## CS6 behavior

There is no CS6 UI behavior in this document. The behavior being mirrored is the
*data contract* implied by CS6: the PSD/PSB native format and the document model
it must round-trip. The Adobe Photoshop File Formats Specification is the
authority for that contract.

Documented format facts the core types must express:

- File signature `8BPS`, version `1` (PSD) or `2` (PSB). Maximum dimension
  30,000 px (PSD) or 300,000 px (PSB).
- Channel count 1–56 including alpha channels; bit depth 1, 8, 16, or 32.
- Color modes: Bitmap=0, Grayscale=1, Indexed=2, RGB=3, CMYK=4, Multichannel=7,
  Duotone=8, Lab=9.
- Channel image data compression: 0 = raw, 1 = RLE (PackBits), 2 = ZIP without
  prediction, 3 = ZIP with prediction.
- Per-layer channel IDs: 0+ = color channels, −1 = transparency mask, −2 = user
  layer mask, −3 = real user layer mask.
- Metadata resource IDs include ICC profile (1039), EXIF (1058/1059), and XMP
  (1060).

These constraints drive the type design below. Anything not stated by the format
spec is marked inferred.

## UI surface

None. This is a headless core library; no Qt types may appear in any crate other
than the bridge crate (`pictura-qt`).

## Parameters & ranges

These are compile-time / construction-time constraints the core types enforce.
They mirror the format limits but are **not** UI controls.

| Constraint | Type | Value | Source |
|---|---|---|---|
| Max dimension, PSD | `u32` | 30,000 | Format spec |
| Max dimension, PSB | `u32` | 300,000 | Format spec |
| Channel count | `u16` | 1–56 | Format spec |
| Bit depth | enum | 1, 8, 16, 32 | Format spec |
| Color mode | enum | Bitmap…Lab (9 values) | Format spec |
| Layer count | `i16` / `u32` | PSD signed, PSB 64-bit lengths | Format spec |
| Tile size | `u32` | implementation-chosen (CS6 default 128 KB bytes) | Adobe performance paper |
| Scratch volume count | `u8` | ≤4 | Adobe performance paper |

## Algorithms & pipeline

### Workspace layout (proposed)

A virtual Cargo workspace with a shared lockfile, shared `target/`, and shared
dependency/lint tables. Layout follows the Cargo book's workspace conventions.

```text
pictura/
├── Cargo.toml               # [workspace] virtual manifest
├── crates/
│   ├── pictura-core/        # no Qt, no GPU; types + document graph
│   ├── pictura-color/       # lcms2 wrapper, color math
│   ├── pictura-codec/       # PSD/PSB/TIFF/PNG/JPEG/… read-write
│   ├── pictura-render/      # tile cache + wgpu compositor
│   ├── pictura-filters/     # CPU kernels (rayon)
│   ├── pictura-script/      # command API + script host
│   └── pictura-qt/          # cxx-qt bridge (only Qt-dependent crate)
└── src/main.rs              # pictura-app binary (Qt startup)
```

Dependency direction is strictly downward: `pictura-app → pictura-qt →
{pictura-script, pictura-render, pictura-filters, pictura-codec} →
{pictura-color, pictura-core}`. `pictura-core` depends on no sibling.

### Key types (proposed signatures)

```rust
// pictura-core
pub struct Document {
    pub id: DocId,
    pub color_mode: ColorMode,
    pub depth: BitDepth,
    pub width: u32,
    pub height: u32,
    pub resolution: Dpi,
    pub profile: Option<ColorProfileRef>,
    pub layers: Vec<Layer>,
    pub channels: Vec<Channel>,      // extra alpha/spot channels
    pub paths: Vec<Path>,
    pub metadata: Metadata,
    pub history: History,
}

pub struct Layer {
    pub id: LayerId,
    pub kind: LayerKind,             // Pixel | Adjustment | Type | Shape | Group | Smart
    pub name: String,
    pub bounds: Rect,
    pub opacity: u8,                 // 0..=255
    pub blend: BlendMode,            // 27 CS6 modes
    pub visible: bool,
    pub clipping: bool,
    pub mask: Option<LayerMask>,
    pub vector_mask: Option<PathRef>,
    pub channels: SmallVec<[ChannelRef; 5]>,
    pub effects: Vec<LayerEffect>,
}

pub struct Channel {
    pub id: ChannelId,               // 0+ color, -1 transparency, -2/-3 masks
    pub kind: ChannelKind,
    pub bounds: Rect,
    pub tiles: TileMap,
}

pub enum PixelBuffer {
    Bitmap(BitBuffer),               // 1-bit
    U8(Buffer<u8>),                  // 8-bit
    U16(Buffer<u16>),                // 16-bit
    F32(Buffer<f32>),                // 32-bit HDR
}

pub struct ColorSpace { /* model + ICC profile reference + intent */ }

pub struct TileCache {
    capacity_bytes: usize,
    entries: LruMap<TileKey, Tile>,
    scratch: ScratchManager,
}

type TileKey = (LayerId, ChannelId, CacheLevel, TileId);
```

`PixelBuffer` is the only pixel container; channels store tiles, and tiles store
one of the `PixelBuffer` variants. 32-bit is `f32` because CS6's 32-bit mode is
floating point HDR. 16-bit is `u16`, 8-bit `u8`, and Bitmap mode is packed bits —
do not force every document through an f32 path.

### Error model: `thiserror` in libraries, `anyhow` in binaries

Split by the standard Rust convention:

- **Library crates** (`pictura-core`, `pictura-color`, `pictura-codec`,
  `pictura-render`, `pictura-filters`, `pictura-script`) define typed error enums
  with `thiserror` so callers can match on failure class (e.g.
  `CodecError::UnsupportedDepth`, `RenderError::DeviceLost`).
- **Binary crate** (`pictura-app`) and top-level UI adapters use `anyhow` to add
  context and collapse errors for logging and dialogs.

Do not return `anyhow::Error` from a library API; do not hand-roll `Display`
for library errors. Boundary conversions (e.g. `image::ImageError` →
`CodecError`) live at the edge of each crate.

### Unsafe policy

- Each crate declares `#![forbid(unsafe_code)]` unless it has a documented
  reason not to.
- Crates expected to opt out (with review): `pictura-codec` (mmap and
  byte-casting of on-disk structures), `pictura-render` (FFI to the GPU stack),
  and `pictura-qt` (cxx-qt generated code).
- Unsafe blocks must be small, have a `// SAFETY:` comment stating the invariant,
  and be covered by a test that exercises the invariant.
- No `transmute` of pixel data; use explicit, reviewed casts (`bytemuck` is the
  proposed helper if added).
- All on-disk and user-provided buffers are untrusted: parsers validate lengths
  before indexing and use `checked_*` arithmetic for offset math.

### Pipeline sketch

```text
file bytes → pictura-codec::decode → Document (tiled channels)
Document + Command → pictura-filters (rayon) → touched tiles
                     or pictura-render (wgpu) → composited texture
touched tiles → TileCache → ScratchManager (when over watermark)
```

## Rust module mapping

| Module (proposed) | Responsibility | Verified crates used |
|---|---|---|
| `pictura-core::document` | Document/Layer/Channel graph | `serde`, `smallvec` (proposal) |
| `pictura-core::tile` | `Tile`, `TileMap`, `TileKey` | — |
| `pictura-core::cache` | `TileCache`, eviction, scratch spill | `memmap2` (proposal) |
| `pictura-core::error` | `CoreError` via `thiserror` | `thiserror` |
| `pictura-color::icc` | Profile parse, transform, soft proof | `lcms2` |
| `pictura-codec::psd` | PSD/PSB read/write | `flate2` (proposal) |
| `pictura-codec::raster` | PNG/JPEG/QOI/EXR dispatch | `image`, `zune-jpeg`, `qoi` |
| `pictura-codec::meta` | EXIF/XMP/IPTC | `kamadak-exif` |
| `pictura-render::compositor` | Blend + composite on GPU | `wgpu` |
| `pictura-render::cpu` | CPU fallback compositor | `rayon`, `half` |
| `pictura-filters::*` | Convolution, blur, gallery kernels | `rayon` |
| `pictura-script::command` | Command enum, undo records | `serde` |
| `pictura-qt::*` | cxx-qt bridge objects | `cxx-qt` |

Verified crate documentation URLs (all confirmed to exist on crates.io):

- `image` 0.25 — `https://docs.rs/image`
- `lcms2` 6.2 — `https://docs.rs/lcms2`
- `wgpu` 30.0 — `https://docs.rs/wgpu`
- `rayon` 1.12 — `https://docs.rs/rayon`
- `serde` 1.0 — `https://docs.rs/serde`
- `kamadak-exif` 0.6 — `https://docs.rs/kamadak-exif`
- `zune-jpeg` 0.5 — `https://docs.rs/zune-jpeg`
- `qoi` 0.4 — `https://docs.rs/qoi`
- `half` 2.7 — `https://docs.rs/half`
- `thiserror` 2.0 — `https://docs.rs/thiserror`
- `anyhow` 1.0 — `https://docs.rs/anyhow`
- `cxx-qt` 0.10 — `https://docs.rs/cxx-qt`
- `qmetaobject` 0.2 — `https://docs.rs/qmetaobject`

`smallvec`, `memmap2`, `flate2`, and `bytemuck` are proposed but were not verified
in this pass; confirm on crates.io before adoption.

## Qt6 component mapping

None inside the core. Only `pictura-qt` links Qt. It is proposed to be built as a
`staticlib` crate imported into CMake via Corrosion (`cxx_qt_import_crate`) and
exposing QML modules for panel models; this follows the documented cxx-qt CMake
integration path. See `ARCH-001`.

## Data-model impact

This section is the model itself, so the impact is the whole document graph.

- **Serialization:** the model maps 1:1 onto PSD/PSB sections (header, color-mode
  data, image resources, layer/mask info, image data). PSB uses the version-2
  marker and 64-bit section lengths. Round-tripping CS6 files requires preserving
  unknown image-resource blocks and additional layer-information keys verbatim.
- **Undo:** records are `Command` values carrying touched tile coordinates and
  prior pixel bytes; snapshots are full-document clones. This matches CS6's
  documented claim that full-image operations cost a full copy while small
  strokes cost less.
- **Metadata:** ICC, EXIF, and XMP are stored as opaque blobs (with typed accessors
  where needed) so unknown metadata survives a save.
- **IDs:** layer/channel IDs are stable within a document and preserved on load,
  matching the format's ID-seed resource behavior.

## Edge cases

- **Depth conversion:** 1/8/16/32-bit paths must be explicit; never silently
  narrow 16→8 or 32→16.
- **CMYK/Lab/Multichannel/Duotone:** supported in the model even if some filters
  are RGB-only; unsupported ops must refuse, not convert silently.
- **Empty layers / zero-area masks:** `Rect` and `TileMap` must accept empty
  extents without panicking.
- **Huge PSB docs:** no whole-image `Vec`; only tiles.
- **Malformed input:** length fields on disk are adversarial (the format is full
  of length markers); every offset update is checked.
- **ZIP-with-prediction (method 3):** must round-trip; if the decoder is
  incomplete, fail with a typed error rather than writing corrupt data.
- **Endianness:** on-disk PSD is big-endian; all readers/writers byte-swap
  explicitly.
- **Integer overflow in `(bottom-top)*(right-left)` tile math:** use `usize`/`u64`
  checked arithmetic.

## Parity acceptance criteria

1. Given a CS6-exported PSD, `pictura-codec` decodes header, channels, depth, and
   color mode to values matching `psd-tools` or Adobe's own readback, and re-encode
   is byte-compatible for the pixel data section within lossless tolerance.
2. Given a PSB with a dimension above 30,000 px, the core accepts it (≤300,000)
   and operates tile-by-tile without a whole-image allocation.
3. Given 16-bit and 32-bit documents, a filter applied to each preserves the
   buffer variant; no implicit down-conversion occurs.
4. Given a malformed length field, decoding returns `CodecError` and never panics
   or reads out of bounds (fuzz corpus required).
5. Given a library crate, its public API returns typed `thiserror` errors; no
   `anyhow::Error` appears in a library public signature (checked by a lint or
   review rule).
6. Given `#![forbid(unsafe_code)]`, every crate that opts out has a documented
   justification and at least one test per unsafe block's invariant.

## Sources

- `https://paulbourke.net/dataformats/psdpsb/psdpsb.html` — Adobe Photoshop File Formats Specification (mirror): header fields, dimension/channel/depth/mode limits, compression codes, channel IDs, resource IDs (ICC/EXIF/XMP), big-endian note.
- `https://doc.rust-lang.org/cargo/reference/workspaces.html` — workspace conventions (virtual manifest, shared lockfile/target, `workspace.dependencies`, `workspace.lints`).
- `https://docs.rs/image` — `image` crate existence/version.
- `https://docs.rs/lcms2` — `lcms2` crate existence/version.
- `https://docs.rs/wgpu` — `wgpu` crate existence/version.
- `https://docs.rs/rayon` — `rayon` crate existence/version.
- `https://docs.rs/serde` — `serde` crate existence/version.
- `https://docs.rs/kamadak-exif` — `kamadak-exif` crate existence/version.
- `https://docs.rs/zune-jpeg` — `zune-jpeg` crate existence/version.
- `https://docs.rs/qoi` — `qoi` crate existence/version.
- `https://docs.rs/half` — `half` crate existence/version.
- `https://docs.rs/thiserror` — `thiserror` crate existence/version.
- `https://docs.rs/anyhow` — `anyhow` crate existence/version.
- `https://docs.rs/cxx-qt` — `cxx-qt` crate existence/version.
- `https://docs.rs/qmetaobject` — `qmetaobject` crate existence/version.
- `https://kdab.github.io/cxx-qt/book/getting-started/5-cmake-integration.html` — cxx-qt staticlib + Corrosion CMake build model.
- `https://web.archive.org/web/20140204041700/http://blogs.adobe.com/crawlspace/2012/10/how-to-tune-photoshop-cs6-for-peak-performance.html` — 128 KB tile default, RAM/scratch model.
- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — CS6 64-bit performance note.

## Open questions

- **Tile size and memory layout** are not fixed in the format; CS6's 128 KB is a byte count, not a pixel geometry. *Resolves with:* a benchmark comparing square 128/256/512 px tiles under the `ARCH-003` budgets.
- **Whether `pictura-core` should be `no_std`-capable** is undecided; it affects the allocator and error types. *Resolves with:* a decision on embedded/CLI reuse (likely no).
- **`memmap2` vs. explicit file I/O for scratch** is deferred. *Resolves with:* measuring mmap page-fault cost versus buffered reads on large PSB files.
- **Which crates must break `forbid(unsafe_code)`** is provisional. *Resolves with:* implementation spikes in codec/render.
- **Exact set of third-party crates** (`smallvec`, `bytemuck`, `flate2`, `quick-xml`) is unverified here. *Resolves with:* crates.io review against the dependency ladder.
- **ZIP-with-prediction round-trip coverage** must be measured. *Resolves with:* a test corpus of CS6-saved PSB files.
