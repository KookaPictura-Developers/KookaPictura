#pragma once

#include <QtCore/QPointF>
#include <QtGui/QColor>
#include <QtGui/QImage>
#include <QtGui/QPolygonF>
#include <QtWidgets/QWidget>

class QMouseEvent;
class QPaintEvent;
class QResizeEvent;
class QWheelEvent;

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

private:
    void centreImage();
    void applyInitialView();

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

    PresentCache presentCache_;
    PresentCache moveBaseCache_;
    PresentCache moveLayerCache_;
    bool presentCacheRebuiltLastPaint_ = false;
    bool presentCacheEnabledForTest_ = true;

    bool movePreviewActive_ = false;
    QImage moveBase_;
    QImage moveLayer_;
    QPointF moveLayerPos_;
    QPointF moveDelta_;
    double moveOpacity_ = 1.0;
};

} // namespace pictura
