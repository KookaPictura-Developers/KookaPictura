#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the layer-lock enforcement checks: llk_move_refusal (313) covers the
// Move tool over a position-locked layer; llk_filter_refusal (314) covers a
// filter over a pixel-locked layer; llk_locked_cursor (315) covers the
// forbidden cursor for a paint tool; llk_paint_refusal (316) covers a refused
// brush stroke writing nothing. Returns 0 when all pass, otherwise the
// self-test failure code.
int runLayerLocksChecks(PicturaMainWindow& frame);
} // namespace pictura
