#pragma once

namespace pictura {

// M44 C3: chosen compact-strip grip height and the "very close to a group"
// proximity band, not sourced CS6 metrics. The band is zero so the strip's own
// margins remain new-group boundaries rather than swallowing the first/last
// group.
constexpr int kCompactGripHeight = 10;
constexpr int kCompactNearBand = 0;

constexpr int kIconStripMinWidth = 40;

// Shared minimum width floor; defined by panel_column.cpp, updated by
// `updateMinimumWidth` and read by the test hooks.
extern int gSharedFloor;

} // namespace pictura
