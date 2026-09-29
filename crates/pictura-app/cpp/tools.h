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

#include "tool_context.h"
#include "tool_registry.h"

#include <algorithm>
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
// The painting brushes (the B group, the S stamps, and the History Brush): they
// share the size ring, the `[` / `]` keys, and the paint cursor policy.
bool isBrushTool(ToolId id);
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

// One context hint for the bottom status bar. `key` is the keycap label; when
// `commandId` is set the label is taken from that command's registered shortcut
// instead of the literal `key`.
struct ToolHint {
    QString key;
    QString text;
    const char* commandId = nullptr;
};

// The active tool's keycap hints, contextual per tool; empty when the tool has
// none (the status bar then falls back to the tool's text hint).
QList<ToolHint> toolHintEntries(ToolId id);

// True when the single active layer a tool edit may target carries the `PIXELS`
// lock (`LockFlags::PIXELS` = 0x02). Shared by the paint press path and the
// cursor branch; defined in tools_marquee.cpp with the other cursor helpers.
bool activePixelLocked(PictureView* view);

// Routes canvas pointer events to the active tool. Implemented tools dispatch
// through the handler registry; the legacy switch covers the rest.
class ToolController : public QObject, public ToolContext {
    Q_OBJECT

public:
    explicit ToolController(QObject* parent = nullptr);

    ToolId activeTool() const { return active_; }
    void setActiveTool(ToolId id);

    // Re-apply the active tool's cursor using the live keyboard modifiers
    // (Shift / Alt select the marquee add / remove cursor variants).
    void refreshCursor();
    // Resolve the cursor from an explicit modifier state. The live path calls
    // the no-arg overload; the offscreen self-test drives this one to check the
    // transient Alt eyedropper without a platform keyboard state.
    void refreshCursor(Qt::KeyboardModifiers mods);
    QString cursorIdForModifiersForTest(ToolId id, int mods) const;

    SelectionMode combineMode() const { return mode_; }
    void setCombineMode(SelectionMode mode);

    MarqueeStyle marqueeStyle() const override { return marqueeStyle_; }
    void setMarqueeStyle(MarqueeStyle style);

    double feather() const override { return feather_; }
    void setFeather(double feather);

    double fixedRatioWidth() const override { return fixedRatioW_; }
    double fixedRatioHeight() const override { return fixedRatioH_; }
    void setFixedRatio(double width, double height);

    int fixedSizeWidth() const override { return fixedSizeW_; }
    int fixedSizeHeight() const override { return fixedSizeH_; }
    void setFixedSize(int width, int height);

    double cropRatio() const override { return cropRatio_; }
    void setCropRatio(double ratio);
    bool cropDeletePixels() const override { return cropDeletePixels_; }
    void setCropDeletePixels(bool on) { cropDeletePixels_ = on; }
    // Esc / the options bar's Cancel: put the crop box back to the canvas.
    void cancelCrop();

    int tolerance() const override { return tolerance_; }
    void setTolerance(int tolerance);

    // Magnetic Lasso options (docs/03-tools/lasso-selection.md): Width 1-256 px,
    // Contrast 1-100 %, Frequency 0-100; CS6 defaults 10 / 10 / 57.
    int magneticWidth() const override { return magneticWidth_; }
    void setMagneticWidth(int width);
    int magneticContrast() const override { return magneticContrast_; }
    void setMagneticContrast(int contrast);
    int magneticFrequency() const override { return magneticFrequency_; }
    void setMagneticFrequency(int frequency);

    bool contiguous() const override { return contiguous_; }
    void setContiguous(bool on);
    bool antiAlias() const { return antiAlias_; }
    bool sampleAllLayers() const { return sampleAllLayers_; }

    int brushSize() const override;
    void setBrushSize(int size);
    int brushHardness() const override;
    void setBrushHardness(int h);
    int brushOpacity() const override;
    void setBrushOpacity(int o);
    int brushFlow() const override;
    void setBrushFlow(int f);
    QString brushMode() const override;
    void setBrushMode(const QString& mode);
    bool autoErase() const override;
    void setAutoErase(bool on);
    // Spot Healing Brush Type (0/1/2) and Healing Brush Aligned.
    int spotHealingType() const override { return spotHealingType_; }
    void setSpotHealingType(int type);
    bool healingAligned() const override { return healingAligned_; }
    void setHealingAligned(bool on) { healingAligned_ = on; }
    // Patch mode (Normal / Content-Aware), Source / Destination, Transparent.
    bool patchContentAware() const override { return patchContentAware_; }
    void setPatchContentAware(bool on) { patchContentAware_ = on; }
    bool patchDestination() const override { return patchDestination_; }
    void setPatchDestination(bool on) { patchDestination_ = on; }
    bool patchTransparent() const override { return patchTransparent_; }
    void setPatchTransparent(bool on) { patchTransparent_ = on; }
    // Content-Aware Move Mode (Move / Extend) and Adaptation (0..4).
    bool contentAwareMoveExtend() const override { return contentAwareMoveExtend_; }
    void setContentAwareMoveExtend(bool on) { contentAwareMoveExtend_ = on; }
    int contentAwareAdaptation() const override { return contentAwareAdaptation_; }
    void setContentAwareAdaptation(int level);
    int redEyePupil() const override { return redEyePupil_; }
    void setRedEyePupil(int pupil) { redEyePupil_ = std::clamp(pupil, 0, 100); }
    int redEyeDarken() const override { return redEyeDarken_; }
    void setRedEyeDarken(int darken) { redEyeDarken_ = std::clamp(darken, 0, 100); }
    ColorReplaceOptions colorReplaceOptions() const override { return colorReplace_; }
    void setColorReplaceOptions(const ColorReplaceOptions& options) { colorReplace_ = options; }
    MixerOptions mixerOptions() const override { return mixer_; }
    void setMixerOptions(const MixerOptions& options) { mixer_ = options; }
    QColor mixerReservoir() const override { return mixerReservoir_; }
    StampOptions stampOptions() const override { return stamp_; }
    void setStampOptions(const StampOptions& options) { stamp_ = options; }
    void setMixerReservoir(const QColor& color) override;
    QColor foreground() const override;
    void setForeground(const QColor& color);
    QColor background() const override;
    void setBackground(const QColor& color);
    void adjustBrushSize(int delta);
    void adjustBrushHardness(int delta);
    // Apply the `[`/`]` brush shortcut by Qt key value or native scan code
    // (evdev 34/35), so it works on EU/Scandinavian layouts. Guarded to the
    // Brush/Pencil tools; returns false when the key is not a brush bracket.
    bool applyBrushShortcut(int key, quint32 nativeScanCode, bool shift);

    void bindCanvas(ImageView* canvas);
    void unbindCanvas();
    ImageView* canvas() const override { return canvas_; }

    // ToolContext: the shared services a handler receives.
    PictureView* view() const override;
    void sampledForeground(const QColor& color) override;
    bool dragging() const override { return dragging_; }
    void setDragging(bool dragging) override { dragging_ = dragging; }
    bool dragCommitted() const override { return dragCommitted_; }
    void setDragCommitted(bool committed) override { dragCommitted_ = committed; }
    SelectionMode dragMode() const override { return dragMode_; }
    void setDragMode(SelectionMode mode) override { dragMode_ = mode; }
    SelectionMode resolveSelectionMode(Qt::KeyboardModifiers mods,
                                       bool hasExistingSelection) const override;
    void refused(const QString& message) override;
    void emitSelectionCommitted() override;
    void refreshAnnotations() override;
    int currentNote() const override { return currentNote_; }
    void setCurrentNote(int index) override;
    void notifyRulerChanged() override { emit rulerChanged(); }
    void notifyCountChanged() override { emit countChanged(); }
    // The options bar's Clear for the active Color Sampler, Note, or Ruler tool.
    bool clearAnnotations();
    void beginContentMove(PictureView* v, const QPointF& imagePos, bool duplicate) override;

    void setViewProvider(std::function<PictureView*()> provider);

    bool hasPendingCrop() const
    {
        ToolHandler* h = registry_.forTool(ToolId::Crop);
        return h && h->hasPendingCrop();
    }
    QRect pendingCropRect() const
    {
        ToolHandler* h = registry_.forTool(ToolId::Crop);
        return h ? h->pendingCropRect() : QRect();
    }
    bool commitCrop();

    static SelectionMode selectionModeForModifiers(SelectionMode base, Qt::KeyboardModifiers mods,
                                                   bool hasExistingSelection);
    QRect marqueeRectForTest(const QPointF& a, const QPointF& b, int mods) const;
    int dragModeForTest() const { return static_cast<int>(dragMode_); }
    int dragModsForTest() const
    {
        ToolHandler* h = registry_.forTool(active_);
        return h ? int(h->dragMods()) : 0;
    }
    // The cursor id a selection-tool drag shows, derived from the mode captured
    // at press (`dragMode_`) rather than the live keyboard state.
    QString dragCursorId() const;
    // The canvas cursor id a selection-tool hover shows under the given live
    // modifiers: the move-selection cursor when neither Shift nor Alt is held
    // and the pointer is over the selection (or Ctrl previews it); otherwise the
    // tool's modifier cursor asset (Shift -> .add, Alt -> .remove). Static so
    // the gate is testable without live keyboard state.
    static QString hoverCursorId(ToolId id, Qt::KeyboardModifiers mods, bool overSelection,
                                 bool ctrlPreview);
    bool contentMoveActiveForTest() const { return contentMove_; }

    // Polygonal Lasso interaction state. `commitPolygonLasso` closes the
    // in-progress path (Enter); `cancelPolygonLasso` discards it (Esc) and
    // returns whether anything was discarded.
    bool commitPolygonLasso();
    bool cancelPolygonLasso();
    // Delete while a click-driven lasso is open: drop its last point.
    bool removeLassoPoint();

    // Free Transform session. `beginFreeTransform` starts a session on `path`
    // and shows its overlay; commit/cancel end it. While a session is active
    // normal tool input is suspended and routed to the session.
    bool beginFreeTransform(const QString& path);
    // Begin a Skew / Distort / Perspective session on `path` (`mode` is
    // "skew", "distort", or "perspective"), showing the same overlay.
    bool beginTransformMode(const QString& path, const QString& mode);
    void commitFreeTransform();
    void cancelFreeTransform();
    bool transformSessionActive() const;

signals:
    void activeToolChanged(ToolId id);
    void brushSizeChanged(int size);
    void magneticWidthChanged(int width);
    void foregroundSampled(const QColor& color);
    void selectionCommitted();
    // A pixel edit was refused because the target layer's pixels are locked.
    void pixelEditRefused(const QString& message);
    // The selection mask moved during a move-from-inside drag; the view changed
    // without a `changed` emission, so the overlay must be refreshed directly.
    void selectionPreviewChanged();
    // The Ruler's measuring line changed (drawn, edited, cleared, or another
    // document's line is now shown); the options bar re-reads its readout.
    void rulerChanged();
    // The Note tool placed or picked note `index` (-1: none); the Notes panel
    // shows it.
    void noteActivated(int index);
    // A Count group or mark changed; the Count options bar re-reads its state.
    void countChanged();
    // The Mixer Brush's paint changed (a stroke, Load, Clean, or Alt-click).
    void mixerReservoirChanged(const QColor& color);

private:
    void applyToolPolicy();
    static bool isSelectionTool(ToolId id);
    bool maybeBeginSelectionMove(PictureView* v, const QPointF& imagePos);
    void cancelSelectionMove();
    void updateSelectionHover(const QPointF& imagePos);
    void dragSelectionMove(const QPointF& imagePos);
    void releaseSelectionMove(const QPointF& imagePos);
    void handlePressed(const QPointF& imagePos, int button, int modifiers);
    void handleMoved(const QPointF& imagePos);
    void handleReleased(const QPointF& imagePos);
    void updateBrushOutline(const QPointF& imagePos);

    void updateTransformOverlay(PictureView* v);
    void setTransformCursor(const QPointF& imagePos);
    // Shared begin: resolve the view, start `mode` (empty = Free), then set up
    // the overlay and pan policy.
    bool beginTransformImpl(const QString& path, const QString& mode);

    ToolRegistry registry_;
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
    double cropRatio_ = 0.0;
    // docs/03-tools/crop-tool.md: Delete Cropped Pixels is on by default.
    bool cropDeletePixels_ = true;
    int magneticWidth_ = 10;
    int magneticContrast_ = 10;
    int magneticFrequency_ = 57;
    bool contiguous_ = true;
    bool antiAlias_ = true;
    bool sampleAllLayers_ = true;

    int brushSize_ = 12;
    int brushHardness_ = 100;
    int brushOpacity_ = 100;
    int brushFlow_ = 100;
    QString brushMode_ = QStringLiteral("normal");
    bool autoErase_ = false;
    int spotHealingType_ = 0;
    bool healingAligned_ = true;
    bool patchContentAware_ = false;
    bool patchDestination_ = false;
    bool patchTransparent_ = false;
    bool contentAwareMoveExtend_ = false;
    int contentAwareAdaptation_ = 2;
    int redEyePupil_ = 50;
    int redEyeDarken_ = 50;
    ColorReplaceOptions colorReplace_;
    MixerOptions mixer_;
    QColor mixerReservoir_{Qt::black};
    StampOptions stamp_;
    QColor foreground_{Qt::black};
    QColor background_{Qt::white};

    bool dragging_ = false;
    bool dragCommitted_ = false;
    bool movingSelection_ = false;
    bool contentMove_ = false;
    bool contentDuplicate_ = false;
    // True while a duplicate content move is showing the pre-cloned pixel
    // preview on the canvas; cleared on release/cancel/tool switch.
    bool contentPreviewActive_ = false;
    bool cursorOverSelection_ = false;
    // The press point for a selection/content move, owned by the controller
    // because the routing is cross-cutting; a tool handler's own drag anchor is
    // private to that handler.
    QPointF selectionMoveAnchor_;

    bool transformDragging_ = false;
    int transformHandle_ = -1;
    int currentNote_ = -1;
};

} // namespace pictura

Q_DECLARE_METATYPE(pictura::ToolId)
