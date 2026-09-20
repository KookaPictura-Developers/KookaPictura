## Why

`read_psd` accepts only RGB and Grayscale, so a Bitmap, Indexed, CMYK, or Lab
PSD is refused with `PsdError::Unsupported` and will not open at all
(`crates/pictura-codec/src/read.rs:46`). The second half of roadmap gap G2 is
that the color-mode-data section — the 768-byte Indexed palette and the Duotone
spec — is preserved opaquely but never interpreted (G3), so even if the mode
gate were widened an Indexed file would still render with wrong colors. This is
the first P4 slice: make the color modes that have a faithful or standard
conversion readable, and be explicit that the document is normalized on save.

## What Changes

- `read_psd` accepts color modes Bitmap (0), Indexed (2), CMYK (4), and Lab (9)
  in addition to Grayscale (1) and RGB (3), and accepts depth 1 for Bitmap
  (Photoshop's Bitmap is 1-bit). It converts each to the engine's working mode:
  Bitmap and Indexed → RGB exactly (bit expansion and palette lookup), CMYK and
  Lab → RGB by standard profile-free formulas marked as approximations. The
  document's `mode` becomes `Rgb` (Grayscale stays Grayscale), `depth` becomes
  `Eight`, and every layer's color planes are converted the same way.
- **Indexed palette interpretation (G3).** The 768-byte color-mode-data section
  is read as 256 red, 256 green, then 256 blue bytes and used to expand the
  single index plane. Because the document is normalized to RGB, the palette is
  consumed and not re-emitted.
- The document records what it was: `Document.source_mode: Option<ColorMode>` is
  `Some(m)` when a file's header mode `m` was normalized, `None` otherwise. The
  app reports it ("Converted from CMYK") so the lossy-in-mode save is not
  silent.
- `write_psd` is unchanged: it keeps writing only RGB/Grayscale documents. A
  file opened from a non-RGB mode therefore saves as RGB — a documented,
  lossy-in-mode save, no longer hidden.
- Deferred and still `PsdError::Unsupported`: Multichannel (7) and Duotone (8)
  (no natural RGB mapping; Duotone depends on the spot-ink spec), and all
  16/32-bit depth (G4 needs a `u16` sample model).
- Odd-row handling: Bitmap depth-1 raw and RLE rows are `ceil(width/8)` bytes,
  MSB-first, 1 = black, 0 = white; ZIP compression at depth 1 stays unsupported.
- New fixture set and psd-tools oracle proving the palette, the CMYK/Lab
  formulas, the Bitmap bit packing, and the normalized round-trip.
- **BREAKING**: none at the consumer API; `Document` gains one `Option` field
  (defaults to `None` via the existing `Default` impls).

## Capabilities

### New Capabilities

- `psd-color-modes`: reading color modes beyond RGB/Grayscale, converting them
  to the working mode (Bitmap, Indexed, CMYK, Lab), interpreting the Indexed
  palette, normalizing the document on read, and reporting the conversion to
  the application.

### Modified Capabilities

- `psd-codec`: the header-validation requirement widens its accepted color-mode
  and depth set, and the composite-read requirement handles Bitmap depth-1 rows
  and stops retaining an Indexed palette verbatim.

## Impact

- `crates/pictura-core/src/lib.rs`: new `Document.source_mode` field and a
  `color_channels`-driven conversion entry point.
- `crates/pictura-codec/src/read.rs`, `common.rs`: mode dispatch, depth-1
  composite and layer row lengths, palette parse, conversions, normalization.
- New `crates/pictura-codec/src/color_mode.rs` (or equivalent) for the
  conversion formulas.
- `crates/pictura-codec/src/tests.rs`, `tests/oracle.rs`,
  `tests/fixtures/{indexed,cmyk,lab,bitmap}.psd`, `scripts/generate-fixtures.py`.
- `crates/pictura-app`: a `mode_notice` invokable, the status-bar notice in
  `frame.cpp`, and one self-test check (exit code 297).
- No new external dependency; the Lab formula is std-only.
