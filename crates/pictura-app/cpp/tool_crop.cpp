#include "tool_handler.h"

#include "image_view.h"
#include "selection_geometry.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QPointF>
#include <QtCore/QRect>
#include <QtGui/QPolygonF>

#include <memory>

namespace pictura {

namespace {

// Crop: drag a rectangle, then commit it on Enter. The staged rectangle is
// owned here; the controller exposes it so the menu and key handler can query
// and commit it.
class CropToolHandler : public ToolHandler {
public:
    void onActivate(ToolContext& ctx) override { ctx_ = &ctx; }

    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        PictureView* v = ctx.view();
        if (!v) {
            return true;
        }
        ctx_ = &ctx;
        ctx.setDragging(true);
        ctx.setDragCommitted(false);
        anchor_ = last_ = imagePos;
        hasPendingCrop_ = false;
        pendingCrop_ = QRect();
        updateOverlay(ctx, imagePos);
        return true;
    }

    void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        if (!ctx.dragging()) {
            return;
        }
        last_ = imagePos;
        updateOverlay(ctx, imagePos);
    }

    void onRelease(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        if (!ctx.dragging()) {
            return;
        }
        ctx.setDragging(false);
        const QRect rect = dragRect(anchor_, imagePos);
        pendingCrop_ = rect;
        hasPendingCrop_ = rect.width() > 0 && rect.height() > 0;
        if (ImageView* canvas = ctx.canvas()) {
            if (hasPendingCrop_) {
                canvas->setOverlayPolygon(QPolygonF(QRectF(rect)));
            } else {
                canvas->clearOverlay();
            }
        }
    }

    bool commitCrop() override
    {
        if (!hasPendingCrop_) {
            return false;
        }
        const QRect rect = pendingCrop_;
        hasPendingCrop_ = false;
        pendingCrop_ = QRect();
        if (!ctx_) {
            return false;
        }
        if (ImageView* canvas = ctx_->canvas()) {
            canvas->clearOverlay();
        }
        PictureView* v = ctx_->view();
        if (!v) {
            return false;
        }
        const bool ok = v->crop(rect.x(), rect.y(), rect.width(), rect.height());
        if (ok) {
            ctx_->emitSelectionCommitted();
        }
        return ok;
    }

    bool hasPendingCrop() const override { return hasPendingCrop_; }
    QRect pendingCropRect() const override { return pendingCrop_; }

private:
    void updateOverlay(ToolContext& ctx, const QPointF& imagePos)
    {
        if (ImageView* canvas = ctx.canvas()) {
            canvas->setOverlayPolygon(QPolygonF(QRectF(dragRect(anchor_, imagePos))));
        }
    }

    QPointF anchor_;
    QPointF last_;
    QRect pendingCrop_;
    bool hasPendingCrop_ = false;
    ToolContext* ctx_ = nullptr;
};

} // namespace

std::unique_ptr<ToolHandler> makeCropToolHandler()
{
    return std::make_unique<CropToolHandler>();
}

} // namespace pictura
