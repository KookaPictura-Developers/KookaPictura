#pragma once

namespace pictura {

class PicturaMainWindow;

// Round-4 shell checks: a whole widget `PanelColumn` tears off into an in-window
// `PanelFloat` overlay that keeps the shared minimum width and a resize grip,
// and re-docks into the splitter on a valid release. Returns 0 when all pass,
// otherwise the failure code. Invoked from runLayersControlsChecks so the token
// order is unchanged.
int runShellRound4Checks(PicturaMainWindow& frame);

} // namespace pictura
