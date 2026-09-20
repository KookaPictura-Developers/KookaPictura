#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the round-3 Layers-panel checks: the rename content band, the top-level
// drop indicator and drag cursor, the Background conversion dialog
// (accept/cancel), the eye-only color tint, the active-highlight clip, the
// thumbnail checkerboard/outline/brackets, row typography roles, the row-height
// floor, and the hidden nesting-lock button. Returns 0 when all pass, otherwise
// the self-test failure code.
int runLayersRound3Checks(PicturaMainWindow& frame);
} // namespace pictura
