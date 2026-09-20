#include "tools.h"

#include "icons.h"
#include "image_view.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QDebug>
#include <QtCore/QElapsedTimer>
#include <QtCore/QPoint>
#include <QtGui/QCursor>
#include <QtGui/QGuiApplication>
#include <QtGui/QPainterPath>
#include <QtWidgets/QApplication>

#include <algorithm>
#include <cmath>
#include <memory>

namespace pictura {

// Concrete handlers live in their own translation units; ToolController owns
// only the registry.
std::unique_ptr<ToolHandler> makeHandToolHandler();
std::unique_ptr<ToolHandler> makeZoomToolHandler();
std::unique_ptr<ToolHandler> makeEyedropperToolHandler();
std::unique_ptr<ToolHandler> makeBrushToolHandler(bool aliased);
std::unique_ptr<ToolHandler> makeMagicWandToolHandler();
std::unique_ptr<ToolHandler> makeQuickSelectionToolHandler();

namespace {

// Document-space radius for the Polygonal Lasso close-click and for treating a
// second rapid press as a double-click. Document pixels, so it shrinks visually
// when zoomed out.
constexpr double kPolygonCloseRadius = 6.0;

} // namespace

ToolController::ToolController(QObject* parent)
    : QObject(parent)
{
    registry_.registerTool(ToolId::Hand, makeHandToolHandler());
    registry_.registerTool(ToolId::Zoom, makeZoomToolHandler());
    registry_.registerTool(ToolId::Eyedropper, makeEyedropperToolHandler());
    registry_.registerTool(ToolId::Brush, makeBrushToolHandler(false));
    registry_.registerTool(ToolId::Pencil, makeBrushToolHandler(true));
    registry_.registerTool(ToolId::MagicWand, makeMagicWandToolHandler());
    registry_.registerTool(ToolId::QuickSelection, makeQuickSelectionToolHandler());
    // A size change from the options bar or `[`/`]` moves the hover ring at
    // once. Query the pointer so a stale position is never reused after leave.
    connect(this, &ToolController::brushSizeChanged, this, [this](int size) {
        if (!canvas_ || (active_ != ToolId::Brush && active_ != ToolId::Pencil)) {
            return;
        }
        const QPoint local = canvas_->mapFromGlobal(QCursor::pos());
        if (canvas_->rect().contains(local)) {
            canvas_->setBrushOutline(size, canvas_->widgetToImage(QPointF(local)));
        }
    });
}

void ToolController::setActiveTool(ToolId id)
{
    if (!toolImplemented(id)) {
        return;
    }
    if (active_ == id) {
        return;
    }
    active_ = id;
    cancelPolygonLasso();
    PictureView* v = view();
    if (v && v->is_painting()) {
        v->cancel_paint();
    }
    dragging_ = false;
    dragCommitted_ = false;
    cancelSelectionMove();
    cursorOverSelection_ = false;
    if (canvas_ && canvas_->movePreviewActive()) {
        canvas_->endMovePreview();
        if (v) {
            v->end_move_preview();
        }
    }
    if (v && v->transform_session_active()) {
        v->cancel_transform();
        transformDragging_ = false;
        transformHandle_ = -1;
        if (canvas_) {
            canvas_->clearTransformPreview();
        }
    }
    if (canvas_) {
        canvas_->clearOverlay();
        canvas_->clearSelectionPreview();
    }
    applyToolPolicy();
    emit activeToolChanged(id);
}

void ToolController::setCombineMode(SelectionMode mode) { mode_ = mode; }

void ToolController::setMarqueeStyle(MarqueeStyle style) { marqueeStyle_ = style; }

void ToolController::setFeather(double feather)
{
    feather_ = feather > 0.0 ? std::min(feather, 250.0) : 0.0;
}

void ToolController::setFixedRatio(double width, double height)
{
    fixedRatioW_ = width > 0.0 ? width : 1.0;
    fixedRatioH_ = height > 0.0 ? height : 1.0;
}

void ToolController::setFixedSize(int width, int height)
{
    fixedSizeW_ = std::max(width, 1);
    fixedSizeH_ = std::max(height, 1);
}

void ToolController::setTolerance(int tolerance)
{
    tolerance_ = std::clamp(tolerance, 0, 255);
}

void ToolController::setContiguous(bool on) { contiguous_ = on; }

int ToolController::brushSize() const { return brushSize_; }

void ToolController::setBrushSize(int size)
{
    const int clamped = std::clamp(size, 1, 5000);
    if (clamped == brushSize_) {
        return;
    }
    brushSize_ = clamped;
    emit brushSizeChanged(brushSize_);
}

int ToolController::brushHardness() const { return brushHardness_; }

void ToolController::setBrushHardness(int h) { brushHardness_ = std::clamp(h, 0, 100); }

int ToolController::brushOpacity() const { return brushOpacity_; }

void ToolController::setBrushOpacity(int o) { brushOpacity_ = std::clamp(o, 0, 100); }

int ToolController::brushFlow() const { return brushFlow_; }

void ToolController::setBrushFlow(int f) { brushFlow_ = std::clamp(f, 0, 100); }

QString ToolController::brushMode() const { return brushMode_; }

void ToolController::setBrushMode(const QString& mode) { brushMode_ = mode; }

bool ToolController::autoErase() const { return autoErase_; }

void ToolController::setAutoErase(bool on) { autoErase_ = on; }

QColor ToolController::foreground() const { return foreground_; }

void ToolController::setForeground(const QColor& color) { foreground_ = color; }

QColor ToolController::background() const { return background_; }

void ToolController::setBackground(const QColor& color) { background_ = color; }

void ToolController::adjustBrushSize(int delta) { setBrushSize(brushSize_ + delta); }

void ToolController::adjustBrushHardness(int delta) { setBrushHardness(brushHardness_ + delta); }

bool ToolController::applyBrushShortcut(int key, quint32 nativeScanCode, bool shift)
{
    PictureView* v = view();
    if (!v) {
        return false;
    }
    const bool paint = active_ == ToolId::Brush || active_ == ToolId::Pencil;
    const int delta = v->brush_shortcut_delta(key, nativeScanCode, shift, paint);
    if (delta == 0) {
        return false;
    }
    // Magnitude 1 is a diameter step, magnitude 5 a hardness step.
    if (std::abs(delta) >= 5) {
        adjustBrushHardness(delta);
    } else {
        adjustBrushSize(delta);
    }
    return true;
}

void ToolController::setViewProvider(std::function<PictureView*()> provider)
{
    viewProvider_ = std::move(provider);
}

PictureView* ToolController::view() const
{
    return viewProvider_ ? viewProvider_() : nullptr;
}

void ToolController::sampledForeground(const QColor& color)
{
    emit foregroundSampled(color);
}

SelectionMode ToolController::resolveSelectionMode(Qt::KeyboardModifiers mods,
                                                   bool hasExistingSelection) const
{
    return selectionModeForModifiers(mode_, mods, hasExistingSelection);
}

void ToolController::refused(const QString& message) { emit pixelEditRefused(message); }

void ToolController::emitSelectionCommitted() { emit selectionCommitted(); }

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
    connect(canvas_, &ImageView::transformCommitRequested, this,
            &ToolController::commitFreeTransform);
    connect(canvas_, &ImageView::transformCancelRequested, this,
            &ToolController::cancelFreeTransform);
    applyToolPolicy();
}

void ToolController::unbindCanvas()
{
    if (!canvas_) {
        return;
    }
    disconnect(canvas_, nullptr, this, nullptr);
    if (canvas_->movePreviewActive()) {
        canvas_->endMovePreview();
        PictureView* previewView = view();
        if (previewView) {
            previewView->end_move_preview();
        }
    }
    if (canvas_->transformPreviewActive()) {
        canvas_->clearTransformPreview();
    }
    transformDragging_ = false;
    transformHandle_ = -1;
    canvas_->clearOverlay();
    canvas_->clearSelectionPreview();
    canvas_->clearBrushOutline();
    canvas_ = nullptr;
    warmView_ = nullptr;
    warmValid_ = false;
    PictureView* v = view();
    if (v && v->is_painting()) {
        v->cancel_paint();
    }
    dragging_ = false;
    dragCommitted_ = false;
    cancelSelectionMove();
    cursorOverSelection_ = false;
}

void ToolController::warmMovePreview()
{
    PictureView* v = view();
    if (!v) {
        warmValid_ = false;
        return;
    }
    if (!v->prepare_move_preview()) {
        warmValid_ = false;
        return;
    }
    warmBase_ = v->move_preview_base();
    warmLayer_ = v->move_preview_layer();
    warmView_ = v;
    warmValid_ = !warmBase_.isNull();
}

void ToolController::applyToolPolicy()
{
    if (!canvas_) {
        return;
    }
    const bool session = transformSessionActive();
    canvas_->setPanEnabled(!session && active_ == ToolId::Hand);
    if (active_ != ToolId::Brush && active_ != ToolId::Pencil) {
        canvas_->clearBrushOutline();
    }
    if (session) {
        return;
    }
    refreshCursor();
    if (active_ == ToolId::Move) {
        warmMovePreview();
    } else {
        warmValid_ = false;
    }
}

void ToolController::handlePressed(const QPointF& imagePos, int button, int modifiers)
{
    if (button != Qt::LeftButton) {
        return;
    }
    PictureView* v = view();
    const Qt::KeyboardModifiers mods = Qt::KeyboardModifiers(modifiers);
    if (v && v->transform_session_active()) {
        const double z = canvas_ ? canvas_->zoom() : 1.0;
        const int hit = v->transform_press(imagePos.x(), imagePos.y(), z,
                                           mods.testFlag(Qt::ShiftModifier),
                                           mods.testFlag(Qt::AltModifier));
        if (hit >= 0) {
            transformDragging_ = true;
            transformHandle_ = hit;
            updateTransformOverlay(v);
            setTransformCursor(imagePos);
        }
        return;
    }
    if (isSelectionTool(active_)) {
        // Capture the modifiers once so the geometry, the release raster, and
        // the drag cursor keep the press-time constraint even if Shift/Alt is
        // released mid-drag.
        dragMods_ = mods;
        const bool ctrl = mods.testFlag(Qt::ControlModifier);
        const bool shift = mods.testFlag(Qt::ShiftModifier);
        const bool alt = mods.testFlag(Qt::AltModifier);
        const bool inside = v && v->has_selection()
            && v->selection_coverage(qRound(imagePos.x()), qRound(imagePos.y())) > 0;
        if (ctrl && inside) {
            beginContentMove(v, imagePos, alt);
            return;
        }
        if (!shift && !alt && maybeBeginSelectionMove(v, imagePos)) {
            return;
        }
    }

    if (ToolHandler* h = registry_.forTool(active_)) {
        if (h->onPress(*this, imagePos, mods)) {
            return;
        }
    }

    switch (active_) {
    case ToolId::Move: {
        if (!v) {
            return;
        }
        if (v->has_selection()) {
            beginContentMove(v, imagePos, mods.testFlag(Qt::AltModifier));
            return;
        }
        QElapsedTimer pressClock;
        pressClock.start();
        const bool alt = mods.testFlag(Qt::AltModifier);
        const bool prepared = alt ? v->begin_move_duplicate() : v->begin_move_preview();
        const qint64 pressNs = pressClock.nsecsElapsed();
        if (qEnvironmentVariableIsSet("PICTURA_PRESS_TRACE") || pressNs > 8000000) {
            qWarning("[move-press] begin_move_preview hit=%d work=%.1fms",
                     (prepared && v->move_preview_cache_hit()) ? 1 : 0, pressNs / 1e6);
        }
        if (!prepared) {
            return;
        }
        dragging_ = true;
        dragCommitted_ = false;
        anchor_ = last_ = imagePos;
        totalDelta_ = QPointF();
        if (canvas_) {
            const bool reuse = warmValid_ && warmView_ == v && v->move_preview_cache_hit();
            if (!reuse) {
                warmBase_ = v->move_preview_base();
                warmLayer_ = v->move_preview_layer();
                warmView_ = v;
                warmValid_ = !warmBase_.isNull();
            }
            canvas_->beginMovePreview(warmBase_, warmLayer_,
                                      QPointF(v->move_preview_x(), v->move_preview_y()),
                                      v->move_preview_opacity() / 255.0);
        }
        return;
    }
    case ToolId::Crop:
        if (!v) {
            return;
        }
        dragging_ = true;
        dragCommitted_ = false;
        anchor_ = last_ = imagePos;
        totalDelta_ = QPointF();
        hasPendingCrop_ = false;
        pendingCrop_ = QRect();
        updateDragOverlay(imagePos);
        return;
    case ToolId::Marquee:
    case ToolId::EllipticalMarquee:
        if (!v) {
            return;
        }
        dragMode_ = selectionModeForModifiers(mode_, mods, v->has_selection());
        dragging_ = true;
        dragCommitted_ = false;
        anchor_ = last_ = imagePos;
        updateMarqueeOverlay(imagePos);
        return;
    case ToolId::Lasso:
        if (!v || !v->begin_lasso(selectionModeString(dragMode_))) {
            return;
        }
        dragging_ = true;
        dragCommitted_ = false;
        last_ = imagePos;
        lassoPolygon_.clear();
        lassoPolygon_ << imagePos;
        if (canvas_) {
            canvas_->setSelectionPreview({lassoPolygon_});
        }
        return;
    case ToolId::PolygonalLasso: {
        if (!v) {
            return;
        }
        const double firstDist = polygonPoints_.isEmpty()
            ? 1e9
            : std::hypot(imagePos.x() - polygonPoints_.first().x(),
                         imagePos.y() - polygonPoints_.first().y());
        const double lastDist = polygonClock_.isValid()
            ? std::hypot(imagePos.x() - lastPolygonPress_.x(),
                         imagePos.y() - lastPolygonPress_.y())
            : 1e9;
        const bool closeClick = polygonInProgress_ && firstDist <= kPolygonCloseRadius;
        const bool doubleClick = polygonInProgress_ && polygonClock_.isValid()
            && polygonClock_.elapsed() <= QApplication::doubleClickInterval()
            && lastDist <= kPolygonCloseRadius;
        if (closeClick || doubleClick) {
            closePolygonLasso();
            return;
        }
        if (!polygonInProgress_) {
            dragMode_ = selectionModeForModifiers(mode_, mods, v->has_selection());
            if (!v->begin_lasso(selectionModeString(dragMode_))) {
                return;
            }
            polygonInProgress_ = true;
            polygonPoints_.clear();
        }
        // ponytail: CS6's Shift 45-degree segment snap is deferred; a plain
        // click adds the vertex at the pointer.
        polygonPoints_ << imagePos;
        lastPolygonPress_ = imagePos;
        polygonClock_.restart();
        v->lasso_add_point(qRound(imagePos.x()), qRound(imagePos.y()));
        if (canvas_) {
            canvas_->setSelectionPreview({polygonPoints_}, false, /*solid=*/true);
        }
        return;
    }
    default:
        return;
    }
}

void ToolController::handleMoved(const QPointF& imagePos)
{
    if (transformSessionActive()) {
        PictureView* v = view();
        const Qt::KeyboardModifiers mods = QGuiApplication::queryKeyboardModifiers();
        if (transformDragging_ && v && canvas_) {
            v->transform_move(imagePos.x(), imagePos.y(), canvas_->zoom(),
                              mods.testFlag(Qt::ShiftModifier), mods.testFlag(Qt::AltModifier));
            updateTransformOverlay(v);
        }
        setTransformCursor(imagePos);
        return;
    }
    updateSelectionHover(imagePos);
    updateBrushOutline(imagePos);
    if (ToolHandler* h = registry_.forTool(active_)) {
        h->onMove(*this, imagePos, QGuiApplication::queryKeyboardModifiers());
        return;
    }
    if (!dragging_ && !polygonInProgress_) {
        return;
    }
    PictureView* v = view();
    if (movingSelection_) {
        dragSelectionMove(imagePos);
        return;
    }

    switch (active_) {
    case ToolId::Move: {
        totalDelta_ += imagePos - last_;
        last_ = imagePos;
        if (canvas_) {
            canvas_->setMovePreviewDelta(totalDelta_);
        }
        return;
    }
    case ToolId::Marquee:
    case ToolId::EllipticalMarquee:
        last_ = imagePos;
        updateMarqueeOverlay(imagePos);
        return;
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
            canvas_->setSelectionPreview({lassoPolygon_});
        }
        return;
    case ToolId::PolygonalLasso:
        if (polygonInProgress_ && canvas_) {
            QPolygonF preview = polygonPoints_;
            preview << imagePos;
            canvas_->setSelectionPreview({preview}, false, /*solid=*/true);
        }
        return;
    default:
        return;
    }
}

void ToolController::handleReleased(const QPointF& imagePos)
{
    if (transformSessionActive()) {
        if (transformDragging_) {
            transformDragging_ = false;
            transformHandle_ = -1;
            if (PictureView* v = view()) {
                v->transform_release();
            }
        }
        return;
    }
    if (ToolHandler* h = registry_.forTool(active_)) {
        h->onRelease(*this, imagePos, QGuiApplication::queryKeyboardModifiers());
        return;
    }
    if (!dragging_) {
        return;
    }
    dragging_ = false;
    PictureView* v = view();

    if (movingSelection_) {
        releaseSelectionMove(imagePos);
        return;
    }

    switch (active_) {
    case ToolId::Move: {
        if (v) {
            v->end_move_preview();
            const int dx = qRound(totalDelta_.x());
            const int dy = qRound(totalDelta_.y());
            if (dx != 0 || dy != 0) {
                v->commit_move(dx, dy);
            }
        }
        if (canvas_) {
            canvas_->endMovePreview();
        }
        return;
    }
    case ToolId::Marquee:
    case ToolId::EllipticalMarquee: {
        const QRect rect = marqueeDragRect(anchor_, imagePos, dragMods_);
        const bool shaped = v && rect.width() > 0 && rect.height() > 0;
        const bool committed = shaped
            && (active_ == ToolId::EllipticalMarquee
                    ? v->select_ellipse(rect.x(), rect.y(), rect.width(), rect.height(),
                                        selectionModeString(dragMode_), feather_)
                    : v->select_rect(rect.x(), rect.y(), rect.width(), rect.height(),
                                     selectionModeString(dragMode_), feather_));
        if (canvas_) {
            canvas_->clearSelectionPreview();
            canvas_->clearDragSizeHint();
        }
        if (committed) {
            emit selectionCommitted();
        }
        return;
    }
    case ToolId::Lasso: {
        const bool committed = v && v->end_lasso(feather_);
        lassoPolygon_.clear();
        if (canvas_) {
            canvas_->clearSelectionPreview();
        }
        if (committed) {
            emit selectionCommitted();
        }
        return;
    }
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

bool ToolController::commitPolygonLasso()
{
    if (!polygonInProgress_) {
        return false;
    }
    closePolygonLasso();
    return true;
}

bool ToolController::cancelPolygonLasso()
{
    if (!polygonInProgress_) {
        return false;
    }
    polygonInProgress_ = false;
    polygonPoints_.clear();
    polygonClock_.invalidate();
    if (canvas_) {
        canvas_->clearSelectionPreview();
    }
    PictureView* v = view();
    if (v) {
        v->cancel_lasso();
    }
    return true;
}

void ToolController::closePolygonLasso()
{
    PictureView* v = view();
    const bool enough = polygonPoints_.size() >= 3;
    const bool committed = enough && v && v->end_lasso(feather_);
    if (v && !committed) {
        v->cancel_lasso();
    }
    polygonInProgress_ = false;
    polygonPoints_.clear();
    polygonClock_.invalidate();
    if (canvas_) {
        canvas_->clearSelectionPreview();
    }
    if (committed) {
        emit selectionCommitted();
    }
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
