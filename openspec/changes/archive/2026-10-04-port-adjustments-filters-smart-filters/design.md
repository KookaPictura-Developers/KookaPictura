# Design

## Context

See `proposal.md` — Why. State the approach must work within:

- `pictura-adjust` already carries most CS6 adjustment variants; `pictura-filters`
  already carries the Artistic family and the Color Lookup / filter kernels. The
  missing pieces are a few kernels (`Lighting`, `Diffuse`, `GlowingEdges`,
  `HdrToning`), two adjustment variants (`ShadowsHighlights`, `ReplaceColor`),
  and the menu/dialog wiring.
- The destructive adjustment path is `apply_op_active_region` →
  `ActiveOp::Adjustment` → `pictura_render::apply_adjustment_region`, which is
  pointwise by contract. Filters run through `apply_filter_active_region`, which
  snapshots for preview and commits one state.
- `command_tree.cpp` keeps the hand-written CS6 menus; `filter_commands.cpp`
  holds the implemented filter rows; a kind is live when it has a `FILTER_ARITIES`
  entry and a `filter_from_kind_params` arm.
- Smart filters exist only as PSD metadata: `SmartObject::smart_filters` is
  parsed from `filterFX` and re-emitted on save, but `pictura-render` never
  composites them and the Layers panel has no rows for them.
- `cxxqt_object.rs` (1227) is at its file-size allowlist ceiling, which may only
  shrink; `filter_map.rs` dropped under the cap when its inline tests moved to
  `filter_map_tests.rs`. New bridges follow the file-level-bridge pattern
  registered in `build.rs`, and new dialogs must be listed in `CMakeLists.txt`
  (no globbing).

## Goals / Non-Goals

**Goals:**
- Close every gap in issues #82/#83/#87 with one engine source of truth and one
  menu source of truth per feature.
- Let an imported `filterFX` chain visibly affect the composite, and expose it
  in the Layers panel with per-filter visibility.
- Keep the destructive Cancel/preview guarantees already established for filters
  and adjustments.

**Non-Goals:**
- Encoding generic `Filter`/`Adjustment` values back into Adobe `Fltr`
  filter ids (only the camera-raw id `2683` is modelled). Inventing ids would
  break PSD round-trips.
- Filter-mask pixels (`FMsk`/`FXid`/`FEid`): not parsed or authored; the group
  mask row shows no mask yet.
- `Flame`, `Tree`, `Picture Frame` (post-CS6), and the full multi-light CS6
  Lighting workspace (one lamp only, matching photorust).

## Decisions

**1. Route each engine-ready capability through the machinery it already fits.**
Shadows/Highlights and Color Lookup are pointwise/dialog-shaped, so they reuse
the existing block/default/`layout` dialogue machinery (`adjustment_defaults`,
`adjustment_params`, `kDialogs`) and the `ActiveOp::Adjustment` preview path.
Color Lookup needed one special case: `set_adjustment_param` cannot rewrite an
embedded 3-D LUT, so the `"preset"` key rebuilds the block (`set_color_lookup_preset`)
instead of writing a scalar. Alternative considered: encode a 1-D lookup list
into `ColorLookupParams` — rejected as a public-struct change to the compositor
for no observable gain.

**2. HDR Toning is a filter-backed neighborhood operator, not an adjustment.**
Its Local Adaptation split needs a Gaussian base, which `pictura-adjust` (no
filter dependency) cannot produce. `HdrToning` therefore lives in
`pictura-filters`, is dispatched by the bridge through the existing
filter preview/commit core (`apply_filter_obj_active_region`), and does **not**
grow `filter_map.rs` (`hdr-toning` is not a menu filter row). It applies to the
active pixel layer, matching Kooka's other destructive adjustments rather than
photorust's flatten-first. Alternative considered: a pointwise
exposure/gamma adapter — rejected because it reproduces none of the presets the
issue asks for.

**3. Replace Color is a dedicated non-modal dialog.**
Its variable-length sample list cannot be carried by the scalar `Field` control
mechanism, and the eyedropper must sample the canvas. The dialog follows
`ColorRangeDialog`'s proven sampler (`sample_argb` + `ToolController::setCanvasSampler`)
and its non-modal open/finish wiring; preview/apply go through the same `ActiveOp`
path so Cancel is bit-identical. Alternative considered: a modal dialog without
sampling — rejected because sampling is the feature.

**4. The Artistic family is menu wiring only.**
All fifteen kernels, `FILTER_ARITIES` rows, and match arms already exist (and are
covered by the arity guard), so the change adds the `Filter ▸ Artistic` leaves
and the `filter_commands.cpp` rows; the engine is untouched.

**5. Smart Filters model the group, parse/author the flags, and composite the
chain over the embedded source.**
- `SmartObject` gains `smart_filters_enabled` (and `filter_mask_*` for a future
  mask). Group `enab` is parsed and authored; the existing per-filter `enab`
  already works.
- The generic `attach_smart_filter` is extracted from the camera-raw-only
  append path so any `SmartFilter` can be attached to a preserved `SoLd` or to
  the typed list.
- `composite_layer_inner` branches to a chain compositor when **every enabled
  filter decodes**, and always applies the chain to the **embedded source**, not
  the baked proxy. This is what prevents double application of an imported
  result; an unknown enabled filter falls back to the proxy path.
- The Layers tree is built as **synthetic rows** (`path/@sf`, `path/@sf/<i>`)
  appended to the existing row projection, with a `synthetic` guard so rename,
  drag, and layer operations skip them. Alternative considered: extending
  `flatten_rows`/`resolve_path` — rejected because those assume every path is a
  real `Layer`.
- Toggling a filter or group eye rewrites the preserved `SoLd`/`SoLE`
  `enab`/`filterFX.enab` in place via `set_smart_filter_enabled` /
  `set_smart_filters_enabled` and syncs the typed view, so the toggle persists
  across a write and re-read.
- `Convert for Smart Filters` delegates to `convert_to_smart_object`.

**6. Adjustment-layer kinds use the existing default encoders.**
`default_adjustment_block` already encodes Levels, Curves, Exposure, Vibrance,
and Black & White defaults, so `adjustment_layer` gains five match arms and the
panel menu gains five rows. Alternative considered: new encoders — unnecessary.

**7. Respect the file-size ceilings by construction.**
New app bridges live in new file-level bridge files registered in `build.rs`;
new C++ dialogs/panel logic live in new translation units listed in
`CMakeLists.txt`. `cxxqt_object.rs` stays within its allowlist ceiling, and
`filter_map.rs` shed its inline tests to `filter_map_tests.rs`, dropping it off
the allowlist entirely.

**8. Opacity-preserving filter kinds skip the unlocked-layer alpha pass.**
`filter_preserves_opacity` names `HdrToning`, `GlowingEdges`, `Lighting`, and
`Diffuse`. The unlocked-layer alpha pass assumes a kernel that maps a flat 255
alpha plane back to 255, which blur and spatial kernels do; these four depend on
absolute channel values, gradients, a darkening base, or pixel position, so
their alpha plane is left exactly as found and alpha is preserved end-to-end.
`ponytail:` a real fix folds alpha into the working buffer so every kernel
receives the fourth plane.

**9. Positional and holistic filters preview the whole layer.**
`filter_preview_needs_whole_layer` escalates `diffuse` (hashes absolute
coordinates), `lighting-effects` (resolves the light span against the crop), and
`hdr-toning` (global pivot); their preview ignores the visible rect so a cropped
section cannot diverge from the commit. HDR Toning's preview apron is
`3 * radius`, matching the Gaussian support.

## Risks / Trade-offs

- [Smart filters are camera-raw only] → every other enabled filter falls back
  to the imported proxy, so nothing is dropped; generic `Fltr` encoding is a
  later change once a real filter-id table exists.
- [Neighborhood HDR preview is more expensive than a pointwise one] → HDR
  Toning previews the whole layer because its global pivot makes a cropped
  section differ from the commit; other spatial filters stay viewport-bounded
  with an apron sized to their support.
- [Replace Color's localized preview uses the whole layer] → the visible-section
  crop would shift layer-local sample coordinates; acceptable for a preview.
- [Synthetic row paths bypass the path core] → dedicated row getters/setters and
  a `synthetic` flag guard rename, drag, and layer operations.
- [Header decoration of the `enab` group flag] → the group `enab` is modelled as
  `smart_filters_enabled`; the filter mask flags are modelled but unusable until
  mask I/O lands.

## Migration Plan

Additive only. The `filterFX` group flag is authored as enabled by default, so
existing documents are unaffected; rollback is a revert of the change. No
on-disk format break.

## Open Questions

_None._
