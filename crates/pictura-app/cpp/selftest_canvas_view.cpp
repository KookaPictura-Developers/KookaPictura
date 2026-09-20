#include "selftest_canvas_view.h"
#include "selftest_report.h"

#include "canvas_range.h"
#include "canvas_scrollbars.h"
#include "frame.h"
#include "image_view.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QPointF>
#include <QtCore/QRectF>
#include <QtCore/QSize>
#include <QtCore/QSizeF>
#include <QtCore/Qt>
#include <QtCore/QtNumeric>
#include <QtGui/QColor>
#include <QtGui/QImage>
#include <QtWidgets/QApplication>
#include <QtWidgets/QScrollBar>

#include <algorithm>
#include <cmath>

namespace {

// Visible portion of the scaled document inside the canvas viewport.
QSizeF visibleCanvas(const pictura::ImageView* canvas)
{
    const QPointF o = canvas->offset();
    const QSize vp = canvas->size();
    const double w = canvas->image().width() * canvas->zoom();
    const double h = canvas->image().height() * canvas->zoom();
    const double vx = std::min(double(vp.width()), o.x() + w) - std::max(0.0, o.x());
    const double vy = std::min(double(vp.height()), o.y() + h) - std::max(0.0, o.y());
    return QSizeF(std::max(0.0, vx), std::max(0.0, vy));
}

// Width of the black brush ring along the horizontal line through its centre.
int measuredRingWidth(pictura::ImageView* canvas, const QPointF& imageCentre,
                      const QSize& imageSize)
{
    const QImage shot = canvas->grab().toImage();
    const double z = canvas->zoom();
    const QPointF o = canvas->offset();
    const int cy = int(std::lround(o.y() + imageCentre.y() * z));
    const int x0 = std::max(0, int(std::lround(o.x())) + 1);
    const int x1 = std::min(shot.width() - 1,
                            int(std::lround(o.x() + imageSize.width() * z)) - 1);
    if (cy < 0 || cy >= shot.height() || x1 < x0) {
        return 0;
    }
    int lo = -1;
    int hi = -1;
    for (int x = x0; x <= x1; ++x) {
        if (qGray(shot.pixel(x, cy)) < 64) {
            if (lo < 0) {
                lo = x;
            }
            hi = x;
        }
    }
    return hi >= lo ? hi - lo + 1 : 0;
}

} // namespace

int pictura::runCanvasViewChecks(pictura::PicturaMainWindow& frame)
{
    // 308: the pan clamp keeps the reveal margin visible on all four axes, and
    // an overshoot slides to the near side instead of wrapping to the far one.
    const bool bigCreated = frame.newDocument(QStringLiteral("CanvasView"), 2000, 2000,
                                              QStringLiteral("rgb"), 8,
                                              QStringLiteral("white"));
    pictura::PictureView* bigView = frame.activeView();
    pictura::ImageView* canvas = frame.imageView();
    if (!bigCreated || !bigView || !canvas) {
        return pictura::selfTest().fail(308, "canvas view fixture");
    }
    const int bigDoc = frame.activeDocumentIndex();
    frame.resize(800, 600);
    QApplication::processEvents();
    canvas->actualPixels();
    QApplication::processEvents();

    const QPointF base = canvas->offset();
    bool marginOk = true;
    bool nearSideOk = true;
    for (int i = 0; i < 4; ++i) {
        canvas->setOffset(base);
        QPointF pan(0.0, 0.0);
        if (i == 0) {
            pan = QPointF(1e6, 0.0);
        } else if (i == 1) {
            pan = QPointF(-1e6, 0.0);
        } else if (i == 2) {
            pan = QPointF(0.0, 1e6);
        } else {
            pan = QPointF(0.0, -1e6);
        }
        canvas->panBy(pan);
        const QSizeF vis = visibleCanvas(canvas);
        if (vis.width() < pictura::kCanvasRevealMarginPx - 1e-6
            || vis.height() < pictura::kCanvasRevealMarginPx - 1e-6) {
            marginOk = false;
        }
        const QPointF off = canvas->offset();
        if (i == 0 && !(off.x() > base.x())) {
            nearSideOk = false;
        }
        if (i == 1 && !(off.x() < base.x())) {
            nearSideOk = false;
        }
        if (i == 2 && !(off.y() > base.y())) {
            nearSideOk = false;
        }
        if (i == 3 && !(off.y() < base.y())) {
            nearSideOk = false;
        }
    }
    ST_BEGIN("canvas_pan_margin");
    ST_PASS("canvas_pan_margin margin=%d nearside=%d", marginOk ? 1 : 0, nearSideOk ? 1 : 0);
    if (!marginOk || !nearSideOk) {
        return pictura::selfTest().fail(308, "pan clamp margin");
    }

    // 309: a scrollbar value round-trips through the canvas offset and the bar
    // follows a canvas pan (offset stays the single source of truth).
    auto* host = qobject_cast<pictura::CanvasScrollBars*>(canvas->parentWidget());
    if (!host) {
        return pictura::selfTest().fail(309, "canvas scrollbars host");
    }
    QScrollBar* hbar = host->horizontalBarForTest();
    const QSizeF bigImage(canvas->image().width(), canvas->image().height());
    const pictura::OffsetRange range =
        pictura::offsetRangeFor(bigImage, canvas->zoom(), QSizeF(canvas->size()));
    const bool shown = !hbar->isHidden();
    const int target = (hbar->minimum() + hbar->maximum()) / 2;
    hbar->setValue(target);
    const bool roundTrip = std::abs(canvas->offset().x()
                                    - (range.minX + double(hbar->value())))
                           < 0.75;
    canvas->panBy(QPointF(40.0, 0.0));
    const pictura::OffsetRange afterRange =
        pictura::offsetRangeFor(bigImage, canvas->zoom(), QSizeF(canvas->size()));
    const bool follows =
        std::abs(hbar->value() - int(std::lround(canvas->offset().x() - afterRange.minX))) <= 1;
    ST_BEGIN("canvas_scrollbar_roundtrip");
    ST_PASS("canvas_scrollbar_roundtrip shown=%d roundtrip=%d follows=%d value=%d",
            shown ? 1 : 0, roundTrip ? 1 : 0, follows ? 1 : 0, hbar->value());
    if (!shown || !roundTrip || !follows) {
        return pictura::selfTest().fail(309, "scrollbar round trip");
    }
    frame.closeDocument(bigDoc, false);

    // 310: both bars hide when the document fits the viewport.
    const bool smallCreated = frame.newDocument(QStringLiteral("CanvasFits"), 16, 16,
                                                QStringLiteral("rgb"), 8,
                                                QStringLiteral("white"));
    pictura::ImageView* small = frame.imageView();
    if (!smallCreated || !small) {
        return pictura::selfTest().fail(310, "canvas fits fixture");
    }
    const int smallDoc = frame.activeDocumentIndex();
    frame.resize(800, 600);
    QApplication::processEvents();
    small->actualPixels();
    QApplication::processEvents();
    auto* smallHost = qobject_cast<pictura::CanvasScrollBars*>(small->parentWidget());
    if (!smallHost) {
        return pictura::selfTest().fail(310, "canvas fits host");
    }
    const bool hidden = smallHost->horizontalBarForTest()->isHidden()
                        && smallHost->verticalBarForTest()->isHidden();
    ST_BEGIN("canvas_scrollbar_hidden");
    ST_PASS("canvas_scrollbar_hidden hidden=%d", hidden ? 1 : 0);
    if (!hidden) {
        return pictura::selfTest().fail(310, "scrollbars not hidden");
    }
    frame.closeDocument(smallDoc, false);

    // 311/312: the drawn ring tracks the brush diameter, and because it lives in
    // image space its on-screen size scales with zoom.
    const bool ringCreated = frame.newDocument(QStringLiteral("BrushRing"), 200, 200,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
    pictura::ImageView* ringCanvas = frame.imageView();
    if (!ringCreated || !ringCanvas) {
        return pictura::selfTest().fail(311, "brush ring fixture");
    }
    const int ringDoc = frame.activeDocumentIndex();
    frame.resize(800, 600);
    QApplication::processEvents();
    ringCanvas->actualPixels();
    QApplication::processEvents();
    const QPointF imageCentre(100.0, 100.0);
    const QSize ringImageSize(200, 200);
    ringCanvas->setBrushOutline(40.0, imageCentre);
    const int widthNarrow = measuredRingWidth(ringCanvas, imageCentre, ringImageSize);
    ringCanvas->setBrushOutline(80.0, imageCentre);
    const int widthWide = measuredRingWidth(ringCanvas, imageCentre, ringImageSize);
    const bool sizeTracked = ringCanvas->hasBrushOutlineForTest()
                             && std::abs(widthNarrow - 40) <= 2
                             && std::abs(widthWide - 80) <= 2;
    ST_BEGIN("canvas_brush_outline_size");
    ST_PASS("canvas_brush_outline_size narrow=%d wide=%d", widthNarrow, widthWide);
    if (!sizeTracked) {
        return pictura::selfTest().fail(311, "brush outline size");
    }

    ringCanvas->setBrushOutline(40.0, imageCentre);
    const int widthAtOne = measuredRingWidth(ringCanvas, imageCentre, ringImageSize);
    ringCanvas->setZoom(2.0, QPointF(ringCanvas->width() / 2.0, ringCanvas->height() / 2.0));
    const int widthAtTwo = measuredRingWidth(ringCanvas, imageCentre, ringImageSize);
    const bool zoomTracked = std::abs(widthAtOne - 40) <= 2
                             && std::abs(widthAtTwo - 80) <= 3
                             && ringCanvas->brushOutlineScreenDiameterForTest() == 80.0;
    ST_BEGIN("canvas_brush_outline_zoom");
    ST_PASS("canvas_brush_outline_zoom one=%d two=%d", widthAtOne, widthAtTwo);
    if (!zoomTracked) {
        return pictura::selfTest().fail(312, "brush outline zoom");
    }
    frame.closeDocument(ringDoc, false);
    return 0;
}
