#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the Batch-6 workspace input checks: the wheel modifier precedence
// (Shift step, Alt/Ctrl+Alt/side-wheel pan, cursor-anchored zoom), the
// cursor-anchored Zoom-tool click, the transient Alt eyedropper for the
// paint tools (sample with no history, cursor swap and restore), and the
// empty-workspace Open gesture gate. Returns 0 when all pass, otherwise the
// self-test failure code.
int runWorkspaceInputChecks(PicturaMainWindow& frame);
} // namespace pictura
