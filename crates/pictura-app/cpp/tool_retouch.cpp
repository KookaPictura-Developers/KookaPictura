// The retouch tools: Blur and Sharpen (`pictura_paint::focus`), Smudge
// (`pictura_paint::smudge`), and Dodge (`pictura_paint::tone`). Each works on
// the pixels already there, more with every pass. It begins a per-dab stroke
// (`cxxqt_object/paint_tools.rs`) and then shares the Brush's live stroke:
// `paint_dab` per move, `end_paint` on release (one history state). Ported
// from photorust's retouch stroke handling.

#include "tool_handler.h"

#include "paint_tip.h"

#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/paint_tools.cxxqt.h"

#include <QtCore/QObject>

#include <memory>

namespace pictura {

namespace {

class RetouchToolHandler : public ToolHandler {
public:
    explicit RetouchToolHandler(ToolId id)
        : id_(id)
    {
    }

    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        PictureView* v = ctx.view();
        if (!v || !v->has_document()) {
            return true;
        }
        if (!begin(ctx, *v)) {
            if (activePixelLocked(v)) {
                ctx.refused(QObject::tr("Could not retouch: the layer's pixels are locked."));
            } else if (!v->active_layer_visible()) {
                ctx.refused(QObject::tr("Could not retouch: the active layer is invisible."));
            } else if (v->active_layer_path().isEmpty()) {
                ctx.refused(QObject::tr("Could not retouch: select a single layer first."));
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
    bool begin(ToolContext& ctx, PictureView& v) const
    {
        if (id_ == ToolId::Dodge) {
            const ToneOptions o = ctx.toneOptions();
            return begin_dodge(v, paintTip(ctx), o.range, o.exposure, o.protectTones);
        }
        const RetouchOptions o = ctx.retouchOptions(id_);
        if (id_ == ToolId::Smudge) {
            return begin_smudge(v, ctx.foreground().rgba(), paintTip(ctx), o.strength, o.mode,
                                o.sampleAllLayers, o.fingerPainting);
        }
        return begin_focus(v, paintTip(ctx), id_ == ToolId::Sharpen, o.strength, o.mode,
                           o.sampleAllLayers, o.protectDetail);
    }

    ToolId id_;
};

} // namespace

std::unique_ptr<ToolHandler> makeRetouchToolHandler(ToolId id)
{
    return std::make_unique<RetouchToolHandler>(id);
}

} // namespace pictura
