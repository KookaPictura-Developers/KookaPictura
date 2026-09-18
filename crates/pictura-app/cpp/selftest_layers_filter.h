#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the Layers-panel filter checks: lfs_name (201), lfs_kind (202),
// lfs_mode (203), lfs_color (204), lfs_none (205), lfs_ancestor (206),
// lfs_toggle (207), lfs_live (208), and lfs_reset (209). Returns 0 when all
// pass, otherwise the self-test failure code.
int runLayersFilterChecks(PicturaMainWindow& frame);
} // namespace pictura
