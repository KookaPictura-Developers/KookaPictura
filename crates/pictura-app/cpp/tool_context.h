#pragma once

#include <QtCore/QPointF>
#include <QtCore/QString>
#include <QtCore/Qt>
#include <QtGui/QColor>

namespace pictura {

class ImageView;
class PictureView;
enum class SelectionMode;
enum class MarqueeStyle;

// The Color Replacement options bar: Mode 0 Hue … 3 Luminosity (2 Color),
// Sampling 0 Continuous / 1 Once / 2 Background Swatch, Limits
// 0 Discontiguous / 1 Contiguous / 2 Find Edges, Tolerance 0-100 %.
struct ColorReplaceOptions {
    int mode = 2;
    int sampling = 0;
    int limits = 1;
    int tolerance = 30;
    bool antialias = true;
};

// The Mixer Brush options bar (all 0-100 %) and its after-stroke toggles.
// Defaults are photorust's "Dry" preset; the spec leaves CS6's open.
struct MixerOptions {
    int wet = 0;
    int load = 50;
    int mix = 0;
    int flow = 100;
    bool loadAfterStroke = false;
    bool cleanAfterStroke = false;
};

// The Clone Stamp and Pattern Stamp options bars. Sample 0 Current Layer /
// 1 Current And Below / 2 All Layers; Ignore Adjustment Layers applies to All
// Layers only. `pattern` indexes the built-in patterns (stamp_pattern_name).
struct StampOptions {
    bool cloneAligned = true;
    int cloneSample = 0;
    bool ignoreAdjustments = false;
    int pattern = 0;
    bool patternAligned = true;
};

// The shared services a tool handler may use, implemented by `ToolController`.
// Kept minimal on purpose: add accessors only as a migrating tool needs them.
struct ToolContext {
    virtual ~ToolContext() = default;
    virtual PictureView* view() const = 0;
    virtual ImageView* canvas() const = 0;
    virtual void sampledForeground(const QColor& color) = 0;

    virtual QColor foreground() const = 0;
    virtual QColor background() const = 0;
    virtual int brushSize() const = 0;
    virtual int brushHardness() const = 0;
    virtual int brushOpacity() const = 0;
    virtual int brushFlow() const = 0;
    virtual QString brushMode() const = 0;
    virtual bool autoErase() const = 0;
    virtual int tolerance() const = 0;
    virtual bool contiguous() const = 0;

    // Spot Healing Brush Type: 0 Proximity Match, 1 Create Texture,
    // 2 Content-Aware. Healing Brush Aligned: keep the sample offset across
    // strokes instead of re-anchoring it to each stroke's start.
    virtual int spotHealingType() const = 0;
    virtual bool healingAligned() const = 0;
    // Patch: Content-Aware rebuilds the selection in place and ignores the
    // drag; Destination applies the selection where it is dragged; Transparent
    // transfers texture only.
    virtual bool patchContentAware() const = 0;
    virtual bool patchDestination() const = 0;
    virtual bool patchTransparent() const = 0;
    // Content-Aware Move: Extend copies instead of moving; Adaptation is
    // 0 Very Strict … 4 Very Loose (2 Medium, the default).
    virtual bool contentAwareMoveExtend() const = 0;
    virtual int contentAwareAdaptation() const = 0;
    // Red Eye: Pupil Size and Darken Amount, 0-100 % (CS6 defaults 50 / 50).
    virtual int redEyePupil() const = 0;
    virtual int redEyeDarken() const = 0;
    virtual ColorReplaceOptions colorReplaceOptions() const = 0;
    virtual MixerOptions mixerOptions() const = 0;
    // The paint on the Mixer Brush (alpha 0: clean); it outlives each stroke.
    virtual QColor mixerReservoir() const = 0;
    virtual void setMixerReservoir(const QColor& color) = 0;
    virtual StampOptions stampOptions() const = 0;

    virtual MarqueeStyle marqueeStyle() const = 0;
    virtual double fixedRatioWidth() const = 0;
    virtual double fixedRatioHeight() const = 0;
    virtual int fixedSizeWidth() const = 0;
    virtual int fixedSizeHeight() const = 0;
    virtual double feather() const = 0;
    // Crop options: aspect ratio (width / height; 0 = unconstrained) and
    // whether a crop discards the pixels outside the canvas.
    virtual double cropRatio() const = 0;
    virtual bool cropDeletePixels() const = 0;
    virtual int magneticWidth() const = 0;
    virtual int magneticContrast() const = 0;
    virtual int magneticFrequency() const = 0;

    // The controller's selection-move service: start a mask/content translate
    // from `imagePos`. The Move handler and the selection pre-block share it.
    virtual void beginContentMove(PictureView* v, const QPointF& imagePos, bool duplicate) = 0;

    virtual bool dragging() const = 0;
    virtual void setDragging(bool dragging) = 0;
    virtual bool dragCommitted() const = 0;
    virtual void setDragCommitted(bool committed) = 0;
    virtual SelectionMode dragMode() const = 0;
    virtual void setDragMode(SelectionMode mode) = 0;
    virtual SelectionMode resolveSelectionMode(Qt::KeyboardModifiers mods,
                                               bool hasExistingSelection) const = 0;

    // Annotations: re-read the document's color samplers and notes onto the
    // canvas; the note shown in the Notes panel (-1 none); the Ruler's line
    // changed, so its readouts must update.
    virtual void refreshAnnotations() = 0;
    virtual int currentNote() const = 0;
    virtual void setCurrentNote(int index) = 0;
    virtual void notifyRulerChanged() = 0;
    // A Count group or mark changed; the options bar re-reads its state.
    virtual void notifyCountChanged() = 0;

    virtual void refused(const QString& message) = 0;
    virtual void emitSelectionCommitted() = 0;
};

} // namespace pictura
