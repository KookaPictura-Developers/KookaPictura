#pragma once

#include <QtCore/QElapsedTimer>
#include <QtCore/QList>
#include <QtCore/QPointF>
#include <QtCore/QString>
#include <QtGui/QColor>
#include <QtGui/QImage>
#include <QtGui/QPolygonF>
#include <QtWidgets/QWidget>

class QMouseEvent;
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

    double zoom() const { return zoom_; }
    QPointF offset() const { return offset_; }

    void setCanvasColor(const QColor& color);
    QColor canvasColor() const { return canvasColor_; }

    // Transparency checkerboard behind the document (Photoshop "Light" grid):
    // two tones in 8-screen-pixel cells, anchored to the document origin.
    static int transparencyCellSize();
    static QColor transparencyColorA();
    static QColor transparencyColorB();

    // When disabled, mouse presses are forwarded as tool events instead of
    // starting a pan. Default true.
    void setPanEnabled(bool enabled);
    bool panEnabled() const { return panEnabled_; }

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
    // toggle: the user is actively defining the selection. Empty clears.
    void setSelectionPreview(const QList<QPolygonF>& loops);
    void clearSelectionPreview();
    bool hasSelectionPreviewForTest() const { return !previewContours_.isEmpty(); }
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
    void mousePressed(const QPointF& imagePos, int button, int modifiers);
    void mouseMoved(const QPointF& imagePos);
    void mouseReleased(const QPointF& imagePos);

protected:
    void paintEvent(QPaintEvent* event) override;
    void wheelEvent(QWheelEvent* event) override;
    void mousePressEvent(QMouseEvent* event) override;
    void mouseMoveEvent(QMouseEvent* event) override;
    void mouseReleaseEvent(QMouseEvent* event) override;
    void resizeEvent(QResizeEvent* event) override;
    void showEvent(QShowEvent* event) override;
    void hideEvent(QHideEvent* event) override;

private:
    void centreImage();
    void applyInitialView();
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
    bool userAdjusted_ = false;
    QPolygonF overlayPolygon_;

    QList<QPolygonF> selectionContours_;
    QList<QPolygonF> previewContours_;
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
