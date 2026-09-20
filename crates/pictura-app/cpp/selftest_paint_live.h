#pragma once

namespace pictura {

class PicturaMainWindow;

// Live brush-stroke regression checks: a dab must be visible on the canvas
// before release at a zoom below and above 100 % (the pre-fix present-cache
// patch never showed it), and a multi-dab stroke must record exactly one
// history state that an undo reverses. Returns 0 when all pass, otherwise the
// self-test failure code. Invoked from runLayersControlsChecks so the token
// order is unchanged.
int runPaintLiveChecks(PicturaMainWindow& frame);

} // namespace pictura
