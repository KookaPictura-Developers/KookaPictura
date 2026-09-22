#pragma once

namespace pictura {

// M44 C3: chosen compact-strip grip height and the "very close to a group"
// proximity band, not sourced CS6 metrics. The band is zero so the strip's own
// margins remain new-group boundaries rather than swallowing the first/last
// group.
constexpr int kCompactGripHeight = 10;
constexpr int kCompactNearBand = 0;

constexpr int kIconStripMinWidth = 40;

// A thin band at the very top and bottom of a normal-mode column's viewport that
// resolves an insert at the column's first/last boundary. Without it the first
// group's tab bar is the topmost surface and the last group stretches to the
// bottom edge, so the column ends were unreachable while the gaps between groups
// worked. ponytail: chosen band, not a sourced CS6 metric.
constexpr int kColumnEdgeBand = 10;

// Shared minimum width floor; defined by panel_column.cpp, updated by
// `updateMinimumWidth` and read by the test hooks.
extern int gSharedFloor;

} // namespace pictura
