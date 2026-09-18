#pragma once

#include <QtCore/QElapsedTimer>
#include <QtCore/QList>
#include <QtCore/QMetaType>
#include <QtCore/QObject>
#include <QtCore/QPointF>
#include <QtCore/QRect>
#include <QtCore/QString>
#include <QtCore/Qt>
#include <QtGui/QColor>
#include <QtGui/QImage>
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
enum class MarqueeStyle { Normal, FixedRatio, FixedSize };

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

// Cursor asset id for a tool under the given modifiers. The marquee tools swap
// to the `.add` / `.remove` variants for Shift / Alt; every other tool keeps
// its plain `tool.<name>` asset.
QString toolCursorId(ToolId id, Qt::KeyboardModifiers mods);

// The distinct slot letters in catalogue order, and the group a letter maps to
// (0 when no tool carries it). Every letter maps to exactly one group.
QList<QChar> toolShortcutKeys();
int toolGroupForKey(QChar key);

// Routes canvas pointer events to the active tool. One switch, not one class per
// tool (see design.md); painting tools with per-tool engines can split later.
class ToolController : public QObject {
    Q_OBJECT

public:
    explicit ToolController(QObject* parent = nullptr);

    ToolId activeTool() const { return active_; }
    void setActiveTool(ToolId id);

    // Re-apply the active tool's cursor using the live keyboard modifiers
    // (Shift / Alt select the marquee add / remove cursor variants).
    void refreshCursor();
    QString cursorIdForModifiersForTest(ToolId id, int mods) const;

    SelectionMode combineMode() const { return mode_; }
    void setCombineMode(SelectionMode mode);

    MarqueeStyle marqueeStyle() const { return marqueeStyle_; }
    void setMarqueeStyle(MarqueeStyle style);

    double feather() const { return feather_; }
    void setFeather(double feather);

    double fixedRatioWidth() const { return fixedRatioW_; }
    double fixedRatioHeight() const { return fixedRatioH_; }
    void setFixedRatio(double width, double height);

    int fixedSizeWidth() const { return fixedSizeW_; }
    int fixedSizeHeight() const { return fixedSizeH_; }
    void setFixedSize(int width, int height);

    int tolerance() const { return tolerance_; }
    void setTolerance(int tolerance);

    bool contiguous() const { return contiguous_; }
    void setContiguous(bool on);
    bool antiAlias() const { return antiAlias_; }
    bool sampleAllLayers() const { return sampleAllLayers_; }

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

    static SelectionMode selectionModeForModifiers(SelectionMode base, Qt::KeyboardModifiers mods,
                                                   bool hasExistingSelection);
    QRect marqueeRectForTest(const QPointF& a, const QPointF& b, int mods) const;
    int dragModeForTest() const { return static_cast<int>(dragMode_); }
    bool contentMoveActiveForTest() const { return contentMove_; }

    // Polygonal Lasso interaction state. `commitPolygonLasso` closes the
    // in-progress path (Enter); `cancelPolygonLasso` discards it (Esc) and
    // returns whether anything was discarded.
    bool commitPolygonLasso();
    bool cancelPolygonLasso();

signals:
    void activeToolChanged(ToolId id);
    void foregroundSampled(const QColor& color);
    void selectionCommitted();
    // The selection mask moved during a move-from-inside drag; the view changed
    // without a `changed` emission, so the overlay must be refreshed directly.
    void selectionPreviewChanged();

private:
    void applyToolPolicy();
    void warmMovePreview();
    PictureView* view() const;
    static bool isSelectionTool(ToolId id);
    bool maybeBeginSelectionMove(PictureView* v, const QPointF& imagePos);
    void beginContentMove(PictureView* v, const QPointF& imagePos, bool duplicate);
    void cancelSelectionMove();
    void updateSelectionHover(const QPointF& imagePos);
    void dragSelectionMove(const QPointF& imagePos);
    void releaseSelectionMove(const QPointF& imagePos);
    void handlePressed(const QPointF& imagePos, int button, int modifiers);
    void handleMoved(const QPointF& imagePos);
    void handleReleased(const QPointF& imagePos);
    void updateDragOverlay(const QPointF& imagePos);
    void updateMarqueeOverlay(const QPointF& imagePos);
    void closePolygonLasso();
    QRect marqueeDragRect(const QPointF& a, const QPointF& b, Qt::KeyboardModifiers mods) const;
    static QRect dragRect(const QPointF& a, const QPointF& b);

    ImageView* canvas_ = nullptr;
    std::function<PictureView*()> viewProvider_;
    ToolId active_ = ToolId::Move;
    SelectionMode mode_ = SelectionMode::New;
    SelectionMode dragMode_ = SelectionMode::New;
    MarqueeStyle marqueeStyle_ = MarqueeStyle::Normal;
    double feather_ = 0.0;
    double fixedRatioW_ = 1.0;
    double fixedRatioH_ = 1.0;
    int fixedSizeW_ = 100;
    int fixedSizeH_ = 100;
    int tolerance_ = 32;
    bool contiguous_ = true;
    bool antiAlias_ = true;
    bool sampleAllLayers_ = true;

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
    bool movingSelection_ = false;
    bool contentMove_ = false;
    bool contentDuplicate_ = false;
    bool cursorOverSelection_ = false;
    QPointF anchor_;
    QPointF last_;
    QPointF totalDelta_;
    QPolygonF lassoPolygon_;
    QPolygonF polygonPoints_;
    QPointF lastPolygonPress_;
    QElapsedTimer polygonClock_;
    bool polygonInProgress_ = false;
    QRect pendingCrop_;
    bool hasPendingCrop_ = false;

    PictureView* warmView_ = nullptr;
    QImage warmBase_;
    QImage warmLayer_;
    bool warmValid_ = false;
};

} // namespace pictura

Q_DECLARE_METATYPE(pictura::ToolId)
