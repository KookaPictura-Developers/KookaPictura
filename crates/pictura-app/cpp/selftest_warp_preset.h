#pragma once

namespace pictura {
class PicturaMainWindow;

// lpr_warp_preset (527): a raster layer warped by a named preset (`Arc`, 50%)
// commits exactly one "Warp" history state and a changed layer rect; `None` and
// an unknown style refuse without adding a state.
int runWarpPresetChecks(PicturaMainWindow& frame);
} // namespace pictura
