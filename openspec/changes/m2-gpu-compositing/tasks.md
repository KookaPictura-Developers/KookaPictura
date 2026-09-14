## 1. App wiring (composited view)

- [x] 1.1 Add `current_buffer(doc)`: use `composite_rgba` when `doc.layers` is
  non-empty, else the embedded `doc.composite`
- [x] 1.2 Route `document_to_image` through `current_buffer` and convert the
  planar buffer to interleaved RGBA8888 `QImage`
- [x] 1.3 Refresh the displayed image from the document on `set_layer_visible`,
  adjustment add/remove, and magic-wand selection so layer edits recomposite
- [x] 1.4 Cover the fallback path with unit tests (layered stack composites,
  uncovered pixels stay transparent, empty stack uses the embedded composite)

## 2. GPU compositor (separable modes)

- [x] 2.1 Add `pictura_render::composite_gpu(doc) -> Result<PixelBuffer, GpuError>`
  with a `GpuError` enum (`Unavailable`, `TooLarge`, `Readback`,
  `UnsupportedMode`, `UnsupportedAdjustment`)
- [x] 2.2 Author the WGSL compute shader: per-channel separable formulas plus
  the whole-RGB DarkenColor/LighterColor branches, and Porter-Duff source-over
  on premultiplied values matching the CPU oracle
- [x] 2.3 Build the wgpu storage buffers and pipeline; assemble source samples
  and mask coverage on the CPU so reads cannot drift from `composite_rgba`
- [x] 2.4 Create the wgpu device once behind `OnceLock<Option<Devices>>`, keeping
  the instance/adapter/device/queue alive for the process
- [x] 2.5 Handle Normal, masks, opacity, and pass-through/isolated group
  semantics identically to the CPU path
- [x] 2.6 Return `GpuError::UnsupportedMode` for Hue/Saturation/Color/Luminosity/
  Dissolve and `UnsupportedAdjustment` for adjustment layers before dispatch

## 3. Fallback and parity verification

- [x] 3.1 Add `composite_gpu_or_cpu(doc)` returning the CPU composite on any
  `GpuError`
- [x] 3.2 Add `tests/gpu_parity.rs`: assert ±1 LSB per channel across Normal and
  every separable mode, skipping with a printed note when no adapter is usable
- [x] 3.3 Test that CPU-only modes return `UnsupportedMode` without panicking
- [x] 3.4 Test that `composite_gpu_or_cpu` is byte-identical to `composite_rgba`
  for a CPU-only stack
- [x] 3.5 Verify `TooLarge` on documents exceeding device buffer/workgroup limits
  and `Readback` on a mapping failure instead of unwinding

## 4. App offscreen GPU path and headless self-test

- [x] 4.1 Offscreen wgpu render in `pictura-app/src/gpu.rs`: Vulkan instance,
  `Rgba8Unorm` target, readback with 256-byte row unpadding, packed RGBA
- [x] 4.2 Wrap the readback as a `QImage` in `PictureView::render_gpu` and feed
  the existing `PictureView::image` display path
- [x] 4.3 Report GPU status through `--self-test` (`gpu=0` fallback,
  `1` non-blank, `2` blank) and fail with a non-zero exit on a blank render
- [x] 4.4 Confirm the layered self-test: red/blue quadrants composited, uncovered
  quadrants transparent (proves the stack, not the embedded composite)
- [x] 4.5 Confirm `--self-test` exits 0 under Xvfb and under a broken ICD that
  forces the CPU fallback
- [x] 4.6 Record the zero-copy interop findings: device/image import works,
  `QRhiWidget` present is blocked by its API, and the wgpu device lacks
  `VK_KHR_swapchain` (GPU-INTEROP-NOTES.md)
