#pragma once

namespace pictura {

class PicturaMainWindow;

// float_child_overlay (440): the platform-adaptive floating-overlay hosting
// check, kept in its own unit so `selftest_shell_round4.cpp` stays inside the
// file-size cap. Returns 0 when it passes, otherwise the failure code.
int runShellRound4FloatCheck(PicturaMainWindow& frame);

} // namespace pictura
