#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the canvas-present checks: present_cache_edge (295) renders an opaque,
// canvas-sized document at fractional zooms where the floor-sized present cache
// falls a pixel short of the checkerboard's rounded document rect, and asserts
// the document's right/bottom edge is covered and the cached and direct present
// paths agree pixel for pixel.
// Returns 0 when all pass, otherwise the self-test failure code.
int runCanvasChecks(PicturaMainWindow& frame);
} // namespace pictura
