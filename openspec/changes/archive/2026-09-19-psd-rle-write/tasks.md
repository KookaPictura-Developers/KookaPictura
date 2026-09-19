## 1. Encoder

- [x] 1.1 In `crates/pictura-codec/src/write.rs`, add a deterministic PackBits row encoder: runs of three or more equal bytes become a repeat packet (control `257 - n`, `n` in 3..=128) and the rest literal packets of up to 128 bytes (control `n - 1`); the choice is fixed so repeated writes are byte-identical.
- [x] 1.2 Add `fn rle_channel(width: usize, height: usize, plane: &[u8]) -> Result<Vec<u8>, PsdError>` that emits the complete on-disk PSD stream: the `COMPRESSION_RLE` (`1`) word, one 2-byte big-endian count per scanline, then the packed rows in row-major order.
- [x] 1.3 Guard each row's packed length: if it exceeds `u16::MAX`, return `PsdError::Invalid` naming the RLE scanline; never truncate the count. Note in a comment that PSD maximum width (30 000) cannot reach the limit.
- [x] 1.4 Redesign `OutChannel` so `Encoded` carries the complete RLE stream (so `declared_len` is the stream length and `write` emits it unchanged) and `Verbatim` stays an already-complete preserved stream emitted byte-for-byte.

## 2. Composite, layer channel, and mask emission

- [x] 2.1 In `write_psd`, replace the raw image-data emission (`COMPRESSION_RAW` + `doc.composite.data` + raw `doc.channels`) with a single `COMPRESSION_RLE` word followed by the `rle_channel` stream of every header channel: the composite color planes and each document extra channel, channel-major then row-major, mirroring `read.rs::read_rle`.
- [x] 2.2 In `write_layer_info`, route every engine-encoded layer channel through `rle_channel` at the layer's channel width/height — the layer color channels (`channel.data`) and the raster mask (`-2`, width × height from the mask rect).
- [x] 2.3 Preserved-verbatim guarantee: `Layer.raw_channels` (unknown compressed streams) continue to use `OutChannel::Verbatim` and are emitted byte-for-byte, never re-encoded; unmodeled blocks, global-layer-mask bytes, mask extras, blending ranges, and section extras are untouched.
- [x] 2.4 Confirm no read path or `psd-layer-io` decode behavior changes.

## 3. Tests

- [x] 3.1 In `crates/pictura-codec/src/tests.rs`, add an encoder unit test: encode a plane with `rle_channel`, decode it with the existing `decode_rle_channel`/`decode_packbits` path, and assert the bytes match (including a repeated-run row and a literal-only row).
- [x] 3.2 Add a "written composite declares compression 1" test: write a default document and assert the image-data section's 2-byte compression word is `1` (offset 38 in the layerless layout).
- [x] 3.3 Add a size test: a compressible image (e.g. a solid 64×64 composite) writes smaller than the raw serialization.
- [x] 3.4 Refresh the `default_document_bytes_are_unchanged` golden: regenerate `crates/pictura-codec/tests/fixtures/default_before.psd` from the RLE output and update the assertion message. This golden change is sanctioned by roadmap P3 (G12) — record it explicitly in the task result.
- [x] 3.5 Confirm the existing round-trip tests (RGB, Grayscale, randomized, layers/group/mask, attributes, background, smart objects) still pass unchanged.
- [x] 3.6 In `crates/pictura-codec/tests/oracle.rs`, confirm the psd-tools/ImageMagick oracle still opens and decodes our RLE output; add an assertion that psd-tools reads the RLE composite/channel pixels if the existing checks do not already cover compression 1.

## 4. Gates

- [x] 4.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, and `cargo test --workspace --doc`.
- [x] 4.2 `bash scripts/verify-full.sh` and `./build/pictura --headless --self-test`; record counts.
- [x] 4.3 `openspec validate psd-rle-write --strict` and `openspec validate --all --strict`.

## 5. Docs and state

- [x] 5.1 Update `docs/dev/psd-support-roadmap.md`: move G12 to shipped (RLE write is the default; composite + layer channels/mask), and note the P2.5/P3 wording that the write baseline is now RLE.
- [x] 5.2 Update `docs/dev/STATE.md` with the shipped change and the new golden baseline.
- [ ] 5.3 Commit the docs change separately with `TASK-ALLOWS-DOCS`.
