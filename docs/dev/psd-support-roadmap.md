# PSD support roadmap

Working note (`docs/dev/`). The behavioral contract lives in
`docs/01-architecture/file-formats.md` and the capability specs
`openspec/specs/psd-codec/`, `openspec/specs/psd-layer-io/`. This note tracks the
distance between the shipped codec and "fully supports PSD" and the order we
close it in.

## Where we are

`pictura-codec` reads and writes a narrow native subset:

- 8-bit only, RGB or Grayscale only.
- Composite compression 0 (raw) / 1 (RLE). Layer channel compression 0 / 1.
- Pixel layers, groups (`lsct`), raster masks (`-2`), `luni`/`lspf`/`lclr`/`iOpa`.
- 18 adjustment keys preserved opaquely; `pictura-render` decodes a subset
  (`nvrt`/`invr`, `post`, `thrs`, `brit`, `levl`, `hue2`, `SoCo` in both the
  4-byte and descriptor forms).
- Document extra channels (saved selections) in the image-data section.

Anything else is either `PsdError::Unsupported` on read or silently dropped.

Shipped: `2026-09-19-psd-opaque-preservation` (P2) captures unmodeled blocks. It
covers document-level tagged blocks including the smart-object source records
`lnkD`/`lnk2`/`lnk3`/`lnkE`, and per-layer `SoLd`/`SoLE`/`plLd`, so a smart
object's bytes and any settings they carry already survive an open→save. What is
missing is owning them: a model to resolve, render, edit, and author them.

## Gap list

| # | Gap | Evidence | Impact |
|---|---|---|---|
| G1 | ZIP / ZIP-with-prediction unsupported (composite + layer channels) | `read.rs` `read_psd` match, `read_channel_data` | **Open blocker** for many real PSDs |
| G2 | Color modes beyond Gray/RGB (Bitmap, Indexed, CMYK, Multichannel, Duotone, Lab) | `read.rs` mode match, `write.rs` mode match | Partly shipped: Bitmap/Indexed/CMYK/Lab read and normalize to RGB; Multichannel/Duotone and the lossy-in-mode save stay open |
| G3 | Color-mode data (Indexed palette, Duotone spec) dropped | `read.rs` skip, `write.rs` zero | Partly shipped: the Indexed palette is interpreted and consumed on read; the Duotone spec stays preserve-only |
| G4 | Bit depth 1/16/32 unsupported (`PixelBuffer` is `Vec<u8>`) | `read.rs` depth check, `write.rs` | Partly shipped: 16/32 read and normalize to 8-bit (`>>8` / `clamp(trunc(f*256))`), all channels narrowed; a true `u16`/`f32` sample model preserving depth stays open |
| G5 | Image resources are parsed; an embedded non-sRGB ICC profile is applied on read (converted to sRGB, stale profile dropped); EXIF/IPTC decode, XMP parse + edit with IIM sync, a File Info dialog, IPTC core-field editing, and user Assign/Convert Profile commands ship; metadata templates, sidecars, and a Color Settings policy layer are open | `read.rs` keep, `write.rs` re-emit; `image_resources.rs` parses; `icc.rs` converts/assigns; `metadata.rs`/`exif.rs`/`iptc.rs`/`xmp.rs` decode and edit | Wide-gamut files render correctly; metadata readable/editable; profiles assignable and convertible |
| G6 | Unknown additional-layer-info keys dropped (effects `lfx2`/`lrFX`, smart objects, text, vector masks, gradient/pattern fills, blend-if, knockout) | `read.rs` `_ => {}`, `write.rs` subset | Loss on open→save; unrendered |
| G7 | `-3` real-user-mask channel, mask params, blend ranges, global layer mask dropped | `read.rs`, `write.rs` | Loss/propagation |
| G8 | Adjustment descriptor payloads preserved but not decoded/rendered (version-3 `phfl` only; curves, exposure, vibrance, B&W, photo filter, channel mixer, gradient map, selective color, and color lookup now decode) | `composite.rs` doc | Layer renders as no-op |
| G9 | ~~PSB write missing~~ PSB write shipped: version-2 container, dimensions to 300 000; tagged-block big-key width + pad framing fixed | `write.rs` | Closed |
| G10 | Unknown blend key aborts the whole file | `read.rs` `from_psd_key(...).ok_or` | Open blocker |
| G11 | Absent merged composite ("Maximize Compatibility" off) unhandled | `read.rs` reads compression unconditionally | Open blocker |
| G12 | ~~Write always raw~~ RLE write shipped; ZIP output still absent | `write.rs` | RLE composite/layer channels/mask now default; ZIP write still missing |
| G13 | Smart objects are preserved opaquely but not modeled: no embedded-source node, so a smart object cannot be resolved, rendered, or re-edited | P2 holds `SoLd`/`SoLE`/`plLd` and `lnkD`/`lnk2`/`lnk3` bytes; nothing consumes them | Open blocker for Camera Raw |
| G14 | No writer for a valid smart-object pair: the `SoLd`/`SoLE` config descriptor, its `lnkD`/`lnk2`/`lnk3` source record, and the matching `uuid` that links them | `write_psd` re-emits preserved bytes but cannot author a new smart object | Open blocker for raw interop |
| G15 | Camera Raw settings are not read or written. Two storage models: `crs:` XMP for a raw opened as a Smart Object, and the `SoLd.filterFX[].Fltr` descriptor for a Camera Raw Filter smart filter | settings are preserved opaquely only; no edit round-trip | Open blocker for Camera Raw |
| G16 | No Adobe round-trip oracle or automated check; Photoshop reopening our PSD is unverified | interop is a claim with no test | Verification gap |
| G17 | Smart filters are unmodeled: `SoLd.filterFX` (Camera Raw Filter, `filterID` 2683), document `FEid`/`FXid`, and the filter mask `FMsk` | P2 preserves the bytes; nothing parses `Fltr` | Open for the CC Camera Raw Filter |

## Phases

**P1 — Interop: open any RGB/Gray PSD.** *(shipped)*
Read ZIP (2) and ZIP-with-prediction (3) for composite and layer channels
(G1); degrade an unknown blend key to Normal instead of aborting (G10); tolerate
an absent merged composite (G11). Adds `flate2` (miniz_oxide backend, pure Rust).
No write-format change (the byte-layout golden is unchanged); write compression
moves to P3. psd-tools cannot author ZIP, so the oracle uses it as a *decoder*
of a hand-built ZIP/ZIP-with-prediction file and compares bytes with `read_psd`.

**P2 — Lossless round-trip.** *(shipped)*
Opaque-preserve what the engine doesn't model so open→save is faithful:
image resources and color-mode data (G3/G5), unknown per-layer tagged blocks,
global layer mask, mask params, blend ranges, the `-3` channel (G6/G7). The
model carries the raw bytes and the codec captures and re-emits them; engine
documents keep empty storage so their bytes are unchanged. A psd-tools-authored
fixture round-trips whole-`Document`-equal and psd-tools still opens our output.
Shipped as `2026-09-19-psd-opaque-preservation`; the document-level tagged blocks
(the smart-object source records) and per-layer config descriptors it preserves
are what P2.5 consumes.

**P2.5 — Smart objects, Camera Raw Filter, and Adobe round-trip.** *(shipped)*
Own what P2 preserves. Model an embedded smart object (source bytes, filename,
filetype, config descriptor, and the `uuid` linking layer to source) on top of
the preserved blocks (G13), and author a valid `SoLd`/`SoLE` pair and its
`lnkD`/`lnk2`/`lnk3` record (G14). For Camera Raw, support the CC smart-filter
model: `SoLd.filterFX[].Fltr` with `filterID` 2683, plus the document `FEid` and
`FMsk` blocks, read and written as settings (G15/G17). Smart objects are
CS6→current CC (tolerant read plus byte-preserving write, proven only on the CC
2021 fixture). The Camera Raw settings model targets the earliest CC Camera Raw
Filter (ACR 8 / PV2012); `crs:` stays preserve-only. The round-trip is proven
against the supplied Photoshop fixtures (G16) by the `psd-tools` oracle; a
manual Photoshop reopen and a CS6/earliest-CC fixture are deferred follow-ups.
Shipped as the archived change `2026-09-19-psd-smart-object-roundtrip`.

**P3 — Render preserved data.** *(in progress)*
Decode the remaining adjustment descriptors and real fill descriptors (G8);
gradient fill layers; layer effects (`lfx2` Drop Shadow, Outer Glow, Inner
Shadow, Inner Glow, Stroke, Color/Gradient/Pattern Overlay, Satin); text (the
remaining kind — vector masks are shipped, archived `vector-mask-render`, and
`vscg` vector fill content is shipped, archived `vector-fill-content`).
**Smart-object source rendering is shipped** (archived
`2026-09-19-smart-object-source-render`): an `Embedded` smart object with no
raster proxy is rendered by decoding its payload and sampling it into the layer
rect (stored merged composite preferred, layers fallback; nearest-neighbour;
`Trnf`/warp deferred). **Adjustment payload decoding is partly shipped**: `expA`
(Exposure), `vibA` (Vibrance), and `blwh` (Black & White) decode to
`pictura-adjust` ops (archived `2026-09-19-adjustment-payload-decode`), and
`phfl` (Photo Filter, version 2) now decodes and encodes too (archived
`2026-09-19-photo-filter-adjustment-decode`) with an encoder and an Adjustments
panel `Photo Filter` entry. Gradient Map (`grdm`) now decodes to
`Adjustment::GradientMap` and encodes too (archived
`2026-09-19-gradient-map-adjustment-decode`) with an encoder and an Adjustments
panel `Gradient Map` entry. The real solid-color fill (`SoCo`) descriptor now
decodes, encodes, and rasterizes (archived
`2026-09-19-solid-color-fill-descriptor`); the real gradient fill (`GdFl`)
descriptor now decodes to `Adjustment::GradientFill`, composites generatively for
all five kinds, is fill content for rasterize, and authors through
`encode_gradient_fill` and `Layer > New Fill Layer > Gradient…` (archived
`2026-09-19-gradient-fill-layer`). Color Balance (`blnc`) now decodes to nine
`i16` shifts plus a luminosity byte and encodes too, with an Adjustments panel
entry (archived `2026-09-19-color-balance-adjustment-decode`). Channel Mixer
(`mixr`) now decodes to `Adjustment::ChannelMixer` and encodes too, with an
Adjustments panel entry; the layout is grounded on **ag-psd** because psd-tools
reads only the red row (archived `channel-mixer-adjustment-decode`). Curves
(`curv`) now decodes composite and per-channel curves into `Adjustment::Curves`
and encodes too, grounded on **ag-psd** with a psd-tools partial check (archived
`curves-adjustment-decode`). Pattern fill
(`PtFl`) now decodes to `Adjustment::PatternFill`, taking its pixels from the
document `Patt`/`Pat2`/`Pat3` pattern library (`pictura-codec::decode_patterns`)
and compositing as tiled content, and is fill content for rasterize (archived
`2026-09-19-pattern-fill-layer`). Layer effects (`lfx2`) now decode the
object-based **Drop Shadow** (`DrSh`), **Outer Glow** (`OrGl`), **Inner Shadow**
(`IrSh`), **Inner Glow** (`IrGl`), and a **Stroke** (`FrFX`),
compositing the shadows and glows behind the layer content, the inner shadow and
inner glow above it, and the stroke as a band at the content edge above it, on
the CPU, with the GPU falling back to CPU (archived
`2026-09-19-layer-effects-drop-shadow`,
`2026-09-19-layer-effects-outer-glow`,
`2026-09-19-layer-effects-inner-shadow`,
`2026-09-19-layer-effects-inner-glow`, and
`2026-09-20-layer-effects-stroke`); the object-based **Color Overlay**
(`SoFi`), **Gradient Overlay** (`GrFl`), and **Pattern Overlay** (`patternFill`)
now decode and composite above the layer content, gated by the content coverage
with the effect blend/opacity, the gradient reusing the shared gradient
geometry and the pattern the document pattern library (archived
`2026-09-20-layer-effects-overlays`); the object-based **Satin** (`ChFX`) now
decodes and composites an interior directional band from the blurred content
matte, optionally inverted, gated by the content coverage, and composited above
it (archived `2026-09-20-layer-effects-satin`); **Bevel & Emboss** (`ebbl`) now
decodes and, for the Inner + Smooth (`InrB`/`SfBL`) slice, composites a lit height
field confined to the content above it, other styles/techniques decoding to a
no-op (archived `2026-09-20-layer-effects-bevel`). The legacy `lrFX` block is
now decoded into the same typed effect model and rendered through the shipped
renderers, with a single `lfx2`-over-`lrFX` resolver (archived
`layer-effects-legacy-lrfx`), so the layer-effects family covers both the
object-based `lfx2` and the legacy `lrFX` encodings. The **Stroke** (`FrFX`) now
also fills from a **gradient** (`PntT` `GrFl`, `Grad`) or **pattern** (`PntT`
`Ptrn`, `Ptrn`) source over the same content-edge band, the last deferred `lfx2`
stroke piece (archived `2026-09-20-layer-effects-stroke-fills`). That change also fixed a cross-cutting bug: every `lfx2`
effect blend mode now decodes the `BlnM` descriptor vocabulary instead of the
layer-key one, and the effect goldens were regenerated. Selective Color (`selc`)
now decodes to `Adjustment::SelectiveColor` and encodes too, with an Adjustments
panel entry; the ten-plate layout (reserved plate 0 plus nine named ranges) is
grounded three ways (libpsd, ag-psd, psd-tools framing) and the kernel follows
libpsd's integer CMYK pipeline (archived `selective-color-adjustment-decode`).
Remaining:
version-3 `phfl` and the text kind (vector masks are shipped,
archived `vector-mask-render`, and `vscg` vector fill content is shipped,
archived `vector-fill-content`).
**Color Lookup (`clrL`) is now shipped** (archived
`2026-09-22-color-lookup-adjustment-decode`): the block decodes to
`Adjustment::ColorLookup`, an embedded `.CUBE` `3DLUT` is sampled trilinearly,
and `encode_color_lookup`/`identity_cube` author a block; abstract-profile,
device-link, and non-`.CUBE` payloads are no-ops (marked ceiling, no Adobe pixel
parity). That closes the whitelisted adjustment-key set; only version-3
`phfl` stays deferred.
**Curves (`curv`) is now shipped**: the original deferral reason — a
single-composite model versus Photoshop's per-channel curves, and an ungrounded
channel-bitmap order — is addressed by the per-channel `CurvesParams` model,
with the per-channel-then-composite order marked an assumption (not
Photoshop-verified). Remaining P3:
those keys and kinds, and write RLE by default (G12).
**RLE write is shipped** (archived
`2026-09-19-psd-rle-write`): the merged composite (color + document extra
channels), layer color channels, and the raster mask are PackBits-encoded;
preserved verbatim channels stay byte-for-byte. ZIP write remains.

**P4 — Color modes and depth.** *(partly shipped)*
Indexed/Bitmap/CMYK/Lab now read and normalize to RGB (G2/G3, archived
`color-mode-read`), with a documented lossy-in-mode save (`write_psd` writes the
working mode) and a status-bar conversion notice in the app. 16/32-bit depth now
reads and normalizes to 8-bit on load too (G4, archived `depth-read`):
`source_depth` records the original, every channel (color, alpha/mask,
spot/extra, and document extras) is narrowed, `write_psd` writes 8-bit
(lossy-in-depth), and the app shows a conversion notice. Multichannel/Duotone
(no natural RGB mapping / needs the spot-ink spec) remains open, along with
write-side re-encoding to the source mode. The remaining depth work is a true
`u16`/`f32` sample model in `PixelBuffer` that preserves depth (no narrowing on
load) and a 32-bit HDR tone map (the shipped path is display-referred, clipping
at 1.0).

**P5 — PSB write / large documents.** *(shipped)* (G9)
`write_psd` emits a version-2 container when the source document was a PSB
(`Document.is_psb`) or either dimension exceeds 30 000, and `write_psb` forces
one; dimensions are accepted to 300 000. The PSB container widens the
layer-and-mask section length, the layer-info length, and each per-channel data
length to `u64`, and the RLE count entries to `u32`. The tagged-block framing it
exposed is now correct: a PSB big key (the psd-tools `_BIG_KEYS` set) carries an
8-byte length; a per-layer block declares an even length with the pad inside it,
while a document-level block declares its exact length and is padded externally
to 4, and a preserved document-level block is re-framed to the output container's
width. `iOpa` is written as a 4-byte `B3x` value. Shipped as the archived change
`2026-09-19-psb-write`; proven by the psd-tools oracle (authored-smart-object
PSB, PSD→PSB reframe, odd/non-4-multiple block framing). Remaining ceilings: an
`8B64` signature is normalized to `8BIM` on re-frame, and the app cannot yet open
a >30 000 PSB (import probe budget).

**P6 — Metadata & ICC integration.** File Info + assign/convert on open/save
(overlaps P2/P4). The resource parser shipped (archived
`2026-09-22-psd-image-resources`): `pictura_codec::decode_image_resources`
returns typed `(id, name, data)` records and exposes the ICC/EXIF/XMP/IPTC ids,
with the raw section still re-emitted byte-for-byte. The embedded **ICC profile
is now applied on read** (archived `2026-09-22-psd-icc-convert`): an RGB file
with a non-sRGB profile is converted to the sRGB working space (composite and
layer color channels, relative colorimetric), the original bytes are recorded in
`Document.source_icc`, and resource 1039 is dropped so the save is not
mis-tagged; a Grayscale or untransformable document is left untouched. A
read-only **File Info** surface is now shipped too (archived
`2026-09-22-psd-file-info`): `File > File Info…` shows decoded EXIF (1058/1059)
and IPTC-IIM (1028) fields plus the raw XMP packet (1060), via
`pictura_codec::{parse_exif, parse_iptc, read_metadata}`; the fixture is proven
against `exiftool`. **IPTC editing and write-back** now ship (archived
`2026-09-22-psd-iptc-write`): the dialog's IPTC page edits the six core fields
and OK writes them into resource 1028 as one undo state
(`frame_image_resource`, `Iptc::set`/`remove`, `encode_iptc`,
`set_iptc_fields`), preserving every other resource byte-for-byte and leaving a
section that does not decode losslessly untouched; a save persists the edit
(proven via `exiftool`), and clearing a field removes its record. Ceilings: XMP
is raw text and is not edited (no field extraction, no IIM↔XMP sync), a
multi-value EXIF tag decodes as raw bytes, only the six core IPTC fields are
editable, and IIM values are written as UTF-8. **Assign Profile and Convert to
Profile** now ship (archived `2026-09-22-assign-convert-profile`): a document
carries an optional working-profile ICC, assign retags without touching pixels,
convert transforms the composite and every layer (including group children) and
retags, the canvas converts the final composite to sRGB for display, and a save
tags resource 1039; each is one undo step. Ceilings: RGB 8-bit only, the three
built-in profiles only (no installed-profile discovery), fixed
relative-colorimetric intent with no black-point compensation, dither, or
flatten. **XMP is now parsed and edited** (archived
`2026-09-22-xmp-metadata`): resource 1060 is decoded into a fixed typed property
set shown in a read-only Description category, and editing the six IPTC-Core
fields writes both the XMP packet and the IIM record so the two channels agree.
The packet is patched by byte-span — unmanaged bytes, unknown namespaces, and
the wrapper survive verbatim — and a packet that cannot be safely rewritten is
left untouched. Ceilings: only the nine managed properties are modelled (IPTC
Extension and arbitrary RDF are preserved but not editable), the raw packet is
read-only, EXIF is not editable (camera data is read-only per `WF-010`), and a
non-primary `x-default` alternative collapses to the edited scalar. Still open:
metadata templates, sidecars, and a Color Settings policy layer (`WF-011`).

## Smart objects and Camera Raw interop

PSD is the one format that can round-trip Camera Raw edits, through embedded
smart objects. Two storage models show up, and the reference files confirm the
container and the CC model.

**Smart object container.** The layer carries a config descriptor in `SoLd`
(legacy `PlLd`) holding placement, transform, warp, and a `uuid`. The
document-level `lnkD`/`lnk2`/`lnk3` tag holds a list of source records; the one
whose `uuid` matches the layer is the embedded source. For embedded content its
`kind` is DATA and its payload is the file bytes. Photoshop also writes an empty
`lnkE`. Confirmed from `assets/test_with_smart_object01.psd`: record version 7,
`filetype` `8BPB`, `creator` `8BIM`, payload an embedded PSB, `child_id` a single
NUL, `mod_time` 0.0, `lock_state` 0.

**Camera Raw settings, two models.**

- A raw opened as a Smart Object stores `crs:` XMP in the embedded raw payload.
  This is the CS6 path; we have no reference file for it.
- A Camera Raw Filter applied as a smart filter stores its settings inside the
  layer's `SoLd` descriptor at `filterFX.filterFXList[].Fltr`, with `filterID`
  2683. This is the a reference build path and `assets/test_with_smart_object02.psd`
  confirms it. `Fltr` uses short keys that map to the `FILT-100` tabs: `Temp`,
  `Tint`, `WBal`, `Sett`, and PV2012 Basic `Ex12`/`Cr12`/`Hi12`/`Sh12`/`Wh12`/
  `Bk12`/`Cl12`/`Vibr`; HSL `RHue`/`RSat`/`GHue`/`GSat`/`BHue`/`BSat` and
  `HA_*`/`SA_*`/`LA_*` per eight ranges; split toning `STSH`/`STSS`/`STHH`/`STHS`/
  `STB`; tone curve `PC_*` parametric and `Crv`/`CrvR`/`CrvG`/`CrvB`; detail
  `Shrp`/`ShpR`/`ShpD`/`ShpM` and `LNR`/`CNR`; lens `LPEn`/`MDis`/`VigA`/`Per*`/
  `DfP*`; effects `GRNA`/`GRNS`/`GRNF` and `PCV*`; calibration `CamP`/`CP_D`/
  `PrVe`. `Dhze` (Dehaze) and `Upri`/`GuUr` (Upright XMP) are CC-only.

**Scope.** Smart objects are CS6→current CC: tolerant read plus byte-preserving
write, proven only on the a reference build fixture. The Camera Raw settings model targets
the earliest CC Camera Raw Filter (ACR 8 / PV2012); `crs:` stays preserve-only.
The app exposes `Layer > Smart Objects > Convert to Smart Object` (archived
`2026-09-19-convert-to-smart-object`): it keeps the raster proxy and authors the
embedded `SoLd`/`lnk2`. `Layer > Rasterize > Smart Object` (archived
`2026-09-19-rasterize-smart-object`) materializes the content into the layer
channels and drops the preserved `SoLd`/`SoLE`/`plLd` block and its document
`lnk*` record (`pictura-codec::remove_linked_source`). `File > Place…` (archived
`2026-09-19-place-smart-object`) inserts a PSD/PSB as a channel-less top
smart-object layer that renders from its embedded source. `Layer > Smart
Objects > Replace Contents…` (archived `2026-09-19-replace-smart-object-contents`)
swaps the embedded source while preserving the layer's transform and re-authors
a fresh link on save. `File > Open As Smart Object…` (archived
`2026-09-19-open-as-smart-object`) opens a PSD/PSB as a new untitled document
with one embedded smart-object layer. `Layer > Smart Objects > Export
Contents…` (archived `2026-09-19-export-smart-object-contents`) writes the stored
payload byte-for-byte and records no state. `Layer > Smart Objects > Edit
Contents…` (archived `2026-09-19-edit-smart-object-contents`) opens the embedded
source in an untitled editor tab and re-embeds it on save. Linked objects are
deferred.

Double-clicking the layer reopens ACR from the stored source and settings. CS6
supports embedded objects only; linked objects (`lnkE`, external paths) are CC
2014, so a raw smart object is self-contained and large.

Open items: a 16-bit fixture, produced later, and whether Photoshop regenerates
or trusts our merged composite.

## Non-PSD image import

`File > Open` and `File > Place…` accept common raster images
(PNG/JPEG/GIF/BMP/TIFF/WebP) beside the native PSD/PSB path; Qt decodes at the
app boundary and the engine builds its own document/layer structures from the
pixels (shipped as `2026-09-19-image-import`). OS file drag-and-drop is shipped
as `2026-09-19-file-drop-routing`: a drop on the document canvas places each
image into the current document, while a drop on the tab strip, menu bar, or
options bar opens each file as its own tab, reusing the same PSD-native vs
Qt-decode routing. Free Transform shipped as `2026-09-19-free-transform-mode`:
a successful place (menu command or canvas drop) selects the new layer and enters
an interactive move/scale/rotate session that commits one state on Enter and
cancels bit-identically on Escape. Skew, distort, perspective, and warp are
deferred follow-ups.

## Reference fixtures

Supplied, currently in `assets/`, to move under
`crates/pictura-codec/tests/fixtures/` when the oracle lands:

- `test_with_smart_object01.psd`: 512x512 RGB 8-bit, one embedded PSB smart
  object with no filter. Confirms the `SoLd`/`lnk2` container schema.
- `test_with_smart_object02.psd`: the same, plus a Camera Raw Filter smart
  filter (`filterFX`, `filterID` 2683) and the document `FEid`/`FMsk` blocks.
  Produced with Photoshop a reference build. Confirms the `Fltr` settings model.

Both are self-produced; record provenance when they move.

To produce later: a 16-bit variant of either, which needs roadmap G4 depth
support.

They feed a round-trip oracle: read with `read_psd`, write with `write_psd`,
re-read, and compare the smart-object and smart-filter blocks with `psd-tools`.
The final check is manual: open our saved file in Photoshop CC and confirm the
smart object and the Camera Raw Filter are present and editable.

## Sequencing rationale

P1 first: it converts the most real files from "refuses to open" to "opens
(possibly flattened)" and is a small, self-contained codec change. P2 second:
today a save silently strips profiles, metadata, effects, smart objects, and
text, which is a data-loss hazard worse than a refusal. P2.5 third: preserving a
smart object is only half the contract; owning it, including the CC Camera Raw
Filter settings it carries, is what Adobe interop is judged on and the only
Camera Raw support we can validate against files we can produce. Rendering
parity (P3+) is the long tail and can proceed feature by feature.
