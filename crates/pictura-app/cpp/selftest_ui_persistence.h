#pragma once

namespace pictura {
class PicturaMainWindow;

// Batch 1 regression checks for the app-UI interaction fixes:
//   lpr_column_railmode  (324) each column's own rail mode round-trips, and a
//                              v7 store with none seeds from the legacy
//                              top-level `panelRailMode`
//   lpn_paint_percent    (325) Hardness/Opacity/Flow render a `%` suffix
//   lpn_feather_no_popup (326) the Feather control is built without a popup
//   lpn_px_tight         (327) a `px` suffix renders immediately after the value
// Returns 0 when all pass, otherwise the self-test failure code.
int runUiPersistenceChecks(PicturaMainWindow& frame);
} // namespace pictura
