#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the canvas-view checks added for the pan clamp, workspace scrollbars,
// and brush-size outline: canvas_pan_margin (308) pans hard in each direction
// and asserts the reveal margin still intersects the viewport without jumping
// to the far side; canvas_scrollbar_roundtrip (309) drives a bar value and
// reads it back through the canvas offset; canvas_scrollbar_hidden (310)
// asserts the bars hide when the document fits; canvas_brush_outline_size (311)
// and canvas_brush_outline_zoom (312) measure the drawn ring against the brush
// diameter and the zoom. Returns 0 when all pass, else the failure code.
int runCanvasViewChecks(PicturaMainWindow& frame);
} // namespace pictura
