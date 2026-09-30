#pragma once

namespace pictura {
class PicturaMainWindow;

// blur_tool (554): the Blur bar defaults to Normal, Strength 50 %, Sample All
// Layers off, and Blur shares the brush size ring; a drag along a black/white
// edge softens both sides in one "Blur" state and leaves the far pixels; on an
// empty layer a drag changes nothing (no state) unless Sample All Layers reads
// the composite.
int runBlurToolChecks(PicturaMainWindow& frame);
} // namespace pictura
