// The stamp tools: Clone Stamp, Pattern Stamp, and History Brush. Each begins
// a Brush stroke whose colour comes from a source (`cxxqt_object/paint_tools.rs`) and
// then shares the Brush's live stroke: `paint_dab` per move, `end_paint` on
// release (one history state). Ported from photorust's clone / pattern stroke
// handling; the History Brush has no photorust source.

#include "tool_handler.h"

#include "paint_tip.h"

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

// Clone Stamp: Alt-click sets the active Clone Source slot's source point;
// each stroke paints what lies at the source offset, through the slot's
// transform about the point the offset was measured at. Aligned keeps the
// offset across strokes; unchecked, every stroke copies from the source point
// again.
// ponytail: no source overlay and no on-canvas source crosshair.
class CloneStampToolHandler : public StampToolHandler {
protected:
    void onAltPress(ToolContext& ctx, const QPointF& imagePos) override
    {
        CloneSource slot = ctx.cloneSource();
        slot.source = QPoint(qRound(imagePos.x()), qRound(imagePos.y()));
        slot.hasSource = true;
        slot.hasOffset = false;
        ctx.setCloneSource(slot);
    }

    bool begin(ToolContext& ctx, PictureView& v, const QPointF& imagePos) override
    {
        CloneSource slot = ctx.cloneSource();
        if (!slot.hasSource) {
            return false;
        }
        const StampOptions o = ctx.stampOptions();
        if (!o.cloneAligned || !slot.hasOffset) {
            slot.anchor = QPoint(qRound(imagePos.x()), qRound(imagePos.y()));
            slot.offset = slot.source - slot.anchor;
            slot.hasOffset = true;
            ctx.setCloneSource(slot);
        }
        const double sx = slot.width / 100.0 * (slot.flipH ? -1.0 : 1.0);
        const double sy = slot.height / 100.0 * (slot.flipV ? -1.0 : 1.0);
        return begin_clone_stamp(v, paintTip(ctx), ctx.brushOpacity(), ctx.brushFlow(),
                                 ctx.brushMode(), slot.offset.x(), slot.offset.y(),
                                 slot.anchor.x() + 0.5, slot.anchor.y() + 0.5, sx, sy,
                                 slot.angle, o.cloneSample, o.ignoreAdjustments);
    }

    void refuseSource(ToolContext& ctx) override
    {
        ctx.refused(ctx.cloneSource().hasSource
                        ? QObject::tr("Clone Stamp: paint away from the source point.")
                        : QObject::tr("Clone Stamp: Alt-click to set a source point first."));
    }
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
        return begin_pattern_stamp(v, paintTip(ctx), ctx.brushOpacity(), ctx.brushFlow(),
                                   ctx.brushMode(), o.pattern, origin.x(), origin.y());
    }
};

// History Brush: paints the active layer as it was in the History panel's
// source state (the oldest state unless another is chosen), in place.
class HistoryBrushToolHandler : public StampToolHandler {
protected:
    bool begin(ToolContext& ctx, PictureView& v, const QPointF&) override
    {
        return begin_history_brush(v, paintTip(ctx), ctx.brushOpacity(), ctx.brushFlow(),
                                   ctx.brushMode());
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
