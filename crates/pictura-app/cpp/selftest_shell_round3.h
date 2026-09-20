#pragma once

namespace pictura {

class PicturaMainWindow;

// Round-3 shell checks: the empty workspace hides its document tab pane, the
// document tab bar carries the scoped medium-weight/padded rule, and the
// `[`/`]` brush shortcut helper maps US keys, evdev scan codes, and the
// non-paint guard. Returns 0 when all pass, otherwise the failure code.
// Invoked from runLayersControlsChecks so the token order is unchanged.
int runShellRound3Checks(PicturaMainWindow& frame);

} // namespace pictura
