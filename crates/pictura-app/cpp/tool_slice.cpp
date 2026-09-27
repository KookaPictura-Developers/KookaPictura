#include "tool_handler.h"

#include "icons.h"
#include "image_view.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/crop_group.cxxqt.h"

#include <QtCore/QRectF>

#include <algorithm>
#include <cmath>
#include <memory>
#include <optional>

namespace pictura {

namespace {

// Show every resolved slice on the canvas: user slices solid blue with a
// numbered badge, auto slices dotted grey, user slice `selected` (a user index,
// -1 for none) with orange handles, plus `dragging` while one is drawn out.
void showSlices(ToolContext& ctx, int selected, const QRectF& dragging = QRectF())
{
    ImageView* canvas = ctx.canvas();
    PictureView* v = ctx.view();
    if (!canvas) {
        return;
    }
    QList<ImageView::SliceOverlay> overlay;
    const int count = v && v->has_document() ? slice_count(*v) : 0;
    for (int i = 0; i < count; ++i) {
        const ::rust::Vec<std::int32_t> f = slice_at(*v, i);
        if (f.size() == 6) {
            overlay.append({QRectF(f[0], f[1], f[2], f[3]), f[4], f[5] >= 0,
                            f[5] >= 0 && f[5] == selected});
        }
    }
    canvas->setSliceOverlay(overlay, dragging);
}

int roundInt(double v) { return int(std::lround(v)); }

// Slice: drag out a user slice (one "Slice" history state); a click adds none.
// Ported from photorust's CanvasView.
class SliceToolHandler : public ToolHandler {
public:
    void onActivate(ToolContext& ctx) override { showSlices(ctx, -1); }

    void onDeactivate(ToolContext& ctx) override
    {
        dragging_ = false;
        if (ImageView* canvas = ctx.canvas()) {
            canvas->clearSliceOverlay();
        }
    }

    void onDocumentRefreshed(ToolContext& ctx) override
    {
        showSlices(ctx, -1, dragging_ ? drag_ : QRectF());
    }

    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        PictureView* v = ctx.view();
        if (!v || !v->has_document()) {
            return true;
        }
        dragging_ = true;
        start_ = imagePos;
        drag_ = QRectF(imagePos, imagePos);
        showSlices(ctx, -1, drag_);
        return true;
    }

    void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        if (!dragging_) {
            return;
        }
        drag_ = QRectF(start_, imagePos).normalized();
        showSlices(ctx, -1, drag_);
    }

    void onRelease(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        if (!dragging_) {
            return;
        }
        dragging_ = false;
        const QRectF box = QRectF(start_, imagePos).normalized();
        drag_ = QRectF();
        PictureView* v = ctx.view();
        if (v && box.width() >= 1.0 && box.height() >= 1.0) {
            const int x = roundInt(box.x());
            const int y = roundInt(box.y());
            add_user_slice(*v, x, y, roundInt(box.right()) - x, roundInt(box.bottom()) - y);
        }
        showSlices(ctx, -1);
    }

private:
    bool dragging_ = false;
    QPointF start_;
    QRectF drag_;
};

// Slice Select: click a user slice to select it (orange handles); clicking an
// auto slice or empty canvas deselects. Drag inside the selected slice to move
// it, or an edge/corner handle to resize it: the document updates live and the
// release records one "Edit Slice" state. Delete removes the selected slice
// ("Delete Slice"); Escape deselects. Ported from photorust's CanvasView.
class SliceSelectToolHandler : public ToolHandler {
public:
    void onActivate(ToolContext& ctx) override
    {
        ctx_ = &ctx;
        showSlices(ctx, selected_);
    }

    void onDeactivate(ToolContext& ctx) override
    {
        grip_ = Grip::None;
        selected_ = -1;
        if (ImageView* canvas = ctx.canvas()) {
            canvas->clearSliceOverlay();
        }
        ctx_ = nullptr;
    }

    void onDocumentRefreshed(ToolContext& ctx) override
    {
        // Undo can remove the selected slice; drop a selection that no longer
        // exists.
        if (selected_ >= 0 && !userRect(ctx, selected_)) {
            selected_ = -1;
        }
        showSlices(ctx, selected_);
    }

    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        PictureView* v = ctx.view();
        if (!v || !v->has_document()) {
            return true;
        }
        ctx_ = &ctx;
        grip_ = Grip::None;
        if (selected_ >= 0) {
            if (const auto rect = userRect(ctx, selected_)) {
                grip_ = gripAt(ctx, *rect, imagePos);
                startRect_ = *rect;
            }
        }
        if (grip_ == Grip::None) {
            // User slices win over the auto slices beneath them.
            selected_ = userSliceAt(ctx, imagePos);
            if (selected_ >= 0) {
                grip_ = Grip::Move;
                startRect_ = *userRect(ctx, selected_);
            }
        }
        start_ = imagePos;
        moved_ = false;
        showSlices(ctx, selected_);
        return true;
    }

    void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        PictureView* v = ctx.view();
        if (grip_ == Grip::None || selected_ < 0 || !v) {
            updateCursor(ctx, imagePos);
            return;
        }
        const QRectF r = dragged(imagePos);
        if (r.width() >= 1.0 && r.height() >= 1.0) {
            moved_ |= set_user_slice(*v, selected_, roundInt(r.x()), roundInt(r.y()),
                                     roundInt(r.width()), roundInt(r.height()), false);
            showSlices(ctx, selected_);
        }
    }

    void onRelease(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        if (grip_ == Grip::None) {
            return;
        }
        onMove(ctx, imagePos, mods);
        PictureView* v = ctx.view();
        const QRectF r = dragged(imagePos);
        if (moved_ && v && r.width() >= 1.0 && r.height() >= 1.0 && r != startRect_) {
            set_user_slice(*v, selected_, roundInt(r.x()), roundInt(r.y()), roundInt(r.width()),
                           roundInt(r.height()), true);
        }
        grip_ = Grip::None;
        showSlices(ctx, selected_);
    }

    // Delete / Backspace (routed through the controller's remove-point hook).
    bool removeLassoPoint() override
    {
        PictureView* v = ctx_ ? ctx_->view() : nullptr;
        if (!v || selected_ < 0) {
            return false;
        }
        const int index = selected_;
        selected_ = -1;
        remove_user_slice(*v, index);
        showSlices(*ctx_, selected_);
        return true;
    }

    // Escape deselects.
    bool cancelPolygonLasso() override
    {
        if (selected_ < 0 || !ctx_) {
            return false;
        }
        selected_ = -1;
        showSlices(*ctx_, selected_);
        return true;
    }

private:
    enum class Grip { None, Move, TopLeft, Top, TopRight, Right, BottomRight, Bottom, BottomLeft, Left };

    static std::optional<QRectF> userRect(ToolContext& ctx, int userIndex)
    {
        PictureView* v = ctx.view();
        const int count = v ? slice_count(*v) : 0;
        for (int i = 0; i < count; ++i) {
            const ::rust::Vec<std::int32_t> f = slice_at(*v, i);
            if (f.size() == 6 && f[5] == userIndex) {
                return QRectF(f[0], f[1], f[2], f[3]);
            }
        }
        return std::nullopt;
    }

    static int userSliceAt(ToolContext& ctx, const QPointF& imagePos)
    {
        PictureView* v = ctx.view();
        const int count = v ? slice_count(*v) : 0;
        for (int i = 0; i < count; ++i) {
            const ::rust::Vec<std::int32_t> f = slice_at(*v, i);
            if (f.size() == 6 && f[5] >= 0 && QRectF(f[0], f[1], f[2], f[3]).contains(imagePos)) {
                return f[5];
            }
        }
        return -1;
    }

    // Handles grab within a fixed 6 screen px, corners before edges.
    static Grip gripAt(ToolContext& ctx, const QRectF& r, const QPointF& p)
    {
        const double zoom = ctx.canvas() ? std::max(ctx.canvas()->zoom(), 1e-6) : 1.0;
        const double grab = 6.0 / zoom;
        const bool left = std::abs(p.x() - r.left()) <= grab;
        const bool right = std::abs(p.x() - r.right()) <= grab;
        const bool top = std::abs(p.y() - r.top()) <= grab;
        const bool bottom = std::abs(p.y() - r.bottom()) <= grab;
        if (p.x() < r.left() - grab || p.x() > r.right() + grab || p.y() < r.top() - grab
            || p.y() > r.bottom() + grab) {
            return Grip::None;
        }
        if (left && top) return Grip::TopLeft;
        if (right && top) return Grip::TopRight;
        if (left && bottom) return Grip::BottomLeft;
        if (right && bottom) return Grip::BottomRight;
        if (left) return Grip::Left;
        if (right) return Grip::Right;
        if (top) return Grip::Top;
        if (bottom) return Grip::Bottom;
        return r.contains(p) ? Grip::Move : Grip::None;
    }

    QRectF dragged(const QPointF& imagePos) const
    {
        const QPointF d = imagePos - start_;
        QRectF r = startRect_;
        switch (grip_) {
        case Grip::None: break;
        case Grip::Move: r.translate(d); break;
        case Grip::TopLeft: r.setTopLeft(r.topLeft() + d); break;
        case Grip::Top: r.setTop(r.top() + d.y()); break;
        case Grip::TopRight: r.setTopRight(r.topRight() + d); break;
        case Grip::Right: r.setRight(r.right() + d.x()); break;
        case Grip::BottomRight: r.setBottomRight(r.bottomRight() + d); break;
        case Grip::Bottom: r.setBottom(r.bottom() + d.y()); break;
        case Grip::BottomLeft: r.setBottomLeft(r.bottomLeft() + d); break;
        case Grip::Left: r.setLeft(r.left() + d.x()); break;
        }
        return r.normalized();
    }

    void updateCursor(ToolContext& ctx, const QPointF& imagePos)
    {
        ImageView* canvas = ctx.canvas();
        if (!canvas) {
            return;
        }
        const auto rect = selected_ >= 0 ? userRect(ctx, selected_) : std::nullopt;
        switch (rect ? gripAt(ctx, *rect, imagePos) : Grip::None) {
        case Grip::Move: canvas->setCursor(Qt::SizeAllCursor); break;
        case Grip::TopLeft:
        case Grip::BottomRight: canvas->setCursor(Qt::SizeFDiagCursor); break;
        case Grip::TopRight:
        case Grip::BottomLeft: canvas->setCursor(Qt::SizeBDiagCursor); break;
        case Grip::Left:
        case Grip::Right: canvas->setCursor(Qt::SizeHorCursor); break;
        case Grip::Top:
        case Grip::Bottom: canvas->setCursor(Qt::SizeVerCursor); break;
        case Grip::None: {
            const ToolInfo& info = toolInfo(ToolId::SliceSelect);
            const QCursor c = cursor(toolCursorId(ToolId::SliceSelect, Qt::NoModifier),
                                     info.hotspotX, info.hotspotY);
            canvas->setCursor(c.pixmap().isNull() ? QCursor(info.cursor) : c);
            break;
        }
        }
    }

    int selected_ = -1;
    Grip grip_ = Grip::None;
    QPointF start_;
    QRectF startRect_;
    bool moved_ = false;
    ToolContext* ctx_ = nullptr;
};

} // namespace

std::unique_ptr<ToolHandler> makeSliceToolHandler()
{
    return std::make_unique<SliceToolHandler>();
}

std::unique_ptr<ToolHandler> makeSliceSelectToolHandler()
{
    return std::make_unique<SliceSelectToolHandler>();
}

} // namespace pictura
