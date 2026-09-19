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
  (`nvrt`/`invr`, `post`, `thrs`, `brit`, `levl`, `hue2`, 4-byte `SoCo`).
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
| G2 | Color modes beyond Gray/RGB (Bitmap, Indexed, CMYK, Multichannel, Duotone, Lab) | `read.rs` mode match, `write.rs` mode match | Open blocker |
| G3 | Color-mode data (Indexed palette, Duotone spec) dropped | `read.rs` skip, `write.rs` zero | Wrong colors / loss |
| G4 | Bit depth 1/16/32 unsupported (`PixelBuffer` is `Vec<u8>`) | `read.rs` depth check, `write.rs` | Open blocker for HDR/16-bit; a 16-bit raw smart object is downgraded on save |
| G5 | Image resources entirely dropped (ICC, EXIF, XMP, IPTC, resolution, paths, slices, alpha names, guides, print) | `read.rs` skip, `write.rs` zero | **Save destroys metadata/profile** |
| G6 | Unknown additional-layer-info keys dropped (effects `lfx2`/`lrFX`, smart objects, text, vector masks, gradient/pattern fills, blend-if, knockout) | `read.rs` `_ => {}`, `write.rs` subset | Loss on open→save; unrendered |
| G7 | `-3` real-user-mask channel, mask params, blend ranges, global layer mask dropped | `read.rs`, `write.rs` | Loss/propagation |
| G8 | Adjustment descriptor payloads preserved but not decoded/rendered (curves, exposure, vibrance, B&W, photo filter, channel mixer, gradient map, selective color, color lookup, real `SoCo`) | `composite.rs` doc | Layer renders as no-op |
| G9 | PSB write missing; write caps at 30 000 px, always version 1 | `write.rs` | Cannot save PSB / huge docs |
| G10 | Unknown blend key aborts the whole file | `read.rs` `from_psd_key(...).ok_or` | Open blocker |
| G11 | Absent merged composite ("Maximize Compatibility" off) unhandled | `read.rs` reads compression unconditionally | Open blocker |
| G12 | Write always raw; no RLE/ZIP output | `write.rs` | Files much larger than Photoshop's |
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

**P3 — Render preserved data.**
Decode the remaining adjustment descriptors and real fill descriptors (G8);
gradient/pattern fill layers; layer effects (`lfx2`/`lrFX`); text; vector masks;
render a smart object's source through the P2.5 model, rasterizing only contents
the renderer cannot reproduce. Write RLE by default (G12).

**P4 — Color modes and depth.**
Indexed/Bitmap/CMYK/Lab/Multichannel/Duotone (G2) and 16/32-bit (G4) through
core model, color management, renderer, and app gating. Largest phase; needs a
16-bit sample representation in `PixelBuffer`.

**P5 — PSB write / large documents.** (G9)

**P6 — Metadata & ICC integration.** File Info + assign/convert on open/save
(overlaps P2/P4).

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

Double-clicking the layer reopens ACR from the stored source and settings. CS6
supports embedded objects only; linked objects (`lnkE`, external paths) are CC
2014, so a raw smart object is self-contained and large.

Open items: a 16-bit fixture, produced later, and whether Photoshop regenerates
or trusts our merged composite.

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
