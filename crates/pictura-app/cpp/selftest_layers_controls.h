#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the Layers-panel control checks: lpc_percent (199) covers the percent
// Opacity field, its five lock toggles, and history-free sync; lpc_nesting (200)
// covers the nesting lock bit, Group Layers refusal, and in-container reorder;
// lpc_preview (214) covers preview-without-history and one commit per edit;
// lpr_percent (215), the inside-box `%`; lpc_lockbadge (216), the right-side lock
// badge; lpr_eye (217), the row's lack of a check state.
// Returns 0 when all pass, otherwise the self-test failure code.
int runLayersControlsChecks(PicturaMainWindow& frame);
} // namespace pictura
