#include "tool_handler.h"

#include "image_view.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/crop_group.cxxqt.h"

#include <QtCore/QRectF>

#include <cmath>
#include <memory>

namespace pictura {

namespace {

// Slice: drag out a user slice (one "Slice" history state). While the tool is
// active the canvas shows every resolved slice: user slices solid blue with a
// numbered badge, auto slices dotted grey. Ported from photorust's CanvasView.
//
// ponytail: selecting, moving, resizing, or deleting a slice is the Slice
// Select tool, which is not implemented; Undo removes a mistaken slice.
class SliceToolHandler : public ToolHandler {
public:
    void onActivate(ToolContext& ctx) override { showSlices(ctx); }

    void onDeactivate(ToolContext& ctx) override
    {
        dragging_ = false;
        if (ImageView* canvas = ctx.canvas()) {
            canvas->clearSliceOverlay();
        }
    }

    void onDocumentRefreshed(ToolContext& ctx) override { showSlices(ctx); }

    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        PictureView* v = ctx.view();
        if (!v || !v->has_document()) {
            return true;
        }
        dragging_ = true;
        start_ = imagePos;
        drag_ = QRectF(imagePos, imagePos);
        showSlices(ctx);
        return true;
    }

    void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        if (!dragging_) {
            return;
        }
        drag_ = QRectF(start_, imagePos).normalized();
        showSlices(ctx);
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
            const int x = int(std::lround(box.x()));
            const int y = int(std::lround(box.y()));
            add_user_slice(*v, x, y, int(std::lround(box.right())) - x,
                           int(std::lround(box.bottom())) - y);
        }
        showSlices(ctx);
    }

private:
    void showSlices(ToolContext& ctx)
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
                overlay.append({QRectF(f[0], f[1], f[2], f[3]), f[4], f[5] >= 0});
            }
        }
        canvas->setSliceOverlay(overlay, dragging_ ? drag_ : QRectF());
    }

    bool dragging_ = false;
    QPointF start_;
    QRectF drag_;
};

} // namespace

std::unique_ptr<ToolHandler> makeSliceToolHandler()
{
    return std::make_unique<SliceToolHandler>();
}

} // namespace pictura
