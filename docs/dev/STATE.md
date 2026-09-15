# STATE — project resume anchor

Snapshot for resuming after a context break. Update after each milestone.

## Where things are

- Repo: `github.com/Zawaro/kooka-pictura`, branch `main`. Docs-only corpus +
  a working Rust/Qt engine.
- Toolchain: Rust 1.98 (`rust-toolchain.toml`), system Qt **6.11.1**, cxx-qt
  **0.10.0**, wgpu **30.0.1**, lcms2 **6.2.0** (system Little CMS 2.19).
- Oracles installed for tests: `psd-tools` 1.19, ImageMagick 7.1.2, `magick`.
- Test suite: **515 tests, 1 ignored** (one pre-existing app `#[ignore]`).
- OpenSpec **1.3.1** (`/usr/bin/openspec`). M0–M26 archived; canonical specs are
  in `openspec/specs/` (58 capabilities, `validate --all --strict`
  green), change history under `openspec/changes/archive/`.
- The C++ app needs **Qt6::Svg** (`Qt6Svg` CMake package) alongside the other Qt
  modules; icons and cursors render through `QSvgRenderer`.

## Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cmake -S . -B build && cmake --build build
xvfb-run -a ./build/pictura --self-test
xvfb-run -a ./build/pictura --self-test crates/pictura-codec/tests/fixtures/two_layers.psd
bash scripts/guard.sh
openspec validate --all --strict
```

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
| `pictura-render` | CPU compositor (27 blend modes, groups, masks, adjustment layers) + GPU compositor (default backend: 26 GPU modes + 5 adjustment layers, `Backend`/`composite_active`) + PSD adjustment encode/decode + `apply_filter` (layer filter gated by mask) + `document_ops` (document resize/canvas/orientation/crop/layer-translate; re-exports `Anchor`/`Resample`) |
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
  `has_document()` for enablement. `main.cpp` shrinks to startup + self-test;
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
  `main.cpp` self-test (exit codes 39–46): tool switching, marquee rect (16 px)
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
  menu-action icons, and the active tool's SVG cursor. `main.cpp` self-test exit
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
  `main.cpp` self-test exit codes 50–52 (`m20_layer count=2 name=1 blend=1
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
  and could not be painted. `main.cpp` self-test exit codes 53–56
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
- `cxxqt_object.rs` + `tools.cpp`: the Move tool previews live **without
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
"Spec workflow (OpenSpec)". M0–M26 are archived; `openspec/specs/` is now the
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

## Next: GPU filters (M27) + GPU painting, zero-copy present, then panel content and image modes (propose via OpenSpec first)

M26 makes the GPU compute backend the default compositor; the CPU compositor is
still the oracle. Next in order:

- **`m27-gpu-filter-acceleration`** — run the `pictura-filters` families as GPU
  compute passes, and extend the same path to GPU painting (the dab loop).
- On-screen **zero-copy present** — manual QRhi + `QWindow` swapchain;
  `QRhiWidget` blocks wgpu device adoption (`crates/pictura-app/GPU-INTEROP-NOTES.md`).
- Off-GUI-thread compositing.
- Real content for the M24 placeholder panels (gradient/pattern presets,
  Properties binding, adjustment presets, libraries, channel/path lists, actions)
  and image modes / bit-depth (16/32-bit, CMYK/Lab gating for filters and
  adjustments), plus Dissolve on GPU.

Process: every new milestone is proposed through OpenSpec first
(`openspec/changes/<name>`, new capabilities), validated, then implemented.
M6 through M26 are archived; their deltas now live in `openspec/specs/`.

## Known risks / open items

- Core API is frozen only where noted; adding fields breaks struct literals.
- PSD descriptor coverage is partial (adjustment layers, layer styles not yet).
- GPU is the default compositor but the CPU compositor remains the oracle.
- A `Dissolve` layer or an unsupported adjustment forces a whole-document CPU
  fallback for that composite.
- The GPU path is readback-bound until zero-copy present lands.
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
