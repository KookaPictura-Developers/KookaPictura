## Context

`read.rs` parses the composite compression word (`read.rs:121`) and each layer
channel's (`read.rs:899`) and dispatches on raw/RLE/ZIP/ZIP-prediction, but
discards the kind. `write.rs` hardcodes `COMPRESSION_RLE` at two sites — the
image-data section (`write.rs:647`) and layer channels/mask (`write.rs:104`,
`:129`) — via `encode_scanlines`/`rle_channel`. `common.rs` already defines
`COMPRESSION_RAW/RLE/ZIP/ZIP_PREDICTION`. `flate2` is a dependency and already
writes zlib (`patterns.rs:358`, tests). The reader's ZIP-prediction undo is
`depth.rs::undo_prediction`; for 8-bit it is a per-row `wrapping_add`, so the
forward transform is a per-row `wrapping_sub` over the concatenated planes with
a row length equal to the channel width.

The writer rejects any non-8-bit document (`write.rs:562`), so ZIP/ZIP-prediction
write only ever meets depth 8.

## Goals / Non-Goals

**Goals:**
- An open→save preserves the source compression for the composite and for layer
  channels (RLE, ZIP, or ZIP-with-prediction).
- A constructed document still writes RLE, so existing output and the RLE golden
  are unchanged.
- `psd-tools` decodes our ZIP/ZIP-prediction output to the same pixels.

**Non-Goals:**
- Per-channel mixed-kind fidelity within a category (one kind per category).
- ZIP for depth 1/16/32 (the writer is 8-bit only).
- A user-facing compression preference (preserve-on-write only).
- Changing the read path or the `RawChannel` verbatim behaviour.

## Decisions

- **Store the kind on `Document`, not on `Channel`.** `Channel` has no `Default`
  and ~185 literal sites, so a per-channel field is a large, low-value edit. Two
  `Document` fields (`composite_compression`, `layer_compression`), default RLE,
  keep the change to `Document::new`, its manual `Default`, and the two read
  literals. Ceiling: a file mixing kinds within a category normalizes to the
  first kind seen; marked `ponytail:`.
- **A `Compression` enum in `pictura-core`** (`Rle` default, `Raw`, `Zip`,
  `ZipPrediction`) carries the kind as a value type; `u16` codes map to/from the
  PSD field.
- **Dispatch in one helper.** Replace `rle_channel` with a `channel_stream(kind,
  width, height, plane, psb)` used by both the layer channel and the mask, and
  add `zip_scanlines(planes, width, height, predict)` for the image-data section;
  the composite path picks `encode_scanlines` or `zip_scanlines` from the kind.
- **Forward prediction reuses the reader's row model.** One `wrapping_sub` pass
  over `height`-row-strided planes, then zlib. Only 8-bit, so no 16/32 shuffle.
- **Preserve-on-write is required for round-trip equality.** Many tests compare
  the whole `Document` after read→write→read; once a compression field exists,
  writing a different kind than read would fail that equality, so the writer must
  honour the field.

## Risks / Trade-offs

- **Round-trip byte equality tests that read raw-composite fixtures** now write
  raw (previously RLE). Their `Document`-equality assertions still hold because
  the field is read back consistently; no test pins the written compression of a
  fixture document.
- **ZIP output size vs RLE** is not always smaller for tiny or noisy channels;
  this change does not choose the smaller, it preserves the source (a
  `ponytail:` note records that a future change could pick the smaller).
- **A corrupted/unknown stored kind** would be a programming error; the writer
  only ever stores the four known kinds.

## Migration Plan

Additive to the model with an RLE default, so constructed documents and the
golden are unchanged. Fixture round-trips gain compression fidelity. No data
migration.

## Open Questions

- Whether a later change should let the user choose compression, or pick the
  smaller of RLE/ZIP per channel, is deferred.
