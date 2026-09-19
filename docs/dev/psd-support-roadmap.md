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

## Gap list

| # | Gap | Evidence | Impact |
|---|---|---|---|
| G1 | ZIP / ZIP-with-prediction unsupported (composite + layer channels) | `read.rs` `read_psd` match, `read_channel_data` | **Open blocker** for many real PSDs |
| G2 | Color modes beyond Gray/RGB (Bitmap, Indexed, CMYK, Multichannel, Duotone, Lab) | `read.rs` mode match, `write.rs` mode match | Open blocker |
| G3 | Color-mode data (Indexed palette, Duotone spec) dropped | `read.rs` skip, `write.rs` zero | Wrong colors / loss |
| G4 | Bit depth 1/16/32 unsupported (`PixelBuffer` is `Vec<u8>`) | `read.rs` depth check, `write.rs` | Open blocker for HDR/16-bit |
| G5 | Image resources entirely dropped (ICC, EXIF, XMP, IPTC, resolution, paths, slices, alpha names, guides, print) | `read.rs` skip, `write.rs` zero | **Save destroys metadata/profile** |
| G6 | Unknown additional-layer-info keys dropped (effects `lfx2`/`lrFX`, smart objects, text, vector masks, gradient/pattern fills, blend-if, knockout) | `read.rs` `_ => {}`, `write.rs` subset | Loss on open→save; unrendered |
| G7 | `-3` real-user-mask channel, mask params, blend ranges, global layer mask dropped | `read.rs`, `write.rs` | Loss/propagation |
| G8 | Adjustment descriptor payloads preserved but not decoded/rendered (curves, exposure, vibrance, B&W, photo filter, channel mixer, gradient map, selective color, color lookup, real `SoCo`) | `composite.rs` doc | Layer renders as no-op |
| G9 | PSB write missing; write caps at 30 000 px, always version 1 | `write.rs` | Cannot save PSB / huge docs |
| G10 | Unknown blend key aborts the whole file | `read.rs` `from_psd_key(...).ok_or` | Open blocker |
| G11 | Absent merged composite ("Maximize Compatibility" off) unhandled | `read.rs` reads compression unconditionally | Open blocker |
| G12 | Write always raw; no RLE/ZIP output | `write.rs` | Files much larger than Photoshop's |

## Phases

**P1 — Interop: open any RGB/Gray PSD.** *(shipped)*
Read ZIP (2) and ZIP-with-prediction (3) for composite and layer channels
(G1); degrade an unknown blend key to Normal instead of aborting (G10); tolerate
an absent merged composite (G11). Adds `flate2` (miniz_oxide backend, pure Rust).
No write-format change (the byte-layout golden is unchanged); write compression
moves to P3. psd-tools cannot author ZIP, so the oracle uses it as a *decoder*
of a hand-built ZIP/ZIP-with-prediction file and compares bytes with `read_psd`.

**P2 — Lossless round-trip.**
Opaque-preserve what the engine doesn't model so open→save is faithful:
image resources and color-mode data (G3/G5), unknown per-layer tagged blocks,
global layer mask, mask params, blend ranges, the `-3` channel (G6/G7). Model
gains a resource/blob store; codec captures and re-emits.

**P3 — Render preserved data.**
Decode the remaining adjustment descriptors and real fill descriptors (G8);
gradient/pattern fill layers; layer effects (`lfx2`/`lrFX`); smart-object and
text raster fallback; vector masks. Write RLE by default (G12).

**P4 — Color modes and depth.**
Indexed/Bitmap/CMYK/Lab/Multichannel/Duotone (G2) and 16/32-bit (G4) through
core model, color management, renderer, and app gating. Largest phase; needs a
16-bit sample representation in `PixelBuffer`.

**P5 — PSB write / large documents.** (G9)

**P6 — Metadata & ICC integration.** File Info + assign/convert on open/save
(overlaps P2/P4).

## Sequencing rationale

P1 first: it converts the most real files from "refuses to open" to "opens
(possibly flattened)" and is a small, self-contained codec change. P2 second:
today a save silently strips profiles, metadata, effects, smart objects, and
text, which is a data-loss hazard worse than a refusal. Rendering parity (P3+)
is the long tail and can proceed feature by feature.
