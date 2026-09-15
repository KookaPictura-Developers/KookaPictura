#include "tools.h"

#include "icons.h"
#include "image_view.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <algorithm>

namespace pictura {

namespace {

const ToolInfo kToolTable[] = {
    {ToolId::Move, "move", "Move", QLatin1Char('V'), Qt::SizeAllCursor,
     "Move: drag to move the active layer"},
    {ToolId::Marquee, "marquee", "Rectangular Marquee", QLatin1Char('M'), Qt::CrossCursor,
     "Marquee: drag to select a rectangle"},
    {ToolId::Lasso, "lasso", "Lasso", QLatin1Char('L'), Qt::CrossCursor,
     "Lasso: drag around a region to select"},
    {ToolId::QuickSelection, "quickselection", "Quick Selection", QLatin1Char('W'), Qt::CrossCursor,
     "Quick Selection: drag to grow a selection"},
    {ToolId::Crop, "crop", "Crop", QLatin1Char('C'), Qt::CrossCursor,
     "Crop: drag a region, press Enter to commit"},
    {ToolId::Eyedropper, "eyedropper", "Eyedropper", QLatin1Char('I'), Qt::CrossCursor,
     "Eyedropper: click to sample a colour"},
    {ToolId::Hand, "hand", "Hand", QLatin1Char('H'), Qt::OpenHandCursor,
     "Hand: drag to pan the canvas"},
    {ToolId::Zoom, "zoom", "Zoom", QLatin1Char('Z'), Qt::CrossCursor,
     "Zoom: click to zoom in, Ctrl/Alt-click to zoom out"},
};
constexpr int kToolCount = int(sizeof(kToolTable) / sizeof(kToolTable[0]));
static_assert(kToolCount == 8, "tool table must cover every ToolId");

int toolIndex(ToolId id) { return static_cast<int>(id); }

} // namespace

const ToolInfo& toolInfo(ToolId id)
{
    const int index = toolIndex(id);
    return (index >= 0 && index < kToolCount) ? kToolTable[index] : kToolTable[0];
}

QString toolIdName(ToolId id)
{
    const int index = toolIndex(id);
    return QString::fromLatin1(
        (index >= 0 && index < kToolCount) ? kToolTable[index].name : kToolTable[0].name);
}

const QList<ToolId>& allToolIds()
{
    static const QList<ToolId> ids = {ToolId::Move,      ToolId::Marquee, ToolId::Lasso,
                                      ToolId::QuickSelection, ToolId::Crop, ToolId::Eyedropper,
                                      ToolId::Hand,      ToolId::Zoom};
    return ids;
}

QString selectionModeString(SelectionMode mode)
{
    switch (mode) {
    case SelectionMode::New:
        return QStringLiteral("new");
    case SelectionMode::Add:
        return QStringLiteral("add");
    case SelectionMode::Subtract:
        return QStringLiteral("subtract");
    case SelectionMode::Intersect:
        return QStringLiteral("intersect");
    }
    return QStringLiteral("new");
}

ToolController::ToolController(QObject* parent)
    : QObject(parent)
{
}

void ToolController::setActiveTool(ToolId id)
{
    if (active_ == id) {
        return;
    }
    active_ = id;
    dragging_ = false;
    dragCommitted_ = false;
    if (canvas_) {
        canvas_->clearOverlay();
    }
    applyToolPolicy();
    emit activeToolChanged(id);
}

void ToolController::setCombineMode(SelectionMode mode) { mode_ = mode; }

void ToolController::setTolerance(int tolerance)
{
    tolerance_ = std::clamp(tolerance, 0, 255);
}

void ToolController::setViewProvider(std::function<PictureView*()> provider)
{
    viewProvider_ = std::move(provider);
}

PictureView* ToolController::view() const
{
    return viewProvider_ ? viewProvider_() : nullptr;
}

void ToolController::bindCanvas(ImageView* canvas)
{
    if (canvas_ == canvas) {
        return;
    }
    unbindCanvas();
    canvas_ = canvas;
    if (!canvas_) {
        return;
    }
    connect(canvas_, &ImageView::mousePressed, this, &ToolController::handlePressed);
    connect(canvas_, &ImageView::mouseMoved, this, &ToolController::handleMoved);
    connect(canvas_, &ImageView::mouseReleased, this, &ToolController::handleReleased);
    applyToolPolicy();
}

void ToolController::unbindCanvas()
{
    if (!canvas_) {
        return;
    }
    disconnect(canvas_, nullptr, this, nullptr);
    canvas_->clearOverlay();
    canvas_ = nullptr;
    dragging_ = false;
    dragCommitted_ = false;
}

void ToolController::applyToolPolicy()
{
    if (!canvas_) {
        return;
    }
    canvas_->setPanEnabled(active_ == ToolId::Hand);
    canvas_->setCursor(cursor(QStringLiteral("tool.") + toolIdName(active_)));
}

void ToolController::handlePressed(const QPointF& imagePos, int button, int modifiers)
{
    if (button != Qt::LeftButton) {
        return;
    }
    PictureView* v = view();

    switch (active_) {
    case ToolId::Hand:
        return;
    case ToolId::Zoom: {
        if (!canvas_) {
            return;
        }
        const Qt::KeyboardModifiers mods = Qt::KeyboardModifiers(modifiers);
        if (mods.testFlag(Qt::ControlModifier) || mods.testFlag(Qt::AltModifier)) {
            canvas_->zoomOut();
        } else {
            canvas_->zoomIn();
        }
        return;
    }
    case ToolId::Eyedropper: {
        if (!v) {
            return;
        }
        const quint32 argb = v->sample_argb(qRound(imagePos.x()), qRound(imagePos.y()));
        if (argb != 0) {
            emit foregroundSampled(QColor::fromRgb(argb));
        }
        return;
    }
    case ToolId::Move:
    case ToolId::Crop:
        if (!v) {
            return;
        }
        dragging_ = true;
        dragCommitted_ = false;
        anchor_ = last_ = imagePos;
        totalDelta_ = QPointF();
        if (active_ == ToolId::Crop) {
            hasPendingCrop_ = false;
            pendingCrop_ = QRect();
            updateDragOverlay(imagePos);
        }
        return;
    case ToolId::Marquee:
        if (!v) {
            return;
        }
        dragging_ = true;
        dragCommitted_ = false;
        anchor_ = last_ = imagePos;
        updateDragOverlay(imagePos);
        return;
    case ToolId::Lasso:
        if (!v || !v->begin_lasso(selectionModeString(mode_))) {
            return;
        }
        dragging_ = true;
        dragCommitted_ = false;
        last_ = imagePos;
        lassoPolygon_.clear();
        lassoPolygon_ << imagePos;
        if (canvas_) {
            canvas_->setOverlayPolygon(lassoPolygon_);
        }
        return;
    case ToolId::QuickSelection:
        if (!v) {
            return;
        }
        dragging_ = true;
        dragCommitted_ = v->quick_select(qRound(imagePos.x()), qRound(imagePos.y()), tolerance_,
                                         selectionModeString(mode_));
        return;
    }
}

void ToolController::handleMoved(const QPointF& imagePos)
{
    if (!dragging_) {
        return;
    }
    PictureView* v = view();

    switch (active_) {
    case ToolId::Move:
        totalDelta_ += imagePos - last_;
        last_ = imagePos;
        return;
    case ToolId::Marquee:
    case ToolId::Crop:
        last_ = imagePos;
        updateDragOverlay(imagePos);
        return;
    case ToolId::Lasso:
        if (!v) {
            return;
        }
        v->lasso_add_point(qRound(imagePos.x()), qRound(imagePos.y()));
        lassoPolygon_ << imagePos;
        if (canvas_) {
            canvas_->setOverlayPolygon(lassoPolygon_);
        }
        return;
    case ToolId::QuickSelection:
        if (!v) {
            return;
        }
        if (v->quick_select(qRound(imagePos.x()), qRound(imagePos.y()), tolerance_,
                            selectionModeString(mode_))) {
            dragCommitted_ = true;
        }
        return;
    default:
        return;
    }
}

void ToolController::handleReleased(const QPointF& imagePos)
{
    if (!dragging_) {
        return;
    }
    dragging_ = false;
    PictureView* v = view();

    switch (active_) {
    case ToolId::Move: {
        const int dx = qRound(totalDelta_.x());
        const int dy = qRound(totalDelta_.y());
        if (v && (dx != 0 || dy != 0)) {
            v->translate_layer(dx, dy);
        }
        return;
    }
    case ToolId::Marquee: {
        const QRect rect = dragRect(anchor_, imagePos);
        const bool committed =
            v && rect.width() > 0 && rect.height() > 0
            && v->select_rect(rect.x(), rect.y(), rect.width(), rect.height(),
                              selectionModeString(mode_));
        if (canvas_) {
            canvas_->clearOverlay();
        }
        if (committed) {
            emit selectionCommitted();
        }
        return;
    }
    case ToolId::Lasso: {
        const bool committed = v && v->end_lasso();
        lassoPolygon_.clear();
        if (canvas_) {
            canvas_->clearOverlay();
        }
        if (committed) {
            emit selectionCommitted();
        }
        return;
    }
    case ToolId::QuickSelection:
        if (canvas_) {
            canvas_->clearOverlay();
        }
        if (dragCommitted_) {
            emit selectionCommitted();
        }
        return;
    case ToolId::Crop: {
        const QRect rect = dragRect(anchor_, imagePos);
        pendingCrop_ = rect;
        hasPendingCrop_ = rect.width() > 0 && rect.height() > 0;
        if (canvas_) {
            if (hasPendingCrop_) {
                canvas_->setOverlayPolygon(QPolygonF(QRectF(rect)));
            } else {
                canvas_->clearOverlay();
            }
        }
        return;
    }
    default:
        return;
    }
}

void ToolController::updateDragOverlay(const QPointF& imagePos)
{
    if (!canvas_) {
        return;
    }
    canvas_->setOverlayPolygon(QPolygonF(QRectF(dragRect(anchor_, imagePos))));
}

QRect ToolController::dragRect(const QPointF& a, const QPointF& b)
{
    const QRectF rect = QRectF(a, b).normalized();
    return QRect(qRound(rect.left()), qRound(rect.top()), qRound(rect.width()),
                 qRound(rect.height()));
}

bool ToolController::commitCrop()
{
    if (!hasPendingCrop_) {
        return false;
    }
    const QRect rect = pendingCrop_;
    hasPendingCrop_ = false;
    pendingCrop_ = QRect();
    if (canvas_) {
        canvas_->clearOverlay();
    }
    PictureView* v = view();
    if (!v) {
        return false;
    }
    const bool ok = v->crop(rect.x(), rect.y(), rect.width(), rect.height());
    if (ok) {
        emit selectionCommitted();
    }
    return ok;
}

} // namespace pictura
