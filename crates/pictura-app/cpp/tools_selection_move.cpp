// Selection-move drag state machine for ToolController: pressing inside a live
// selection translates the mask instead of starting a new marquee. Split from
// tools.cpp to keep each translation unit under the size cap.

#include "tools.h"

#include "image_view.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

namespace pictura {

bool ToolController::isSelectionTool(ToolId id)
{
    return id == ToolId::Marquee || id == ToolId::EllipticalMarquee || id == ToolId::Lasso
        || id == ToolId::PolygonalLasso;
}

bool ToolController::maybeBeginSelectionMove(PictureView* v, const QPointF& imagePos)
{
    if (!v || !v->has_selection()
        || v->selection_coverage(qRound(imagePos.x()), qRound(imagePos.y())) <= 0
        || !v->begin_selection_move()) {
        return false;
    }
    contentMove_ = false;
    movingSelection_ = true;
    dragging_ = true;
    dragCommitted_ = false;
    anchor_ = last_ = imagePos;
    refreshCursor();
    return true;
}

void ToolController::beginContentMove(PictureView* v, const QPointF& imagePos, bool duplicate)
{
    if (!v || !v->begin_selection_move()) {
        return;
    }
    contentMove_ = true;
    contentDuplicate_ = duplicate;
    movingSelection_ = true;
    dragging_ = true;
    dragCommitted_ = false;
    anchor_ = last_ = imagePos;
    refreshCursor();
}

void ToolController::cancelSelectionMove()
{
    contentMove_ = false;
    contentDuplicate_ = false;
    if (!movingSelection_) {
        return;
    }
    movingSelection_ = false;
    PictureView* v = view();
    if (v) {
        v->cancel_selection_move();
    }
}

void ToolController::updateSelectionHover(const QPointF& imagePos)
{
    PictureView* v = view();
    cursorOverSelection_ = isSelectionTool(active_) && v && v->has_selection()
        && v->selection_coverage(qRound(imagePos.x()), qRound(imagePos.y())) > 0;
    refreshCursor();
}

void ToolController::dragSelectionMove(const QPointF& imagePos)
{
    PictureView* v = view();
    if (!v) {
        return;
    }
    const int dx = qRound(imagePos.x() - anchor_.x());
    const int dy = qRound(imagePos.y() - anchor_.y());
    if (v->preview_selection_move(dx, dy)) {
        emit selectionPreviewChanged();
    }
}

void ToolController::releaseSelectionMove(const QPointF& imagePos)
{
    if (!movingSelection_) {
        return;
    }
    movingSelection_ = false;
    PictureView* v = view();
    const int dx = qRound(imagePos.x() - anchor_.x());
    const int dy = qRound(imagePos.y() - anchor_.y());
    if (contentMove_) {
        contentMove_ = false;
        if (v && (dx != 0 || dy != 0)) {
            if (v->move_selection_content(dx, dy, contentDuplicate_)) {
                emit selectionCommitted();
            } else {
                v->cancel_selection_move();
            }
        } else if (v) {
            v->cancel_selection_move();
        }
        contentDuplicate_ = false;
        refreshCursor();
        return;
    }
    if (v && (dx != 0 || dy != 0)) {
        if (v->preview_selection_move(dx, dy) && v->commit_selection_move()) {
            emit selectionCommitted();
        }
    } else if (v) {
        v->cancel_selection_move();
    }
    refreshCursor();
}

} // namespace pictura
