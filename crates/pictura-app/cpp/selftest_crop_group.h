#pragma once

namespace pictura {
class PicturaMainWindow;

// crop_group (532): Perspective Crop stages a dragged quad, Enter warps the
// document to it (new size, one "Perspective Crop" state, the quad's content
// fills the canvas), a degenerate quad is refused and kept, and Escape discards
// a quad. The Slice tool shows the unsliced document as one auto slice, a drag
// adds one "Slice" state and a numbered user slice, a click adds none, Undo
// removes it from the overlay, and leaving the tool hides the overlay.
int runCropGroupChecks(PicturaMainWindow& frame);
} // namespace pictura
