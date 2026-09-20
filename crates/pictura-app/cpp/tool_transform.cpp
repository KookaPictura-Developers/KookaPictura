#include "tools.h"

#include "image_view.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

namespace pictura {

bool ToolController::beginFreeTransform(const QString& path)
{
    PictureView* v = view();
    if (!v || !v->begin_free_transform(path)) {
        return false;
    }
    transformDragging_ = false;
    transformHandle_ = -1;
    if (canvas_) {
        canvas_->beginTransformPreview(v->move_preview_base(), v->move_preview_layer(),
                                      QPointF(v->move_preview_x(), v->move_preview_y()),
                                      v->move_preview_opacity() / 255.0);
        updateTransformOverlay(v);
        canvas_->setPanEnabled(false);
        canvas_->setFocus();
    }
    return true;
}

bool ToolController::transformSessionActive() const
{
    PictureView* v = view();
    return v && v->transform_session_active();
}

void ToolController::updateTransformOverlay(PictureView* v)
{
    if (!canvas_ || !v) {
        return;
    }
    canvas_->setTransformQuad(v->transform_quad());
    canvas_->setTransformPreview(v->transform_scale_x(), v->transform_scale_y(),
                                 v->transform_angle(), v->transform_dx(), v->transform_dy());
}

void ToolController::setTransformCursor(const QPointF& imagePos)
{
    PictureView* v = view();
    if (!canvas_ || !v) {
        return;
    }
    const int hit = v->transform_hit_test(imagePos.x(), imagePos.y(), canvas_->zoom());
    switch (hit) {
    case 0:
    case 2:
        canvas_->setCursor(Qt::SizeFDiagCursor);
        break;
    case 1:
    case 3:
        canvas_->setCursor(Qt::SizeBDiagCursor);
        break;
    case 4:
    case 6:
        canvas_->setCursor(Qt::SizeVerCursor);
        break;
    case 5:
    case 7:
        canvas_->setCursor(Qt::SizeHorCursor);
        break;
    case 9:
        canvas_->setCursor(transformDragging_ ? Qt::ClosedHandCursor : Qt::OpenHandCursor);
        break;
    case 8:
        canvas_->setCursor(Qt::CrossCursor);
        break;
    default:
        canvas_->setCursor(Qt::ArrowCursor);
        break;
    }
}

void ToolController::commitFreeTransform()
{
    PictureView* v = view();
    if (!v || !v->transform_session_active()) {
        return;
    }
    v->commit_transform();
    transformDragging_ = false;
    transformHandle_ = -1;
    if (canvas_) {
        canvas_->clearTransformPreview();
    }
    applyToolPolicy();
}

void ToolController::cancelFreeTransform()
{
    PictureView* v = view();
    if (!v || !v->transform_session_active()) {
        return;
    }
    v->cancel_transform();
    transformDragging_ = false;
    transformHandle_ = -1;
    if (canvas_) {
        canvas_->clearTransformPreview();
    }
    applyToolPolicy();
}

} // namespace pictura
