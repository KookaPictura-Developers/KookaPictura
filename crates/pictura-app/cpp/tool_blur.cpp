// The Blur tool (`pictura_paint::focus`): a brush that softens the active
// layer where it passes, more with every pass. It begins a per-dab stroke
// (`cxxqt_object/paint_tools.rs`) and then shares the Brush's live stroke:
// `paint_dab` per move, `end_paint` on release (one "Blur" state). Ported from
// photorust's retouch stroke handling.

#include "tool_handler.h"

#include "paint_tip.h"

#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/paint_tools.cxxqt.h"

#include <QtCore/QObject>

#include <memory>

namespace pictura {

namespace {

class BlurToolHandler : public ToolHandler {
public:
    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        PictureView* v = ctx.view();
        if (!v || !v->has_document()) {
            return true;
        }
        const BlurOptions o = ctx.blurOptions();
        if (!begin_blur(*v, paintTip(ctx), o.strength, o.mode, o.sampleAllLayers)) {
            if (activePixelLocked(v)) {
                ctx.refused(QObject::tr("Could not blur: the layer's pixels are locked."));
            } else if (!v->active_layer_visible()) {
                ctx.refused(QObject::tr("Could not blur: the active layer is invisible."));
            } else if (v->active_layer_path().isEmpty()) {
                ctx.refused(QObject::tr("Could not blur: select a single layer first."));
            }
            return true;
        }
        ctx.setDragging(true);
        ctx.setDragCommitted(false);
        v->paint_dab(imagePos.x(), imagePos.y(), 1.0);
        return true;
    }

    void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        if (!ctx.dragging()) {
            return;
        }
        if (PictureView* v = ctx.view()) {
            v->paint_dab(imagePos.x(), imagePos.y(), 1.0);
        }
    }

    void onRelease(ToolContext& ctx, const QPointF&, Qt::KeyboardModifiers) override
    {
        if (!ctx.dragging()) {
            return;
        }
        ctx.setDragging(false);
        if (PictureView* v = ctx.view()) {
            v->end_paint();
        }
    }
};

} // namespace

std::unique_ptr<ToolHandler> makeBlurToolHandler()
{
    return std::make_unique<BlurToolHandler>();
}

} // namespace pictura
