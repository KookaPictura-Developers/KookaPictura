#include "tools.h"

#include "icons.h"
#include "image_view.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <algorithm>

namespace pictura {

namespace {

const ToolInfo kToolTable[] = {
    {ToolId::Move, "move", "Move", QLatin1Char('V'), Qt::SizeAllCursor,
     "Move: drag to move the active layer", 1, true, 2, 2},
    {ToolId::Marquee, "marquee", "Rectangular Marquee", QLatin1Char('M'), Qt::CrossCursor,
     "Marquee: drag to select a rectangle", 2, true, 12, 12},
    {ToolId::EllipticalMarquee, "ellipticalmarquee", "Elliptical Marquee", QLatin1Char('M'),
     Qt::CrossCursor, "Elliptical Marquee: not implemented yet", 2, false, 12, 12},
    {ToolId::Lasso, "lasso", "Lasso", QLatin1Char('L'), Qt::CrossCursor,
     "Lasso: drag around a region to select", 3, true, 12, 12},
    {ToolId::PolygonalLasso, "polygonallasso", "Polygonal Lasso", QLatin1Char('L'),
     Qt::CrossCursor, "Polygonal Lasso: not implemented yet", 3, false, 12, 12},
    {ToolId::MagneticLasso, "magneticlasso", "Magnetic Lasso", QLatin1Char('L'),
     Qt::CrossCursor, "Magnetic Lasso: not implemented yet", 3, false, 12, 12},
    {ToolId::MagicWand, "magicwand", "Magic Wand", QLatin1Char('W'), Qt::CrossCursor,
     "Magic Wand: not implemented yet", 4, false, 12, 12},
    {ToolId::QuickSelection, "quickselection", "Quick Selection", QLatin1Char('W'),
     Qt::CrossCursor, "Quick Selection: drag to grow a selection", 4, true, 12, 12},
    {ToolId::Crop, "crop", "Crop", QLatin1Char('C'), Qt::CrossCursor,
     "Crop: drag a region, press Enter to commit", 5, true, 12, 12},
    {ToolId::PerspectiveCrop, "perspectivecrop", "Perspective Crop", QLatin1Char('C'),
     Qt::CrossCursor, "Perspective Crop: not implemented yet", 5, false, 12, 12},
    {ToolId::Slice, "slice", "Slice", QLatin1Char('C'), Qt::CrossCursor,
     "Slice: not implemented yet", 5, false, 12, 12},
    {ToolId::SliceSelect, "sliceselect", "Slice Select", QLatin1Char('C'), Qt::CrossCursor,
     "Slice Select: not implemented yet", 5, false, 12, 12},
    {ToolId::Eyedropper, "eyedropper", "Eyedropper", QLatin1Char('I'), Qt::CrossCursor,
     "Eyedropper: click to sample a colour", 6, true, 2, 22},
    {ToolId::ColorSampler, "colorsampler", "Color Sampler", QLatin1Char('I'), Qt::CrossCursor,
     "Color Sampler: not implemented yet", 6, false, 2, 22},
    {ToolId::Ruler, "ruler", "Ruler", QLatin1Char('I'), Qt::CrossCursor,
     "Ruler: not implemented yet", 6, false, 2, 22},
    {ToolId::Note, "note", "Note", QLatin1Char('I'), Qt::CrossCursor,
     "Note: not implemented yet", 6, false, 2, 22},
    {ToolId::Count, "count", "Count (Extended)", QLatin1Char('I'), Qt::CrossCursor,
     "Count (Extended): not implemented yet", 6, false, 12, 12},
    {ToolId::SpotHealingBrush, "spothealingbrush", "Spot Healing Brush", QLatin1Char('J'),
     Qt::CrossCursor, "Spot Healing Brush: not implemented yet", 7, false, 2, 22},
    {ToolId::HealingBrush, "healingbrush", "Healing Brush", QLatin1Char('J'), Qt::CrossCursor,
     "Healing Brush: not implemented yet", 7, false, 2, 22},
    {ToolId::Patch, "patch", "Patch", QLatin1Char('J'), Qt::CrossCursor,
     "Patch: not implemented yet", 7, false, 12, 12},
    {ToolId::ContentAwareMove, "contentawaremove", "Content-Aware Move", QLatin1Char('J'),
     Qt::CrossCursor, "Content-Aware Move: not implemented yet", 7, false, 12, 12},
    {ToolId::RedEye, "redeye", "Red Eye", QLatin1Char('J'), Qt::CrossCursor,
     "Red Eye: not implemented yet", 7, false, 12, 12},
    {ToolId::Brush, "brush", "Brush", QLatin1Char('B'), Qt::CrossCursor,
     "Brush: drag to paint the foreground colour", 8, true, 2, 22},
    {ToolId::Pencil, "pencil", "Pencil", QLatin1Char('B'), Qt::CrossCursor,
     "Pencil: drag to paint a hard aliased line", 8, true, 2, 22},
    {ToolId::ColorReplacement, "colorreplacement", "Color Replacement", QLatin1Char('B'),
     Qt::CrossCursor, "Color Replacement: not implemented yet", 8, false, 2, 22},
    {ToolId::MixerBrush, "mixerbrush", "Mixer Brush", QLatin1Char('B'), Qt::CrossCursor,
     "Mixer Brush: not implemented yet", 8, false, 2, 22},
    {ToolId::CloneStamp, "clonestamp", "Clone Stamp", QLatin1Char('S'), Qt::CrossCursor,
     "Clone Stamp: not implemented yet", 9, false, 2, 22},
    {ToolId::PatternStamp, "patternstamp", "Pattern Stamp", QLatin1Char('S'), Qt::CrossCursor,
     "Pattern Stamp: not implemented yet", 9, false, 2, 22},
    {ToolId::HistoryBrush, "historybrush", "History Brush", QLatin1Char('Y'), Qt::CrossCursor,
     "History Brush: not implemented yet", 10, false, 2, 22},
    {ToolId::ArtHistoryBrush, "arthistorybrush", "Art History Brush", QLatin1Char('Y'),
     Qt::CrossCursor, "Art History Brush: not implemented yet", 10, false, 2, 22},
    {ToolId::Eraser, "eraser", "Eraser", QLatin1Char('E'), Qt::CrossCursor,
     "Eraser: not implemented yet", 11, false, 2, 22},
    {ToolId::BackgroundEraser, "backgrounderaser", "Background Eraser", QLatin1Char('E'),
     Qt::CrossCursor, "Background Eraser: not implemented yet", 11, false, 2, 22},
    {ToolId::MagicEraser, "magiceraser", "Magic Eraser", QLatin1Char('E'), Qt::CrossCursor,
     "Magic Eraser: not implemented yet", 11, false, 2, 22},
    {ToolId::Gradient, "gradient", "Gradient", QLatin1Char('G'), Qt::CrossCursor,
     "Gradient: not implemented yet", 12, false, 2, 22},
    {ToolId::PaintBucket, "paintbucket", "Paint Bucket", QLatin1Char('G'), Qt::CrossCursor,
     "Paint Bucket: not implemented yet", 12, false, 2, 22},
    {ToolId::Blur, "blur", "Blur", QChar(), Qt::CrossCursor, "Blur: not implemented yet", 13,
     false, 2, 22},
    {ToolId::Sharpen, "sharpen", "Sharpen", QChar(), Qt::CrossCursor,
     "Sharpen: not implemented yet", 13, false, 2, 22},
    {ToolId::Smudge, "smudge", "Smudge", QChar(), Qt::CrossCursor,
     "Smudge: not implemented yet", 13, false, 2, 22},
    {ToolId::Dodge, "dodge", "Dodge", QLatin1Char('O'), Qt::CrossCursor,
     "Dodge: not implemented yet", 14, false, 2, 22},
    {ToolId::Burn, "burn", "Burn", QLatin1Char('O'), Qt::CrossCursor,
     "Burn: not implemented yet", 14, false, 2, 22},
    {ToolId::Sponge, "sponge", "Sponge", QLatin1Char('O'), Qt::CrossCursor,
     "Sponge: not implemented yet", 14, false, 2, 22},
    {ToolId::Pen, "pen", "Pen", QLatin1Char('P'), Qt::CrossCursor,
     "Pen: not implemented yet", 15, false, 2, 2},
    {ToolId::FreeformPen, "freeformpen", "Freeform Pen", QLatin1Char('P'), Qt::CrossCursor,
     "Freeform Pen: not implemented yet", 15, false, 2, 2},
    {ToolId::AddAnchorPoint, "addanchorpoint", "Add Anchor Point", QChar(), Qt::CrossCursor,
     "Add Anchor Point: not implemented yet", 15, false, 2, 2},
    {ToolId::DeleteAnchorPoint, "deleteanchorpoint", "Delete Anchor Point", QChar(),
     Qt::CrossCursor, "Delete Anchor Point: not implemented yet", 15, false, 2, 2},
    {ToolId::ConvertPoint, "convertpoint", "Convert Point", QChar(), Qt::CrossCursor,
     "Convert Point: not implemented yet", 15, false, 2, 2},
    {ToolId::HorizontalType, "horizontaltype", "Horizontal Type", QLatin1Char('T'),
     Qt::IBeamCursor, "Horizontal Type: not implemented yet", 16, false, 12, 12},
    {ToolId::VerticalType, "verticaltype", "Vertical Type", QLatin1Char('T'),
     Qt::IBeamCursor, "Vertical Type: not implemented yet", 16, false, 12, 12},
    {ToolId::HorizontalTypeMask, "horizontaltypemask", "Horizontal Type Mask", QLatin1Char('T'),
     Qt::IBeamCursor, "Horizontal Type Mask: not implemented yet", 16, false, 12, 12},
    {ToolId::VerticalTypeMask, "verticaltypemask", "Vertical Type Mask", QLatin1Char('T'),
     Qt::IBeamCursor, "Vertical Type Mask: not implemented yet", 16, false, 12, 12},
    {ToolId::PathSelection, "pathselection", "Path Selection", QLatin1Char('A'),
     Qt::CrossCursor, "Path Selection: not implemented yet", 17, false, 12, 12},
    {ToolId::DirectSelection, "directselection", "Direct Selection", QLatin1Char('A'),
     Qt::CrossCursor, "Direct Selection: not implemented yet", 17, false, 12, 12},
    {ToolId::Rectangle, "rectangle", "Rectangle", QLatin1Char('U'), Qt::CrossCursor,
     "Rectangle: not implemented yet", 18, false, 12, 12},
    {ToolId::RoundedRectangle, "roundedrectangle", "Rounded Rectangle", QLatin1Char('U'),
     Qt::CrossCursor, "Rounded Rectangle: not implemented yet", 18, false, 12, 12},
    {ToolId::Ellipse, "ellipse", "Ellipse", QLatin1Char('U'), Qt::CrossCursor,
     "Ellipse: not implemented yet", 18, false, 12, 12},
    {ToolId::Polygon, "polygon", "Polygon", QLatin1Char('U'), Qt::CrossCursor,
     "Polygon: not implemented yet", 18, false, 12, 12},
    {ToolId::Line, "line", "Line", QLatin1Char('U'), Qt::CrossCursor,
     "Line: not implemented yet", 18, false, 12, 12},
    {ToolId::CustomShape, "customshape", "Custom Shape", QLatin1Char('U'), Qt::CrossCursor,
     "Custom Shape: not implemented yet", 18, false, 12, 12},
    {ToolId::ObjectRotate, "objectrotate", "Object Rotate (Extended)", QLatin1Char('K'),
     Qt::CrossCursor, "Object Rotate (Extended): not implemented yet", 19, false, 12, 12},
    {ToolId::ObjectRoll, "objectroll", "Object Roll (Extended)", QLatin1Char('K'),
     Qt::CrossCursor, "Object Roll (Extended): not implemented yet", 19, false, 12, 12},
    {ToolId::ObjectPan, "objectpan", "Object Pan (Extended)", QLatin1Char('K'),
     Qt::CrossCursor, "Object Pan (Extended): not implemented yet", 19, false, 12, 12},
    {ToolId::ObjectSlide, "objectslide", "Object Slide (Extended)", QLatin1Char('K'),
     Qt::CrossCursor, "Object Slide (Extended): not implemented yet", 19, false, 12, 12},
    {ToolId::ObjectScale, "objectscale", "Object Scale (Extended)", QLatin1Char('K'),
     Qt::CrossCursor, "Object Scale (Extended): not implemented yet", 19, false, 12, 12},
    {ToolId::CameraRotate, "camerarotate", "Camera Rotate (Extended)", QLatin1Char('N'),
     Qt::CrossCursor, "Camera Rotate (Extended): not implemented yet", 20, false, 12, 12},
    {ToolId::CameraRoll, "cameraroll", "Camera Roll (Extended)", QLatin1Char('N'),
     Qt::CrossCursor, "Camera Roll (Extended): not implemented yet", 20, false, 12, 12},
    {ToolId::CameraPan, "camerapan", "Camera Pan (Extended)", QLatin1Char('N'),
     Qt::CrossCursor, "Camera Pan (Extended): not implemented yet", 20, false, 12, 12},
    {ToolId::CameraWalk, "camerawalk", "Camera Walk (Extended)", QLatin1Char('N'),
     Qt::CrossCursor, "Camera Walk (Extended): not implemented yet", 20, false, 12, 12},
    {ToolId::CameraZoom, "camerazoom", "Camera Zoom (Extended)", QLatin1Char('N'),
     Qt::CrossCursor, "Camera Zoom (Extended): not implemented yet", 20, false, 12, 12},
    {ToolId::Hand, "hand", "Hand", QLatin1Char('H'), Qt::OpenHandCursor,
     "Hand: drag to pan the canvas", 21, true, 9, 2},
    {ToolId::RotateView, "rotateview", "Rotate View", QLatin1Char('R'), Qt::CrossCursor,
     "Rotate View: not implemented yet", 22, false, 12, 12},
    {ToolId::Zoom, "zoom", "Zoom", QLatin1Char('Z'), Qt::CrossCursor,
     "Zoom: click to zoom in, Ctrl/Alt-click to zoom out", 23, true, 9, 2},
};
constexpr int kToolCount = int(sizeof(kToolTable) / sizeof(kToolTable[0]));
static_assert(kToolCount == 71, "tool table must cover every ToolId");

int toolIndex(ToolId id) { return static_cast<int>(id); }

} // namespace

const ToolInfo& toolInfo(ToolId id)
{
    const int index = toolIndex(id);
    return (index >= 0 && index < kToolCount) ? kToolTable[index] : kToolTable[0];
}

bool toolImplemented(ToolId id) { return toolInfo(id).implemented; }

QString toolIdName(ToolId id)
{
    const int index = toolIndex(id);
    return QString::fromLatin1(
        (index >= 0 && index < kToolCount) ? kToolTable[index].name : kToolTable[0].name);
}

QList<QChar> toolShortcutKeys()
{
    QList<QChar> keys;
    for (ToolId id : allToolIds()) {
        const QChar key = toolInfo(id).shortcut;
        if (!key.isNull() && !keys.contains(key)) {
            keys << key;
        }
    }
    return keys;
}

int toolGroupForKey(QChar key)
{
    if (key.isNull()) {
        return 0;
    }
    const QChar upper = key.toUpper();
    for (ToolId id : allToolIds()) {
        const QChar shortcut = toolInfo(id).shortcut;
        if (!shortcut.isNull() && shortcut.toUpper() == upper) {
            return toolInfo(id).group;
        }
    }
    return 0;
}

const QList<ToolId>& allToolIds()
{
    static const QList<ToolId> ids = {
        ToolId::Move,             ToolId::Marquee,          ToolId::EllipticalMarquee,
        ToolId::Lasso,            ToolId::PolygonalLasso,   ToolId::MagneticLasso,
        ToolId::MagicWand,        ToolId::QuickSelection,   ToolId::Crop,
        ToolId::PerspectiveCrop,  ToolId::Slice,            ToolId::SliceSelect,
        ToolId::Eyedropper,       ToolId::ColorSampler,     ToolId::Ruler,
        ToolId::Note,             ToolId::Count,            ToolId::SpotHealingBrush,
        ToolId::HealingBrush,     ToolId::Patch,            ToolId::ContentAwareMove,
        ToolId::RedEye,           ToolId::Brush,            ToolId::Pencil,
        ToolId::ColorReplacement, ToolId::MixerBrush,       ToolId::CloneStamp,
        ToolId::PatternStamp,     ToolId::HistoryBrush,     ToolId::ArtHistoryBrush,
        ToolId::Eraser,           ToolId::BackgroundEraser, ToolId::MagicEraser,
        ToolId::Gradient,         ToolId::PaintBucket,      ToolId::Blur,
        ToolId::Sharpen,          ToolId::Smudge,           ToolId::Dodge,
        ToolId::Burn,             ToolId::Sponge,           ToolId::Pen,
        ToolId::FreeformPen,      ToolId::AddAnchorPoint,   ToolId::DeleteAnchorPoint,
        ToolId::ConvertPoint,     ToolId::HorizontalType,   ToolId::VerticalType,
        ToolId::HorizontalTypeMask, ToolId::VerticalTypeMask, ToolId::PathSelection,
        ToolId::DirectSelection,  ToolId::Rectangle,        ToolId::RoundedRectangle,
        ToolId::Ellipse,          ToolId::Polygon,          ToolId::Line,
        ToolId::CustomShape,      ToolId::ObjectRotate,     ToolId::ObjectRoll,
        ToolId::ObjectPan,        ToolId::ObjectSlide,      ToolId::ObjectScale,
        ToolId::CameraRotate,     ToolId::CameraRoll,       ToolId::CameraPan,
        ToolId::CameraWalk,       ToolId::CameraZoom,       ToolId::Hand,
        ToolId::RotateView,       ToolId::Zoom,
    };
    return ids;
}

const QList<ToolId>& implementedToolIds()
{
    static const QList<ToolId> ids = {
        ToolId::Move,   ToolId::Marquee, ToolId::Lasso, ToolId::QuickSelection,
        ToolId::Crop,   ToolId::Eyedropper, ToolId::Hand, ToolId::Zoom,
        ToolId::Brush,  ToolId::Pencil,
    };
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
    if (!toolImplemented(id)) {
        return;
    }
    if (active_ == id) {
        return;
    }
    active_ = id;
    PictureView* v = view();
    if (v && v->is_painting()) {
        v->cancel_paint();
    }
    dragging_ = false;
    dragCommitted_ = false;
    if (canvas_ && canvas_->movePreviewActive()) {
        canvas_->endMovePreview();
        if (v) {
            v->end_move_preview();
        }
    }
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

int ToolController::brushSize() const { return brushSize_; }

void ToolController::setBrushSize(int size) { brushSize_ = std::clamp(size, 1, 5000); }

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
    if (canvas_->movePreviewActive()) {
        canvas_->endMovePreview();
        PictureView* previewView = view();
        if (previewView) {
            previewView->end_move_preview();
        }
    }
    canvas_->clearOverlay();
    canvas_ = nullptr;
    PictureView* v = view();
    if (v && v->is_painting()) {
        v->cancel_paint();
    }
    dragging_ = false;
    dragCommitted_ = false;
}

void ToolController::applyToolPolicy()
{
    if (!canvas_) {
        return;
    }
    canvas_->setPanEnabled(active_ == ToolId::Hand);
    const ToolInfo& info = toolInfo(active_);
    const QCursor toolCursor =
        cursor(QStringLiteral("tool.") + toolIdName(active_), info.hotspotX, info.hotspotY);
    canvas_->setCursor(
        toolCursor.pixmap().isNull() ? QCursor(info.cursor) : toolCursor);
    if (active_ == ToolId::Move) {
        PictureView* v = view();
        if (v) {
            v->prepare_move_preview();
        }
    }
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
    case ToolId::Brush:
    case ToolId::Pencil: {
        if (!v) {
            return;
        }
        const bool aliased = active_ == ToolId::Pencil;
        if (!v->begin_paint(foreground_.rgba(), background_.rgba(), brushSize_, brushHardness_,
                            100, 0, brushOpacity_, brushFlow_, 25, brushMode_, aliased,
                            autoErase_)) {
            return;
        }
        dragging_ = true;
        dragCommitted_ = false;
        v->paint_dab(imagePos.x(), imagePos.y(), 1.0);
        return;
    }
    case ToolId::Move:
        if (!v || !v->begin_move_preview()) {
            return;
        }
        dragging_ = true;
        dragCommitted_ = false;
        anchor_ = last_ = imagePos;
        totalDelta_ = QPointF();
        if (canvas_) {
            canvas_->beginMovePreview(
                v->move_preview_base(), v->move_preview_layer(),
                QPointF(v->move_preview_x(), v->move_preview_y()),
                v->move_preview_opacity() / 255.0);
        }
        return;
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
    default:
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
    case ToolId::Move: {
        totalDelta_ += imagePos - last_;
        last_ = imagePos;
        if (canvas_) {
            canvas_->setMovePreviewDelta(totalDelta_);
        }
        return;
    }
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
    case ToolId::Brush:
    case ToolId::Pencil:
        if (v) {
            v->paint_dab(imagePos.x(), imagePos.y(), 1.0);
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
    case ToolId::Brush:
    case ToolId::Pencil:
        if (v) {
            v->end_paint();
        }
        return;
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
