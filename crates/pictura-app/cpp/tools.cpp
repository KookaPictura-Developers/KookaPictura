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
std::unique_ptr<ToolHandler> makeMoveToolHandler();
std::unique_ptr<ToolHandler> makeCropToolHandler();
std::unique_ptr<ToolHandler> makeMarqueeToolHandler();
std::unique_ptr<ToolHandler> makeEllipticalMarqueeToolHandler();
std::unique_ptr<ToolHandler> makeLassoToolHandler();
std::unique_ptr<ToolHandler> makePolygonalLassoToolHandler();

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
    registry_.registerTool(ToolId::Move, makeMoveToolHandler());
    registry_.registerTool(ToolId::Crop, makeCropToolHandler());
    registry_.registerTool(ToolId::Marquee, makeMarqueeToolHandler());
    registry_.registerTool(ToolId::EllipticalMarquee, makeEllipticalMarqueeToolHandler());
    registry_.registerTool(ToolId::Lasso, makeLassoToolHandler());
    registry_.registerTool(ToolId::PolygonalLasso, makePolygonalLassoToolHandler());
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
    // Drop the outgoing handler's per-tool state (in-progress polygon, warm
    // preview) before the switch, while the old tool is still active.
    if (ToolHandler* old = registry_.forTool(active_)) {
        old->onDeactivate(*this);
    }
    active_ = id;
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
    // Let the active handler release anything tied to this canvas (an
    // in-progress polygon, the warm Move preview) before it goes away.
    if (ToolHandler* h = registry_.forTool(active_)) {
        h->onDeactivate(*this);
    }
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
    PictureView* v = view();
    if (v && v->is_painting()) {
        v->cancel_paint();
    }
    dragging_ = false;
    dragCommitted_ = false;
    cancelSelectionMove();
    cursorOverSelection_ = false;
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
    if (ToolHandler* h = registry_.forTool(active_)) {
        h->onActivate(*this);
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
        // A press inside a live selection moves the mask or its content instead
        // of starting a new shape; the selection handler never sees the event.
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
        h->onPress(*this, imagePos, mods);
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
    // The selection/content move is cross-cutting: route it before the active
    // tool handler so the shape tools do not also consume the move.
    if (movingSelection_) {
        dragSelectionMove(imagePos);
        return;
    }
    if (ToolHandler* h = registry_.forTool(active_)) {
        h->onMove(*this, imagePos, QGuiApplication::queryKeyboardModifiers());
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
    if (movingSelection_) {
        dragging_ = false;
        releaseSelectionMove(imagePos);
        return;
    }
    if (ToolHandler* h = registry_.forTool(active_)) {
        h->onRelease(*this, imagePos, QGuiApplication::queryKeyboardModifiers());
    }
}

bool ToolController::commitPolygonLasso()
{
    if (ToolHandler* h = registry_.forTool(active_)) {
        return h->commitPolygonLasso();
    }
    return false;
}

bool ToolController::cancelPolygonLasso()
{
    if (ToolHandler* h = registry_.forTool(active_)) {
        return h->cancelPolygonLasso();
    }
    return false;
}

bool ToolController::commitCrop()
{
    // The staged crop survives a tool switch, so reach the Crop handler
    // directly rather than through whatever is active now.
    if (ToolHandler* h = registry_.forTool(ToolId::Crop)) {
        return h->commitCrop();
    }
    return false;
}

} // namespace pictura
