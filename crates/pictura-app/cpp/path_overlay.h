#pragma once

#include "image_view.h"

namespace pictura {

class PictureView;

// The Work Path as a canvas overlay: every subpath's curve, plus the anchors
// and handle lines of subpath `pointsOf`: every subpath's when it is -1, none
// when it names no subpath.
// Defined in tool_pen.cpp.
ImageView::PathOverlay workPathOverlay(const PictureView& v, int pointsOf);

} // namespace pictura
