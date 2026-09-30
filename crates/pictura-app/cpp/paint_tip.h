#pragma once

#include "tool_context.h"

#include "pictura_app/src/cxxqt_object/paint_tools.cxxqt.h"

namespace pictura {

// The controller's brush size, hardness, and Brush Tip Shape as the paint-tool
// bridge's tip.
inline PaintTip paintTip(const ToolContext& ctx)
{
    return PaintTip{ctx.brushSize(), ctx.brushHardness(), ctx.brushRoundness(), ctx.brushAngle(),
                    ctx.brushSpacing()};
}

} // namespace pictura
