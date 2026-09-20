#include "tool_handler.h"

#include "image_view.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QDebug>
#include <QtCore/QElapsedTimer>
#include <QtGui/QImage>

#include <memory>

namespace pictura {

namespace {

// Move: whole-layer translate and, when a selection exists, the content move.
// It owns the warm-preview cache so the first press after activation skips the
// expensive region build.
class MoveToolHandler : public ToolHandler {
public:
    void onActivate(ToolContext& ctx) override { warm(ctx); }

    void onDeactivate(ToolContext&) override
    {
        warmView_ = nullptr;
        warmBase_ = QImage();
        warmLayer_ = QImage();
        warmValid_ = false;
    }

    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        PictureView* v = ctx.view();
        if (!v) {
            return true;
        }
        if (v->has_selection()) {
            ctx.beginContentMove(v, imagePos, mods.testFlag(Qt::AltModifier));
            return true;
        }
        QElapsedTimer pressClock;
        pressClock.start();
        const bool alt = mods.testFlag(Qt::AltModifier);
        const bool prepared = alt ? v->begin_move_duplicate() : v->begin_move_preview();
        const qint64 pressNs = pressClock.nsecsElapsed();
        if (qEnvironmentVariableIsSet("PICTURA_PRESS_TRACE") || pressNs > 8000000) {
            qWarning("[move-press] begin_move_preview hit=%d work=%.1fms",
                     (prepared && v->move_preview_cache_hit()) ? 1 : 0, pressNs / 1e6);
        }
        if (!prepared) {
            return true;
        }
        ctx.setDragging(true);
        ctx.setDragCommitted(false);
        last_ = imagePos;
        totalDelta_ = QPointF();
        if (ImageView* canvas = ctx.canvas()) {
            const bool reuse = warmValid_ && warmView_ == v && v->move_preview_cache_hit();
            if (!reuse) {
                warmBase_ = v->move_preview_base();
                warmLayer_ = v->move_preview_layer();
                warmView_ = v;
                warmValid_ = !warmBase_.isNull();
            }
            canvas->beginMovePreview(warmBase_, warmLayer_,
                                     QPointF(v->move_preview_x(), v->move_preview_y()),
                                     v->move_preview_opacity() / 255.0);
        }
        return true;
    }

    void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        if (!ctx.dragging()) {
            return;
        }
        totalDelta_ += imagePos - last_;
        last_ = imagePos;
        if (ImageView* canvas = ctx.canvas()) {
            canvas->setMovePreviewDelta(totalDelta_);
        }
    }

    void onRelease(ToolContext& ctx, const QPointF&, Qt::KeyboardModifiers) override
    {
        if (!ctx.dragging()) {
            return;
        }
        ctx.setDragging(false);
        if (PictureView* v = ctx.view()) {
            v->end_move_preview();
            const int dx = qRound(totalDelta_.x());
            const int dy = qRound(totalDelta_.y());
            if (dx != 0 || dy != 0) {
                v->commit_move(dx, dy);
            }
        }
        if (ImageView* canvas = ctx.canvas()) {
            canvas->endMovePreview();
        }
    }

private:
    void warm(ToolContext& ctx)
    {
        PictureView* v = ctx.view();
        if (!v) {
            warmValid_ = false;
            return;
        }
        if (!v->prepare_move_preview()) {
            warmValid_ = false;
            return;
        }
        warmBase_ = v->move_preview_base();
        warmLayer_ = v->move_preview_layer();
        warmView_ = v;
        warmValid_ = !warmBase_.isNull();
    }

    QPointF last_;
    QPointF totalDelta_;
    PictureView* warmView_ = nullptr;
    QImage warmBase_;
    QImage warmLayer_;
    bool warmValid_ = false;
};

} // namespace

std::unique_ptr<ToolHandler> makeMoveToolHandler()
{
    return std::make_unique<MoveToolHandler>();
}

} // namespace pictura
