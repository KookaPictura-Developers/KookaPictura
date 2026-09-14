## 1. Core layer/channel/mask model (M1-A)

- [x] 1.1 Add `PsdRect` with signed `i32` edges and `width()`/`height()`
- [x] 1.2 Add `Channel { id: i16, data: Vec<u8> }` for planar per-channel data
- [x] 1.3 Add `LayerMask { rect, default_color, disabled, flags, data }`
- [x] 1.4 Add `Layer { name, rect, blend, opacity, clipping, visible, mask, adjustment, channels, children, is_group }`
- [x] 1.5 Add `BlendMode` with the 27 CS6 layer modes plus group-only `PassThrough`
- [x] 1.6 Add `BlendMode::LAYER_MODES` and `to_psd_key`/`from_psd_key` with `None` for unknown keys
- [x] 1.7 Add `layers` and document-level `channels` to `Document`; initialize empty in `Document::new`
- [x] 1.8 Write the authoritative type contract in `crates/pictura-core/MODEL.md`
- [x] 1.9 Unit-test all 27 blend keys round-trip, Pass Through is group-only, and unknown keys return `None`
- [x] 1.10 Unit-test layer/group/mask construction, signed/off-canvas rect sizes, and empty `Document::new`

## 2. PSD layer-section read (M1-B)

- [x] 2.1 Add a bounds-checked `Reader` returning `PsdError::Truncated` instead of panicking
- [x] 2.2 Read the layer-and-mask section lengths (4-byte PSD, 8-byte PSB) and the layer info length
- [x] 2.3 Parse the signed layer count and read each layer record
- [x] 2.4 Parse the layer mask / adjustment data block and mask flags
- [x] 2.5 Parse `'luni'` Unicode names and prefer them over the Pascal name
- [x] 2.6 Parse `'lsct'` section markers (divider/open/closed) and prefer the `lsct` blend key
- [x] 2.7 Read per-channel image data with raw (`0`) and PackBits RLE (`1`) decode
- [x] 2.8 Size the `-2` mask channel by the mask rect and consume `-3` without modeling it
- [x] 2.9 Assemble the bottom-first layer tree from flat records in `build_tree`
- [x] 2.10 Skip global layer mask info by its declared length
- [x] 2.11 Return `Unsupported` for ZIP channel compression (`2`/`3`)

## 3. PSD layer-section write (M1-B)

- [x] 3.1 Flatten the layer tree depth-first bottom-first with divider/folder records
- [x] 3.2 Emit layer records with bounds, channel info, blend key, opacity, clipping, and flags
- [x] 3.3 Emit mask blocks and the `-2` mask channel
- [x] 3.4 Emit the Pascal name plus a `'luni'` Unicode name block
- [x] 3.5 Emit the `'lsct'` section marker with the group blend key
- [x] 3.6 Write opaque adjustment blocks back with their original key and bytes
- [x] 3.7 Pad records/tagged blocks to their required alignment and emit zero-length global mask info
- [x] 3.8 Wire the layer section into `write_psd` alongside the unchanged composite path

## 4. Oracle fixtures and differential tests (M1-C)

- [x] 4.1 Generate deterministic fixtures with `psd-tools` in `scripts/generate-fixtures.py`
- [x] 4.2 Add `two_layers.psd`, `group.psd`, `masked.psd`, `gray.psd` fixtures
- [x] 4.3 Author `adjustment.psd` by setting adjustment tagged blocks on stripped pixel layers
- [x] 4.4 Document the fixtures and regeneration in `tests/fixtures/README.md`
- [x] 4.5 Read every fixture and assert dimensions and color mode
- [x] 4.6 Assert the two-layer tree names and bounds, and the group tree and Pass Through blend
- [x] 4.7 Assert the masked fixture exposes an 8×8 mask with 64 bytes of data
- [x] 4.8 Assert adjustment keys and payload bytes survive read and round-trip
- [x] 4.9 Add `scripts/validate_output.py` to open codec output in `psd-tools`
- [x] 4.10 Add a test that `psd-tools` reads a codec-written extra channel as composite alpha

## 5. Integration and validation (M1-D)

- [x] 5.1 Round-trip a document with layers, a group, and a mask to exact equality
- [x] 5.2 Round-trip Pass Through groups and grayscale layers
- [x] 5.3 Assert malformed inputs (truncation, bogus count, bogus channel length, bad signatures) error without panicking
- [x] 5.4 Keep the existing composite/extra-channel and LCG property tests green
- [x] 5.5 Run `cargo test --workspace` and `cargo clippy --all-targets -- -D warnings`
- [x] 5.6 Validate this change with `openspec validate m1-document-model --strict`
