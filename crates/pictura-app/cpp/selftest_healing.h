#pragma once

namespace pictura {
class PicturaMainWindow;

// healing_tools (536): the Spot Healing Brush rebuilds a blemish from its
// surroundings and records one "Spot Healing Brush" state; the Healing Brush
// refuses to paint before an Alt-click sample; the Count (Extended) tool adds
// numbered marks (one "New Count" state each) shown on the canvas overlay and
// Clear removes them ("Clear Counts").
int runHealingChecks(PicturaMainWindow& frame);

// patch_tool (537): the Patch cursor is an arrow with its hotspot at the tip;
// the Patch options bar disables Source/Destination under
// Content-Aware; a drag outside the selection outlines a region; dragging the
// outline onto clean pixels repairs the region in one "Patch Tool" state and
// leaves the sampled area alone; a click without a drag records nothing.
int runPatchChecks(PicturaMainWindow& frame);
} // namespace pictura
