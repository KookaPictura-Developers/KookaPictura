// The shape tools (`pictura_core::shape`, bridged by `cxxqt_object/shapes.rs`):
// Rectangle, Rounded Rectangle, Ellipse, and Polygon. A drag previews the
// outline on the canvas and lands on release in the options bar's Mode: a new
// shape layer (Shape), a Work Path component (Path), or foreground pixels on
// the active layer (Pixels). Shift squares the box off (the Polygon snaps its
// turn to 15°); Alt grows it from the press point. The modifiers are read live,
// so pressing one mid-drag changes the preview. Ported from photorust's
// CanvasView shape drag (shapeOutlineFor / paintShapeOverlay / drawShape).
//
// ponytail: a click does not open CS6's "Create Rectangle" size dialog; no
// Stroke, gradient / pattern Fill, geometry pop-up (Fixed Size, Proportional,
// Star), path operations, or Align Edges.

#include "tool_handler.h"

#include "image_view.h"
#include "path_overlay.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/shapes.cxxqt.h"

#include <QtCore/QLineF>
#include <QtCore/QObject>
#include <QtGui/QPainterPath>

#include <array>
#include <memory>

namespace pictura {

namespace {

int shapeKind(ToolId id)
{
    switch (id) {
    case ToolId::RoundedRectangle:
        return 1;
    case ToolId::Ellipse:
        return 2;
    case ToolId::Polygon:
        return 3;
    default:
        return 0;
    }
}

class ShapeToolHandler : public ToolHandler {
public:
    explicit ShapeToolHandler(ToolId id)
        : kind_(shapeKind(id))
    {
    }

    void onActivate(ToolContext& ctx) override { refreshOverlay(ctx, {}); }

    void onDeactivate(ToolContext& ctx) override
    {
        ctx.setDragging(false);
        if (ImageView* canvas = ctx.canvas()) {
            canvas->clearPathOverlay();
        }
    }

    void onDocumentRefreshed(ToolContext& ctx) override
    {
        if (!ctx.dragging()) {
            refreshOverlay(ctx, {});
        }
    }

    void onOptionsChanged(ToolContext& ctx) override
    {
        if (!ctx.dragging()) {
            refreshOverlay(ctx, {});
        }
    }

    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        PictureView* v = ctx.view();
        if (!v || !v->has_document()) {
            return true;
        }
        press_ = imagePos;
        ctx.setDragging(true);
        return true;
    }

    void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        if (ctx.dragging()) {
            refreshOverlay(ctx, preview(ctx, imagePos, mods));
        }
    }

    void onRelease(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        if (!ctx.dragging()) {
            return;
        }
        ctx.setDragging(false);
        PictureView* v = ctx.view();
        if (v && v->has_document()) {
            land(ctx, *v, imagePos, mods);
        }
        refreshOverlay(ctx, {});
    }

private:
    // The bridge's shared arguments: the drag and the modifiers.
    struct Drag {
        std::array<double, 4> points;
        bool shift;
        bool alt;

        ::rust::Slice<const double> slice() const { return {points.data(), points.size()}; }
    };

    Drag drag(const QPointF& end, Qt::KeyboardModifiers mods) const
    {
        return {{press_.x(), press_.y(), end.x(), end.y()},
                mods.testFlag(Qt::ShiftModifier),
                mods.testFlag(Qt::AltModifier)};
    }

    QPainterPath preview(ToolContext& ctx, const QPointF& end, Qt::KeyboardModifiers mods) const
    {
        const ShapeOptions o = ctx.shapeOptions();
        const Drag d = drag(end, mods);
        const ::rust::Vec<double> k =
            shape_outline(kind_, d.slice(), d.shift, d.alt, o.radius, o.sides);
        QPainterPath path;
        const int n = int(k.size() / 6);
        if (n < 2) {
            return path;
        }
        path.moveTo(k[0], k[1]);
        for (int i = 0; i < n; ++i) {
            const int a = i * 6;
            const int b = ((i + 1) % n) * 6;
            path.cubicTo(QPointF(k[a + 4], k[a + 5]), QPointF(k[b + 2], k[b + 3]),
                         QPointF(k[b], k[b + 1]));
        }
        path.closeSubpath();
        return path;
    }

    void land(ToolContext& ctx, PictureView& v, const QPointF& end, Qt::KeyboardModifiers mods)
    {
        // A click, or a drag under two screen pixels, draws nothing.
        const double zoom = ctx.canvas() ? qMax(ctx.canvas()->zoom(), 1e-6) : 1.0;
        if (QLineF(press_, end).length() * zoom < 2.0) {
            return;
        }
        const ShapeOptions o = ctx.shapeOptions();
        const Drag d = drag(end, mods);
        const std::uint32_t color = ctx.foreground().rgba();
        switch (o.mode) {
        case 1:
            shape_add_path(v, kind_, d.slice(), d.shift, d.alt, o.radius, o.sides);
            break;
        case 2:
            if (!shape_fill_pixels(v, kind_, d.slice(), d.shift, d.alt, o.radius, o.sides,
                                   color)) {
                reportRefusal(ctx, v);
            }
            break;
        default: {
            const QString created =
                shape_add_layer(v, kind_, d.slice(), d.shift, d.alt, o.radius, o.sides, color);
            if (!created.isEmpty()) {
                ctx.notifyLayerCreated(created);
            }
            break;
        }
        }
    }

    static void reportRefusal(ToolContext& ctx, PictureView& v)
    {
        if (activePixelLocked(&v)) {
            ctx.refused(QObject::tr("Could not fill: the layer's pixels are locked."));
        } else if (!v.active_layer_visible()) {
            ctx.refused(QObject::tr("Could not fill: the active layer is invisible."));
        } else if (v.active_layer_path().isEmpty()) {
            ctx.refused(QObject::tr("Could not fill: select a single layer first."));
        }
    }

    // The live outline; in Path mode the Work Path it joins is drawn too.
    static void refreshOverlay(ToolContext& ctx, const QPainterPath& outline)
    {
        ImageView* canvas = ctx.canvas();
        if (!canvas) {
            return;
        }
        PictureView* v = ctx.view();
        ImageView::PathOverlay overlay;
        if (v && v->has_document() && ctx.shapeOptions().mode == 1) {
            overlay = workPathOverlay(*v, -2);
        }
        overlay.preview = outline;
        canvas->setPathOverlay(overlay);
    }

    const int kind_;
    QPointF press_;
};

} // namespace

std::unique_ptr<ToolHandler> makeShapeToolHandler(ToolId id)
{
    return std::make_unique<ShapeToolHandler>(id);
}

} // namespace pictura
