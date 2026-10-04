# Proposal

## Why

Issues #82 and #83 closed with parts of the CS6 Image ▸ Adjustments and Filter
port unfinished and untracked (Shadows/Highlights, Replace Color, destructive
Color Lookup, HDR Toning, Lighting Effects, Diffuse, Glowing Edges, and the
Artistic family), the New Adjustment Layer menu covered only 11 of 16 kinds
(#87), and Smart Filters (#166) existed only as PSD `filterFX` metadata with no
rendering or UI. This change closes those gaps so the menus match CS6 and
imported smart-filter data is rendered.

## What Changes

- **Image ▸ Adjustments.** Add Shadows/Highlights and Replace Color; make
  destructive Color Lookup live with the seven CS6 presets; add HDR Toning with
  its 17 Local Adaptation presets. Color Lookup and HDR Toning were engines
  without dialogs.
- **Filter menu.** Expose the 15 already-implemented Artistic filters under
  `Filter ▸ Artistic`, and add the Lighting Effects, Diffuse, and Glowing Edges
  kernels. `Flame`, `Tree`, and `Picture Frame` stay non-goals (post-CS6; see
  `docs/06-filters/render-filters.md`).
- **New Adjustment Layer.** Add the missing Levels, Curves, Exposure, Vibrance,
  and Black & White kinds (engine encoders already exist).
- **Smart Filters.** Composite an imported/authored `filterFX` chain over the
  embedded source; parse and author the group enable flag; show a `Smart
  Filters` parent row with one child per filter and visibility toggles in the
  Layers panel; add `Filter ▸ Convert for Smart Filters`. Generic
  filter/adjustment ids are not encoded as smart filters yet, and the filter
  mask is not modelled (see design non-goals).

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `imaging/image-adjustments`: add the Shadows/Highlights and Replace Color
  requirements; make the Color Lookup requirement destructive-dialog aware.
- `color/hdr-toning`: add the destructive HDR Toning adjustment and its preset
  table.
- `imaging/render-filters`: add Lighting Effects; record Flame/Tree/Picture
  Frame as explicit non-goals at the requirement level.
- `imaging/stylize-filters`: add Diffuse and Glowing Edges.
- `imaging/filter-app-ui`: expose the `Filter ▸ Artistic` submenu, enable the
  three new filter kernels, and add `Convert for Smart Filters`.
- `imaging/adjustment-layers`: add the Levels, Curves, Exposure, Vibrance, and
  Black & White layer kinds.
- `codec/psd-smart-filters`: parse/author the `filterFXStyle` group enable flag
  and attach a smart filter generically.
- `compositing/smart-object-rendering`: composite the ordered, enabled
  `filterFX` chain over the embedded source instead of the baked proxy.
- `ui/layers-panel`: render `Smart Filters` parent/child rows with per-filter
  and group visibility toggles.

## Impact

- `crates/pictura-adjust`: `ShadowsHighlights`, `ReplaceColor` variants; Color
  Lookup presets; HDR Toning moved to a filter-backed operator.
- `crates/pictura-filters`: `HdrToning`, `Lighting`, `Diffuse`, `GlowingEdges`
  kernels and `Filter` variants.
- `crates/pictura-render`: `filterFX` chain compositing, color-lookup preset
  rebuild, adjustment-dialog layouts/defaults, preview apron.
- `crates/pictura-codec`: `SmartObject` group flags, generic
  `attach_smart_filter`, `filterFXStyle` `enab` round-trip.
- `crates/pictura-core`: `SmartObject` group-enable/mask fields.
- `crates/pictura-app`: new bridges (`image_hdr_toning`, `image_replace_color`,
  `layers_smart_filters`), new dialogs (`hdr_toning_dialog`,
  `replace_color_dialog`), `frame_menus_adjust`/`command_tree` wiring, and the
  Layers-panel Smart Filters tree. New Qt Test suites; every new `.cpp`/`.h` is
  added to `CMakeLists.txt` explicitly.
- No new dependencies.

## Provenance

Ported from perfecto25/photorust (GPL-3.0-or-later, DCO certified; see issue
#1) with `Source:`/`Co-authored-by:` trailers. Behavioural parity where an
oracle exists; the raw/HDR/shadows approximations are marked as such.
