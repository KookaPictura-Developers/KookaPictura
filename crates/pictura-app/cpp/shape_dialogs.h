#pragma once

#include <array>

class QWidget;

namespace pictura {

enum class ToolId;

// "This operation will turn a live shape into a regular path. Continue?" with
// Yes / No and "Don't show again" (persisted in the session store; once set,
// the answer is always Yes). True to go ahead; `asked` says whether the
// prompt was shown.
bool confirmLiveShapeToPath(QWidget* parent, bool* asked = nullptr);

// What a shape tool's Create dialog asks for: the size (px), From Center, the
// Rounded Rectangle's corner radii (top-left, top-right, bottom-right,
// bottom-left), and the Polygon's sides, Smooth Corners, Star, Indent Sides By
// (%), and Smooth Indents.
struct CreateShapeValues {
    double width = 100.0;
    double height = 100.0;
    bool fromCenter = false;
    std::array<double, 4> radii{10.0, 10.0, 10.0, 10.0};
    int sides = 5;
    bool smoothCorners = false;
    bool star = false;
    double indent = 50.0;
    bool smoothIndents = false;
};

// The "Create Rectangle" / "Create Rounded Rectangle" / "Create Ellipse" /
// "Create Polygon" dialog a click opens; edits `values` and returns true on OK.
bool execCreateShapeDialog(ToolId id, CreateShapeValues& values, QWidget* parent);

} // namespace pictura
