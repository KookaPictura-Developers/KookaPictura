#include "tool_handler.h"

#include "image_view.h"

#include <memory>

namespace pictura {

namespace {

// The Hand tool pans through ImageView's pan-enabled mode; no pointer hook is
// needed here.
class HandToolHandler : public ToolHandler {
};

class ZoomToolHandler : public ToolHandler {
public:
    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        ImageView* canvas = ctx.canvas();
        if (!canvas) {
            return true;
        }
        // Anchor the step at the clicked image point, not the canvas centre.
        const QPointF anchor = imagePos * canvas->zoom() + canvas->offset();
        const bool out = mods.testFlag(Qt::ControlModifier) || mods.testFlag(Qt::AltModifier);
        canvas->setZoom(canvas->zoom() * (out ? 1.0 / 1.2 : 1.2), anchor);
        return true;
    }
};

} // namespace

std::unique_ptr<ToolHandler> makeHandToolHandler()
{
    return std::make_unique<HandToolHandler>();
}

std::unique_ptr<ToolHandler> makeZoomToolHandler()
{
    return std::make_unique<ZoomToolHandler>();
}

} // namespace pictura
