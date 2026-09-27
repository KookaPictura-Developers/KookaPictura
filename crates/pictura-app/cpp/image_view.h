#pragma once

#include <QtCore/QElapsedTimer>
#include <QtCore/QLineF>
#include <QtCore/QList>
#include <QtCore/QPointF>
#include <QtCore/QRectF>
#include <QtCore/QString>
#include <QtGui/QColor>
#include <QtGui/QImage>
#include <QtGui/QPolygonF>
#include <QtGui/QTransform>
#include <QtWidgets/QWidget>

#include <algorithm>

class QPainter;

class QMouseEvent;
class QKeyEvent;
class QPaintEvent;
class QResizeEvent;
class QWheelEvent;
class QShowEvent;
class QHideEvent;
class QTimer;

namespace pictura {

// The central document canvas: paints a QImage under a pan/zoom transform over
// a plain canvas colour. GPU compositing is out of scope (CPU image for now).
class ImageView : public QWidget {
    Q_OBJECT

public:
    explicit ImageView(QWidget* parent = nullptr);

    // Replace the image and reset the view: fit-and-centre when the image is
    // larger than the viewport, otherwise 100% centred.
    void setImage(const QImage& image);

    // Replace the image but keep the current zoom and pan.
    void replaceImage(const QImage& image);

    // Overwrite the canvas at document-space (x, y) with a region-sized image
    // under CompositionMode_Source (no blending), then invalidate the zoom
    // present cache so the next paint rebuilds it. A null/empty region and a
    // null canvas are no-ops.
    void blitRegion(const QImage& region, int x, int y);

    const QImage& image() const { return image_; }

    // Zoom about a cursor position so the point under the cursor stays put.
    void zoomAt(const QPointF& cursor, int angleDelta);
    void panBy(const QPointF& delta);

    void zoomIn();
    void zoomOut();
    void fitOnScreen();
    void actualPixels();

    // Set an absolute zoom about a widget-space anchor (clamped to 0.01x..32x).
    void setZoom(double zoom, const QPointF& anchor);

    // Set the widget-space offset directly, clamped by the reveal margin. The
    // workspace scrollbars use this; every other path goes through clampOffset.
    void setOffset(const QPointF& offset);

    double zoom() const { return zoom_; }
    QPointF offset() const { return offset_; }

    // Brush-size ring drawn in image space under the pan/zoom transform. The
    // diameter is in image pixels, so a zoomed canvas scales the ring for free.
    void setBrushOutline(double diameter, const QPointF& imagePos);
    void clearBrushOutline();
    bool hasBrushOutlineForTest() const { return brushOutlineActive_; }
    double brushOutlineDiameterForTest() const { return brushOutlineDiameter_; }
    // Device-pixel ring diameter for the current zoom (what a render shows).
    double brushOutlineScreenDiameterForTest() const
    {
        return brushOutlineDiameter_ * zoom_;
    }

    void setCanvasColor(const QColor& color);
    QColor canvasColor() const { return canvasColor_; }

    // Transparency checkerboard behind the document (the "Light" grid):
    // two tones in 8-screen-pixel cells, anchored to the document origin.
    static int transparencyCellSize();
    static QColor transparencyColorA();
    static QColor transparencyColorB();

    // When disabled, mouse presses are forwarded as tool events instead of
    // starting a pan. Default true.
    void setPanEnabled(bool enabled);
    bool panEnabled() const { return panEnabled_; }

    // Transient Space-key pan: while on, a left drag pans and the hand cursors
    // are used regardless of the active tool, and the tool's mouse-press is not
    // emitted. The owner restores the pan/cursor state on release.
    void setSpacePan(bool on);
    bool spacePanForTest() const { return spacePan_; }

    void setOverlayPolygon(const QPolygonF& polygon);
    void clearOverlay();

    // Committed-selection marching ants. `encoded` is `"x,y x,y ..."` loops
    // joined by `;` (see PictureView::selection_contour); empty clears.
    void setSelectionContour(const QString& encoded);
    void clearSelectionContour();
    void setSelectionEdgesVisible(bool on);
    bool selectionEdgesVisible() const { return selectionEdgesVisible_; }
    bool hasSelectionContourForTest() const { return !selectionContours_.isEmpty(); }
    int selectionContourLoopCountForTest() const { return selectionContours_.size(); }

    // Live selection preview while a tool drags (the rubber band). Drawn with the
    // same animated marching-ants pen as a committed selection, and replaced by
    // the committed contour on release. Not affected by the Selection Edges
    // toggle: the user is actively defining the selection. Empty clears. When
    // `closed` is false the loops are drawn as open polylines (the Polygonal
    // Lasso rubber band must not show a phantom closing edge).
    void setSelectionPreview(const QList<QPolygonF>& loops, bool closed = true, bool solid = false);
    void clearSelectionPreview();
    // A click-driven lasso's origin: a small hollow square, fixed on screen,
    // marking the point the outline closes to. Cleared with the preview.
    void setSelectionPreviewOrigin(const QPointF& imagePos);
    bool hasSelectionPreviewOriginForTest() const { return previewOriginActive_; }
    bool hasSelectionPreviewForTest() const { return !previewContours_.isEmpty(); }
    bool selectionPreviewOpenForTest() const
    {
        return !previewContours_.isEmpty() && !selectionPreviewClosed_;
    }
    bool selectionPreviewSolidForTest() const { return selectionPreviewSolid_; }
    int selectionPreviewLoopCountForTest() const { return previewContours_.size(); }
    int selectionPreviewPointCountForTest() const
    {
        return previewContours_.isEmpty() ? 0 : previewContours_.first().size();
    }

    // Move-tool live preview: draw a cached base plus the moved layer at a live
    // image-space offset, so the drag never composites the document.
    void beginMovePreview(const QImage& base, const QImage& layer, const QPointF& layerPos,
                          double opacity);
    void setMovePreviewDelta(const QPointF& delta);
    void endMovePreview();
    bool movePreviewActive() const { return movePreviewActive_; }

    // Free Transform preview: same cached base/layer as the Move preview, but the
    // layer is drawn under a live similarity transform (scale/rotate/translate
    // about the layer centre) and the quad with its eight handles is overlaid.
    void beginTransformPreview(const QImage& base, const QImage& layer, const QPointF& layerPos,
                               double opacity);
    void setTransformPreview(double scaleX, double scaleY, double angleRadians, double dx,
                             double dy);
    // Projective layer preview: nine space-separated coefficients in
    // `QTransform(m11,m12,m13,m21,m22,m23,m31,m32,m33)` order, mapping source
    // document space onto the target quad. Overrides the similarity preview
    // until the next `beginTransformPreview`.
    void setTransformPreviewProjective(const QString& matrix9);
    void setTransformQuad(const QString& encoded);
    void clearTransformPreview();
    bool transformPreviewActive() const { return transformActive_; }

    // The document-space matrix the live preview applies to the cached layer
    // (`c + R·S·(u − c) + d`). paintEvent draws through the same matrix, so a
    // gesture's preview mapping can be checked against the committed mapping
    // without rendering.
    QTransform transformPreviewMatrix() const;

    // Crop box (image space): shades outside it and draws rule-of-thirds
    // guides, its frame, and eight handles.
    void setCropBox(const QRectF& box);
    void clearCropBox();
    bool hasCropBoxForTest() const { return !cropBox_.isNull(); }

    // Perspective Crop quad (TL, TR, BR, BL, image space): shades outside it and
    // draws its edges, a perspective-following 3x3 grid, and corner handles.
    void setPerspectiveCropQuad(const QPolygonF& quad);
    void clearPerspectiveCropQuad();
    bool hasPerspectiveCropQuadForTest() const { return perspectiveQuad_.size() == 4; }

    // Slice overlay (image space): user slices solid blue with a numbered
    // badge, auto slices dotted grey, the selected slice with orange handles,
    // plus the slice being dragged out.
    struct SliceOverlay {
        QRectF rect;
        int number = 0;
        bool user = false;
        bool selected = false;
    };
    void setSliceOverlay(const QList<SliceOverlay>& slices, const QRectF& dragging = QRectF());
    void clearSliceOverlay();
    int sliceOverlayCountForTest() const { return int(sliceOverlay_.size()); }
    bool sliceOverlayHasSelectionForTest() const
    {
        return std::any_of(sliceOverlay_.cbegin(), sliceOverlay_.cend(),
                           [](const SliceOverlay& s) { return s.selected; });
    }

    // Annotation overlay (image space): numbered color-sampler crosshairs and
    // note glyphs (the `currentNote` index outlined), shown with any tool, plus
    // the Ruler's measuring line while that tool is active.
    void setAnnotationOverlay(const QList<QPointF>& samplers, const QList<QPointF>& notes,
                              int currentNote);
    void setRulerLine(const QLineF& line);
    void clearRulerLine();
    int samplerOverlayCountForTest() const { return int(samplerOverlay_.size()); }
    int noteOverlayCountForTest() const { return int(noteOverlay_.size()); }
    bool hasRulerLineForTest() const { return rulerShown_; }

    // Live marquee size readout ("W x H"), painted as a tooltip offset from the
    // mapped cursor. Empty text or clearDragSizeHint() hides it.
    void setDragSizeHint(const QString& text, const QPointF& imagePos);
    void clearDragSizeHint();
    bool hasDragSizeHintForTest() const { return dragSizeActive_ && !dragSizeText_.isEmpty(); }

    // Test hooks for the internal present cache.
    bool presentCacheRebuiltOnLastPaint() const { return presentCacheRebuiltLastPaint_; }
    int presentCacheRebuildCount() const { return presentCache_.rebuilds; }
    QSize presentCacheImageSize() const { return presentCache_.scaled.size(); }
    qint64 presentCacheImageKey() const { return presentCache_.key; }
    void setPresentCacheEnabledForTest(bool enabled);

    // Map a widget-space point to document/image coordinates.
    QPointF widgetToImage(const QPointF& widgetPos) const;

signals:
    void zoomChanged(double zoom);
    // The pan or zoom changed by any path (pan, zoom, fit, 100%, resize, the
    // Navigator proxy). The workspace scrollbars project this into their ranges.
    void viewChanged();
    void mousePressed(const QPointF& imagePos, int button, int modifiers);
    void mouseMoved(const QPointF& imagePos);
    void mouseReleased(const QPointF& imagePos);
    // Free Transform Enter/Return (commit) and Escape (cancel).
    void transformCommitRequested();
    void transformCancelRequested();

protected:
    void paintEvent(QPaintEvent* event) override;
    void wheelEvent(QWheelEvent* event) override;
    void mousePressEvent(QMouseEvent* event) override;
    void mouseMoveEvent(QMouseEvent* event) override;
    void mouseReleaseEvent(QMouseEvent* event) override;
    void keyPressEvent(QKeyEvent* event) override;
    void resizeEvent(QResizeEvent* event) override;
    void showEvent(QShowEvent* event) override;
    void hideEvent(QHideEvent* event) override;
    void leaveEvent(QEvent* event) override;

private:
    void paintCropGroupOverlays(QPainter& painter);
    void paintAnnotations(QPainter& painter);
    QList<QPointF> samplerOverlay_;
    QList<QPointF> noteOverlay_;
    int currentNote_ = -1;
    QLineF rulerLine_;
    bool rulerShown_ = false;
    QPolygonF perspectiveQuad_;
    QRectF cropBox_;
    QList<SliceOverlay> sliceOverlay_;
    QRectF sliceDrag_;
    void centreImage();
    void applyInitialView();
    void clampOffset();
    void updateAntsTimer();

    struct PresentCache {
        qint64 key = 0;
        double zoom = 0.0;
        bool valid = false;
        int rebuilds = 0;
        QImage scaled;
    };
    const QImage* cachedScaled(PresentCache& cache, const QImage& source);

    QImage image_;
    QColor canvasColor_{Qt::darkGray};
    double zoom_ = 1.0;
    QPointF offset_;
    QPointF last_;
    bool panEnabled_ = true;
    bool panning_ = false;
    bool spacePan_ = false;
    bool userAdjusted_ = false;
    QPolygonF overlayPolygon_;

    QList<QPolygonF> selectionContours_;
    QList<QPolygonF> previewContours_;
    bool selectionPreviewClosed_ = true;
    bool selectionPreviewSolid_ = false;
    bool previewOriginActive_ = false;
    QPointF previewOrigin_;
    bool selectionEdgesVisible_ = true;
    int antsPhase_ = 0;
    QTimer* antsTimer_ = nullptr;

    PresentCache presentCache_;
    bool presentCacheRebuiltLastPaint_ = false;
    bool presentCacheEnabledForTest_ = true;

    bool movePreviewActive_ = false;
    QImage moveBase_;
    QImage moveLayer_;
    QPointF moveLayerPos_;
    QPointF moveDelta_;
    double moveOpacity_ = 1.0;

    bool transformActive_ = false;
    double transformScaleX_ = 1.0;
    double transformScaleY_ = 1.0;
    double transformAngle_ = 0.0;
    double transformDx_ = 0.0;
    double transformDy_ = 0.0;
    QTransform transformProjective_;
    bool transformProjectiveActive_ = false;
    QPolygonF transformQuad_;

    QString dragSizeText_;
    QPointF dragSizeImagePos_;
    bool dragSizeActive_ = false;

    bool brushOutlineActive_ = false;
    double brushOutlineDiameter_ = 0.0;
    QPointF brushOutlineImagePos_;

    // Drag-start latency probe: reports the press -> handler -> first move ->
    // paint gaps to stderr when a drag start exceeds one frame, or always when
    // PICTURA_PRESS_TRACE is set. Left in deliberately; it is a few loads.
    struct PressTrace {
        QElapsedTimer clock;
        qint64 pressHwMs = -1;
        qint64 handledNs = -1;
        qint64 firstMoveNs = -1;
        qint64 firstMoveHwMs = -1;
        bool armed = false;
    };
    PressTrace pressTrace_;
};

} // namespace pictura
