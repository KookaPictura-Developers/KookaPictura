# Proposal: properties-panel

## Why

Issue #70 replaces the "No Properties" `PlaceholderPanel` with a real CS6
Properties panel. In CS6 the panel is where the active layer's adjustment
parameters are shown and edited. The engine and bridge cannot support that yet:
adjustments are opaque `AdjustmentData` with no parameter get/set accessor and no
edit session, so there is nothing to bind an editor to.

This change ships the read-only slice that the existing bridge can actually
back: identify the active adjustment layer and name it. It is deliberately a
scoped slice, not a fake full panel.

## What Changes

- `PropertiesPanel` (`panels/properties_panel.{h,cpp}`): a `QWidget` with
  `setView`/`refresh`. With no document, or an active layer that is not an
  adjustment, it shows "No Properties"; with an active adjustment layer it names
  that layer.
- The active row is found by scanning `layer_row_count()` for the row whose
  `layer_row_path` equals `active_layer_path()`.
- Replace the `PlaceholderPanel` at the Properties slot with `PropertiesPanel`,
  keeping `objectName` `propertiesPanel`.
- Refresh from the shell's `retargetDock()` so active-layer and selection changes
  reach the panel.
- Qt Test suite `tst_properties_panel`, added to `PICTURA_QT_TESTS`.

## Non-Goals

- **Editing adjustment parameters is deferred.** There is no
  adjustment-parameter get/set bridge and no edit session; adjustments are opaque
  `AdjustmentData`, so the panel is display-only. Add editing when a parameter
  bridge and an edit session exist.
- No adjustment controls, no layer-property editors, and no non-adjustment
  properties (blend, opacity) in this slice.

## Capabilities

### New Capabilities

- `ui/properties-panel`: the read-only Properties panel contract — active
  adjustment layer identified and named, empty state otherwise.

## Impact

- `pictura-app` C++ shell (`cpp/panels/properties_panel.*`,
  `cpp/frame*.{h,cpp}`, `cpp/tests/tst_properties_panel.cpp`, root
  `CMakeLists.txt`). No Rust, no bridge.
- No new dependency.
