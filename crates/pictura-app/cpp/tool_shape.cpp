// The shape tools (`pictura_core::shape`, bridged by `cxxqt_object/shapes.rs`):
// Rectangle, Rounded Rectangle, Ellipse, and Polygon. A drag previews the
// outline on the canvas and lands on release in the options bar's Mode: a new
// shape layer (Shape), a Work Path component (Path), or foreground pixels on
// the active layer (Pixels). Shift squares the box off (the Polygon snaps its
// turn to 15°); Alt grows it from the press point. The modifiers are read live,
// so pressing one mid-drag changes the preview. A click instead opens the
// tool's Create dialog and places the shape at the click (or centred on it).
// Outside Path mode the active shape layer's outline is drawn with its anchors,
// ready for Direct Selection. Ported from photorust's CanvasView shape drag
// (shapeOutlineFor / paintShapeOverlay / drawShape).
//
// ponytail: no Stroke, gradient / pattern Fill, geometry pop-up (Fixed Size,
// Proportional), path operations, or Align Edges.

#include "tool_handler.h"

#include "image_view.h"
#include "path_overlay.h"
#include "shape_dialogs.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/paths.cxxqt.h"
#include "pictura_app/src/cxxqt_object/shapes.cxxqt.h"

#include <QtCore/QLineF>
#include <QtCore/QObject>
#include <QtGui/QPainterPath>

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
        : id_(id)
        , kind_(shapeKind(id))
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
            refreshOverlay(ctx, outlinePath(dragSpec(ctx, imagePos, mods)));
        }
    }

    void onRelease(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        if (!ctx.dragging()) {
            return;
        }
        ctx.setDragging(false);
        refreshOverlay(ctx, {});
        PictureView* v = ctx.view();
        if (!v || !v->has_document()) {
            return;
        }
        // A press that wobbles under two screen pixels is a click.
        const double zoom = ctx.canvas() ? qMax(ctx.canvas()->zoom(), 1e-6) : 1.0;
        if (QLineF(press_, imagePos).length() * zoom < 2.0) {
            createAtClick(ctx, *v);
        } else {
            const ShapeSpec spec = dragSpec(ctx, imagePos, mods);
            const QRectF drawn = outlinePath(spec).boundingRect();
            if (land(ctx, *v, spec) && !drawn.isEmpty()) {
                values_.width = drawn.width();
                values_.height = drawn.height();
            }
        }
        refreshOverlay(ctx, {});
    }

private:
    // The spec's options from the options bar; the geometry is the caller's.
    ShapeSpec baseSpec(ToolContext& ctx) const
    {
        const ShapeOptions o = ctx.shapeOptions();
        ShapeSpec spec{};
        spec.kind = kind_;
        spec.r_tl = spec.r_tr = spec.r_br = spec.r_bl = o.radius;
        spec.sides = o.sides;
        spec.indent = 50.0;
        return spec;
    }

    ShapeSpec dragSpec(ToolContext& ctx, const QPointF& end, Qt::KeyboardModifiers mods) const
    {
        ShapeSpec spec = baseSpec(ctx);
        spec.x0 = press_.x();
        spec.y0 = press_.y();
        spec.x1 = end.x();
        spec.y1 = end.y();
        spec.shift = mods.testFlag(Qt::ShiftModifier);
        spec.alt = mods.testFlag(Qt::AltModifier);
        return spec;
    }

    static QPainterPath outlinePath(const ShapeSpec& spec)
    {
        const ::rust::Vec<double> k = shape_outline(spec);
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

    // The Create dialog, seeded with the last shape's size and the options
    // bar's radius and sides the first time; the shape's top-left corner (or
    // centre) goes at the click.
    void createAtClick(ToolContext& ctx, PictureView& v)
    {
        const ShapeOptions o = ctx.shapeOptions();
        if (!dialogSeeded_) {
            values_.radii.fill(o.radius);
            values_.sides = o.sides;
            dialogSeeded_ = true;
        }
        if (!execCreateShapeDialog(id_, values_, ctx.canvas())) {
            return;
        }
        ShapeSpec spec = baseSpec(ctx);
        spec.boxed = true;
        spec.x0 = press_.x() - (values_.fromCenter ? values_.width / 2.0 : 0.0);
        spec.y0 = press_.y() - (values_.fromCenter ? values_.height / 2.0 : 0.0);
        spec.x1 = spec.x0 + values_.width;
        spec.y1 = spec.y0 + values_.height;
        spec.r_tl = values_.radii[0];
        spec.r_tr = values_.radii[1];
        spec.r_br = values_.radii[2];
        spec.r_bl = values_.radii[3];
        spec.sides = values_.sides;
        spec.star = values_.star;
        spec.indent = values_.indent;
        spec.smooth_corners = values_.smoothCorners;
        spec.smooth_indents = values_.smoothIndents;
        land(ctx, v, spec);
    }

    bool land(ToolContext& ctx, PictureView& v, const ShapeSpec& spec)
    {
        const std::uint32_t color = ctx.foreground().rgba();
        switch (ctx.shapeOptions().mode) {
        case 1:
            return shape_add_path(v, spec);
        case 2:
            if (!shape_fill_pixels(v, spec, color)) {
                reportRefusal(ctx, v);
                return false;
            }
            return true;
        default: {
            const QString created = shape_add_layer(v, spec, color);
            if (created.isEmpty()) {
                return false;
            }
            ctx.notifyLayerCreated(created);
            return true;
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

    // The live outline over either the Work Path it joins (Path mode) or the
    // active shape layer's outline with its anchors.
    static void refreshOverlay(ToolContext& ctx, const QPainterPath& outline)
    {
        ImageView* canvas = ctx.canvas();
        if (!canvas) {
            return;
        }
        PictureView* v = ctx.view();
        ImageView::PathOverlay overlay;
        if (v && v->has_document()) {
            const bool pathMode = ctx.shapeOptions().mode == 1;
            path_set_layer_target(*v, !pathMode);
            if (pathMode) {
                overlay = workPathOverlay(*v, -2);
            } else if (path_target_is_layer(*v)) {
                overlay = workPathOverlay(*v, -1);
                overlay.handles.clear();
            }
        }
        overlay.preview = outline;
        canvas->setPathOverlay(overlay);
    }

    const ToolId id_;
    const int kind_;
    QPointF press_;
    CreateShapeValues values_;
    bool dialogSeeded_ = false;
};

} // namespace

std::unique_ptr<ToolHandler> makeShapeToolHandler(ToolId id)
{
    return std::make_unique<ShapeToolHandler>(id);
}

} // namespace pictura
