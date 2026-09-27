#pragma once

namespace pictura {
class PicturaMainWindow;

// click_deselect (531): with a selection, a click outside it (no drag) with the
// Rectangular Marquee, the Lasso, or the Polygonal Lasso (a double-click on one
// spot) deselects as one "Deselect" state that Reselect undoes; a Shift-click
// (Add mode) leaves the selection and history alone.
int runClickDeselectChecks(PicturaMainWindow& frame);
} // namespace pictura
