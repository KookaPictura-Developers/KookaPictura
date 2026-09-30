#pragma once

#include "tool_context.h"

#include "pictura_app/src/cxxqt_object/paint_tools.cxxqt.h"

namespace pictura {

// The controller's brush size, hardness, Brush Tip Shape, and dynamics as the
// paint-tool bridge's tip.
inline PaintTip paintTip(const ToolContext& ctx)
{
    const BrushDynamics d = ctx.brushDynamics();
    return PaintTip{ctx.brushSize(), ctx.brushHardness(), ctx.brushRoundness(), ctx.brushAngle(),
                    ctx.brushSpacing(), d.scatter, d.count, d.sizeJitter, d.angleJitter,
                    d.roundnessJitter};
}

} // namespace pictura
