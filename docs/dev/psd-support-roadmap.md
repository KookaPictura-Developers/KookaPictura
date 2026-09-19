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

In flight: `openspec/changes/psd-opaque-preservation` (P2) captures unmodeled
blocks. It covers document-level tagged blocks including the smart-object source
records `lnkD`/`lnk2`/`lnk3`/`lnkE`, and per-layer `SoLd`/`SoLE`/`plLd`. So a
smart object's bytes, and the `crs:` Camera Raw settings the embedded payload
carries, already survive an open→save once that change lands. What is missing is
owning them: a model to read, render, edit, and author them.

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
| G15 | Camera Raw `crs:` settings are not read or written; the embedded raw's XMP and the document XMP resource are preserved opaquely only | opening a raw as a Smart Object cannot round-trip a settings edit | Open blocker for raw interop |
| G16 | No Adobe round-trip oracle or reference fixtures; Photoshop reopening our PSD is unverified | interop is a claim with no test | Verification gap |

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
fixture round-trips whole-`Document`-equal and psd-tools still opens our output. In flight as
`openspec/changes/psd-opaque-preservation`; document-level tagged blocks (the
smart-object source records) and per-layer config descriptors fall out of it.

**P2.5 — Smart objects and Adobe round-trip.**
Own what P2 preserves. Model an embedded smart object (source bytes, filename,
filetype, config descriptor, and the `uuid` linking layer to source) on top of
the preserved blocks (G13). Read the `crs:` Camera Raw settings that ride in the
embedded payload and the document XMP resource, and write them back (G15). When
a raw is opened as a Smart Object, author a valid `SoLd`/`SoLE` pair and its
`lnkD`/`lnk2`/`lnk3` record so Photoshop reopens it editable (G14). This half
depends on the raw decoder for the layer raster, see `FILT-100`/`WF-012`. Prove
the round-trip against Photoshop-produced fixtures (G16). Tracked as
`openspec/changes/psd-smart-object-roundtrip`.

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

PSD is the one format that can round-trip Camera Raw edits, and it does it
through embedded smart objects (`WF-012`). The mechanism, confirmed against
psd-tools and Adobe's metadata documentation:

- The layer carries a config descriptor in `SoLd`/`SoLE` (legacy `plLd`), holding
  placement, transform, warp, and the `uuid`.
- The document-level `lnkD`/`lnk2`/`lnk3` tagged block holds a list of source
  records. The one whose `uuid` matches the layer is the embedded source; for
  embedded content its `kind` is DATA and its payload is the file bytes.
- For a raw, that payload is the original raw file. The Camera Raw `crs:`
  settings are XMP in the embedded file's metadata, so preserving the payload
  preserves the settings.
- Double-clicking the layer in Photoshop reopens ACR from those bytes and
  settings. CS6 supports embedded objects only; linked objects (`lnkE`, external
  paths) are CC 2014, so a CS6 raw smart object is self-contained and large.

Unknowns that need a reference file, not more reading: whether `crs:` sits in
the embedded payload, the document XMP resource, or the config descriptor; which
`SoLd` keys Photoshop validates; the linked-record version and the
`child_id`/`mod_time`/`lock_state` fields; and whether Photoshop trusts our
merged composite.

## Reference fixtures (later phase)

Adobe round-trip cannot be proven without Photoshop-produced files. Add them
when the whole pipeline is validated, as reference fixtures under
`crates/pictura-codec/tests/fixtures/`:

- a PSD with an embedded raw smart object, ideally with non-default `crs:`
  settings;
- a 16-bit equivalent;
- an embedded raster smart object (no raw) as a control.

They feed a round-trip oracle: read with `read_psd`, write with `write_psd`,
re-read, and compare the smart-object blocks and settings with `psd-tools`. The
final check is manual: open our saved file in Photoshop and confirm the smart
object is present and editable. Fixture provenance must be recorded, self-
produced, no Adobe assets.

## Sequencing rationale

P1 first: it converts the most real files from "refuses to open" to "opens
(possibly flattened)" and is a small, self-contained codec change. P2 second:
today a save silently strips profiles, metadata, effects, smart objects, and
text, which is a data-loss hazard worse than a refusal. P2.5 third: preserving a
raw smart object is only half the contract; owning it is what lets Camera Raw
edits survive a round-trip and what Adobe interop is judged on. Rendering parity
(P3+) is the long tail and can proceed feature by feature.
