#pragma once

namespace pictura {
class PicturaMainWindow;

// magnetic_lasso (530): on a white image with a black square, clicks placed
// two pixels inside the square's edges close into a selection whose border
// snapped onto the edges (edge columns/rows are selected that a straight
// outline through the same clicks would miss) as one history state; the live
// outline is drawn closed back to an origin marker; Delete
// peels fastening points back and then abandons the trace; Escape leaves the
// selection and history untouched; Frequency 100 fastens automatically as the
// wire lengthens where Frequency 0 does not; `]` / `[` step Width by 1 px and
// the options-bar Width field follows.
int runMagneticLassoChecks(PicturaMainWindow& frame);
} // namespace pictura
