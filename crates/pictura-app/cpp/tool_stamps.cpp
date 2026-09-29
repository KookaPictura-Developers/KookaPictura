// The stamp tools: Clone Stamp, Pattern Stamp, and History Brush. Each begins
// a Brush stroke whose colour comes from a source (`cxxqt_object/paint_tools.rs`) and
// then shares the Brush's live stroke: `paint_dab` per move, `end_paint` on
// release (one history state). Ported from photorust's clone / pattern stroke
// handling; the History Brush has no photorust source.

#include "tool_handler.h"

#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/paint_tools.cxxqt.h"

#include <QtCore/QObject>
#include <QtCore/QPoint>
#include <QtGui/QColor>

#include <memory>

namespace pictura {

namespace {

// The press / drag / release shared by the three stamps; `begin` starts the
// tool's stroke and reports whether it did.
class StampToolHandler : public ToolHandler {
public:
    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        PictureView* v = ctx.view();
        if (!v || !v->has_document()) {
            return true;
        }
        if (mods.testFlag(Qt::AltModifier)) {
            onAltPress(ctx, imagePos);
            return true;
        }
        if (!begin(ctx, *v, imagePos)) {
            if (activePixelLocked(v)) {
                ctx.refused(QObject::tr("Could not paint: the layer's pixels are locked."));
            } else if (!v->active_layer_visible()) {
                ctx.refused(QObject::tr("Could not paint: the active layer is invisible."));
            } else if (v->active_layer_path().isEmpty()) {
                ctx.refused(QObject::tr("Could not paint: select a single layer first."));
            } else {
                refuseSource(ctx);
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

protected:
    virtual bool begin(ToolContext& ctx, PictureView& v, const QPointF& imagePos) = 0;
    // A refusal the layer checks do not explain, e.g. a missing source.
    virtual void refuseSource(ToolContext&) {}

    // Alt samples the foreground, as with the Brush.
    virtual void onAltPress(ToolContext& ctx, const QPointF& imagePos)
    {
        PictureView* v = ctx.view();
        const quint32 argb = v->sample_argb(qRound(imagePos.x()), qRound(imagePos.y()));
        if (argb != 0) {
            ctx.sampledForeground(QColor::fromRgb(argb));
        }
    }
};

// Clone Stamp: Alt-click sets the source point; each stroke paints what lies at
// the source offset. Aligned keeps the offset across strokes; unchecked, every
// stroke copies from the source point again.
// ponytail: no Clone Source panel (one source, no scale / rotate / overlay) and
// no on-canvas source crosshair.
class CloneStampToolHandler : public StampToolHandler {
protected:
    void onAltPress(ToolContext&, const QPointF& imagePos) override
    {
        source_ = QPoint(qRound(imagePos.x()), qRound(imagePos.y()));
        hasSource_ = true;
        hasOffset_ = false;
    }

    bool begin(ToolContext& ctx, PictureView& v, const QPointF& imagePos) override
    {
        if (!hasSource_) {
            return false;
        }
        const StampOptions o = ctx.stampOptions();
        if (!o.cloneAligned || !hasOffset_) {
            offset_ = source_ - QPoint(qRound(imagePos.x()), qRound(imagePos.y()));
            hasOffset_ = true;
        }
        return begin_clone_stamp(v, ctx.brushSize(), ctx.brushHardness(), ctx.brushOpacity(),
                                 ctx.brushFlow(), ctx.brushMode(), offset_.x(), offset_.y(),
                                 o.cloneSample, o.ignoreAdjustments);
    }

    void refuseSource(ToolContext& ctx) override
    {
        ctx.refused(hasSource_
                        ? QObject::tr("Clone Stamp: paint away from the source point.")
                        : QObject::tr("Clone Stamp: Alt-click to set a source point first."));
    }

private:
    QPoint source_;
    QPoint offset_;
    bool hasSource_ = false;
    bool hasOffset_ = false;
};

// Pattern Stamp: paints the chosen pattern, pinned to the document when
// Aligned and to each stroke's start otherwise.
// ponytail: built-in generated patterns only; Impressionist is not wired.
class PatternStampToolHandler : public StampToolHandler {
protected:
    bool begin(ToolContext& ctx, PictureView& v, const QPointF& imagePos) override
    {
        const StampOptions o = ctx.stampOptions();
        const QPoint origin =
            o.patternAligned ? QPoint() : QPoint(qRound(imagePos.x()), qRound(imagePos.y()));
        return begin_pattern_stamp(v, ctx.brushSize(), ctx.brushHardness(), ctx.brushOpacity(),
                                   ctx.brushFlow(), ctx.brushMode(), o.pattern, origin.x(),
                                   origin.y());
    }
};

// History Brush: paints the active layer as it was in the History panel's
// source state (the oldest state unless another is chosen), in place.
class HistoryBrushToolHandler : public StampToolHandler {
protected:
    bool begin(ToolContext& ctx, PictureView& v, const QPointF&) override
    {
        return begin_history_brush(v, ctx.brushSize(), ctx.brushHardness(), ctx.brushOpacity(),
                                   ctx.brushFlow(), ctx.brushMode());
    }

    void refuseSource(ToolContext& ctx) override
    {
        ctx.refused(QObject::tr("History Brush: the source state has no matching pixel layer."));
    }
};

} // namespace

std::unique_ptr<ToolHandler> makeCloneStampToolHandler()
{
    return std::make_unique<CloneStampToolHandler>();
}

std::unique_ptr<ToolHandler> makePatternStampToolHandler()
{
    return std::make_unique<PatternStampToolHandler>();
}

std::unique_ptr<ToolHandler> makeHistoryBrushToolHandler()
{
    return std::make_unique<HistoryBrushToolHandler>();
}

} // namespace pictura
