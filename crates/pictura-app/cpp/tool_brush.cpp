#include "tool_handler.h"

#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QObject>
#include <QtGui/QColor>

#include <memory>

namespace pictura {

namespace {

// Brush and Pencil share this handler; `aliased_` selects the hard-edged
// Pencil rasterisation.
class BrushToolHandler : public ToolHandler {
public:
    explicit BrushToolHandler(bool aliased)
        : aliased_(aliased)
    {
    }

    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        PictureView* v = ctx.view();
        if (!v) {
            return true;
        }
        // Transient Alt eyedropper: sample the pointer without starting a
        // stroke, leaving the active tool unchanged.
        if (mods.testFlag(Qt::AltModifier)) {
            const quint32 argb = v->sample_argb(qRound(imagePos.x()), qRound(imagePos.y()));
            if (argb != 0) {
                ctx.sampledForeground(QColor::fromRgb(argb));
            }
            return true;
        }
        if (!v->begin_paint(ctx.foreground().rgba(), ctx.background().rgba(), ctx.brushSize(),
                            ctx.brushHardness(), 100, 0, ctx.brushOpacity(), ctx.brushFlow(), 25,
                            ctx.brushMode(), aliased_, ctx.autoErase())) {
            if (topmostPixelLocked(v)) {
                ctx.refused(QObject::tr("Could not paint: the layer's pixels are locked."));
            } else if (!v->active_layer_visible()) {
                ctx.refused(QObject::tr("Could not paint: the active layer is invisible."));
            } else if (v->active_layer_path().isEmpty()) {
                ctx.refused(QObject::tr("Could not paint: select a single layer first."));
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

private:
    bool aliased_;
};

}

std::unique_ptr<ToolHandler> makeBrushToolHandler(bool aliased)
{
    return std::make_unique<BrushToolHandler>(aliased);
}

} // namespace pictura
