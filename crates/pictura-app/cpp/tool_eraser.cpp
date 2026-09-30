// The Eraser: a Brush stroke that erases the active layer to transparency, or
// to the background colour on the Background and transparency-locked layers
// (`pictura_paint::eraser`). Erase To History, or Alt held at the press, paints
// the History Brush's source state back instead. Ported from photorust's erase
// mode.

#include "tool_handler.h"

#include "paint_tip.h"

#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/paint_tools.cxxqt.h"

#include <QtCore/QObject>

#include <memory>

namespace pictura {

namespace {

class EraserToolHandler : public ToolHandler {
public:
    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        PictureView* v = ctx.view();
        if (!v || !v->has_document()) {
            return true;
        }
        const EraserOptions o = ctx.eraserOptions();
        const bool toHistory = o.toHistory || mods.testFlag(Qt::AltModifier);
        if (!begin_eraser(*v, ctx.background().rgba(), paintTip(ctx), o.mode,
                          ctx.brushOpacity(), ctx.brushFlow(), toHistory)) {
            if (activePixelLocked(v)) {
                ctx.refused(QObject::tr("Could not erase: the layer's pixels are locked."));
            } else if (!v->active_layer_visible()) {
                ctx.refused(QObject::tr("Could not erase: the active layer is invisible."));
            } else if (v->active_layer_path().isEmpty()) {
                ctx.refused(QObject::tr("Could not erase: select a single layer first."));
            } else if (toHistory) {
                ctx.refused(
                    QObject::tr("Erase to History: the source state has no matching pixel layer."));
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

std::unique_ptr<ToolHandler> makeEraserToolHandler()
{
    return std::make_unique<EraserToolHandler>();
}

} // namespace pictura
