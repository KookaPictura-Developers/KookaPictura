# M0.5 GPU path — findings

Status: offscreen wgpu render works and is wired into the display path. The
zero-copy interop mechanism (shared Vulkan device + `QRhiTexture::createFrom`)
works in isolation; presenting that texture through a `QRhiWidget` is blocked by
the widget's API.

## Environment

- wgpu 30.0.1 (features: `std`, `vulkan`, `wgsl`), pollster 1.0.1, ash 0.38
- adapter: NVIDIA GeForce RTX 3090, Vulkan 1.4.341, driver 610.57.04
- Qt 6.11.1 (system), cxx-qt 0.10.0, Rust 1.98.1

## What works: offscreen render -> CPU -> QImage

`crates/pictura-app/src/gpu.rs` builds a wgpu Vulkan instance/device, renders a
fullscreen-triangle gradient into an offscreen `Rgba8Unorm` texture, copies it
to a host buffer, unpads the 256-byte row alignment, and returns packed RGBA.
`cxxqt_object::render_gpu` wraps that as a `QImage` so it flows through the
existing `PictureView::image` / `ImageView` display path.

`--self-test` now reports the GPU status and proves the displayed image is not
blank by sampling 64 points through `QImage`:

```
pictura gpu: adapter="NVIDIA GeForce RTX 3090" backend=Vulkan device_type=DiscreteGpu
pictura self-test: image=512x512 codec_loaded=0 gpu=1
pictura self-test: nonblank=1 distinct=64
pictura self-test: zoom=1,197 pan_ok=1
```

Status codes from `render_gpu`: 0 = no Vulkan adapter (CPU fallback kept),
1 = rendered non-blank, 2 = rendered blank (self-test fails, exit 4).

Fallback proof (broken ICD, no adapter):

```
$ xvfb-run -a env VK_ICD_FILENAMES=/nonexistent.json ./build/pictura --self-test
pictura self-test: image=512x512 codec_loaded=0 gpu=0
pictura self-test: nonblank=1 distinct=64
EXIT=0
```

## Zero-copy attempt

Direction tried: wgpu owns the Vulkan instance/device/texture; Qt imports them.

`--interop-probe` creates a wgpu Vulkan device and texture, exports the raw
handles through the wgpu-hal escape hatch (`Instance::as_hal` / `Device::as_hal`
/ `Texture::as_hal`, all `unsafe`), then a small hand-written C++ bridge
(`cpp/interop.cpp`) hands them to Qt:

- `QVulkanInstance::setVkInstance(wgpu VkInstance)` then `create()`
- `QRhi::create(QRhi::Vulkan, params, {}, &QRhiVulkanNativeHandles{physDev, dev, gfxQueueFamilyIdx, gfxQueueIdx})`
- `QRhiTexture::createFrom(QRhiTexture::NativeTexture{object = VkImage})`

Result (under Xvfb, XCB platform):

```
pictura interop: device extensions QRhi needs: VK_KHR_swapchain, VK_EXT_vertex_attribute_divisor, VK_KHR_create_renderpass2, VK_KHR_depth_stencil_resolve, VK_KHR_fragment_shading_rate
pictura interop: QRhi backend=Vulkan device=NVIDIA GeForce RTX 3090 adopted_wgpu_device=1 queue_family=0
pictura interop: QRhiTexture::createFrom(wgpu VkImage)=1 (512x512)
pictura interop-probe: qrhi_import=1
```

`adopted_wgpu_device=1` means QRhi's `nativeHandles()->dev` is the exact
`VkDevice` wgpu created, and `createFrom` wrapped the wgpu-owned `VkImage`. So
same-device, same-image sharing works on this driver. The remaining gap is
presentation, not transfer.

## Where it breaks

1. `QRhiWidget` owns its `QRhi` and offers no injection point. Its public API is
   `setApi(Api)`, `initialize()`, `render()`, `rhi()` (a getter). Internally it
   is driven by `QPlatformBackingStoreRhiConfig` and creates the RHI itself
   (`QRhiWidgetPrivate::rhi`). There is no `setRhi` / `adoptDevice`. An imported
   wgpu device therefore cannot back a `QRhiWidget`, which is the route the task
   suggested. Zero-copy present requires a manually managed `QRhi` + `QWindow`
   swapchain (Qt's `rhiwindow` pattern), replacing `QRhiWidget`, which is an
   architectural change larger than a spike.

2. The imported device lacks `VK_KHR_swapchain`. `create_device` never creates a
   surface, so wgpu enables no WSI extensions. QRhi still created offscreen, but
   an on-screen swapchain on this device would need the extension enabled at
   device creation (or a wgpu surface) and the queue-family/queue-index contract
   confirmed.

3. The platform plugin must support Vulkan. Under `QT_QPA_PLATFORM=offscreen`
   the probe fails before wgpu is even involved:

   ```
   qt[1]: This plugin does not support createPlatformVulkanInstance
   qt[1]: QVulkanInstance: Failed to initialize Vulkan
   pictura interop: QVulkanInstance::create() rejected the imported instance
   pictura interop-probe: qrhi_import=0
   ```

   `--self-test` is unaffected: the offscreen wgpu render does not need a
   platform Vulkan instance.

4. `QRhiTexture::createFrom` returns true but does not establish a Vulkan layout
   contract. The probe passes `VK_IMAGE_LAYOUT_GENERAL`; real use must call
   `setNativeLayout()` after wgpu renders and synchronize queue access
   (semaphores/fences) between the two engines, which is unverified here.

5. cxx-qt 0.10 has no `QRhi` / `QRhiWidget` bindings, so all interop code is
   hand-written C++ (`cpp/interop.cpp`) reached from Rust through numeric handle
   getters. Raw handles outlive the wgpu objects unless the wgpu device/texture
   are kept alive, which `InteropState` does.

## Recommendation

Keep the offscreen -> readback -> `QImage` path as the Stage 1 default for M1:
it is correct, deterministic, and GPU-optional. Revisit zero-copy present only
if profiling shows readback dominates, and do it with a manual QRhi + QWindow
swapchain plus the wgpu device-import recipe above; enable `VK_KHR_swapchain`
and define layout/synchronization before claiming parity.

Files: `crates/pictura-app/src/gpu.rs`,
`crates/pictura-app/src/cxxqt_object.rs`, `crates/pictura-app/src/lib.rs`,
`crates/pictura-app/cpp/main.cpp`, `crates/pictura-app/cpp/interop.{h,cpp}`,
`crates/pictura-app/Cargo.toml`, `CMakeLists.txt`.
