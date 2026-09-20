#pragma once

#include "tool_context.h"

#include <QtCore/QPointF>
#include <QtCore/Qt>

namespace pictura {

// One tool's pointer behavior. Returning true from `onPress` consumes the
// event; the other hooks are fire-and-forget. The default no-ops let a handler
// override only what it needs.
class ToolHandler {
public:
    virtual ~ToolHandler() = default;
    virtual bool onPress(ToolContext& ctx, const QPointF& imagePos,
                         Qt::KeyboardModifiers mods)
    {
        return false;
    }
    virtual void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) {}
    virtual void onRelease(ToolContext& ctx, const QPointF& imagePos,
                           Qt::KeyboardModifiers mods)
    {
    }
};

} // namespace pictura
