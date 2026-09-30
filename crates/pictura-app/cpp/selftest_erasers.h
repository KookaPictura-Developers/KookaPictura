#pragma once

namespace pictura {
class PicturaMainWindow;

// eraser_tool (547): the Eraser bar's Mode defaults to Brush with Erase to
// History off; on the Background a stroke paints the background colour in one
// "Eraser" state; Alt-drag paints the oldest state back; on an ordinary layer
// it erases to transparency; Block greys Opacity.
int runEraserChecks(PicturaMainWindow& frame);

// background_eraser_tool (550): the bar defaults to Continuous / Contiguous /
// 50 %; a stroke with the crosshair on the sky beside a subject turns the
// Background into a layer and erases the sky to transparency in one
// "Background Eraser" state, keeping the subject.
int runBackgroundEraserChecks(PicturaMainWindow& frame);

// magic_eraser_tool (551): the bar defaults to Tolerance 32 with Anti-alias
// and Contiguous on; a click on a red square erases it to transparency in one
// "Magic Eraser" state and keeps a separate red square; with Contiguous off a
// click on white erases all the white.
int runMagicEraserChecks(PicturaMainWindow& frame);
} // namespace pictura
