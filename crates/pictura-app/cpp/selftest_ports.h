#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the checks for features ported from photorust (clipboard, Magnetic
// Lasso, click-to-deselect, the Crop tool group, Open Recent), in order;
// returns the first failing code, or 0. Split from selftest_layers_controls.cpp
// to keep it under its size cap.
int runPortChecks(PicturaMainWindow& frame);
} // namespace pictura
