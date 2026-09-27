#pragma once

namespace pictura {
class PicturaMainWindow;

// healing_tools (536): the Spot Healing Brush rebuilds a blemish from its
// surroundings and records one "Spot Healing Brush" state; the Healing Brush
// refuses to paint before an Alt-click sample; the Count (Extended) tool adds
// numbered marks (one "New Count" state each) shown on the canvas overlay and
// Clear removes them ("Clear Counts").
int runHealingChecks(PicturaMainWindow& frame);
} // namespace pictura
