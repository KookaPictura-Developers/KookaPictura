#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the shared numeric-field checks:
//   lpn_scrub (303)        a label scrub changes the value, Shift scales up 10x
//                          and Ctrl down 10x, with one commit on release
//   lpn_popup_keys (304)   the popup slider steps on Left/Right and jumps on
//                          Home/End/PageUp/PageDown regardless of focus
//   lpn_jump_track (305)   a press-drag on the tracking slider follows the pointer
//   las_no_scratch (306)   a normal launch seeds no scratch document
//   lpn_brush_resync (307) the brush-size signal resyncs the options-bar field
// Returns 0 when all pass, otherwise the self-test failure code.
int runNumericFieldChecks(PicturaMainWindow& frame);
} // namespace pictura
