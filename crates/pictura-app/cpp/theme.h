#pragma once

#include <QtCore/QString>

namespace pictura {

// Single source of truth for the application theme. Applies the Fusion style
// and a dark QPalette for one of four brightness levels (0 darkest .. 3
// lightest). No widget may hard-code a frame colour.
class Theme {
public:
    static constexpr int kLevelCount = 4;
    static constexpr int kDefaultLevel = 1;

    // M44: chosen chrome dimensions (unsourced CS6 metrics), shared so the
    // stylesheet and the splitter agree.
    static constexpr int kPanelBorderWidth = 1;
    static constexpr int kGroupDividerWidth = 4;

    // Clamp a level into [0, kLevelCount).
    static int clampLevel(int level);

    // CS6-style chrome QSS for `level`.
    static QString styleSheet(int level);

    // Apply the Fusion style and the palette for `level`. Call once after the
    // QApplication exists, and again whenever the level changes.
    static void apply(int level);
};

} // namespace pictura
