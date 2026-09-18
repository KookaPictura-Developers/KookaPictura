# STATE — project resume anchor

Snapshot for resuming after a context break. Update after each milestone.

## Where things are

- Repo: `github.com/Zawaro/kooka-pictura`, branch `main`. Docs-only corpus +
  a working Rust/Qt engine.
- Toolchain: Rust 1.98 (`rust-toolchain.toml`), system Qt **6.11.1**, cxx-qt
  **0.10.0**, wgpu **30.0.1**, lcms2 **6.2.0** (system Little CMS 2.19).
- Oracles installed for tests: `psd-tools` 1.19, ImageMagick 7.1.2, `magick`.
- Test suite: **589 tests, 0 failed, 7 ignored** (the M29 `move_profile_*` pair,
  the M31 `region_move_timing_4000`, the M33 `m33_composite_profile_*` pair, the
  M34 `m34_undo_profile_4000`, and the M35 `m35_region_refresh_profile_4000`;
  M44 added the `gpu_parity` fresh-white-document regression; counted from
  `cargo test --workspace`, excluding the pre-existing ignored `pictura-render`
  doctest, which makes the raw ignored count 8).
- OpenSpec **1.3.1** (`/usr/bin/openspec`). M0–M43 archived; canonical specs are
  in `openspec/specs/` (60 specs, `validate --all --strict` green), change
  history under `openspec/changes/archive/`. The headless/CI/build-speed
  infrastructure change `ci-headless-and-speedup` is **archived** (not part of
  the panels program). The M44 panel/theme polish change `m44-panel-theme-polish`
  is **implemented and verified** (archive/commit deferred); the M45 panel-fixes
  change `m45-panel-fixes` is **implemented and verified** (archive/commit
  deferred); **M46** layer filtering/search is next.
- The C++ app needs **Qt6::Svg** (`Qt6Svg` CMake package) alongside the other Qt
  modules; icons and cursors render through `QSvgRenderer`.

## Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace          # preferred; cargo test --workspace is the fallback
cargo test --workspace --doc           # doctests (nextest does not run them)
cmake -S . -B build -G Ninja -DCMAKE_EXE_LINKER_FLAGS=-fuse-ld=lld && cmake --build build --parallel
./build/pictura --headless --self-test
./build/pictura --headless --self-test crates/pictura-codec/tests/fixtures/two_layers.psd
bash scripts/guard.sh
openspec validate --all --strict
```

## Code health (LOC guardrail)

Every source file is under the **1000-LOC hard cap** (target <800; `AGENTS.md`
rule). `scripts/check-file-size.sh` is the guard, `scripts/file-size-allowlist.txt`
the exception list, and `scripts/verify-fast.sh` runs it. A completed
code-splitting pass (`docs/dev/refactor-code-splitting.md`) took the sixteen
over-cap files down to one by pure moves — build, byte-identical self-test
stderr, the Rust suite, and the specs green throughout:

- **C++ app TUs** — `panels/panel_group.cpp` → `panel_group{,_menu,_test}.cpp`;
  `panels/layers_panel.cpp` → `layers_panel{,_actions,_menu,_test}.cpp` +
  `layers_panel_internal.h`; `panels/panel_column.cpp` →
  `panel_column{,_drag,_iconic,_menu,_test}.cpp` + `panel_float.cpp` +
  `panel_column_internal.h`; `frame.cpp` →
  `frame{,_columns,_session,_menus,_build,_test}.cpp`. New TUs are registered in
  `CMakeLists.txt`.
- **Self-test** — the `--self-test` block moved out of `main.cpp` into
  `selftest.cpp` / `selftest.h` (`int runSelfTest(QApplication&, bool, const
  QString&, PicturaMainWindow&, PictureView*, const QImage&, bool, int)`).
  `main.cpp` is now the startup path only (7320 → 180 LOC) that calls
  `runSelfTest`; the self-test lines live in `selftest.cpp`.
- **cxx-qt bridge** — `cxxqt_object.rs` is a Rust-2018 root that keeps the
  cxx-qt bridge and shared private helpers, with concern submodules under
  `src/cxxqt_object/`: `impl_{core,layers,selection,transform,paint,history,filters}.rs`,
  `helpers.rs`, `helpers_composite.rs`, `tests.rs`, `tests_impl.rs`. The root +
  directory layout keeps the generated-header path and the 14 C++ includes
  untouched.
- **Engine crates** — `pictura-filters` (`artistic/filters.rs` →
  `artistic/filters/{effects,brush,common,tests}.rs`; `lib.rs` → `filter.rs`),
  `pictura-render` (`lib.rs` → `composite.rs` + `tests/`; `gpu.rs` →
  `gpu/{mod,backend,shader}.rs`; `gpu_filter.rs` →
  `gpu_filter/{mod,plan,resources}.rs`; `document_ops/layer_ops.rs` →
  `document_ops/layer_ops/{mod,create,paths,properties,tests}.rs`),
  `pictura-adjust` (`lib.rs` → `types/common/apply/tonal/color/auto/tests.rs`),
  `pictura-codec` (`lib.rs` → `error/common/read/write/tests.rs`), and the
  integration tests (`tests/oracle.rs` → `tests/oracle/`;
  `tests/gpu_parity.rs` → `tests/gpu_parity/`).

One file remains allowlisted: `crates/pictura-app/cpp/selftest.cpp` (7212 LOC).
It was extracted whole first to protect the verification oracle; subdividing it
by self-test section is a deliberate later step, out of this pass.

## Crates

| Crate | Responsibility |
|---|---|
| `pictura-core` | Document/Layer/Channel/Mask/BlendMode(27+pass)/AdjustmentData; no deps |
| `pictura-codec` | PSD/PSB read/write: composite, layers, masks, adjustment keys, document channels |
| `pictura-color` | ICC profiles (sRGB/AdobeRGB/ProPhoto), convert/assign, intents, BPC |
| `pictura-adjust` | 15 destructive adjustments (`apply`) |
| `pictura-filters` | blur/sharpen/noise + stylize/other + pixelate + distort + render filters (`Filter` + `apply`); seeded filters; `artistic` module (15 CS6 Artistic filters with shared `reduce`/`noise`/`texture` helpers) + the four remaining families — Brush Strokes, Sketch, Texture, Oil Paint (29 filters, same shared helpers) |
| `pictura-select` | selection coverage mask, boolean/modify ops, wand, color range; `Selection::{rect,ellipse,polygon}` rasterizers + `CombineMode`/`combine_with` |
| `pictura-ops` | image resize (Nearest/Bilinear/Bicubic), canvas size (9 anchors), rotate/flip + arbitrary rotation; ImageMagick oracle |
| `pictura-render` | CPU compositor (27 blend modes, groups, masks, adjustment layers) + GPU compositor (default backend: 26 GPU modes + 5 adjustment layers, `Backend`/`composite_active`/`composite_region_active` (dirty-rect composite, byte-identical to the sub-rect); GPU-native 8-bit data path; 2-D compute dispatch so large documents like 4000² composite on the GPU instead of falling back to the CPU above the old ~4.19 MP 1-D workgroup ceiling) + GPU filter path (`filter_gpu_available`/`apply_filter_active`, byte-exact CPU-parity kernels — the nine convolution kernels plus the M28 heavy window/effect kernels Surface Blur, Maximum, Minimum, Median, Custom 5×5, Oil Paint; same 2-D dispatch for >4.19 MP) + PSD adjustment encode/decode + `apply_filter` (layer filter gated by mask, GPU-accelerated when `gpu_enabled`) + `document_ops` (document resize/canvas/orientation/crop/layer-translate; `translate_layer_active` composites through the active backend, `translate_layer_rect` shifts a layer rect (and mask) without recompute, CPU `translate_layer`/`recompute` remain the oracle; re-exports `Anchor`/`Resample`) |
| `pictura-testkit` | golden compare/hash + `pictura-diff` CLI |
| `pictura-paint` | dab-splatting brush/pencil stroke engine — tip coverage, spacing, flow/opacity, paint modes; depends on `pictura-core` |
| `pictura-app` | cxx-qt `PictureView` QObject + Qt C++ shell: `commands` (command registry + full documented CS6 menu tree), `frame` (`PicturaMainWindow`: menu bar, tabbed document area with a `PictureView`+`ImageView` per document, file lifecycle New/Open/Save/Save As/Revert/Close/Close All/Exit, status bar, docks, screen modes), `theme` (Fusion dark palette, 4 brightness levels), `session` (XDG state store), real Layers/History/Navigator/Color/Swatches/Info/Histogram panel docks replacing the debug dock (backed by the document and history models, with a frame-owned `ColorState` fed by the Eyedropper), tool layer (`tools`/`toolbox`/`options_bar` — Move/Marquee/Lasso/Quick Selection/Crop/Eyedropper/Hand/Zoom/Brush/Pencil), paint bridge (`begin_paint`/`paint_dab`/`end_paint`/`cancel_paint`/`is_painting`) with a live paint options bar, zoom/pan, GPU demo |

## Milestones done

- **M0/M0.5** — cxx-qt↔Qt6 walking skeleton, PSD codec, harness, CI, GPU spike.
  GPU: offscreen wgpu→readback→QImage works; QRhi imports the wgpu VkDevice/image;
  on-screen present via `QRhiWidget` is blocked (`crates/pictura-app/GPU-INTEROP-NOTES.md`).
- **M1** — layer/channel/mask model + layered PSD round-trip (+psd-tools oracle).
- **M2/M2.5** — CPU compositor (27 modes) + ImageMagick oracle; GPU compositor
  (22 modes, Δ0); app renders the layer stack.
- **M3** — ICC color management (+ ImageMagick oracle).
- **M4 (A/B/C)** — 15 adjustments; adjustment layers (decoded: `nvrt`,`post`,
  `thrs`,`brit`,`levl`,`hue2`; others preserved-only); app adjustment UI.
- **M5 (A/B/C)** — selection math (+ IM morphology oracle); selection↔PSD
  channels; selection-masked adjustments in the app.
- **M6** — `pictura-filters`: blur (Gaussian/Box/Motion/Radial/Average/
  Blur(+More)/Surface), sharpen (Sharpen(+More)/Edges/Unsharp Mask), noise
  (Add Noise seeded/Median/Despeckle). ImageMagick oracle: Gaussian/Box/Median
  exact, USM ±6; Motion classified no-equivalent (IM kernel is one-sided). 47
  filter tests. OpenSpec change `m6-filters`, tasks checked.
- **M6-C** — filter integration: `pictura-render::apply_filter(layer, filter,
  mask)` (destructive, selection/mask-confined, alpha + layer meta preserved) and
  the app `apply_filter(kind)` command + dock control. Headless self-test proves
  confinement (`changed_inside=12 changed_outside=0`). OpenSpec change
  `m6-filter-integration`, tasks checked.
- **M7** — Stylize + Other filters: `Maximum`, `Minimum`, `Offset`, `High Pass`,
  `Custom` (Other); `Emboss`, `Find Edges`, `Solarize` (Stylize). ImageMagick
  oracle exact (Δ0) for Maximum/Minimum/Offset-wrap/Custom/Solarize; Emboss,
  Find Edges, High Pass, Offset-fill classified no-equivalent. App filter kinds
  added. OpenSpec change `m7-stylize-other`, tasks checked.
- **M8** — Pixelate filters: `Mosaic`, `Crystallize`, `Facet`, `Fragment`,
  `Mezzotint`, `Pointillize`, `Color Halftone`. Oracle: Mosaic exact (block
  average when the cell divides the dimensions); the other six no-equivalent
  with recorded deltas; seeded filters reproducible. App filter kinds added.
  OpenSpec change `m8-pixelate`, tasks checked.
- **M9** — Distort warps: `Twirl`, `Pinch`, `Spherize`, `Ripple`, `Wave`
  (single-image inverse-mapping with bilinear resampling; alpha untouched; Wave
  seeded, `repeat_edge` selectable). All five classified no-equivalent against
  the closest ImageMagick operator (Twirl closest at Δ124 vs `-swirl 45`);
  measured deltas recorded. App filter kinds added. OpenSpec change
  `m9-distort`, tasks checked.
- **M10** — `pictura-ops` image operations: `resize` (Nearest/Bilinear/Bicubic),
  `resize_canvas` (9 anchors, grow/shrink), orientation (`rotate90_cw/ccw`,
  `rotate180`, flips, `rotate_arbitrary`). ImageMagick oracle: Nearest and the
  exact right-angle rotations/flips and 3-channel canvas are exact (Δ0); Bicubic
  matches `-filter catrom` (tol 1; `cubic` is a B-spline); Bilinear is exact on
  upscale but no-equivalent downscale; `rotate_arbitrary` measured on the central
  region (max 8 at 30°, 45° no-equivalent). OpenSpec change `m10-image-ops`,
  tasks checked; app/document integration deferred.
- **M11** — Distort filters, part 2: `PolarCoordinates`, `Shear`, `ZigZag`,
  `OceanRipple` (inverse-mapping warps with bilinear resampling; alpha
  untouched; Shear `fill` selectable, Ocean Ripple seeded). All nine Distort
  filters are now implemented, and all four new ones are classified
  **no-equivalent** against their closest ImageMagick operators with measured
  deltas (Polar Δ189/58 and 194/59 vs `-distort Polar`/`DePolar`; Shear Δ255/18
  vs `-shear`; ZigZag Δ170/14 vs `-swirl`; Ocean Ripple Δ227/54 vs `-wave`).
  Shear's curve range check (`x`/`y` in `-1..=1`) added. App filter kinds added.
  OpenSpec change `m11-distort2` MODIFIED the canonical `distort-filters` spec
  (7 MODIFIED + 4 ADDED); tasks checked.
- **M12** — Document operations: document-level resize/canvas/rotate/flip in
  `pictura-render`; reuses `pictura-ops`; structural oracle via psd-tools +
  composite consistency + exactness identities. OpenSpec change
  `m12-document-ops`, archived.
- **M13** — Image ops app UI: `PictureView` commands `resize_image` /
  `resize_canvas` / `rotate_doc` / `flip_doc` (validate → doc op → clear
  selection → recomposite), dock "Image" section (spin boxes + resample and
  9-anchor combos + orientation buttons), and self-test doc-op checks
  (exact remap + rejection + selection-clear assertions, exit codes 19–21).
  `pictura-render` re-exports `Anchor`/`Resample`. OpenSpec change
  `m13-image-ops-ui` (capability `image-ops-app-ui`), archived.
- **M14** — Undo/Redo: snapshot history in `crates/pictura-app/src/history.rs`
  (two-stack, depth 20 = CS6 default, doc+selection clones); capture wired
  into all eleven mutating commands (pre-state clone on success only);
  `open()` resets. `PictureView::undo`/`redo`/`can_undo`/`can_redo`/
  `history_depth`; dock buttons + Ctrl+Z/Ctrl+Y; self-test proves bit-exact
  undo/redo, redo invalidation, open reset (exit codes 22/23).
  OpenSpec change `m14-undo-history` (capability `edit-history`), archived.
- **M15** — Render filters: `Clouds`, `DifferenceClouds`, `Fibers`,
  `LensFlare` in `pictura-filters/src/render.rs` (seeded lattice value noise,
  Difference blend, x-elongated fibers, additive lens flare with
  `LensType`); all four classified no-equivalent (closed Adobe models) and
  verified by 13 property tests. App kinds + combo + self-test confinement
  check (exit 24). Lighting Effects and Scripted Patterns remain future.
  OpenSpec change `m15-render-filters` (capability `render-filters`),
  archived.
- **M16** — App shell foundation: replaced the M0 debug window with a
  CS6-shaped frame. New `pictura-app` C++ units: `commands` (declarative
  registry: stable id, path, label, shortcut, enablement, dispatch),
  `command_tree` (full documented CS6 tree — 523 documented leaves plus 21
  implemented commands and 47 separators; unimplemented leaves disabled), `frame`
  (`PicturaMainWindow`: menu bar, central canvas, status bar with view-options
  popup, dock registration with duplicate `objectName` rejection, screen modes
  `F`/`Shift+F`, canvas colour `Space+F`, `Tab`/`Shift+Tab` hide-all),
  `theme` (Fusion + dark palette, four brightness levels, `Shift+F1`/`F2`),
  `session` (atomic `QSaveFile` XDG state, schema-versioned). Bridge gains
  `has_document()` for enablement. `main.cpp` shrinks to startup + the self-test
  call (`selftest.cpp`);
  new self-test checks (exit codes 25–32) cover menu order, dispatch/inertness,
  no-document enablement, brightness, screen-mode cycle, session round-trip,
  duplicate panel rejection, and hide-all. OpenSpec change `m16-app-shell`
  (capabilities `command-registry`, `workspace-persistence`; MODIFIED
  `application-shell`), archived.
- **M17** — Document lifecycle and multi-document tabs. Rust bridge
  `PictureView` gains `new_document(width,height,mode,depth,background)`
  (8-bit Grayscale/RGB, white/transparent only), `save(path)` (atomic
  temp+rename through `write_psd`), `is_dirty()`, `file_path()`, plus
  `path`/`dirty` state; dirty is set at all 11 mutating-command
  history-capture sites and cleared by open/save. C++ shell: `new_document_dialog`
  (New Document dialog), `dialogs` (`askUnsaved` Save/Discard/Cancel with a
  non-interactive test policy), `frame` rewritten around a `QTabWidget` document
  area (one `PictureView`+`ImageView` per document, active-document targeting
  for all menu handlers/dock/status, tab titles with a `*` modified marker,
  File New/Open/Save/Save As/Revert/Close/Close All/Exit handlers, recent-files
  list persisted in the session store), `session` gains a bounded `recent`
  list, and `commands.h`/`command_tree.cpp` file-lifecycle leaves become
  implemented. `main.cpp` starts empty, opens the CLI path into a document, or
  creates a scratch document for the GPU smoke test; self-test exit codes 33–37
  cover New + white fill, Save As/open pixel round-trip, dirty set/cleared, tab
  switching, and close prompt Cancel/Discard. Verified: `cargo` suite clean
  (401 tests, 0 failed, 1 ignored — unchanged), fixture and no-argument
  self-tests exit 0, `guard.sh` OK. OpenSpec change `m17-document-lifecycle`
  (capabilities `document-lifecycle`, `document-tabs`; MODIFIED
  `application-shell`), archived.
  **Bug fixed (document tab reorder):** the document `QTabWidget` was movable
  but nothing connected `QTabBar::tabMoved`, so dragging a tab left `docs_` in
  the old order while `tabs_->currentIndex()`, `viewAt` and `removeDocument`
  indexed the new one — the wrong document became active, closed, or returned.
  `frame.cpp` now connects `tabMoved(from,to)` to `docs_.move(from,to)` (guarded
  by valid indices); Qt keeps the dragged-to-current tab current, so the active
  document is unchanged. Regression: self-test exit **196**
  `doc_tab_reorder aligned=1` (`PicturaMainWindow::reorderDocumentsForTest`),
  which also exercises `viewAt`/`documentName`/`activeDocumentIndex` after the
  move.
- **M18** — Toolbox and core tools. Engine: `pictura-select` gains
  `Selection::{rect,ellipse,polygon}` coverage rasterizers and
  `CombineMode {New,Add,Subtract,Intersect}` + `combine_with`; `pictura-render`
  document ops gain `crop_document` (reuses the M12 canvas offset; clamps and
  shifts layers/masks/channels) and `translate_layer` (shifts the topmost pixel
  layer's rect). Rust bridge: `select_rect`/`select_ellipse`/`begin_lasso`/
  `lasso_add_point`/`end_lasso`/`quick_select`/`crop`/`translate_layer`/
  `sample_argb`/`selection_bounds`, with dirty/history capture on the new
  mutating ops. C++ shell: `image_view.{h,cpp}` gains pointer signals
  (`mousePressed/moved/released` in image coordinates), `setPanEnabled`, and an
  overlay polygon; new `tools.{h,cpp}` (`ToolId {Move,Marquee,Lasso,
  QuickSelection,Crop,Eyedropper,Hand,Zoom}`, `SelectionMode`, `ToolController`),
  `toolbox.{h,cpp}` (Tools dock, letter shortcuts), `options_bar.{h,cpp}`
  (context-sensitive options bar); `frame.{h,cpp}` hosts them, routes canvas
  events, exposes `activeTool/setActiveTool/foregroundColor/hasPendingCrop/
  commitCrop`. Commands `image.crop`, `view.options`, `window.panels.tools`.
  `selftest.cpp` self-test (exit codes 39–46): tool switching, marquee rect (16 px)
  + ellipse (12 px), combine modes (16/28/12/4), lasso (25 px) + short-lasso
  rejection, quick selection, crop remap, layer move, eyedropper sample
  (`ffff0000`/`00000000`). Verified: build green, fixture and no-arg self-tests
  exit 0, `cargo fmt/clippy` clean, 411 tests (0 failed, 1 ignored; +10 engine
  tests), `openspec validate --all --strict` 40/40 pre-archive (42 after),
  `guard.sh` OK. OpenSpec change `m18-toolbox-tools` (capabilities
  `tool-framework`, `shape-selection-tools`, `canvas-tools`), archived.
- **M19** — SVG icon set and cursors. New `assets/icons/` (40 original
  independent-creation SVGs: `app`, eight `tool.*`, and 31 implemented-command icons named
  by command id, e.g. `file.saveAs.svg`, `view.screenMode.full.svg`) and
  `assets/cursors/` (eight `tool.*` SVG cursors; eye-dropper hotspot (2,22),
  others (12,12)); `assets/pictura.qrc` bundles all 48. `icons.{h,cpp}` provides
  `QIcon pictura::icon(id)` and `QCursor pictura::cursor(id)` (QSvgRenderer,
  DPR-scaled render, existence-guarded so unknown ids are silent). Build gains
  `Qt6::Svg`, `CMAKE_AUTORCC`, the `.qrc`, and the `Qt6::Svg` link. Wiring:
  application/window icon, Tools-panel action icons, the 31 implemented
  menu-action icons, and the active tool's SVG cursor. `selftest.cpp` self-test exit
  codes 47–49: all 40 icons resolve + unknown is null, all 8 cursors resolve,
  window icon set. Verified: build green, fixture and no-arg self-tests exit 0
  with no Qt warnings, `cargo fmt/clippy` clean, 411 tests (0 failed, 1 ignored;
  unchanged), `openspec validate --all --strict` 43/43 pre-archive (44 after),
  `guard.sh` OK. OpenSpec change `m19-svg-icons` (capabilities `icon-assets`,
  `svg-cursors`), archived.
- **M20** — Panel parity. `history.rs` becomes a labeled linear model (one
  labeled state per undoable step, depth 20, plus up to 10 named snapshots);
  `edit-history` undo/redo semantics are unchanged. Bridge gains
  `layer_blend`/`set_layer_blend`, `layer_opacity`/`set_layer_opacity`,
  `set_layer_name`, `move_layer`, and `layer_thumbnail`. Seven `QDockWidget`
  panels in `crates/pictura-app/cpp/panels/` (`layers_panel`, `history_panel`,
  `navigator_panel`, `color_panel` + `ColorState`, `swatches_panel`,
  `info_panel`, `histogram_panel`; objectNames `layersPanel`, `historyPanel`,
  `navigatorPanel`, `colorPanel`, `swatchesPanel`, `infoPanel`,
  `histogramPanel`). `frame` registers the docks (replacing the debug dock),
  rebinds them on active-document change, owns a `ColorState` fed by the
  Eyedropper (`ToolController::foregroundSampled`), routes
  `ImageView::mouseMoved` into the Info panel, and implements checkable
  `Window > Panels` toggles for Navigator/History/Color/Swatches/Info/Histogram
  in addition to Layers/Tools. `CMakeLists.txt` gains the panel sources.
  `selftest.cpp` self-test exit codes 50–52 (`m20_layer count=2 name=1 blend=1
  badblend=1 opacity=128 dirty=1`; `m20_history states=5 open_label=1 grew=1
  jump=1 snapshot=1`; `m20_panels registered=7 toggled=7`), and the
  headless-shutdown hang is fixed by disabling the interactive
  unsaved-document prompt before the self-test quit timer. Deferred non-goals:
  group-tree expansion, drag-reorder, layer lock flags, clipping/link/color
  labels, filter/search row, swatch library file I/O, Info color samplers,
  Histogram source/cache states, and history branching beyond the bounded
  stack. Verified: build green, fixture and no-arg self-tests exit 0, `cargo
  fmt/clippy` clean, 414 tests (0 failed, 1 ignored), `openspec validate --all
  --strict` 45/45 pre-archive. OpenSpec change m20-panels (capabilities
  layers-panel, history-panel, navigator-panel, color-swatches-panel,
  info-histogram-panel), archived.
- **M21** — Paint engine. New `pictura-paint` crate (depends on `pictura-core`
  only): dab-splatting stroke engine with a procedural round/elliptical tip
  (size 1..5000, hardness, roundness, angle, flip), anti-aliased Brush vs
  aliased Pencil, fixed and velocity-driven spacing with residue carry-over,
  per-pixel coverage accumulation with a flow/opacity model
  (`acc = 1-(1-acc)(1-flow*tip)`, composited alpha `= opacity*acc`), and paint
  modes Normal/Dissolve/Behind/Clear; 25 in-crate tests. Bridge `PictureView`
  gains `begin_paint`/`paint_dab`/`end_paint`/`cancel_paint`/`is_painting`, a
  pre-stroke base plus working document, live image refresh per dab, and one
  labelled history state ("Brush"/"Pencil") per completed stroke. Qt:
  `ToolId::Brush`/`Pencil` (values 8/9), `B`/`Shift+B` cycling, size and hardness
  shortcuts (`[`/`]`, `Shift+[`/`Shift+]`), a paint options bar (size, hardness,
  opacity, flow, mode, Pencil Auto Erase), and `ToolController` stroke routing.
  Bug fixed: `PictureView::new_document` now creates one raster layer (opaque for
  "white", transparent for "transparent"), matching
  `docs/10-workflow-io/open-and-new.md`; previously a fresh document had no layer
  and could not be painted. `selftest.cpp` self-test exit codes 53–56
  (`m21_stroke ended=1 painted=196 dirty=1 hist=2`; `m21_opacity a1=84 a2=140`;
  `m21_aliased pencil_ok=1 brush_aa=1`; `m21_undo changed=1 restored=1`), with
  identical output on fixture and no-argument runs. Deferred non-goals:
  sampled/bristle/erodible/airbrush tips, Shape Dynamics, Scattering, Texture,
  Dual Brush, Color Dynamics, Transfer, Brush Pose, airbrush time build-up,
  tablet pressure mapping, `.abr` presets, Brush/Brush Presets panels, HUD, the
  remaining 23 paint modes, 16/32-bit and non-RGB painting, lock transparency,
  and sparse tile scratch storage. Verified: `cmake --build build` OK, both
  self-tests exit 0 with no FAILs, `cargo fmt/clippy` clean, 439 tests (0 failed,
  1 ignored; +25 `pictura-paint` tests), `openspec validate --all --strict`
  50/50 pre-archive. OpenSpec change m21-paint-engine (capabilities paint-engine,
  brush-tools; MODIFIED tool-framework), archived.
- **M22** — Artistic filters. New `pictura-filters::artistic` module implements
  the 15 CS6 Artistic filters — Colored Pencil, Cutout, Dry Brush, Film Grain,
  Fresco, Neon Glow, Paint Daubs, Palette Knife, Plastic Wrap, Poster Edges,
  Rough Pastels, Smudge Stick, Sponge, Underpainting, Watercolor — as
  behavioural-parity models (Adobe's kernels are closed), each carrying a
  `// ponytail:` ceiling note. Shared helpers: `artistic/reduce.rs`
  (`posterize`, `edge_magnitude`), `artistic/noise.rs` (seeded value noise via
  `ChaCha8Rng`), and `artistic/texture.rs` (`TextureSurface`
  Brick/Burlap/Canvas/Sandstone, `emboss`, `TextureOptions { surface, scaling,
  relief, light_direction, invert }`). All 15 are `Filter` variants with typed
  parameters, in-range validation, alpha preservation, and seeded determinism for
  the stochastic ones; new enums `BrushType`
  (Simple/LightRough/DarkRough/WideSharp/WideBlurry/Sparkle) and
  `TextureSurface`. App `filter_from_kind` maps the 15 kebab-case kinds with fixed
  in-range defaults and `seed: 1`. Self-test exit codes 57–58
  (`m22_applied applied=15/15`; `m22_deterministic=1`), measured on fixture and
  no-arg runs. Deferred non-goals: Filter Gallery dialog and cumulative stack,
  Smart Filter entries, `Edit > Fade`, 16/32-bit and CMYK/Lab gating, `Load
  Texture` file I/O, and the remaining families (Brush Strokes, Sketch, Texture,
  Oil Paint). Verified: `cmake --build build` OK; both self-tests exit 0 with no
  FAILs; `cargo fmt --all --check` OK; `cargo clippy --workspace --all-targets --
  -D warnings` OK; 465 tests (0 failed, 1 ignored; +26 filter tests); `openspec
  validate --all --strict` 52/52 pre-archive. OpenSpec change m22-artistic-filters
  (capability artistic-filters), archived.
- **M23** — CS6 UI chrome. `theme.{h,cpp}` gains `Theme::styleSheet(int level)`,
  a CS6-style QSS built from the four-level dark ramp (menu bar, options bar,
  dock tabs/title bars, tool buttons, status bar, scrollbars, menus, tooltips,
  panel content, push buttons); `Theme::apply` now sets the palette then the
  stylesheet. `toolbox.{h,cpp}` is rebuilt as a two-column icon-button grid
  (10 tools) replacing the single-column toolbar, plus a
  `ForegroundBackgroundWidget` (overlapping fg/bg swatches, active target,
  reset) and a screen-mode button emitting `screenModeRequested()`. `ColorState`
  gains an active target (`foregroundActive()`/`setForegroundActive()`/
  `activeChanged`) shared by the Color panel and the toolbox control.
  `frame.cpp` tabifies the default docks into CS6 groups — Color+Swatches,
  Layers+History, Navigator+Info+Histogram — and sets the canvas default colour
  to the CS6 dark grey `QColor(37,37,37)` (the four-entry cycle is unchanged).
  Reference screenshot: `docs/02-ui-ux/reference/cs6-workspace.png`. Self-test
  hygiene: an isolated `XDG_STATE_HOME` (`QTemporaryDir`) is set before the frame
  is constructed, so a user-saved layout cannot affect the checks. Self-test
  exit codes 59–61, measured identically on fixture and no-argument runs:
  `m23_stylesheet applied=1 levels=4 distinct=1`;
  `m23_toolbox dock=1 buttons=11 fgbg=1`; `m23_groups grouped=4/4`. Deferred
  non-goals: pixel-exact CS6 metrics and icon art, HUD/on-image displays, new
  panels (Gradients/Patterns/Properties/Adjustments/Libraries/Channels/Paths/
  Brush), workspace presets/switcher, icon-collapse docks, floating-panel drop
  zones, and deeper Layers-panel internals beyond M20. Verified:
  `cmake --build build` OK; both self-tests exit 0 with no FAILs; `cargo fmt
  --all --check` OK; `cargo clippy --workspace --all-targets -- -D warnings`
  OK; 465 tests (0 failed, 1 ignored; unchanged — no Rust changes); `openspec
  validate --all --strict` 53/53 pre-archive. OpenSpec change
  m23-cs6-ui-chrome (MODIFIED application-shell, tool-framework), archived.
- **M24** — Panel rail and right-side placeholders. `panels/placeholder_panel.{h,cpp}`
  adds one shared `pictura::PlaceholderPanel(title, message, parent)` dock with a
  centred CS6 empty-state label; `panels/panel_rail.{h,cpp}` adds
  `pictura::PanelRail`, a vertical icon-only `QToolBar` (objectName `panelRail`).
  `frame` creates the eight new placeholder docks — objectNames `gradientsPanel`,
  `patternsPanel`, `propertiesPanel`, `adjustmentsPanel`, `librariesPanel`,
  `channelsPanel`, `pathsPanel`, `actionsPanel` (Properties shows "No Properties"),
  structural only, no real functionality yet — and tabifies the right docks into
  the three CS6 groups Color+Swatches+Gradients+Patterns,
  Properties+Adjustments+Libraries, and Layers+Channels+Paths (tabify + raise the
  first). The rail carries five glyph buttons for History, Actions, Info,
  Navigator, Histogram; each button and its matching `Window > Panels` command
  share one toggle path, and the button's checked state tracks the dock's
  visibility. `commands.h`/`command_tree.cpp` gain ids and implemented, checkable
  `Window > Panels` entries for the eight new panels; `CMakeLists.txt` gains the
  two new sources. Self-test exit codes 61 (grouping, revised), 62 (panels), and
  63 (rail), measured identically on fixture and no-argument runs:
  `m24_groups grouped=8/8`; `m24_panels found=8/8 properties_empty=1`;
  `m24_rail actions=5 toggled=1 synced=1`. Deferred non-goals: real content for the
  placeholder panels (gradient/pattern presets, Properties binding, adjustment
  presets, libraries, channel/path lists, actions), icon-collapse auto-collapse,
  floating-panel drop zones, workspace presets/switcher, and panel-title-bar menus.
  Verified: `cmake --build build` OK; both self-tests exit 0 with no FAILs;
  `cargo fmt --all --check` OK; `cargo clippy --workspace --all-targets --
  -D warnings` OK; 465 tests (0 failed, 1 ignored; unchanged — no Rust changes);
  `openspec validate --all --strict` 53/53 pre-archive. OpenSpec change
  m24-panel-rail (capability panel-rail; MODIFIED application-shell), archived.
- **M25** — Remaining filter families. `pictura-filters` gains the four remaining
  CS6 families, 29 filters total: **Brush Strokes** (`brush_strokes.rs`: Accented
  Edges, Angled Strokes, Crosshatch, Dark Strokes, Ink Outlines, Spatter, Sprayed
  Strokes, Sumi-e), **Sketch** (`sketch/{mod,relief,paper}.rs`: Bas Relief, Chalk
  & Charcoal, Charcoal, Chrome, Conté Crayon, Graphic Pen, Halftone Pattern, Note
  Paper, Photocopy, Plaster, Reticulation, Stamp, Torn Edges, Water Paper),
  **Texture** (`texture.rs`: Craquelure, Grain, Mosaic Tiles, Patchwork, Stained
  Glass, Texturizer), and **Oil Paint** (`oil_paint.rs`). All are
  behavioural-parity models (Adobe kernels closed), reusing the M22 shared helpers
  `artistic::{reduce,noise,texture}` (posterize, edge_magnitude, clamp_u8,
  value_noise, surface_height, emboss, `TextureOptions`). New `Filter` enums
  `StrokeDirection`, `LightDirection`, `HalftoneType`, `GrainType` and 29 variants
  in `lib.rs`; `apply` dispatches to the family functions. Colour-dependent Sketch
  filters carry explicit `foreground`/`background` RGB (Bas Relief included —
  dark/recessed→foreground, light/raised→background); Conté Crayon and Texturizer
  take shared `TextureOptions`; seeded filters carry `seed: u64` and are
  bit-reproducible. **Oil Paint is a deliberate non-parity divergence:** CS6
  hard-requires a supported GPU (closed OpenCL kernel, no CPU fallback), so this
  is a CPU behavioural model (gradient-orientation directional edge-stopping
  smoothing + a luma/scale/bristle height field shaded Lambert/Blinn-Phong from
  `angular_direction`/`shine`); it is deterministic (no seed) and a GPU compute
  path is the deferred upgrade. **Accented Edges** is neutral (no-op) at Edge
  Brightness 25 by contract; the app default is set to 38 (CS6's dialog default)
  so the menu item visibly acts. App: `filter_from_kind` maps all 29 kebab-case
  kinds with fixed in-range defaults (`seed: 1` for seeded ones). Self-test exit
  codes **68/69** (`m25_applied applied=29/29`; `m25_deterministic=1`), identical
  on fixture and no-argument runs. Self-test label disambiguation: the earlier
  canvas-perf checks were renamed `m25_*` → `canvas_*` (`canvas_centre`,
  `canvas_middle_pan`, `canvas_move`, `canvas_preview_cache`) so the `m25_` prefix
  belongs to the actual M25 milestone; exit codes 64–67 unchanged. Verified:
  `cmake --build build` OK; both self-tests exit 0; `cargo fmt/clippy` clean;
  **508 tests (0 failed, 1 ignored)** — up from 465; `openspec validate --all
  --strict` 54/54 pre-archive. OpenSpec change `m25-filter-families` adds
  capabilities `brush-stroke-filters`, `sketch-filters`, `texture-filters`,
  `oil-paint-filter` (4 new → **57** capabilities after archive). Deferred
  non-goals: Filter Gallery dialog and cumulative/reorder stack, Smart Filters,
  `Edit > Fade`, 16/32-bit and CMYK/Lab gating, `Load Texture` file I/O, and an
  Oil Paint GPU compute pass.

- **M26** — GPU compute backend (GPU default). `pictura-render::gpu` gains
  `Backend { Gpu, Cpu }`, `gpu_available()` (cached probe), and
  `composite_active(doc, gpu_enabled) -> (PixelBuffer, Backend)`; re-exported from
  the crate root. The compute pipeline/bind-group layout/params are cached once per
  device (was rebuilt per composite). The params uniform was moved from
  process-global into the per-composite `Gpu` — a real race fix for parallel
  composites. Compositor completeness: the four **non-separable** modes
  Hue/Saturation/Color/Luminosity now run on the GPU (mode ids 23–26; separable
  ids 1–22 unchanged); the app's five **adjustment layers** (invert, posterize,
  threshold, brightness/contrast, hue/saturation) are applied on the GPU at the
  layer's stack position, matching the CPU oracle; **Dissolve** is the only
  remaining CPU-only blend mode (random, not bit-reproducible). Layer source/mask
  uploads are now rect-only. Parity: separable 26 modes max delta 0 LSB,
  non-separable 4 modes max delta 0 LSB, adjustments 0 LSB except hue/saturation
  1 LSB (all within ±1). App: an app-wide `gpuCompute` preference (default `true`)
  is persisted in the XDG session store, schema **v2** (a missing field or
  schema-1 store loads `true`). A checkable implemented command
  **`view.gpuCompute`** ("Use GPU Compute", View menu after View > Options)
  toggles it, enabled only when an adapter is present (greyed when none). The
  status bar shows `GPU` / `CPU` / `CPU (no GPU)`. Bridge `PictureView` gains
  `set_gpu_compute`, `gpu_compute`, `gpu_available`, `active_backend`;
  `current_buffer`/`document_to_image` route through `composite_active`, so the
  canvas, panels, painting refresh, and every recomposite use the GPU by default,
  falling back to CPU (per call) for `Dissolve`, unsupported adjustments, or any
  `GpuError`. Self-test exit codes **70/71**: `m26_gpu available=1 default_on=1
  off_cpu=1 on_back=1`; `m26_session gpu_off=1 gpu_on=1 default=1` — identical on
  fixture and no-argument runs. Timing evidence (1024×1024, 4 pixel layers, debug
  test build): `m26 timing 1024x1024x4layers: cpu 485 ms, gpu 744 ms` (the GPU
  path is readback-bound). Verified: `cmake --build build` OK; both self-tests
  exit 0; `cargo fmt/clippy` clean; **515 tests (0 failed, 1 ignored)** — up from
  508 (514 before the close-out timing test); `openspec validate --all --strict`
  58/58. OpenSpec change `m26-gpu-compute` adds capability `gpu-compute-backend`
  and MODIFIES `gpu-compositing` (→ **58** capabilities after archive). Deferred:
  on-screen zero-copy present (manual QRhi + QWindow swapchain; `QRhiWidget`
  blocks device adoption — see `crates/pictura-app/GPU-INTEROP-NOTES.md`),
  off-GUI-thread compositing, GPU filters and GPU painting (next change
  `m27-gpu-filter-acceleration`), and Dissolve on GPU.
- **M27** — GPU-native acceleration. The compositor data path is now 8-bit end
  to end: layer sources upload as raw planar 8-bit channel samples over the layer
  rect, the mask uploads as an 8-bit plane, the working canvas is packed 8-bit
  RGBA in a storage buffer, and output is read back as packed 8-bit — no
  host-side full-canvas `f32` buffer and no `f32` readback (was the M26
  bottleneck). Public API and ±1 LSB parity unchanged. Release timing,
  1024×1024×4 pixel layers: **CPU 72 ms → GPU 19 ms (~3.8×; pre-M27 was ~1.2×)**.
  Parity: separable 26 modes 0 LSB, non-separable 4 modes 0 LSB, five adjustment
  layers 0 LSB except hue/saturation 1 LSB, group/mask scenes 0 LSB. **GPU filter
  acceleration:** new `pictura-render` API `filter_gpu_available()` and
  `apply_filter_active(filter, buf, gpu_enabled) -> Result<Backend, FilterError>`
  (re-exported from the crate root); a wgpu compute shader (kernel / separable-H /
  separable-V / motion / combine modes, weights from the CPU kernel builders)
  accelerates nine filters **byte-exact (0 LSB vs the CPU oracle, alpha
  bit-identical)** — Gaussian Blur, Box Blur, Motion Blur, Blur, BlurMore,
  Sharpen, SharpenMore, Unsharp Mask, High Pass — while every other filter falls
  back to the CPU `pictura_filters::apply` byte-identically (including the M22
  Artistic and M25 Brush Strokes/Sketch/Texture families and Oil Paint).
  `pictura_render::apply_filter` gained a `gpu_enabled` parameter and routes
  through `apply_filter_active`; the app passes the M26 `gpuCompute` flag, so
  filters are GPU-accelerated by default and CPU when disabled or no adapter
  exists. `pictura_render::gpu::shared_device()` (crate-private) now serves both
  the compositor and the filter path; the filter path no longer opens a second
  Vulkan device. Self-test exit code **72**: `m27_filter byte_identical=1` — a
  Gaussian Blur through the GPU path and the CPU path produces byte-identical
  images, identical on fixture and no-argument runs. Verified: `cmake --build
  build` OK; both self-tests exit 0; `cargo fmt/clippy` clean; **519 tests (0
  failed, 1 ignored)** — up from 515; `openspec validate --all --strict` 59/59.
  OpenSpec change `m27-gpu-acceleration` adds capability
  `gpu-filter-acceleration` and MODIFIES `gpu-compositing` (→ **59** capabilities
  after archive). Deferred: GPU kernels for the painterly/stochastic filter
  families and Oil Paint; GPU painting/brush; on-screen zero-copy present;
  off-GUI-thread compute; Dissolve on GPU.
- **M28** — GPU heavy filters. `crates/pictura-filters/tests/profile.rs` profiles the
  CPU filters at 1024² (release) and ranks them by cost; the heavy **deterministic**
  kernels were then ported onto the M27 GPU filter path. Six new kernels —
  **Surface Blur, Maximum, Minimum, Median, Custom (5×5), Oil Paint** — match the
  CPU oracle within ±1 LSB (mostly 0) with alpha bit-identical. Surface Blur is a
  bilateral `SURFACE` window (spatial Gaussian × luma-range Gaussian, `f32`
  accumulation, `round_away` quantize); Median is an **exact order statistic**
  (binary search over the byte value), not a separable approximation;
  Maximum/Minimum use the morphology window; Custom adds the 5×5 `KERNEL` path.
  **Oil Paint now runs on the GPU** (four passes: luma → edge-stopping directional
  aggregation → height/bristle → Lambert/Blinn-Phong shading), resolving the M25
  CPU-only deferral. The GPU filter path therefore accelerates the previously-named
  nine kernels plus these six. Measured GPU speedups at 1024² (release, this box):
  Surface Blur **7161 → 83 ms (~86×)**, Median **1273 → 6.2 ms (~205×)**, Oil Paint
  **10063 → 15.3 ms (~658×; the brief's baseline was ~1446 ms)**, Custom
  byte-exact. **Selection is profile-driven:** a filter is GPU-accelerated only if
  it is high-cost (≥ ~150 ms at 1024²) *and* reaches ±1 LSB. The stochastic/seeded
  filters stay on the CPU with a byte-identical fallback — `Crystallize`
  (3114 ms), `Watercolor`, `Conté Crayon`, `Paint Daubs`, `Dry Brush`,
  `Ocean Ripple`, `Spatter`, `Sponge`, `Palette Knife`, `Add Noise`,
  `Colored Pencil` — because their RNG stream cannot be reproduced bit-exactly on
  the GPU; the warps/distort and render filters are likewise deferred. Self-test
  exit code **73**: `m28_heavy byte_identical=1` (Surface Blur and Median produce
  identical images through the GPU and CPU paths), identical on fixture and
  no-argument runs. Verified: `cmake --build build` OK; both self-tests exit 0;
  `cargo fmt/clippy` clean; **523 tests (0 failed, 1 ignored)** — up from 519;
  `openspec validate --all --strict` 60/60 pre-archive. OpenSpec change
  `m28-gpu-heavy-filters` MODIFIES `gpu-filter-acceleration` (no new capability →
  **59** capabilities after archive). Deferred: stochastic-family GPU kernels via
  CPU-pre-generated RNG fields, GPU painting/brush, on-screen zero-copy present,
  off-GUI-thread compute, Dissolve on GPU.
- **M29** — large-document performance. A 4000×4000 layer move was 3–5 s per
  update; two root causes were fixed. **GPU dispatch cliff:** the compositor and
  filter paths dispatched a 1-D grid (`ceil(n/64)` workgroups, limit 65535 ≈
  4.19 MP), so 4000² (16.7 MP, 250 000 workgroups) silently fell back to the CPU.
  Both now use a **2-D dispatch** (`x = min(ceil(n/64), 65535)`,
  `y = ceil(ceil(n/64)/x)`, shader index `gid.x + gid.y * (grid_x*64)`);
  rejection only when the 2-D workgroup product is exceeded (~2.8×10¹⁴ px).
  4000² now composites on the GPU (`Backend::Gpu`): **CPU 887 ms → GPU 332 ms**
  (release, 3 layers, 1 LSB). Accelerated filters (Surface Blur, Median) also run
  on the GPU at >4.19 MP. **App-side per-update costs:** `sample_argb`
  re-composited the whole document (~286 ms) — it now reads the cached `image`
  QImage (**0 ms**); `layer_thumbnail(24)` built a full-size RGBA image — it now
  gathers the planar channels straight into a ≤24 px buffer (**90 ms → 0.02 ms**);
  the histogram scans all pixels — it now downsamples to ≤512² first
  (~145 ms → <10 ms); `begin_move_preview` cloned the whole document — it now
  hides the layer in place (**344 → ~306 ms**); `commit_move` composites through
  the new GPU-aware `translate_layer_active` (**739 → ~403 ms**). New
  render-crate API `translate_layer_active(doc, dx, dy, gpu_enabled)` (the CPU
  `translate_layer`/`recompute` remain the oracle). Self-test unchanged codes; the
  temporary 4000² timing block was removed (self-test stays fast) and the
  `move_profile_4000`/`move_profile_1024` tests are `#[ignore]`d (run with
  `cargo test -p pictura_app --release -- --ignored --nocapture move_profile`).
  Verified: `cmake --build build` OK; both self-tests exit 0; `cargo fmt/clippy`
  clean; **530 tests (0 failed, 3 ignored)** — up from 523; `openspec validate
  --all --strict` 60/60. The M29 change MODIFIES `gpu-compositing`,
  `gpu-filter-acceleration`, `info-histogram-panel`, `layers-panel`,
  `document-canvas`; no new capability (→ **59** capabilities after archive).
  Remaining bottlenecks: the history capture still clones the whole document
  (~60 ms/state, and up to 20 states of memory) — **copy-on-write or tile diffs**
  is the deferred fix. (The initial "readback-bound" guess was measured wrong in
  M32: the 4000² composite is dominated by host-side CPU per-pixel assembly, not
  the 64 MB readback — see below.)
- **M30** — canvas transparency and clipping. `ImageView::paintEvent` now draws a
  **transparency checkerboard** behind the document so pixels with alpha < 255
  reveal it (a fully transparent document shows the checkerboard; opaque pixels
  cover it). The checkerboard is **screen-space** (constant 8 px cells,
  independent of zoom), **anchored to the document origin** (stable while
  panning), rendered with a cached 2×2-cell `QPixmap` tile via a brush origin and
  clipped to the document rect ∩ viewport (O(1), never allocated beyond the
  screen). Two light tones `#FFFFFF` / `#CCCCCC` (Photoshop "Light" grid). All
  canvas content — the composited image, the Move-tool live preview layer, and the
  selection overlay — is now **clipped to the document rect**, so a layer dragged
  outside the canvas is cropped instead of drawn over the surrounding area. Test
  hooks `ImageView::transparencyCellSize()/transparencyColorA()/transparencyColorB()`
  were added. Self-test exit code **74**: `m30_canvas checker=1 clipped=1` (a
  transparent document shows both checker tones inside the document rect and the
  canvas colour outside; a preview layer dragged past the canvas edge is clipped),
  identical on fixture and no-argument runs. Verified: `cmake --build build` OK;
  both self-tests exit 0; `cargo fmt/clippy` clean; **530 tests (0 failed, 3
  ignored)** (no Rust change; the check is in the C++ self-test); `openspec
  validate --all --strict` 60/60. The M30 change MODIFIES `application-shell`
  (no new capability → **59** capabilities after archive). Deferred: the
  `Transparency & Gamut` preferences pane (grid size None/Small/Medium/Large and
  colour sets Light/Medium/Dark/Red/Custom), the `View > Show > Transparency Grid`
  toggle, gamut warning, and the GPU/RHI-backed canvas.
- **M31** — region (dirty-rect) compositing. New render API
  `pictura_render::composite_region_active(doc, rect: PsdRect, gpu_enabled) ->
  (PixelBuffer, Backend)` composites only the (clamped) rect and returns a
  rect-sized buffer **byte-identical (0 LSB)** to the corresponding
  sub-rectangle of `composite_active`, verified across separable, non-separable
  (Hue), adjustment-layer, masked and isolated-group scenes. The GPU path threads
  a region origin/size/stride so the canvas, per-layer sources, mask and group
  inner canvases are region-sized and only the region is dispatched and read
  back; the CPU fallback is a full composite + slice (`ponytail:`).
  `composite_gpu`/`composite_active` are unchanged (the region kernel over the
  full rect). Measured: 4000² with a 512² region **52 ms vs 3082 ms full (59×)**.
  New `pictura_render::translate_layer_rect(doc, dx, dy)` shifts a layer's rect
  (and mask) **without** recomputing (the CPU `translate_layer`/`recompute`
  remain the oracle). App: `PictureView` caches the composited document `QImage`;
  `refresh_region(rect)` composites the rect through the active backend and
  patches the cached image (and `doc.composite`) so pixels outside the rect are
  untouched. `commit_move` now invalidates `old ∪ new` layer bounds and refreshes
  only that region; `paint_dab` invalidates the stroke's `dirty()` rect. All
  other mutations keep the full recomposite path (the incremental-extension
  point). Measured region-move: a 64² region **0.497 ms vs 585 ms full
  (~1170×)**. A guard (`REGION_REFRESH_BUDGET = 1_000_000` px) falls back to the
  full recomposite for large dirty unions, because the cached image is patched
  with per-pixel `QImage::set_pixel_color` (FFI per pixel) — the `ponytail:`
  upgrade is a C++ `ImageView::blitRegion` via
  `QPainter::CompositionMode_Source`, or M34's GPU-resident present. Self-test
  exit code **75**: `m31_region moved=1 outside_unchanged=1 undo=1` and
  `m31_region_large moved=1 vacated=1 undo=1` (the latter drives the
  full-recomposite fallback), identical on fixture and no-argument runs.
  Verified: `cmake --build build` OK; both self-tests exit 0; `cargo fmt/clippy`
  clean; **536 tests (0 failed, 3 ignored)** — up from 530; `openspec validate
  --all --strict` 60/60. The M31 change MODIFIES `gpu-compositing` and
  `document-canvas` (no new capability → **59** capabilities after archive).
- **M32** — interactive-path region completion and present caching.
  `begin_move_preview` no longer composites the whole document: it
  region-composites only the moved layer's clamped rectangle with that layer
  hidden (`composite_region_active`) and blits it into a clone of the cached
  canvas, **byte-identical** to a full composite with the layer hidden; it falls
  back to the full path when the layer has no image, the clamped rect is empty or
  over `REGION_REFRESH_BUDGET`, or the cache is null/stale. `set_layer_visible`
  refreshes only the toggled layer's influence rectangle — a raster layer's
  `rect`, or a bounded adjustment layer's mask rect (enabled + data + zero
  `default_color`) — and falls back to a full recomposite for groups, unbounded
  adjustments and masks with a non-zero default. `ImageView` now presents from a
  zoom cache keyed on the source `cacheKey()` and the zoom, built **through a
  `QPainter`** so it is pixel-identical to the previous transform draw (a
  `QImage::scaled` smooth build diverged from the painter's non-smooth filter and
  fractional-offset phase — that was found and fixed); the cache is invalidated on
  image or zoom change and falls back to the transform draw above a 64 MP bound.
  Test hooks: `setPresentCacheEnabledForTest`,
  `presentCacheRebuiltOnLastPaint/RebuildCount/ImageSize/ImageKey`; a
  `topmost_pixel_layer_index` bridge accessor was added for the self-test. The
  measurement that scoped M32: a full 4000² GPU composite costs ~254 ms, split
  `build_source` 146 ms (58%), `to_pixel_buffer` 35 ms, `mapped.to_vec` 22 ms,
  `zero_canvas` 18 ms, `build_mask` 16 ms, readback submit/poll/map 7 ms, GPU
  dispatch 0.2 ms — **host-side CPU assembly dominates, not the readback**, which
  is why M32 deferred the Qt Quick zero-copy present (M34) and why M33 targets the
  assembly. Measured move preview (4000², release, RTX 3090): the old body (hide
  layer → full composite) **276 ms**; the new region body with a canvas-sized layer
  is over the per-pixel blit budget and falls back (**280–290 ms**, unchanged),
  while a moved **512²** layer takes the region path at **~30 ms (~10×)**. Self-test
  exit codes **76/77/78**: `present_cache reused=1 zoom_rebuild=1 src=32x32 z0=1
  z1=2 size=64x64 stable=1 identical=1` and `m32_region preview_base=1
  visibility=1`, identical on fixture and no-argument runs. Verified: `cmake --build
  build` OK; both self-tests exit 0; `cargo fmt/clippy` clean; **539 tests (0
  failed, 3 ignored)** — up from 536; `openspec validate --all --strict` 60/60. The
  M32 change MODIFIES `document-canvas` (no new capability → **59** capabilities
  after archive).
- **M33** — full-composite throughput. `build_source` assembles layer sources with
  whole-row `copy_from_slice` per plane (row-wise fast path; the per-pixel scan
  remains the fallback when a channel does not cover the clamped row intersection,
  preserving grayscale 2-plane, missing-G/B-aliases-channel-0, and
  missing-alpha-fills-255 semantics); `build_mask` fills the influence rect
  row-wise when there is no enabled data-carrying mask (per-pixel `mask_alpha`
  retained otherwise); the readback de-interleaves directly from the mapped
  staging slice into the planar `PixelBuffer` with no host packed RGBA
  intermediate (`to_pixel_buffer` walks 4-byte chunks via `as_chunks::<4>`); the
  canvas is cleared with a GPU command (`clear_buffer`) instead of a host
  full-canvas zero write. Output stays byte-identical (GPU 0 LSB, ±1 LSB vs the
  CPU oracle, alpha unchanged) — `cargo test --workspace` includes the new
  equivalence tests `m33_rowwise_source_matches_per_pixel` and
  `m33_rowwise_mask_matches_per_pixel`, and `gpu_parity` (18 tests) passes.
  Measured 4000² (2 RGB layers, release, RTX 3090): total **~123 ms (from
  ~254 ms, ~2×)**; `zero_canvas` **0.05 ms (from 18)**, `build_source` **~69 ms
  (from 146)**, `build_mask` **~15 ms (from 16)**, readback (transfer +
  de-interleave) **~38 ms (from ~57 = 22 `to_vec` + 35 de-interleave)**,
  dispatch 0.2 ms. 1024² total **~4.3 ms**. The remainder is dominated by the
  per-composite source/mask **upload** (~128 MB + 16 MB per composite), which only
  resident per-layer GPU buffers would remove (deferred). M33 stopped here
  because the measured bottleneck after the change is the upload, and residency
  needs content versioning (deferred); `build_mask` gained least because its
  upload dominates the removed `mask_alpha` calls. Self-test: unchanged codes;
  **541 tests, 0 failed, 5 ignored** (was 539, 3 ignored); `openspec validate
  --all --strict` 60/60; the M33 change MODIFIES `gpu-compositing` only (no new
  capability → **59** after archive).
- **M36 — layer attributes end-to-end** (the first milestone of the Layers-panel
  program; see `docs/dev/layers-panel-program.md`). `pictura_core::Layer` gained
  `fill: u8` (default 255), `lock: LockFlags` (newtype, `TRANSPARENCY|PIXELS|
  POSITION`, `all()` = 0x07) and `color: ColorLabel` (None/Red/Orange/Yellow/
  Green/Blue/Violet/Gray); 37 explicit `Layer { .. }` literals across 14 files
  were updated with the defaults (plus the `..bare` update-syntax layer in
  `pictura-core`'s `masked` test). The compositor now uses effective layer alpha
  = `opacity/255 × fill/255` in both the CPU oracle and the GPU shader (a new
  `fill` uniform word; groups use 1.0), byte-identical to the previous composite
  at `fill == 255` and within ±1 LSB on the GPU. The codec reads/writes the
  `lspf` (lock), `lclr` (color) and `iOpa` (fill) additional-layer blocks,
  omitted at defaults — a default document's `write_psd` output is byte-identical
  (proven against `crates/pictura-codec/tests/fixtures/m36_default_before.psd`),
  and psd-tools reads the new attributes back. The bridge gained
  `layer_fill`/`set_layer_fill`, `layer_lock`/`set_layer_lock`,
  `layer_color`/`set_layer_color` with the frozen refusal rules (fill refused for
  group/Background/fully-locked; opacity refused for Background/fully-locked;
  lock/color refused for Background) and history labels `Fill Opacity`/`Lock`/
  `Layer Color`. The panel gained a Fill spinbox, a four-button lock strip and an
  eight-entry color-label context menu. Honest limits: CS6's `lspf` "Lock All"
  high-bit `0x80000000` encoding is not handled (writes `0x07`); `layer_kind`'s
  `"background"` is a name+index heuristic (M37 replaces it); the color context
  menu is not greyed on the Background row (the bridge refuses). The `lspf`/
  `iOpa`/`lclr` source disagreements are recorded in
  `docs/dev/layers-panel-program.md` §5. Self-test exit codes 86–89:
  `m36_attrs fill=1 lock=1 color=1 undo=1`, identical on the fixture and
  no-argument runs. Verified: `cmake --build build` OK; both self-tests exit 0;
  `cargo fmt --all --check`/`cargo clippy --workspace --all-targets --
  -D warnings` clean; **556 tests, 0 failed, 7 ignored** (up from 546/7; the
  ignored set is unchanged, so the raw `cargo test` ignored count is 8 with the
  `pictura-render` doctest); `openspec validate --all --strict` 60/60. The M36
  change MODIFIES `layers-panel` and `psd-layer-io` and ADDs to
  `layer-compositing` (no new capability → **59** capabilities after archive).
  Deferred: group Fill in compositing (CS6 has no group Fill; the value
  round-trips and is ignored), the forced type/shape locks, and a first-class
  Background flag (M37).

- **M37 — layer creation and grouping** (the second milestone of the Layers-panel
  program; see `docs/dev/layers-panel-program.md`). New
  `crates/pictura-render/src/document_ops/layer_ops.rs` (418 lines) adds five pure
  functions over `&mut Document` — `add_layer(doc, above, name) -> i32`,
  `add_group(doc, above, name) -> i32`, `duplicate_layer(doc, index) -> i32`,
  `group_layer(doc, index) -> i32`, `ungroup_layer(doc, index) -> bool` — plus
  `next_layer_name(doc, prefix)`, re-exported from `document_ops` and the crate
  root. Insertion is "directly above `above`" = `above + 1` in the bottom-first
  stack, clamped to the top; a negative sentinel (no selection) or an out-of-range
  value also lands on top. `duplicate_layer` deep-clones the node (children,
  channels, mask, adjustment and all attributes) directly above the source and
  names the copy `"<name> copy"`; `group_layer` wraps the target in place (the
  group takes the layer's slot, the layer becomes its only child) and names it
  `"Group N"` via `next_layer_name`; `ungroup_layer` splices the children back in
  order and refuses a non-group (state unchanged). A new layer is a
  document-sized transparent raster layer (channels `0/1/2/-1` of `w*h` zero
  bytes, `Normal`/opacity 255/fill 255/visible) and leaves `composite_rgba`
  unchanged; a new group is an empty `is_group` node with a `Normal` blend (not
  `PassThrough`) and an empty `{0,0,0,0}` rectangle. Ceiling: the empty layer is
  stored document-sized and costs `w*h*4` bytes before it is painted — Photoshop
  stores nothing until a dab — marked with a `// ponytail:` empty-rect upgrade in
  the module. The bridge `PictureView` gains `add_layer`/`add_group`/
  `duplicate_layer`/`group_layer`/`ungroup_layer` `#[qinvokable]`, each
  `recomposite()`-then-`record()` with labels `New Layer` / `New Group` /
  `Duplicate Layer` / `Group Layers` / `Ungroup Layers`; a failed op records
  nothing. The panel adds **New Group** then **New Layer** buttons before Delete
  (Add Adjustment / New Group / New Layer / Delete / Move Up / Move Down) and
  `LayersPanel::currentLayer()`/`selectLayer(int)`; the five `Layer` leaves are
  frozen to `command_ids` (`layer.new.layer`, `layer.new.group`,
  `layer.duplicate.layer`, `layer.group.layers`, `layer.ungroup.layers`) and
  handled in `frame.cpp` against the active view's current layer. Honest limit:
  `Group Layers`/`Ungroup Layers` (and `Duplicate Layer`) act on the **single**
  selected layer — CS6 groups a multi-selection; M39's selection work upgrades
  these to per-selection operations (the handler carries the `// ponytail:` note).
  Self-test exit codes 90–94: `m37_create new=1 group=1 duplicate=1 ungroup=1
  undo=1` (count growth, an inert transparent layer, a `" copy"` duplicate name,
  wrap/unwrap ordering, one history step per op, and five undos restoring the
  start), identical on the no-argument and `two_layers.psd` runs. Verified:
  `cmake --build build` OK; both self-tests exit 0 with no FAILs; `cargo fmt --all
  --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean;
  **568 tests, 0 failed, 7 ignored** (up from 556/7; the ignored set is unchanged,
  so the raw `cargo test --workspace` ignored count is 8 with the pre-existing
  `pictura-render` doctest); `openspec validate m37-layer-creation --strict` valid
  and `openspec validate --all --strict` 60/60. The M37 change MODIFIES
  `layers-panel` (no new capability → **59** capabilities after archive).
  Deferred: a New Layer / New Group **dialog** (neutral-color fill, blend/opacity,
  use-previous-as-clipping), multi-selection grouping, and empty-rect layer
  storage.

- **M38 — icon and cursor library** (a user-requested interruption to the
  Layers-panel program; see `docs/dev/layers-panel-program.md` and the contract
  `docs/dev/m38-icon-cursor-library.md`; OpenSpec change
  `m38-icon-cursor-library`). The full CS6 toolbox catalogue and icon/cursor set
  landed in the Qt shell only — no Rust, bridge, document, compositor, or PSD
  change. **Assets:** `assets/icons/` now holds 137 SVGs and `assets/cursors/`
  71; `assets/pictura.qrc` was regenerated to 208 entries with the on-disk set
  equal to the listed set (all parse under `xmllint`, none use `<text>`, and a
  48 px render check found no blank and no solid-fill assets). **Tool
  catalogue:** `ToolId`/`ToolInfo` in `tools.{h,cpp}` expanded from the M19/M23
  ten to all **71** CS6 tools in catalogue order, each carrying `name`, `label`,
  `shortcut`, `Qt::CursorShape`, `hint`, `group` 1..23, `implemented`, and
  `hotspotX/Y`; `allToolIds()` returns 71, `implementedToolIds()` 10,
  `toolImplemented()` answers membership, and `static_assert(kToolCount == 71)`
  guards the table (the separate `ToolCatalogueEntry` layer in the design was
  collapsed into the existing `ToolInfo` table). **Toolbox:** rebuilt as one
  button per group (23 slots, single column); a slot shows its current member
  (first implemented member, else first) and gets a flyout triangle plus a
  hold/right-click menu only when the group has more than one member; Alt-click
  cycles the implemented members; an all-unimplemented slot is disabled with the
  exact `<label> — not implemented yet` tooltip; `setActiveTool` refuses an
  unimplemented id. The fg/bg swatch widget and the Screen Mode button are
  unchanged below the slots; `slotButtons()` exposes the slots for the
  self-test. **Cursors:** `cursor(id, hotX, hotY)` was added (the 24×24 centre
  default is preserved); the hardcoded eyedropper hotspot is gone and per-tool
  hotspots come from the catalogue; a null or failed SVG render falls back to
  the tool's `Qt::CursorShape`, so a missing asset can never yield an invisible
  cursor. **Panels:** rail buttons carry their `window.panels.<panel>` icons
  (history/actions/info/navigator/histogram — none null); the Layers strip is
  the CS6-order icon set `link, fx, mask, fillAdjustment, group, newLayer,
  delete` (link/fx/mask disabled "not implemented yet";
  fillAdjustment/group/newLayer/delete wired to the existing behaviour); the
  History snapshot button uses `history.snapshot`. Existing objectNames and
  `currentLayer()`/`selectLayer()`/`setView` are unchanged. **Self-tests:**
  `m38_tools icons=1 cursors=1 slots=1 guard=1` (exit codes 95–98) and
  `m38_panels rail=1 strip=1 history=1` (codes 99–101), both identical on the
  no-argument and `two_layers.psd` runs; all earlier lines/codes are unchanged
  (`m23_toolbox dock=1 buttons=24 fgbg=1` still passes). Two fixes during the
  milestone: `assets/cursors/tool.move.svg` was clobbered by a parallel asset
  agent and restored from git before the required compound rewrite, and
  `tool.brush`/`tool.pencil` had no cursor assets at all (the original defect)
  and were added. **Honest limits:** dock-tab window icons were not wired (only
  the rail, Layers strip, and History snapshot get icons); the 3D-object,
  3D-camera, and Count slots are Extended-only and every all-unimplemented slot
  is disabled; the blur/sharpen/smudge slot has no default letter (flyout or
  Alt-click only); the toolbox 1-/2-column toggle is not present (single column
  only); and the **functionality** of the 61 new catalogue entries is not
  implemented — assets and disabled UI only. Verified: `cargo fmt --all
  --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean;
  **568 tests, 0 failed, 7 ignored** (unchanged — no Rust change; the raw
  `cargo test --workspace` ignored count is 8 with the pre-existing
  `pictura-render` doctest); `openspec validate m38-icon-cursor-library
  --strict` valid and `openspec validate --all --strict` 60/60; both self-tests
  exit 0. The M38 change ADDs to `application-shell` and `layers-panel` and
  MODIFIES `tool-framework`, `panel-rail`, `svg-cursors` (no new capability →
  **59** capabilities after archive). Because it claimed the M38 number, the
  Layers-panel program's stages shift by one: **M39** panel anatomy, **M40**
  filtering/search, **M41** remaining management ops, **M42** styles/effects,
  **M43** smart objects / vector masks / artboards / layer comps; the deferred
  canvas-performance tracks stay by name.

- **M39 — panel anatomy** (the Layers-panel program's panel-anatomy milestone
  after the M38 interruption; see
  `docs/dev/layers-panel-program.md`; OpenSpec change `m39-panel-anatomy`, brief
  `docs/dev/m39-panel-anatomy.md`). The Layers panel becomes a full expandable
  tree over the document's existing `Layer.children` hierarchy, and edits become
  path-addressed and selection-based. **Path/tree core**
  (`crates/pictura-render/src/document_ops/layer_ops.rs`): a frozen path grammar
  (`segment *("/" segment)`, digits with no leading zeros, bottom-first),
  `resolve_path`/`resolve_path_mut`/`parent_path`, `flatten_rows ->
  Vec<(String,u32)>` (depth-first, topmost-first `(path, depth)` pairs), and
  `is_background` (the single source of truth; the app's `is_background_layer`
  delegates). Batch ops `set_visible_paths`/`apply_visibility`/`set_blend_paths`/
  `set_opacity_paths`/`set_fill_paths`/`set_lock_paths`/`set_color_paths`/
  `delete_paths`/`duplicate_paths`/`group_paths`/`ungroup_paths`/`rename_path`/
  `move_path`/`add_layer_in`/`add_group_in` apply the frozen per-node skip vs
  whole-op refuse table (only `group_paths` refuses the whole op); structural ops
  resolve-then-apply deepest-first with ancestor-descendant dropping so indices
  never invalidate. `move_path` refuses the Background and fully-locked layers
  (the LAY-002 ruling). **Bridge** (`cxxqt_object/impl_layers.rs`): `layer_row_count` + 17
  `layer_row_*` getters over the projection (path, depth, name, kind, visible,
  blend, opacity, fill, lock, color, clipping, has-mask, has-adjustment,
  expandable, child-count, thumbnail, mask-thumbnail); `set_layer_name_path`/
  `move_layer_path`; `QStringList` batch mutators for visible/blend/opacity/fill/
  lock/color plus `apply_visibility(paths,label)`, `delete_layers`,
  `duplicate_layers`, `group_layers`, `ungroup_layers`; `add_layer_in`/
  `add_group_in`. Undo contract: `recomposite()` then `record(label)` only when
  the changed count is non-zero; a zero-change call records nothing and emits no
  `changed`. Labels per the design (`Set Visibility`, `Solo Visibility`,
  `Restore Visibility`, `Group Layers`, …). Every legacy top-level
  `layer_*(i)`/`set_layer_*(i)`/M37 op is left **byte-identical** — the new path
  surface is the tree implementation and nothing calls legacy after the panel
  migration — with the deliberate asymmetry that legacy `move_layer` still swaps
  unconditionally while `move_layer_path` refuses. Thumbnails:
  `layer_row_thumbnail(i,size,entire_document)` (layer bounds vs
  document-positioned) and `layer_row_mask_thumbnail`; groups and adjustments
  return a null image (the delegate draws a folder glyph). **Panel**:
  `LayersModel` becomes a `QAbstractItemModel` over the flat projection (roles,
  `internalPointer` path, `CheckStateRole`→visibility, `EditRole`→rename);
  `LayerRowDelegate` paints the eye (with a press hit-test), thumbnail/folder
  glyph, name, color swatch, clip indent + base underline, mask thumbnail, and
  `fx` badge (null-asset safe); a `QTreeView` with `ExtendedSelection`,
  `SelectRows`, `uniformRowHeights`, `expandsOnDoubleClick(false)`,
  `dragEnabled(false)`; panel-side expansion (`QSet<QString>`, default collapsed,
  new groups expanded) and path-based re-selection; multi-selection routing for
  the header controls and Delete/Duplicate/Group/Ungroup (one undo step each);
  solo (`Alt`-click) snapshots visibility and uses `apply_visibility` for a
  one-step exact restore; rename `Tab`/`Shift+Tab` moves to the next/previous
  visible row with no wrap; tooltips `"<name> (<kind>)"`; Panel Options (Medium /
  Entire Document / Expand New Effects on) persisted in session **schema v3**
  (`layersThumbSize`/`layersThumbContents`/`layersExpandNewEffects`); the panel
  menu (`layersPanelMenu`) + row context menu with the wired commands only and a
  `Color Label` submenu; eye right-click show-only/show-all. The seven-button
  strip is unchanged and reordering is menu-only. **Fixes/decisions:**
  `PicturaMainWindow::saveSession()` now loads before writing so the v3 fields
  are not clobbered; `layerTooltip` now always returns `"<name> (<kind>)"` (the
  old helper predated M39 and would have failed the new tooltip check).
  **Self-tests:** `m39_tree` (102/103), `m39_multi` (104/105), `m39_solo` (106),
  `m39_rename` (107), `m39_options` (108), `m39_badges` (109), `m39_menus` (110),
  `m39_tooltip` (111), `m39_strip` (112); all earlier lines/codes unchanged.
  **Honest limits:** the clip-indent rendering is not positively proven (no
  bridge operation can set `Layer.clipping` and no fixture has a clipped layer —
  `m39_badges clip=1` only proves the role is plumbed and `clipBase` is not
  spuriously set); group thumbnails are folder glyphs (no group composite);
  drag-reorder is explicitly deferred to M41; multi-row move is deferred;
  expansion is session-only (not persisted); solo is one undo step per direction.
  Verified: `cargo fmt --all --check` and `cargo clippy --workspace --all-targets
  -- -D warnings` clean; **588 tests, 0 failed, 7 ignored** (up from 568/7; the
  ignored set is unchanged, so the raw `cargo test --workspace` ignored count is
  8 with the pre-existing `pictura-render` doctest); `openspec validate
  m39-panel-anatomy --strict` valid and `openspec validate --all --strict`
  60/60; both self-tests exit 0. The M39 change MODIFIES `layers-panel` (no new
  capability → **59** capabilities after archive). Deferred within the program:
  filtering/search (M40), the remaining management ops and drag-reorder (M41),
  styles/effects (M42), smart objects / vector masks / artboards / layer comps
  (M43).

- **M40 — CS6 Tools panel** (a second user-requested interruption to the
  Layers-panel program; OpenSpec change `m40-tools-panel`, brief
  `docs/dev/m40-tools-panel.md`). The Tools panel becomes CS6-shaped. **Flyout:**
  `ToolSlotButton` paints a 5 px filled triangle at the **bottom-right** when the
  slot's group has ≥2 members (counting unimplemented members); `MenuButtonPopup`
  and `setMenu` are gone so the icon is centred with no stock arrow. A **300 ms**
  hold timer opens the group `QMenu` below the button (screen-clamped);
  **right-click opens immediately**; a release before the timeout selects
  normally; `Alt`+click still cycles. Menu items carry the group's key via
  `QAction::setShortcut` + `setShortcutVisibleInContextMenu(true)` +
  `setShortcutContext(Qt::WidgetWithChildrenShortcut)` (no window-global shortcut,
  so a disabled item cannot steal the key); unimplemented members stay disabled
  with the "not implemented yet" tooltip. `refreshSlot()` no longer calls
  `setShortcut`. **Columns:** a custom `QDockWidget` title bar (`Tools` label + a
  flat double-arrow button, `objectName` `toolsColumnToggle`) toggles one/two
  columns; the icon shows the **target** layout
  (`assets/icons/panel.columnsTwo.svg` in one column, `panel.columnsOne.svg` in
  two — both new, 24×24 stroke-`#c8c8c8`, added to the regenerated qrc); reflow is
  row-major `(i/2, i%2)` with `minimumWidth` 66 → 104; the fg/bg widget and Screen
  Mode button stay pinned below the slots. **Standalone dock:**
  `setAllowedAreas(Left|Right)` and `setFeatures(Movable|Floatable|Closable)`;
  tabification refused by an event filter plus a reactive
  `PicturaMainWindow::ensureToolsNotTabified()` (float → re-add → show) driven from
  `dockLocationChanged`/`topLevelChanged`. **Shift cycling:**
  `toolShortcutKeys()`/`toolGroupForKey()` in `tools.{h,cpp}` and
  `Toolbox::handleToolKey(key, shift)`; `frame.cpp` registers one plain and one
  `Shift`+letter `QShortcut` per distinct key routed to it, replacing the
  hard-coded B / Shift+B `cyclePaintTool`. A plain letter activates the slot's
  current member; `Shift` cycles to the next **implemented** member (skipping
  unimplemented, wrapping); an all-unimplemented group is a no-op;
  `setShiftKeyForToolSwitch(false)` makes the plain letter cycle. Gated on a new
  session preference `useShiftKeyForToolSwitch` (default **true**) — **no UI
  yet**; M41 adds the Preferences dialog (General + Interface) and gives it a home
  alongside `Auto-Collapse Iconic Panels`. **Session v4:** `toolsColumns` (1|2)
  and `useShiftKeyForToolSwitch` with per-key defaults;
  `PicturaMainWindow::saveSession()` still loads-then-writes so unknown keys
  survive; `columnsChanged` persists. **Self-tests:** `m40_columns` (113/114),
  `m40_flyout` (115), `m40_keys` (116), `m40_shift` (117, driven through the real
  `QShortcut`/`QKeyEvent` path), `m40_dock` (118), `m40_session` (119);
  `m23_toolbox` tightened from a loose `buttons < 10` to `buttons != 25` plus
  `toggle=1` (the new title-bar toggle is found by `objectName` and is not one of
  the 23 slots). All earlier lines/codes unchanged. **Honest limits:** Qt has no
  clean per-dock tabify veto — the event filter only covers the toolbox body and a
  drop can briefly tabify before the reactive re-dock; the
  **right-click-immediate and quick-release-select timing are not self-tested**
  (the paths exist but only triangle/member-split/popup-placement are asserted);
  the preference has no UI until M41; the two-column width is 66/104 rather than
  the sketched 34/64 (the swatch + dock chrome need the width); `m40_columns`
  asserts widening via `minimumWidth` (the actual mechanism) with actual width
  only non-decreasing. No Rust changes: **588 tests, 0 failed, 7 ignored**
  (unchanged; the raw ignored count is 8 with the pre-existing
  `pictura-render` doctest). Verified: `cargo fmt --all --check` and
  `cargo clippy --workspace --all-targets -- -D warnings` clean;
  `TASK_ALLOWS_DOCS=1 bash scripts/verify-fast.sh` → `verify-fast: OK`
  (588 tests, 0 failed); both self-tests exit 0 with all m20–m40 lines `=1`;
  `openspec validate m40-tools-panel --strict` valid and
  `openspec validate --all --strict` 60/60. The M40 change MODIFIES/ADDs to
  `tool-framework` and ADDs to `application-shell` (no new capability → **59**
  capabilities after archive).

- **M41 — CS6 panel column** (a third user-requested interruption to the
  Layers-panel program; OpenSpec change `m41-panel-column`, brief
  `docs/dev/m41-panel-column.md`). The fifteen right-hand `QDockWidget`s are
  replaced by a custom column of plain content widgets. **Hosts:** `PanelGroup`
  is a `QTabWidget` with `setTabPosition(QTabWidget::North)` forced; the tab
  text is the panel title and there is **no separate group label** (a
  single-panel group still shows its tab). `PanelColumn` is a `QScrollArea` over
  a vertical `QSplitter` of groups with no hard panel minimums, so the window
  shrinks freely. **Groups (CS6 Essentials, fixed):** `Color | Swatches |
  Styles`; `Adjustments` (plus a hidden `Properties` tab); `Layers | Channels |
  Paths`; `Navigator | Histogram | Info`; iconic `History`, `Actions`; and a
  hidden overflow group `Gradients | Patterns | Libraries` (kept reachable from
  `Window → Panels`). `Styles` is a new placeholder; Properties is folded into
  Adjustments. **Width toggle:** a `panelColumnToggle` double-arrow in the column
  header switches **normal ⇄ iconic**. **Iconic mode** is a vertical icon strip
  with group dividers, labels-on-widen (threshold 120 px), and a `Qt::Popup`
  flyout per panel that reparents the panel in and restores it on close.
  Per-group **Collapse to Icons** reuses the same component; **Minimize** rolls a
  group up to its tab bar (distinct from iconic). **Tab context menu** (right-click
  a group's tab bar), exactly: `Close`, `Close Panel Group`, `Minimize`,
  `Collapse to Icons`, sep, `Auto-Collapse Iconic Panels` (checkable),
  `Auto-Show Hidden Panels` (checkable), sep, `Interface Options…`. **Drag &
  drop:** in-group reorder, cross-group regroup, between-groups insert (new
  group), with a **3 px `#2a7fff` drop indicator** (`panelDropIndicator`) — a
  vertical marker at the tab index or a full-width horizontal bar at the group
  boundary; hidden on commit/cancel/out. **Tear-off:** leaving the column floats
  the source group in a `Qt::Tool` `panelFloat`; dragging it back re-docks at the
  index; the float is hidden/`deleteLater`-ed when emptied. Drops are
  remove-then-insert, so a panel is never double-parented. **M24 `PanelRail`
  deleted** (`panel_rail.{h,cpp}` removed from disk and CMake, creation + five
  `setPanelChecked` connections gone). `Window → Panels` is the single visibility
  path; the `m24_rail` self-test (code 63) was repointed to it (`actions=5
  toggled=1 norail=1`). **Tools panel fixes:** the `"Tools"` title label is gone
  (the `toolsColumnToggle` stays); the hard-coded 66/104 widths became content-fit
  **34 / 65** px; `ForegroundBackgroundWidget` scales to the column (30/40) and
  no longer widens the dock. M40 flyout/hold/right-click/shortcuts/Shift-cycling/
  left-right dock all intact. **Session v5:** `panelRailMode` (`normal`|`iconic`,
  default normal), `railWidth` (0 = derive), `autoCollapseIconic` (default
  **false**), `autoShowHidden` (default **false**), `panelGroups` (per-group
  `order`/`visible`/`minimized`/`collapsed`), `schemaVersion` 5.
  `saveSession()` starts from the parsed on-disk object so unknown keys survive;
  a v4 store loads with v5 defaults. **Preferences dialog** (new
  `preferences_dialog.{h,cpp}`, `objectName` `preferencesDialog`, modeless, page
  list + `QStackedWidget`) with exactly two real pages, **General** (the real
  brightness setting) and **Interface** (`Use Shift Key For Tool Switch`,
  `Auto-Collapse Iconic Panels`, `Auto-Show Hidden Panels`). Wired to
  `Edit → Preferences → General` (`edit.preferences.general`, implemented) and
  `→ Interface` (`edit.preferences.interface`, implemented); the other nine
  Preferences leaves stay disabled no-ops (enablement unchanged).
  `PanelColumn::interfaceOptionsRequested()` opens Interface. **M40's UI-less
  preference now has a home:** `Use Shift Key For Tool Switch` →
  `Toolbox::setShiftKeyForToolSwitch` → `handleToolKey`, persisted.
  **`Auto-Collapse Iconic Panels`** (default off) — when a flyout closes and the
  column is in normal mode it returns to iconic; inert when already iconic; no
  per-panel expand-in-place state machine. **`Auto-Show Hidden Panels`** (default
  off) — the iconic strip includes hidden panels' icons and opening one reveals
  it; inert: no hover-at-edge gesture. **Self-tests**, new codes **120–130**:
  `m41_tabs`(120), `m41_width`(121), `m41_iconic`(122), `m41_menu`(123),
  `m41_minimize`(124), `m41_prefs`(125), `m41_drag`(126), `m41_tearoff`(127),
  `m41_session`(128), `m41_tools`(130). Code **129** (`m41_rail`) was folded into
  the repurposed `m24_rail` rather than allocated. Earlier codes all still pass
  with adapted internals (`m24_groups`/`m24_panels` now assert `PanelColumn`
  membership via `groupOfForTest`, `m38`/`m39`/`m40_dock` use
  `QWidget`/`PanelColumn`). **Harness fix (not product):** `main.cpp` now forces
  `QT_QPA_PLATFORM=xcb` for `--self-test` when both `DISPLAY` and
  `WAYLAND_DISPLAY` are set and no platform is pinned. Without it, a live Wayland
  session leaked through `xvfb-run`, Qt picked Wayland, and a
  programmatically-opened `QMenu` popup could not grab and was dismissed — making
  `m40_flyout` (code 115) flaky (**3/8** pass under Phase A; clean HEAD measured
  **8/8** only by luck, **3/12** in the agent's sample). Deterministic **5/5**
  no-arg after the guard. **Honest limits:** multi-monitor tear-off untested and
  the float is not screen-clamped; no cross-process drag; no translucent drag
  ghost (the float is the feedback); float chrome is a plain WM-framed
  `Qt::Tool`; Escape-cancel is not key-bound (`cancelDrag()` is programmatic
  only); drop-on-gap needs the ~4 px splitter or a group's top/bottom half;
  torn-off floats are not serialized; `panelGroups` keys groups by first-ever
  panel objectName and skips a stale name; iconic label threshold (120 px) and
  popup size are unsourced constants; the frame's own minimum width is ~776 px
  from the M40 options bar (column min is 64 px) so shrink-to-nothing is
  untestable; `Styles` has no dedicated SVG asset; the group collapse-icon row is
  horizontal while the column iconic strip is vertical (shared flyout/button
  code, not the identical widget). No Rust/bridge/codec/compositor/PSD change.
  **Capability:** ADD `panel-column`; the M24 `panel-rail` requirement is
  REMOVED (its spec persists without the rail requirement); MODIFIED
  `application-shell`, `tool-framework`, `workspace-persistence`. 59 canonical
  specs → **60 after archive**. Verified: `cargo fmt --all --check` and
  `cargo clippy --workspace --all-targets -- -D warnings` clean;
  `TASK_ALLOWS_DOCS=1 bash scripts/verify-fast.sh` → `verify-fast: OK`;
  **588 tests, 0 failed, 7 ignored** (unchanged — no Rust change; the raw
  ignored count is 8 with the pre-existing `pictura-render` doctest); both
  self-tests exit 0 with the new `m41_*` lines and every earlier m20–m40 line
  unchanged; `openspec validate m41-panel-column --strict` valid and
  `openspec validate --all --strict` 60/60 pre-archive. The M41 change MODIFIES
  `application-shell`, `tool-framework`, `workspace-persistence` and REMOVES the
  M24 `panel-rail` requirement, ADDing the `panel-column` capability (→ **60**
  capabilities after archive).

- **M42 — panel refinements** (a fourth user-requested interruption to the
  Layers-panel program; OpenSpec change `m42-panel-refinements`, brief
  `docs/dev/m42-panel-refinements.md`, research `docs/dev/m42-panel-menus.md`).
  **Phase A — chrome fixes.** The normal-mode `PanelColumn` minimum width is the
  widest visible group's `sizeHint` clamped to `[180,320]` (iconic-strip floor
  40); entering iconic now starts at the smallest possible width instead of
  keeping the prior splitter width. Iconic-strip buttons grew `24→30` (pixmap
  `16→20`) and Tools slots `30→32` (icons `20→22`). The floated Tools dock hugs
  its content height (the trailing stretch is zeroed on `topLevelChanged(true)`
  and the body layout invalidated so the two-column floor is not cached) and
  stays width-tight (1-col min = content = 36, 2-col 69).
  `ForegroundBackgroundWidget` gains the CS6 double-arrow swap control top-right
  (the default-colors X is kept) wired to the **`X`** key (was free; not in the
  tool catalogue or the frame's shortcut list). **Menu-bar overlay root cause
  (item 8):** `frame.cpp` constructed `actionsPanel_` **twice**; the first
  `PlaceholderPanel("Actions")` was never added to a group, so it remained a
  direct child of the main window and painted its dim "Actions" label over
  `File`/`Edit` — that was the "File Act…" artifact (confirmed against
  `/tmp/opencode/clean_wide.png`; a fresh `XDG_STATE_HOME` reproduced it, so it
  was not the persisted layout). Fixed by deleting the duplicate.
  Defence-in-depth: `saveState()` records `layoutRevision` and
  `restoreStoredLayout` discards a stored layout whose revision mismatches
  (`kLayoutRevision=2`), so the pre-M41 dock blob is no longer restored (**the
  session gains `layoutRevision`; no schema bump**). **Phase B — compact-strip
  drag** reuses the M41 `beginPanelDrag`/`updateDrag`/`commitDrop`/`resolveDrop`
  path (no second drag system): `resolveDrop` gained an `onStrip` branch with
  `stripInsertionIndexAt`, `applyStripDrop` rewrites order via
  `PanelGroup::setPanelOrder` or reuses `takePanel`/`insertPanel`/
  `cleanupEmptyGroup` across groups, and a dedicated `stripIndicator_` draws the
  blue line (the normal indicator lives in the hidden scroll viewport). Commit is
  queued (`Qt::QueuedConnection`) so the strip can rebuild after the button's
  event returns. **Phase C — compact flyout.** Opens on the **inner** side,
  derived from the column's geometry vs its window (right-edge column ⇒ popup to
  the left), clamped to `QScreen::availableGeometry`. The open icon is a
  checkable/pressed strip button cleared on restore. The popup is now
  **group-styled**: a `panelFlyoutHeader` (`panelFlyoutTitle` + stretch +
  `panelFlyoutClose`) above the detached panel, with a new
  `assets/icons/panel.closeChevron.svg` (double right chevron, 24×24 `#c8c8c8`,
  qrc regenerated) for the close button. Still `Qt::Popup` (click-away), still
  reparents the panel back exactly once. **Phase D — per-widget header menu.** A
  `▾` `QToolButton` (`panelWidgetMenu_<panelName>`) is installed as
  `QTabWidget::setCornerWidget(..., Qt::TopRightCorner)` on each `PanelGroup`, and
  follows the **current tab** (menu + tooltip switch on `currentChanged`); hidden
  when the current panel has no menu table. Menu contents are transcribed from
  `docs/dev/m42-panel-menus.md` (CS6 panel fly-outs, order best-effort;
  `[toggle]`/radio entries checkable). Unimplemented entries ship **disabled**
  with `"<label> — not implemented yet"`. `Close`/`Close Panel Group` are
  excluded (they stay on the M41 tab menu). **Wired:** Layers — New Layer…,
  Duplicate Layer/Group…, Delete Layer/Group, New Group…, Group Layers, Ungroup
  Layers, Hide Layers, Arrange ▸ Move Layer Up/Down, Panel Options…; History —
  Step Forward/Backward, New Snapshot…; Adjustments — Invert, Posterize,
  Threshold, Brightness/Contrast, Hue/Saturation. Everything else disabled (all
  of Channels, Paths, Color, Swatches, Styles, Navigator, Histogram, Info,
  Actions, Properties; Channels/Paths `Panel Options…` stay disabled — no options
  dialog exists for placeholder panels). Gradients/Patterns/Libraries get no
  header button (not CS6 panels: picker pop-ups / CC-only Libraries). **Phase E —
  in-window float overlay.** `PanelFloat` is no longer a `Qt::Tool` top-level: it
  is a plain child of the main window (`Qt::Widget`, `WA_StyledBackground`,
  `#panelFloat{background:#3a3a3a;border:1px solid #555}`), raised, clipped to
  `centralWidget()`'s rect and clamped there on every header drag (`moveFloat`).
  It is parented to `window()` **not** `centerSplitter`, because
  `QSplitter::childEvent` auto-inserts non-window children as panes. Re-dock is
  the existing `applyGroupDrop` (remove-then-insert; overlay destroyed once
  emptied). `m41_tearoff` is unchanged and proves the same claim (group left the
  column, contains its panels, re-docks, no float remains); windowness is now
  proven separately by `m42_float_overlay` (`!isWindow()`). **Self-tests**
  131–139: `m42_minwidth`(131), `m42_iconic`(132), `m42_dragstrip`(133),
  `m42_flyout`(134), `m42_widgetmenu`(135), `m42_float_overlay`(136),
  `m42_fgbg`(137), `m42_menubar`(138), `m42_tools`(139); `m41_width`(121) was
  amended (output format unchanged) and all 120–140 lines are `=1`. **Honest
  limits:** the per-panel menu order/separators are the research doc's
  best-effort reconstruction; the disabled entries are stubs (no invented
  dialogs); Layers `Panel Options…`/History `New Snapshot…` open modal dialogs so
  the test asserts structural wiring, not execution; `New Snapshot…` is marked
  disabled in the research doc but was wired to the existing snapshot behavior;
  Adjustments route through the Layers panel's view (the Adjustments placeholder
  has no view handle); the float overlay does not persist position, is not
  re-clamped on window resize, and cannot cover the docked Tools panel (clamped
  to the central-widget rect); min widths/icon sizes are chosen constants, not
  CS6 metrics; `m42_tools` tolerates ±8 px on the float height under xvfb; the
  stale-layout revision guard is defence-in-depth (proven not to be the overlay's
  cause); drop-on-stack from the strip requires the column to be expanded
  mid-drag. No Rust change: **588 tests, 0 failed, 7 ignored** (unchanged; the
  raw ignored count is 8 with the pre-existing `pictura-render` doctest).
  **Capability:** MODIFIES `panel-column`, `tool-framework`, `application-shell`;
  **no new capability** → **60** canonical specs after archive. Verified:
  `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --
  -D warnings` clean; `TASK_ALLOWS_DOCS=1 bash scripts/verify-fast.sh` →
  `verify-fast: OK`; both self-tests exit 0 with all m42 lines `=1` and every
  earlier m20–m41 line unchanged; `openspec validate m42-panel-refinements
  --strict` valid and `openspec validate --all --strict` 61/61.

- **M43 — panel multicolumn** (a fifth user-requested interruption to the
  Layers-panel program; OpenSpec change `m43-panel-multicolumn`, brief
  `docs/dev/m43-panel-multicolumn.md`). **Phase A — drag/chrome/float.** A tab
  press drags/floats **only that panel** (a one-panel float built via `takePanel`
  + a fresh wired `PanelGroup`), an empty-header press drags/floats the **whole
  group**; both work docked and for a group already floating
  (`PanelColumn::beginPanelDrag`/`createFloat` payload-aware). The panel tab bar
  carries `objectName` `panelTabBar` and scoped QSS — selected tab `${base}`
  (identical to the `QTabWidget::pane`/widget background), unselected
  `${window}` (hover `${hover}`); the document bar is `documentTabBar`,
  unaffected (pixel sample: active `srgb(35,35,35)` = base, inactive
  `srgb(43,43,43)` = window). The corner `▾` is fixed with `ElideRight` +
  `setExpanding(false)` + the corner width added to `updateMinimumWidth`, so it
  is fully visible at the column minimum. `placeFlyout` uses the **actual button
  geometry** and the column's side, clamping only the inner coordinate so it can
  never flip outward or overlap the button. The Tools dock gets
  `setFixedWidth(content)` + `QSizePolicy::Fixed` horizontal + a `Resize`
  event-filter clamp (belt-and-braces for QMainWindow's internal splitter); the
  separator no longer resizes it, min == max == content width in 1- and 2-column
  modes and while floating, with the M42 float-height and the M40 standalone-dock
  contract intact. **`D`** resets fg/bg to default (black/white) through the
  existing `resetColors()`; the M42 `X` swap is kept (`D` was free — not in the
  tool catalogue). Compact icon buttons grew again (30 → 34 px, pixmap 24).
  **Phase B — multi-column host.** `centerSplitter_` now hosts an ordered set:
  left `PanelColumn`s, the document tabs (`objectName` `documentTabs`), right
  `PanelColumn`s; stretch stays on the tabs. `PanelColumn::side()` is derived
  from the splitter index vs `documentTabs` (not geometry). Columns are
  **created on drop** (`createPanelColumn(side)`) and **removed when empty**
  (`removeColumnIfEmpty`, dynamic-only), with the frame owning the wiring
  (`stateChanged`, `interfaceOptionsRequested`). `resolveDrop` gained a
  `DropKind` enum — `Reorder`, `IntoGroup`, `AboveGroup`, `BelowGroup`,
  `OnStrip`, `NewColumnLeft`, `NewColumnRight`, `Outside` — with a **28 px outer
  band / over-the-Tools-dock** rule for new columns, group top/bottom halves for
  above/below, tab-bar hits for into-group, cross-column `IntoGroup` delegation,
  and compact mode handled first so strip drags still reorder. One `#2a7fff`
  indicator marks every candidate. `flyoutSide()` is now `side() == Right ?
  "left" : "right"`. `Window → Panels` targets the owning column;
  `setPanelsHidden`/screen modes iterate all columns. **Phase C — session v6.**
  `panelColumns: [{side, order, groups:[{name, order, visible, minimized,
  collapsed}]}]` with `schemaVersion` **6**; the legacy flat `panelGroups` is
  still written as a mirror and a **v5 store loads as a single right-hand
  column** (synthesised when `panelColumns` is absent); load-then-write keeps
  unknown keys; the M42 `layoutRevision` guard and all v4/v5 keys survive.
  `applyPanelSession` rebuilds N columns and re-applies rail mode to all;
  `clearDynamicColumns` for re-apply. `m41_session`'s schema assertion changed
  from `==5` to `>=5` (the schema advanced — the only earlier-check change).
  **Self-tests** 140–151: `m43_tabdrag`(140), `m43_tabcolors`(141),
  `m43_corner`(142), `m43_newcolumn`(143), `m43_intogroup`(144),
  `m43_boundary`(145), `m43_singlefloat`(146), `m43_tools`(147),
  `m43_icon`(148), `m43_flyout`(149), `m43_dreset`(150), `m43_session`(151);
  all older codes green. **Honest limits:** cross-column drops delegate only
  `into-group` — a whole group dropped onto another existing column (not the
  outer edge) resolves as tear-off rather than a cross-column move; new columns
  are only allocated at the workspace outer edges or over the Tools dock;
  compact "above the first group" boundary is unreachable (the gap resolves as
  on-strip); the vertical group order **inside** a column is written but not
  re-applied on restore (pre-existing M41 behaviour); one workspace-wide
  `panelRailMode`/`railWidth`; float existence/position not persisted; only the
  left+right pair is self-tested (not multiple columns on the same side); the
  primary right column is never removable (an all-left layout leaves it present
  but empty); the Tools `setFixedWidth` clamp is verified under xcb only;
  `headerCornerWidthForTest` over-reserves when the `▾` is hidden. No Rust
  change: **588 tests, 0 failed, 7 ignored** (unchanged — no Rust change; the raw
  ignored count is 8 with the pre-existing `pictura-render` doctest).
  **Capability:** MODIFIES `panel-column`, `application-shell`, `tool-framework`,
  `workspace-persistence`; **no new capability** → **60** canonical specs after
  archive. Verified: `cmake --build` clean; both self-tests exit 0 (no-arg 5/5)
  with all m43 lines `=1` and every earlier m20–m42 line unchanged except the
  `m41_session` `>=5` schema assertion; `cargo fmt --all --check` and `cargo
  clippy --workspace --all-targets -- -D warnings` clean; `TASK_ALLOWS_DOCS=1
  bash scripts/verify-fast.sh` → `verify-fast: OK`; `openspec validate
  m43-panel-multicolumn --strict` valid and `openspec validate --all --strict`
  61/61.

- **M44 — panel/theme polish** (a sixth user-requested interruption to the
  Layers-panel program; OpenSpec change `m44-panel-theme-polish`, brief
  `docs/dev/m44-panel-theme-polish.md`). **Phase A — new-document canvas bug
  (E1).** On startup with no PSD the canvas showed the M0.5 GPU demo's repeating
  black/white/green/red banding instead of the white scratch document. Root
  cause: `PictureView::render_gpu` (the M0.5 GPU spike now in
  `cxxqt_object/impl_core.rs`)
  offscreen-rendered `crate::gpu::render_gradient` and **assigned it directly to
  `rust.image` without touching `rust.doc` or setting `display_dirty`**; startup
  creates a white scratch document, calls `render_gpu()`, then presents
  `view->image()`, so the canvas showed the gradient while the document composite
  stayed white — any later recomposite derives the display from `doc.composite`,
  which is why moving a layer turned it white. Fix: removed the image
  assignment; `render_gpu` is now a pure smoke probe (`Rendered { distinct, .. }`)
  that never mutates the display image (a pre-fix capture had 8634 distinct
  canvas colours, 1 after). New Rust regression
  `crates/pictura-render/tests/gpu_parity.rs::fresh_white_document_composites_uniform_white_twice`
  (37×23, CPU oracle + GPU first and second composite byte-equal white; self-skips
  without an adapter, **runs** on the RTX 3090). The startup check that previously
  asserted the gradient was non-blank became `fresh_white=1`; self-test **153**
  `m44_newdoc white=1 uniform=1 immediate=1`. **Phase B — theme/borders/style.**
  **W1** the collapse chevrons were inverted; swapped the `panel.columnsOne`/
  `columnsTwo` mapping so the toggle shows the correct target state (**154**
  `m44_chevrons`). **W2** the "last item active" default was
  **`PanelColumn::restorePanelState`**: replaying `PanelGroup::setPanelVisible`
  left the **last visible** tab current after a session restore (every normal
  launch after the first), fixed with `setCurrentToFirstVisible()` at the end of
  each group's restore (**155** `m44_defaultactive`). **W3** tab colours: the
  measured visible widget surface is `${window}` (`#2b2b2b`), not the QSS pane;
  active tab = `${window}`, inactive = `${base}` (`#232323`), panel groups get
  `QTabWidget#panelGroupTabs::pane { background: ${window} }`, document tabs
  unchanged (**156** `m44_tabswap`; the M43 `m43_tabcolors` direction corrected).
  **W6** group divider: splitter `setHandleWidth(kGroupDividerWidth = 6)` +
  `QSplitter#panelColumnSplitter::handle { background: ${border} }` (**159**).
  **C2** the compact-strip divider is now 2 px `${border}` (dark grey) instead of
  white (**161**). **C4** strip labels now **elide as soon as there is any room**
  (`QFontMetrics::elidedText`), replacing the fixed `>= 120 px` show/hide
  threshold (**164**). **F1/F2** document tab bar: `QTabBar#documentTabBar {
  border-right: 1px solid ${border}; border-top: 0 }` plus
  `QTabWidget#documentTabs::pane { border-top: 0 }` (**165**; F2 is a no-op at the
  sampled pixels — the only top line was the options bar's own bottom border).
  **S1** darker grey 1 px `${border}` on `#toolsPanel`, `#panelGroupTabs`,
  `#panelColumnIconStrip`, `#panelColumnContainer` (**166**). **S2** removed the
  inline `#panelFloat`/`#panelIconFlyout`/`#panelFlyoutHeader` stylesheets;
  docked, popup and floating now all take the `${window}` surface + `${border}`
  border from theme. Shared constants `Theme::kPanelBorderWidth = 1`,
  `Theme::kGroupDividerWidth = 6`. **Phase C — drag/dock.** **T1** the floating
  Tools dock height is now locked (both axes fixed while floating; the M40
  four-area dock contract is intact — `m40_dock` now asserts all four areas).
  **T2/W5** `CreatePanelColumn(side, anchor)` inserts immediately before/after an
  **anchor** column (not only the splitter ends); `columnEdgeAnchorAt()` yields a
  new-column target beside any `PanelColumn`; `newColumnSideAt` maps the Tools
  dock's side; the Tools dock uses `setAllowedAreas(AllDockWidgetAreas)` with a
  width-or-height lock per dock side. Same M43 `DropKind` resolver, one `#2a7fff`
  indicator (**158** `m44_docksides toolbar=1 column=1 workspace=1 float=1`).
  **W4** `createFloat` no longer cleaned the source group on a tab drag (the
  source tab bar owns the implicit mouse grab), so the float now **tracks the
  cursor until release**; the source is cleaned on commit/cancel (**157**
  `m44_floatdrag`). **C3** `resolveIconicDrop` proximity rule: icon hit → into
  that group; divider → new group at that boundary; inside a group container →
  on-strip/boundary; otherwise new group at the top/bottom end. Each group is a
  `panelIconGroup` container with a `panelIconGroupGrip` (`•••`) drag handle above
  its icons, and the container gets a `${base}` background/border so the icons
  read as one group (**162** `m44_compactdrop`, **163** `m44_draghandle`).
  **160** `m44_popupstyle parity=1` — the flyout shares the docked group's styling.
  **Self-tests** 153–166: `m44_newdoc`(153), `m44_chevrons`(154),
  `m44_defaultactive`(155), `m44_tabswap`(156), `m44_floatdrag`(157),
  `m44_docksides`(158), `m44_divider`(159), `m44_popupstyle`(160),
  `m44_compactdivider`(161), `m44_compactdrop`(162), `m44_draghandle`(163),
  `m44_elide`(164), `m44_filebar`(165), `m44_panelborder`(166); exit **152**
  remains the user's headless-platform check; all earlier m20–m43 lines unchanged
  except the corrected `m43_tabcolors` direction and the strengthened `m40_dock`.
  **Honest limits:** widget columns dock only left/right of another
  column/workspace (the host is a horizontal splitter) — top/bottom is supported
  for the Tools **dock** only; the compact "very close above/below ⇒ into" zone is
  the group container including the grip; multiple columns on one side with a
  column drag in flight remain best-effort; the M44 spec's parenthetical describes
  the pane as `${base}` while the measured surface/implementation uses `${window}`
  (the behavioural scenario holds); F2 is a no-op at sampled pixels; elide width
  is an approximate row allowance; the Rust regression pins the composite
  invariant but not the demo overlay itself (that is pinned by `m44_newdoc` + the
  startup `fresh_white` assertion). **Capability:** MODIFIES `panel-column`,
  `application-shell`, `tool-framework`, `document-canvas`; **no new capability**
  → **60** canonical specs after archive. Verified: `cmake --build` clean;
  `./build/pictura --headless --self-test` exit 0 with all `m44_*` `=1`; the PSD
  headless self-test exit 0; `cargo fmt --all --check` and `cargo clippy
  --workspace --all-targets -- -D warnings` clean; `cargo test --workspace`
  **589 tests, 0 failed, 7 ignored** (up from 588/7; the raw ignored count is 8
  with the pre-existing `pictura-render` doctest); `openspec validate
  m44-panel-theme-polish --strict` valid and `openspec validate --all --strict`
  61/61.

- **M45 — panel fixes** (a seventh user-requested interruption to the
  Layers-panel program; OpenSpec change `m45-panel-fixes`, brief
  `docs/dev/m45-panel-fixes.md`). **Phase A — Tools toolbar.** **T1 sizing:**
  `updateContentMetrics` previously fixed only one axis and read `sizeHint()`
  before the reflowed grid was active, while `topLevelChanged` latched a
  `floatHeight_`; the M43 width lock and M44 height lock could each keep a size
  from the previous column count, so 1↔2 read as 1-col ≈507 px and 2-col ≈862 px.
  Both axes now come from one content formula (`contentWidth` + a new
  `contentHeight` = title bar + margins + `rows*slot + gaps` + fg/bg + screen
  mode + body spacing), released and re-fixed after `layout()->activate()`;
  measured `w1=36 h1=862 w2=69 h2=507`, stable across the toggle. **T2:** the dock
  allowed areas revert to **left/right only** (M44 had `AllDockWidgetAreas`),
  keeping the M40 contract and the per-side lock. **T3:** the floating toolbar now
  docks **beside any widget column** via the existing `columnEdgeAnchorAt` + a new
  `PanelColumn::showEdgeDropIndicator(side)` (reusing the single `#2a7fff`
  indicator) and `commitToolboxDrop` (inserts at the anchor's splitter index); the
  toolbox title-bar drag emits `toolbarDragMoved`/`toolbarDragFinished`. It is
  hosted as a **central-splitter pane** (a `QDockWidget` cannot sit *between*
  columns); its title-bar re-float is not re-wired yet. **Phase B — widget
  panel.** **W1/W2/W3/W6 indicator correctness:** `DropTarget` gained
  `PanelColumn* owner`; `resolveDrop` sets it (`this` for local/workspace-edge/
  iconic, the **anchor** for beside-column, and the **destination column** for a
  delegated cross-column `IntoGroup`); `updateDrag` renders through the owner and
  clears the previous owner's line on change. Root cause: `resolveDrop` delegated
  the target but `showIndicatorFor` still ran on the **source column**, mapping
  the target's tab-bar x through the wrong `scroll_->viewport()` — so a
  right-hand target mapped off to the left (W1) or outside/clipped (W2). Tab
  inserts draw at `target.group->tabInsertionX(target.tabIndex)` (W3) and bottom
  boundaries at the last visible group's bottom edge (W6), in the owning column.
  `kEdgeInside` reduced 6→0 so an inside-edge tab insert is not mistaken for a
  new-column anchor. **W4 emptied column:** one `PanelColumn::maybeRemoveSelf()`
  (calls `frame->removeColumnIfEmpty(this)`) runs from `commitDrop`, `cancelDrag`,
  `closeGroup`, `showPanel(name,false)`, `restoreFlyoutPanel`;
  `removeColumnIfEmpty` **rehomes still-live groups into the primary column** via
  the new `PanelColumn::adoptGroup` (so panels survive for a later Window-menu
  show) and keeps a column that still owns a float. **W5 minimize actually
  collapses:** `PanelGroup::applyMinimize` clamps the **group's** `maximumHeight`
  (and size policy) to the tab-bar height, saving/restoring
  `savedGroupMaxHeight_`; the tab menu entry is state-derived — **"Expand Panel"**
  while minimized, "Minimize" otherwise (`tabMenuActionsForTest` applies the same
  substitution). **W7 never clip:** the scroll area's horizontal policy changed
  `AlwaysOff → ScrollBarAsNeeded`, tab text elides, the corner button keeps its
  reserved width. **W8 shared floor:** one `constexpr int kPanelMinWidth = 180`
  for every normal-mode column (the compact strip keeps `kIconStripMinWidth = 40`),
  replacing the per-column widest-derived floor, so columns share the floor and
  none can vanish. **Phase C — compact parity.** **C1 the popup is a real
  group:** `ensureFlyout` no longer builds a bespoke one-tab header;
  `openIconFlyout` finds the group, `PanelGroup::setCurrentPanel(clicked)`,
  records its index, and reparents the **whole `PanelGroup`** into the popup (it
  stays in `groups_`); `restoreFlyoutGroup` inserts it back at the recorded
  splitter index exactly once, guarded by `restoringFlyout_`/`flyoutGroup_`.
  Docked, popup and floating are now the **same widget instance** (same tabs, `▾`
  menu, minimize, drag, styling). **C2 group-drag line:** in compact mode a
  whole-group drag (`!dragIsPanel_`) anchors `stripIndicator_` to the target
  group's **container top −1** (above the `•••` grip dots), not the first icon
  button; panel drags keep the M42/M44 anchoring. **Self-tests** 167–179
  (`m45_tools_sizing` 167, `m45_tools_sides` 168, `m45_tools_beside_column` 169,
  `m45_indicator_side` 170, `m45_indicator_cross_column` 171,
  `m45_indicator_rightmost_tab` 172, `m45_empty_column_removed` 173,
  `m45_minimize_collapse` 174, `m45_indicator_bottom` 175, `m45_no_clip` 176,
  `m45_min_width_floor` 177, `m45_popup_group` 178, `m45_compact_group_line`
  179); exit **152** remains the headless-platform check. **Honest limits:** the
  toolbar is hosted as a central-splitter pane, so it cannot sit *between* columns
  as a dock and its title-bar re-float is not re-wired yet; a narrow column's
  group content can still scroll (W7's stated trade-off); `removeColumnIfEmpty`
  deliberately does not tear down a column that owns a live float; drag *from
  inside* the popup is not automatically tested; `PanelGroup::detachPanel`/
  `attachPanel`/`detached_` are now unused (left in place) and `theme.cpp`'s
  `panelFlyoutHeader` selector is dead (untouched); C2 falls back to the last strip
  box when the boundary maps to a group hidden from the strip; chosen constants
  remain unsourced CS6 metrics. No Rust change: **589 tests, 0 failed, 7 ignored**
  (unchanged; the raw ignored count is 8 with the pre-existing `pictura-render`
  doctest). **Capability:** MODIFIES `panel-column`, `application-shell`,
  `tool-framework`; **no new capability** → **60** canonical specs after archive.
  Verified: `cmake --build` clean; `./build/pictura --headless --self-test` exit 0
  with all `m45_*` `=1`; the PSD headless self-test exit 0; `cargo fmt --all
  --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean;
  `cargo test --workspace` **589 tests, 0 failed, 7 ignored**; `openspec validate
  m45-panel-fixes --strict` valid and `openspec validate --all --strict` 61/61.

- **CI headless and build speed** (infrastructure, not a milestone; archived
  OpenSpec change `ci-headless-and-speedup`). `main.cpp` gains an explicit **`--headless`**
  flag: it selects the offscreen QPA plugin before `QApplication` (when
  `QT_QPA_PLATFORM` is unset), wins over the `--self-test` xcb override, and
  implies `--self-test` when no document is given so it never blocks in
  `app.exec()`. The self-test asserts `QApplication::platformName() ==
  "offscreen"` and exits `152` otherwise. `--interop-probe` is excluded (it
  needs a real platform Vulkan instance); explicit `QT_QPA_PLATFORM=offscreen`
  and `xvfb-run` still work. Build speed: a new `.cargo/config.toml` links with
  `lld`, `[profile.test]` compiles dependencies at `opt-level = 0` with the
  workspace crates pinned at `2`, and CMake uses Ninja + `--parallel` with the
  self-test run headless. CI is rebuilt into three jobs — `rust` (fmt, clippy,
  nextest, doctests), `qt-headless` (pinned Qt 6.11.1 + CMake/Ninja +
  `--headless --self-test`), and `oracles` (ImageMagick + `psd-tools`, full
  `cargo test --workspace`) — with registry/sccache caching. No Rust API,
  document, codec, compositor, or dependency change: **588 tests, 0 failed, 7
  ignored** (unchanged).

## Canvas viewport & performance (post-M24 pass)

Not an OpenSpec capability — a correctness/performance pass; the intended
behaviour (move-tool behaviour, budget, suspected bottlenecks, acceptance
checks) is written up in `docs/dev/canvas-view-spec.md`.

- `image_view.{h,cpp}`: `setImage` now fits-and-centres (fit when the image
  exceeds the viewport, else 100 % centred) and re-applies that initial view on
  resize until the user pans/zooms; middle-button drag pans the canvas
  regardless of the active tool; `fitOnScreen`/`actualPixels` re-arm the
  initial view.
- `frame.{h,cpp}`: `refresh()` keeps the canvas/status/menu updates synchronous
  but defers the expensive panel refresh (`retargetDock`) behind a single-shot
  120 ms `QTimer`, so a burst of `changed` signals no longer blocks the canvas
  repaint; tab add/remove/switch force an immediate panel refresh.
- `cxxqt_object/impl_transform.rs` + `tools.cpp`: the Move tool previews live **without
  compositing during the drag**. `begin_move_preview()` caches a base image (the
  document composited with the moved topmost raster layer hidden), the layer's
  own image, its document-space top-left, and its opacity; `move_preview_base/
  layer/x/y/opacity` and `end_move_preview()` expose/clear it.
  `ImageView::beginMovePreview(base, layer, layerPos, opacity)` +
  `setMovePreviewDelta(delta)` + `endMovePreview()` draw the cached base then the
  moved layer at the live Qt delta (source-over with `setOpacity`) in
  `paintEvent`. `ToolController` Move press seeds the preview, move updates only
  the delta, and release calls `end_move_preview` → `commit_move(dx, dy)` once
  (a single composite, exactly one "Move Layer" history state) →
  `endMovePreview`; switching tools or rebinding the canvas cancels it. The old
  per-event `move_preview(dx, dy)` is retained only for the self-test
  (`// ponytail: slow path`). One drag = one history state; undo restores the
  pre-drag pixels.
- Move-tool root cause, measured: each mouse-move previously ran `move_preview`
  → `pictura_render::translate_layer` (full composite) plus `document_to_image`
  (a **second** full composite) and a full planar→RGBA conversion. A single
  1024×1024 two-layer `composite_rgba` measures ~**257 ms in the debug build**
  (the old CMake default) and ~**39 ms in the optimized build** (~6.6×). The one
  remaining cost is the single commit composite on mouse-up; the live preview is
  source-over only (non-Normal blend modes, masks, and clipping are not
  reproduced mid-drag, but the committed image is exact).
- `CMakeLists.txt` now defaults `CMAKE_BUILD_TYPE` to `RelWithDebInfo` when
  unset; Corrosion maps any non-Debug config to cargo `--release`, so the Rust
  crate is built optimized too (verified: `build/libpictura_app.a` is the release
  artifact).

Self-test exit codes 64–67, measured identically on fixture and no-argument
runs: `canvas_centre offset=(270.691, 5) zoom=1`; `canvas_middle_pan
delta=(30,15)`; `canvas_move preview=1 hist=1 undo=1`; `canvas_preview_cache
began=1 base=1 layer=1 hist_unchanged=1`. These labels were renamed `m25_*` →
`canvas_*` during M25 so the `m25_` prefix belongs to the actual milestone; the
exit codes are unchanged. Gates green:
`cmake --build build`; both self-tests exit 0; `cargo fmt/clippy/test` clean
(no new Rust tests; the app crate stays green). The headless self-test cannot
measure frame timing, so the interactive feel (pan/zoom/move latency on large
documents) still needs a real GUI check.

## Spec workflow (OpenSpec)

OpenSpec is the per-change requirements layer over `docs/`. See `AGENTS.md`
"Spec workflow (OpenSpec)". M0–M34 are archived; `openspec/specs/` is now the
canonical contract, with the per-change history under
`openspec/changes/archive/`. New work starts as a new change under
`openspec/changes/` (not as code), with `proposal.md`, `design.md`, `tasks.md`,
and `specs/<capability>/spec.md` deltas, archived into `openspec/specs/` when
complete.

## Conventions (keep doing)

- Task briefs live in `docs/dev/m*-*.md`; docs changes need a commit message
  containing `TASK-ALLOWS-DOCS` or `TASK_ALLOWS_DOCS=1` for `guard.sh`.
- Each milestone: freeze interfaces → dispatch 2–3 `general` sub-agents on
  **disjoint files/crates** → orchestrator integrates, un-ignores oracle tests,
  verifies, commits. Never let an implementer verify its own work without an
  independent oracle (psd-tools / ImageMagick / the app self-test).
- Oracles: don't fake tolerances. Where ImageMagick/Photoshop semantics diverge,
  reclassify as "no faithful equivalent" and use property/known-value tests.

## Next: panels program (M46–M49), canvas perf series deferred

### Panels program (M36–M49) — M36–M41 done; M42–M45 panel refinements/multicolumn/polish/fixes done; M46 layer filtering/search next

The CS6 Layers panel program's research, gap analysis, and staged plan live in
`docs/dev/layers-panel-program.md`. **M36 — layer attributes end-to-end** (change
`openspec/changes/m36-layer-attributes`) and **M37 — layer creation and grouping**
(change `openspec/changes/m37-layer-creation`) are implemented and verified (see
the milestone entries above): M36 added `Layer.fill`/`lock`/`color`, `opacity ×
fill` compositing on CPU and GPU, `lspf`/`lclr`/`iOpa` PSD I/O, the bridge
getters/setters, and the Fill/lock/color panel controls; M37 added
`document_ops::layer_ops` New Layer / New Group / Duplicate / Group / Ungroup, the
bridge methods, the panel buttons and the five `Layer` menu commands.

**M39 — panel anatomy** (change `openspec/changes/m39-panel-anatomy`, brief
`docs/dev/m39-panel-anatomy.md`) is implemented and independently verified (see
the milestone entry above): the full expandable layer tree, the frozen layer-path
grammar and depth-first topmost-first projection, the path/batch bridge API,
the `QAbstractItemModel` tree + delegate row anatomy, multi-selection with the
per-node refusal table, solo visibility, `Tab` rename, Panel Options (session
schema v3), the panel/row menus, tooltips, and the explicit drag-reorder
deferral to M47.

**M38 was a user-requested interruption: the full CS6 toolbox icon/cursor
library and the panel icons** (`openspec/changes/m38-icon-cursor-library`,
contract `docs/dev/m38-icon-cursor-library.md`) — the frozen 71-tool catalogue,
the full icon/cursor asset set, the single-column flyout toolbox, and the
panel/Layers/History icons. It is implemented and verified (see the milestone
entry above); it took the M38 number, so the panel stages shifted by one:
**M39 — panel anatomy** is done (see the milestone entry above).

**M40 was a second user-requested interruption — the CS6 Tools panel**
(`openspec/changes/m40-tools-panel`, brief `docs/dev/m40-tools-panel.md`): the
custom lower-right flyout triangle, hold/right-click flyout with shortcut keys,
the one/two-column double-arrow toggle, the standalone dock (left/right only, no
tab groups), and the generic `Shift`+letter group cycling gated by a session
`Use Shift Key For Tool Switch` preference (session schema v4). It is
**implemented and independently verified** (see the milestone entry above).

**M41 was a third user-requested interruption — the CS6 panel column**
(`openspec/changes/m41-panel-column`, brief `docs/dev/m41-panel-column.md`):
content widgets moved under a `PanelColumn`: in normal mode vertical tab groups
with the tabs explicitly on top, in compact mode an icon strip with group
dividers, labels-on-widen, and `Qt::Popup` flyouts that close on click-away; the
`Auto-Collapse Iconic Panels` preference defaults **off**, the double-chevron
toggle, fixed CS6 Essentials groups, and the hard panel minimums removed with a
scroll so the window resizes freely; session **v5** adds
`panelRailMode`/`railWidth`/`autoCollapseIconic`, and the Preferences dialog
(General + Interface panes) gives `Use Shift Key For Tool Switch` and
`Auto-Collapse Iconic Panels` a UI — M40's previously UI-less `Use Shift Key For
Tool Switch` now has its home there. It is **implemented and independently
verified** (see the milestone entry above). **M42 was a fourth user-requested
interruption — panel refinements** (`openspec/changes/m42-panel-refinements`,
brief `docs/dev/m42-panel-refinements.md`, research
`docs/dev/m42-panel-menus.md`): the compact-strip drag/reorder, the group-styled
compact flyout, the per-widget header menus, the in-window float overlay, the
bounded normal-mode width / larger icons, and the menu-bar overlay root cause (a
duplicate `actionsPanel_`). It is **implemented and independently verified** (see
the milestone entry above). **M43 was a fifth user-requested interruption — panel
multicolumn** (`openspec/changes/m43-panel-multicolumn`, brief
`docs/dev/m43-panel-multicolumn.md`): the multi-column host of
create-on-drop/remove-when-empty columns, tab-vs-group drag and single-panel
floats, the unified `DropKind`/`resolveDrop` drop targets, the `panelTabBar` tab
colours, the fixed corner button and actual-geometry inner-side flyout, the
fixed-width Tools dock, the `D` colour reset, and session **v6**
  (`panelColumns`). It is **implemented and independently verified** (see the
  milestone entry above). **M44 was a sixth user-requested interruption — panel/
  theme polish** (`openspec/changes/m44-panel-theme-polish`, brief
  `docs/dev/m44-panel-theme-polish.md`): the new-document canvas E1 root cause
  (the GPU demo image assigned straight to `rust.image`) and its fix, the
  chevron/tab-colour/default-active corrections, the theme border/tab/flyout/elide
  unification, the float-drag continuation and any-side docking, and the compact
  group-relative drop with drag handles. It is **implemented and independently
  verified** (see the milestone entry above). **M45 was a seventh user-requested
  interruption — panel fixes** (`openspec/changes/m45-panel-fixes`, brief
  `docs/dev/m45-panel-fixes.md`): the resolver-owner drop indicator, the single
  emptied-column cleanup, the group-height minimize with its state-derived label,
  the one-formula Tools sizing and left/right-only beside-column pane, the shared
  minimum-width floor with no clipping, and the whole-group compact popup. It is
  **implemented and independently verified** (see the milestone entry above).
  Because the M40/M41/M42/M43/M44/M45 interruptions claim six numbers the
  Layers-panel program had reserved, that program shifts by **six**: **M46**
  layer filtering/search (the six-dimension
  filter/search row) is next, **M47** remaining management
  (rasterize/merge/flatten/link/select-similar/convert-background/
  layer-via-copy-cut, the New Layer/Group dialogs, and the deferred drag-reorder
  with its recorded drop rules), **M48** styles/effects, and **M49** smart
  objects / vector masks / artboards-as-non-goal / layer comps. M36's confirmed
  ceilings — the `layer_kind` `"background"` name+index heuristic and the forced
  type/shape locks — land in M47, and M37's single-layer grouping
  limit was lifted by M39's multi-selection (the `is_background` single source of
  truth and the path/batch selection ops). The pre-shift numbers still stand in
  `docs/dev/layers-panel-program.md`; this file is the up-to-date anchor.

> These numbers reuse M36–M38 previously sketched for canvas performance below.
> `docs/dev/canvas-compositing-plan.md` is frozen and still uses them, so read
> those tracks by name (history COW, resident GPU sources, 256² tiles), not by
> number; they are deferred until after M49.

M31 removed the full composite and readback from every move and paint
(dirty-rect compositing), M32 removed it from the move-preview base and the
visibility toggle and cached the present-scale, and M33 removed the host-side
per-pixel assembly from the remaining full composites; every other mutation still
composites and reads back the **whole** document, and the CPU compositor and
`pictura_filters::apply` are still the oracles. The research and the M31–M35 plan
are written up in `docs/dev/canvas-compositing-plan.md`; the M32 brief with the
measured phase table is `docs/dev/m32-interactive-canvas.md`, and the M33 brief is
`docs/dev/m33-composite-throughput.md`. The 4000² full composite is now ~123 ms,
dominated by the per-composite source/mask **upload** (~128 + 16 MB), not the GPU
dispatch (~0.2 ms) or the ~38 ms readback, so a zero-copy present still saves
little while the upload stays.

The perf series was **paused** for **M34 — composite coherence and cheap
undo/redo** (OpenSpec change `m34-composite-coherence`, **implemented**; brief
`docs/dev/m34-composite-coherence.md`): the canvas rebuild persists its rendered
result into `doc.composite` (RGBA for an RGB document, plane-count-preserving
for a non-RGB mode), Save serializes that current composite, `undo`/`redo`
restore the display from the snapshot composite instead of a full composite
(unconditionally), and the `record` call moves after the composite step so the
snapshot carries it. It is app-local and bounded: no `write_psd` format change,
no new capability. Next in order:

- **M35 — region blit in C++ (implemented; archive pending).** OpenSpec change
  `m35-cpp-region-blit` (MODIFIED `document-canvas`; no new capability → **59**
  after archive), brief `docs/dev/m35-cpp-region-blit.md`. `PictureView` no longer
  maintains a region-patched full-resolution `QImage`: `refresh_region` composites
  the region, patches the planar `doc.composite` with `copy_from_slice`, converts
  only the region-sized buffer to a `QImage`, sets a `display_dirty` marker and
  emits a new `region_blitted(QImage, x, y)` signal; `ImageView::blitRegion` paints
  it with `QPainter` + `CompositionMode_Source` (invalidating the present zoom
  cache so the next paint rebuilds it identically). The per-pixel
  `QImage::set_pixel_color` loop and `REGION_REFRESH_BUDGET` are deleted, so a large
  dirty region is blitted in C++ instead of forcing a full document composite.
  `image()` rebuilds from `doc.composite` when `display_dirty` (with an explicit
  in-stroke guard), `sample_argb` reads the planar composite directly, and
  `move_preview_base` builds from the current composite. Measured 4000² (release,
  GPU): a 1024² region refresh **~33 ms → ~8.8 ms** (composite 4.95 + composite
  patch 1.81 + region convert 2.04) plus one `QPainter::drawImage` blit in C++; a
  512² region refresh **3.29 ms**. The old path was a 27.3 ms per-pixel FFI blit
  plus a ~5.6 ms composite. Honest notes: `refresh_region` no longer emits
  `changed`, so panel refresh on the region path is debounced in `frame.cpp`; a
  region blit invalidates the present zoom cache (one rescale on the next paint,
  the same cost as the old `replaceImage`); `begin_move_preview` now pays one planar
  clone + conversion per drag start; mid-stroke `sample_argb` reads the pre-stroke
  composite (unreachable while painting). `m31_region_large` now asserts the region
  path ran *and* the canvas equals a full recomposite (strictly stronger than
  before), and `move_preview_region` no longer has an oversized-rect fallback
  because the budget is gone. Verified: `cargo test --workspace` **546 tests, 0
  failed, 7 ignored** (up from 544, 6 ignored; the new
  `m35_region_refresh_profile_4000`), `cargo fmt`/`clippy` clean, both self-tests
  exit 0 with `m35_region_blit region=1 changed=0 canvas=1 rebuilt=1 cache=1` and
  `m35_region_large region=1 recomposite=0 canvas=1`, `openspec validate --all
  --strict` 60/60.

- **M34 — composite coherence and cheap undo/redo (implemented; archived).**
  OpenSpec change `m34-composite-coherence` (MODIFIED
  `edit-history`, `document-lifecycle`; no new capability → **59** after archive).
  The canvas rebuild now persists its rendered frame into `doc.composite`
  (`store_composite`: RGBA for an RGB document, colour-plane-count-preserving
  otherwise), `recomposite` renders → stores → builds the `QImage` from the
  rendered frame, `refresh_region` patches the composite on its region path, and
  `record` runs after the composite step so every snapshot carries a current
  composite. `undo`/`redo` therefore rebuild the display with
  `buffer_to_image(&snapshot.doc.composite)` instead of a full composite, and
  Save serializes the current composite. Measured 4000² (2 RGB layers, release,
  GPU): the old undo display path `document_to_image(gpu=true)` **163.16 ms** vs
  the new `buffer_to_image(snapshot.composite)` **40.94 ms** (~4×); the remaining
  per-snapshot cost is the whole-document `History::capture` clone at **59.49 ms**
  (COW/tile-diff history remains deferred). Honest limits: a layered **grayscale**
  document with transparent coverage restores opaque because its composite stays
  1-plane, and a dimension-changing op on grayscale keeps the pre-existing
  4-plane outcome. Verified: `cargo test --workspace` **544 tests, 0 failed, 6
  ignored** (up from 541, 5 ignored; the new `m34_undo_profile_4000`), `cargo
  fmt`/`clippy` clean, both self-tests exit 0 with
  `m34_coherent composite=1 undo=1 save=1`, `openspec validate --all --strict`
  60/60.
- **M33 — full-composite throughput (done; archived).** OpenSpec change
  `m33-composite-throughput` (MODIFIED `gpu-compositing`; no new capability),
  implemented and verified. Row-wise source/mask assembly, a fused planar readback
  that skips the packed `Vec`, and a GPU command-buffer canvas clear took the
  4000² two-layer composite ~254 ms → ~123 ms (~2×), byte-identical. The remaining
  bottleneck is the per-composite upload; resident per-layer GPU source buffers are
  deferred (they need content versioning).
- **GPU-resident zero-copy present (deferred; was planned as M34)** via Qt Quick
  (`QQuickRhiItem` sharing the window's `QRhi` +
  `QQuickWindow::createTextureFromRhiTexture()`, or one shared Vulkan device via
  `QQuickGraphicsDevice::fromDeviceObjects(...)`); `QRhiWidget` cannot adopt the
  wgpu device. Deferred because it removes only the ~38 ms readback of a ~123 ms
  composite, not the upload.
- **256² GPU tiles (deferred) + LRU + seam gutters + mipmaps**
  (Graphite-style), only if pan/zoom over documents larger than VRAM demands it;
  includes display-time LoD so a zoomed-out view composites a proxy.

Deferred canvas-performance tracks (previously sketched as M36–M38; those
numbers are now claimed by the panels program above, so these are deferred
until after M49). The remaining canvas-performance tracks — history
copy-on-write / tile diffs, resident per-layer GPU source buffers, 256² tiles +
LoD, plus the GPU-resident zero-copy present — each need their own design (the
small, app-local region-blit slice landed as M35 above):

- **Cheap undo/redo + composite coherence/save** — landed as M34
  (`m34-composite-coherence`); see the brief `docs/dev/m34-composite-coherence.md`.
  The original concern — persisting the rendered composite into `doc.composite`
  changes what `write_psd` serializes — is handled by storing the rendered RGBA
  frame for RGB and preserving the composite's colour-plane count for a non-RGB
  mode, so the byte layout is unchanged.

- **M35 — region blit in C++ / `REGION_REFRESH_BUDGET` removal — landed.** See
  the milestone entry above; OpenSpec change `m35-cpp-region-blit`, brief
  `docs/dev/m35-cpp-region-blit.md`. The per-pixel `QImage::set_pixel_color` loop
  and the budget fallback are gone; a dirty region of any size takes the region
  path.
- **History copy-on-write / tile diffs** — the history capture still clones
  the whole document (~60 ms per state at 4000², and holds up to 20 states).
- **Resident per-layer GPU source buffers and shader-side planar output** —
  deferred from M33. Keeping a layer's source plane resident on the GPU across a
  composite session needs content versioning to detect a changed layer; the
  remaining composite cost is the per-composite upload (~128 MB + 16 MB at
  4000²), which residency would remove. A shader-side planar output would remove
  the ~32 ms readback de-interleave.
- **Transparency grid preferences** — M30's checkerboard is fixed at an 8 px
  Light (`#FFFFFF`/`#CCCCCC`) grid; the `Transparency & Gamut` preferences pane
  (grid size None/Small/Medium/Large, colour sets Light/Medium/Dark/Red/Custom),
  the `View > Show > Transparency Grid` toggle, and gamut warning are deferred.
- **GPU painterly/stochastic filters and GPU painting** — the remaining M22
  Artistic and M25 Brush Strokes/Sketch/Texture families (`Watercolor`,
  `Conté Crayon`, `Paint Daubs`, `Dry Brush`, `Ocean Ripple`, `Spatter`,
  `Sponge`, `Palette Knife`, `Add Noise`, `Colored Pencil`, `Crystallize`) on the
  GPU by **pre-generating their seeded RNG fields on the CPU and running only the
  spatial work on the GPU**, since the RNG stream cannot be reproduced
  bit-exactly on the GPU, and the paint dab loop (GPU painting).
- Real content for the M24 placeholder panels (gradient/pattern presets,
  Properties binding, adjustment presets, libraries, channel/path lists, actions)
  and image modes / bit-depth (16/32-bit, CMYK/Lab gating for filters and
  adjustments).

Process: every new milestone is proposed through OpenSpec first
(`openspec/changes/<name>`, new capabilities), validated, then implemented.
M6 through M34 are archived; their deltas now live in `openspec/specs/`.

## Known risks / open items

- Core API is frozen only where noted; adding fields breaks struct literals.
- PSD descriptor coverage is partial (adjustment layers, layer styles not yet).
- GPU is the default compositor and the default filter path, but the CPU
  compositor and `pictura_filters::apply` remain the oracles.
- A `Dissolve` layer or an unsupported adjustment forces a whole-document CPU
  fallback for that composite.
- Fifteen filter kernels are GPU-accelerated (the blur/sharpen/High Pass family
  plus the M28 heavy window/effect set); the stochastic/seeded filters and the
  warps/distort and render filters fall back to the CPU oracle byte-for-byte.
- The GPU compositor and filter path are host-side bound, not readback-bound: at
  4000² the composite is now dominated by the per-composite source/mask upload
  (~128 + 16 MB) after M33 removed the per-pixel assembly, while the 64 MB
  readback is ~6 ms (see `docs/dev/canvas-compositing-plan.md` §2.1); resident
  per-layer buffers (deferred) and zero-copy present (M34) both aim at this.
- M35 (`m35-cpp-region-blit`) removed both the per-pixel `QImage::set_pixel_color`
  blit and the 1 MP `REGION_REFRESH_BUDGET` fallback: the planar composite is the
  authoritative canvas, `refresh_region` signals the region, and C++
  `ImageView::blitRegion` (`QPainter`, `CompositionMode_Source`) paints it. A
  Display-resolution proxy (LoD) is still absent, so a zoomed-out composite still
  covers the whole document.
- History capture clones the whole document for each state, so large documents
  pay both RAM (up to 20 states) and latency (~60 ms/state at 4000²); copy-on-write
  or tile diffs are the deferred fix.
- The GPU path still silently falls back to the CPU on any `GpuError`.
- `pictura-app` has one `#[ignore]`d interop test.
- The recent-files menu is rebuilt at startup, so a file opened in-session
  appears there only after restart.
- Quick Selection is a wand-union approximation, not a true Photoshop quick
  selection; crop is destructive (no crop region / no non-destructive re-crop);
  selection marching ants are not implemented — only a rubber band during drag
  and the committed bounds are shown.
- Icon art is a first functional pass; a visual refinement pass can change SVG
  paths without any code change.
- Cursors render at a single DPR (no per-screen 2×/3× cursor variants yet).
- The M20 Layers panel shows top-level rows only: group-tree expansion,
  drag-reorder, layer lock flags, clipping/link/color labels, and a
  filter/search row are not implemented.
- Swatch library file I/O, Info color samplers, and Histogram source/cache
  states are not implemented.
- Painting is limited to 8-bit RGB single raster layers; the coverage scratch is
  a layer-sized buffer (sparse tiles deferred).
- Only the Normal/Dissolve/Behind/Clear paint modes exist; there is no tablet
  pressure mapping or brush presets yet.
- Artistic filters are behavioural-parity models without an Adobe oracle; the
  Filter Gallery UI, Smart Filters, and depth/mode gating are not implemented.
- The CS6 chrome is a defensible dark look, not a pixel-exact match (exact CS6
  colours/metrics are unsourced).
- Panel contents beyond M20 and workspace presets/icon-collapse docks are not
  implemented.
- The M24 panels are structural placeholders with empty states, not features;
  icon-collapse, workspace presets, and panel-title-bar menus are not
  implemented.
- Oil Paint is a CPU behavioural model: CS6 requires a supported GPU (closed
  OpenCL kernel, no CPU fallback), so the result is a deliberate non-parity
  divergence rather than verified parity.
- The OS font/filter gallery UI is still absent.
