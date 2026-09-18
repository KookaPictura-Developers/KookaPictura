#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the Layers-panel control checks: lpc_percent (199) covers the percent
// Opacity field, its five lock toggles, and history-free sync; lpc_nesting (200)
// covers the nesting lock bit, Group Layers refusal, and in-container reorder.
// Returns 0 when both pass, otherwise the self-test failure code.
int runLayersControlsChecks(PicturaMainWindow& frame);
} // namespace pictura
