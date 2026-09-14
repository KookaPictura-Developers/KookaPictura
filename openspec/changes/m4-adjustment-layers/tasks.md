## 1. Model (pictura-core)

- [x] 1.1 Add `AdjustmentData { key: [u8; 4], data: Vec<u8> }` and `Layer.adjustment: Option<AdjustmentData>`.
- [x] 1.2 Default `adjustment` to `None` for pixel and group layers; update constructors and test fixtures.
- [x] 1.3 Confirm `pictura-core/Cargo.toml` gains no `pictura-adjust` dependency.

## 2. Codec (pictura-codec)

- [x] 2.1 Add the adjustment-key whitelist (`levl`, `curv`, `brit`, `expA`, `vibA`, `hue2`, `hue `, `blwh`, `phfl`, `mixr`, `gdrm`, `invr`, `nvrt`, `post`, `thrs`, `selc`, `clrL`) and `is_adjustment_key`.
- [x] 2.2 Capture the first recognised adjustment block's key and payload verbatim while reading a layer record.
- [x] 2.3 Write the stored key and payload bytes back verbatim in `write_layer_info`.
- [x] 2.4 Add a unit round-trip test that writes and reads several adjustment layers with byte equality.
- [x] 2.5 Add a psd-tools-authorised oracle test (`adjustment.psd`) asserting key and payload bytes survive a full write/read.

## 3. Decode subset (pictura-render)

- [x] 3.1 Add the `pictura-adjust` dependency to `pictura-render`.
- [x] 3.2 Implement `decode_adjustment` for `nvrt`/`invr`, `post`, `thrs`, `brit`, and `hue2`/`hue `, plus `levl`.
- [x] 3.3 Return `None` for unknown keys and out-of-range payloads; never error.
- [x] 3.4 Add encoders (`encode_invert`, `encode_posterize`, `encode_threshold`, `encode_brightness_contrast`, `encode_hue_saturation`) mirroring the psd-tools byte layouts.
- [x] 3.5 Test each supported key decodes and that unknown/invalid payloads decode to `None`.

## 4. Compositing (pictura-render)

- [x] 4.1 Implement `composite_adjustment` to apply the decoded adjustment to the running backdrop buffer.
- [x] 4.2 Gate the adjusted output through the layer's mask, opacity, and blend via the existing `blend_into` path, skipping transparent backdrop pixels.
- [x] 4.3 Test an Invert adjustment layer over a pixel layer against applying Invert to the flattened composite within ±1.
- [x] 4.4 Test that a zero mask value hides the effect and opacity 128 scales it.
- [x] 4.5 Test that an undecodable key composites identically to no adjustment layer.

## 5. Verification

- [x] 5.1 `cargo test --workspace` green.
- [x] 5.2 Adjustment layers round-trip key and bytes through `write_psd`/`read_psd`.
- [x] 5.3 `scripts/guard.sh` green.
