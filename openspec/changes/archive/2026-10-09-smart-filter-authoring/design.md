# Design

## Context

- `SmartObject.smart_filters: Vec<SmartFilter>` is the typed view; the preserved
  `SoLd`/`SoLE` `filterFX` bytes remain the source of truth for save, exactly as
  the existing enable-toggle path treats them.
- `pictura_codec::attach_smart_filter` already inserts/replaces one entry and
  rewrites the preserved descriptor, and `pictura_render::apply_pictura_raw`
  already routes the one modelled filter (Camera Raw, `filterID` 2683) through
  that attach.
- `SmartObject.filter_mask: Option<LayerMask>` exists but is never decoded;
  `apply_smart_filter_chain` already applies `Some` mask pixels.

## Approach

### Codec edits over the preserved descriptor

`delete_smart_filter`, `reorder_smart_filters`, and `clear_smart_filters` reuse
the module's existing parse → mutate → `serialize_config` → `store_config`
pipeline (the same one `set_smart_filter_enabled` uses). They operate on
`filterFX.filterFXList` through the existing `filter_fx_mut`/`get_object_item_mut`
accessors, drop the whole `filterFX` object when the list empties, and then sync
`SmartObject.smart_filters` so the typed view matches. A layer with no preserved
block edits only the typed view; the writer authors it on save. Out-of-range
indices return `PsdError::Invalid` before any mutation.

Reordering moves the entry in both the descriptor list and the typed list, so
the render order (`Vec` order, bottom-up) follows the panel order.

### Engine operations

The new `document_ops/smart_filters.rs` resolves the path, requires a
non-group, non-adjustment layer with a smart object, and delegates to the codec.
Adding builds a `SmartFilter { enabled: true }` and calls the existing
`attach_smart_filter`. `attach_smart_filter` replaces a same-`filterID` entry
rather than stacking a duplicate; that ceiling is marked in the module doc and
is acceptable because the render chain models one Camera Raw filter.

### App command

`clear_layer_smart_filters` mirrors the existing toggle bridge: it mutates the
document, recomposites, and records one "Clear Smart Filters" state. The
`Layer > Smart Filter > Clear Smart Filters` command is enabled only when
`layer_smart_filter_count(currentPath) > 0`. The `currentPath` may be a synthetic
`"0/@sf/2"` row; the bridge's `base_path` strips the suffix.

## Deferred

### Filter-mask pixels (item 1)

The filter-mask pixels are not decodable from the current corpus. Neither
`assets/test_with_smart_object02.psd` nor any other fixture carries `FEid`,
`FXid`, or `FMsk` (confirmed with a raw byte search and with `psd-tools`
1.19), and the repo docs list the `FXid`/`FEid` parameter mapping and the
filter-mask density/feather serialization as open questions
(`docs/05-layers/smart-filters.md`, `docs/dev/psd-support-roadmap.md` G17).
`psd-tools` does not model a filter mask at all. Implementing a decoder now
would be an unsourced guess, so the preserved bytes stay authoritative and
`filter_mask` stays `None`. The container is prepared: `SmartObject.filter_mask`
and `apply_mask` already exist, so a future change only needs the decoder plus
a reference fixture. Consequently the `Disable Filter Mask` / `Delete Filter
Mask` menu leaves remain registered stubs.

### Routing Filter-menu filters into the stack (item 3)

Every filter except Camera Raw (`filterID` 2683) is unknown to
`decode_smart_filter`, so appending one would disable
`composite_smart_filtered_source` (it requires every enabled filter to decode)
and fall back to the proxy — the user would see no effect. Routing arbitrary
filters needs a filter registry (id ↔ renderable op ↔ options codec) that does
not exist; the existing `apply_pictura_raw` path is the one working instance of
the desired pattern. This is deferred rather than faked.

### Panel drag-reorder / thumbnail / blend-options dialog

The `Smart Filters` row children are currently non-draggable, and the panel's
row-menu surface only handles the layer row. Wiring drag-reorder to the new
engine op, a mask thumbnail, and a per-filter Blend Options dialog is UI work
that depends on the deferred mask and blend-options model; it is out of scope
for this change.
