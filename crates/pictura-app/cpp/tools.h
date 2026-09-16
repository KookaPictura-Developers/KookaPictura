#pragma once

#include <QtCore/QList>
#include <QtCore/QMetaType>
#include <QtCore/QObject>
#include <QtCore/QPointF>
#include <QtCore/QRect>
#include <QtCore/QString>
#include <QtCore/Qt>
#include <QtGui/QColor>
#include <QtGui/QPolygonF>

#include <functional>
#include <utility>

namespace pictura {

class ImageView;
class PictureView;

// The 71-tool CS6 catalogue in frozen table order. The 10 implemented tools
// keep their M19 names; the enum order matches `kToolTable` in tools.cpp.
enum class ToolId {
    Move,
    Marquee,
    EllipticalMarquee,
    Lasso,
    PolygonalLasso,
    MagneticLasso,
    MagicWand,
    QuickSelection,
    Crop,
    PerspectiveCrop,
    Slice,
    SliceSelect,
    Eyedropper,
    ColorSampler,
    Ruler,
    Note,
    Count,
    SpotHealingBrush,
    HealingBrush,
    Patch,
    ContentAwareMove,
    RedEye,
    Brush,
    Pencil,
    ColorReplacement,
    MixerBrush,
    CloneStamp,
    PatternStamp,
    HistoryBrush,
    ArtHistoryBrush,
    Eraser,
    BackgroundEraser,
    MagicEraser,
    Gradient,
    PaintBucket,
    Blur,
    Sharpen,
    Smudge,
    Dodge,
    Burn,
    Sponge,
    Pen,
    FreeformPen,
    AddAnchorPoint,
    DeleteAnchorPoint,
    ConvertPoint,
    HorizontalType,
    VerticalType,
    HorizontalTypeMask,
    VerticalTypeMask,
    PathSelection,
    DirectSelection,
    Rectangle,
    RoundedRectangle,
    Ellipse,
    Polygon,
    Line,
    CustomShape,
    ObjectRotate,
    ObjectRoll,
    ObjectPan,
    ObjectSlide,
    ObjectScale,
    CameraRotate,
    CameraRoll,
    CameraPan,
    CameraWalk,
    CameraZoom,
    Hand,
    RotateView,
    Zoom
};
enum class SelectionMode { New, Add, Subtract, Intersect };

struct ToolInfo {
    ToolId id;
    const char* name;
    const char* label;
    QChar shortcut;
    Qt::CursorShape cursor;
    const char* hint;
    int group;
    bool implemented;
    int hotspotX;
    int hotspotY;
};

const ToolInfo& toolInfo(ToolId id);
const QList<ToolId>& allToolIds();
const QList<ToolId>& implementedToolIds();
bool toolImplemented(ToolId id);
QString selectionModeString(SelectionMode mode);

// Asset base name for a tool ("move", "quickselection"), used for the
// `tool.<name>` icon and cursor ids.
QString toolIdName(ToolId id);

// Routes canvas pointer events to the active tool. One switch, not one class per
// tool (see design.md); painting tools with per-tool engines can split later.
class ToolController : public QObject {
    Q_OBJECT

public:
    explicit ToolController(QObject* parent = nullptr);

    ToolId activeTool() const { return active_; }
    void setActiveTool(ToolId id);

    SelectionMode combineMode() const { return mode_; }
    void setCombineMode(SelectionMode mode);

    int tolerance() const { return tolerance_; }
    void setTolerance(int tolerance);

    int brushSize() const;
    void setBrushSize(int size);
    int brushHardness() const;
    void setBrushHardness(int h);
    int brushOpacity() const;
    void setBrushOpacity(int o);
    int brushFlow() const;
    void setBrushFlow(int f);
    QString brushMode() const;
    void setBrushMode(const QString& mode);
    bool autoErase() const;
    void setAutoErase(bool on);
    QColor foreground() const;
    void setForeground(const QColor& color);
    QColor background() const;
    void setBackground(const QColor& color);
    void adjustBrushSize(int delta);
    void adjustBrushHardness(int delta);

    void bindCanvas(ImageView* canvas);
    void unbindCanvas();
    ImageView* canvas() const { return canvas_; }

    void setViewProvider(std::function<PictureView*()> provider);

    bool hasPendingCrop() const { return hasPendingCrop_; }
    QRect pendingCropRect() const { return pendingCrop_; }
    bool commitCrop();

signals:
    void activeToolChanged(ToolId id);
    void foregroundSampled(const QColor& color);
    void selectionCommitted();

private:
    void applyToolPolicy();
    PictureView* view() const;
    void handlePressed(const QPointF& imagePos, int button, int modifiers);
    void handleMoved(const QPointF& imagePos);
    void handleReleased(const QPointF& imagePos);
    void updateDragOverlay(const QPointF& imagePos);
    static QRect dragRect(const QPointF& a, const QPointF& b);

    ImageView* canvas_ = nullptr;
    std::function<PictureView*()> viewProvider_;
    ToolId active_ = ToolId::Move;
    SelectionMode mode_ = SelectionMode::New;
    int tolerance_ = 32;

    int brushSize_ = 12;
    int brushHardness_ = 100;
    int brushOpacity_ = 100;
    int brushFlow_ = 100;
    QString brushMode_ = QStringLiteral("normal");
    bool autoErase_ = false;
    QColor foreground_{Qt::black};
    QColor background_{Qt::white};

    bool dragging_ = false;
    bool dragCommitted_ = false;
    QPointF anchor_;
    QPointF last_;
    QPointF totalDelta_;
    QPolygonF lassoPolygon_;
    QRect pendingCrop_;
    bool hasPendingCrop_ = false;
};

} // namespace pictura

Q_DECLARE_METATYPE(pictura::ToolId)
