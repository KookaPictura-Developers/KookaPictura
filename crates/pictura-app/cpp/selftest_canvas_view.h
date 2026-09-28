#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the canvas-view checks added for the pan clamp, workspace scrollbars,
// and brush-size outline: canvas_pan_margin (308) pans hard in each direction
// and asserts the reveal margin still intersects the viewport without jumping
// to the far side; canvas_scrollbar_roundtrip (309) drives a bar value and
// reads it back through the canvas offset; canvas_scrollbar_visible (310)
// asserts both bars stay visible when the document fits and the canvas still
// pans; canvas_brush_outline_size (311)
// and canvas_brush_outline_zoom (312) measure the drawn ring against the brush
// diameter and the zoom; present_filter_boundary (539) pins the sampling filter
// and present-level formula; navigator_pyramid_level (540) pins the navigator
// thumbnail source; canvas_present_contract (541) pins level-crop reuse across a
// pan, level-0 crop identity against a full-resolution draw, and no `changed` on
// a region refresh. Returns 0 when all pass, else the failure code.
int runCanvasViewChecks(PicturaMainWindow& frame);
} // namespace pictura
