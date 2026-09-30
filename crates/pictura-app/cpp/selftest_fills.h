#pragma once

namespace pictura {
class PicturaMainWindow;

// gradient_tool (552): G selects the Gradient and Shift+G the Paint Bucket;
// the bar defaults to the first gradient, Linear, Normal, 100 %, Transparency
// on; a drag shows its axis and on release draws black to white along it in
// one "Gradient" state that undo takes back; a click draws nothing; Reverse
// swaps the ends; a Radial gradient stays inside the selection; the preset
// menu picks the gradient.
int runGradientToolChecks(PicturaMainWindow& frame);

// paint_bucket_tool (553): the bar defaults to Foreground, Tolerance 32,
// Anti-alias and Contiguous on, the pattern picker greyed; a click fills the
// connected white in one "Paint Bucket" state and keeps the red squares; All
// Layers reads the composite and fills the active layer; with Contiguous off
// both squares fill; Pattern fills with the tile; locked pixels refuse.
int runPaintBucketChecks(PicturaMainWindow& frame);
} // namespace pictura
