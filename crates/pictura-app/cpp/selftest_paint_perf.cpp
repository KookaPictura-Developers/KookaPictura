#include "selftest_paint_perf.h"
#include "selftest_report.h"

#include "commands.h"
#include "frame.h"
#include "image_view.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QObject>
#include <QtGui/QImage>
#include <QtWidgets/QApplication>

#include <algorithm>

namespace {

// Byte-for-byte widget-image equality (both grabs share the same format).
bool sameImage(const QImage& a, const QImage& b)
{
    if (a.size() != b.size()) {
        return false;
    }
    for (int y = 0; y < a.height(); ++y) {
        for (int x = 0; x < a.width(); ++x) {
            if (a.pixel(x, y) != b.pixel(x, y)) {
                return false;
            }
        }
    }
    return true;
}

} // namespace

int pictura::runPaintPerfChecks(pictura::PicturaMainWindow& frame)
{
    // pp_dab_region (344): each paint_dab reports a dab-sized dirty rect, not
    // the accumulated stroke union. With a 12 px step over a long stroke the
    // cumulative bug would report a rect that grows with the whole stroke;
    // here every rect stays near the brush diameter. No wall-clock is measured
    // (flaky); the region/behavior is the assertion.
    {
        const bool created = frame.newDocument(QStringLiteral("PaintRegion"), 256, 256,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
        PictureView* view = frame.activeView();
        if (!created || !view) {
            return pictura::selfTest().fail(344, "paint region fixture");
        }
        const int doc = frame.activeDocumentIndex();
        view->set_active_layer(QStringLiteral("0"));

        int blits = 0;
        int maxW = 0;
        int firstX = -1;
        int lastX = -1;
        auto conn = QObject::connect(
            view, &PictureView::regionBlitted,
            [&blits, &maxW, &firstX, &lastX](const QImage& region, int x, int) {
                ++blits;
                maxW = std::max(maxW, region.width());
                if (firstX < 0) {
                    firstX = x;
                }
                lastX = x;
            });

        const bool begun = view->begin_paint(0xFFFF0000u, 0xFFFFFFFFu, 8, 100, 100, 0, 100, 100,
                                             150, QStringLiteral("normal"), false, false);
        int dabs = 0;
        for (int i = 0; i < 16; ++i) {
            if (view->paint_dab(40.0 + i * 12.0, 128.0, 1.0)) {
                ++dabs;
            }
        }
        const bool ended = view->end_paint();
        QObject::disconnect(conn);

        const int span = lastX - firstX;
        const bool regionPerDab = blits >= dabs && dabs >= 16;
        const bool dabSized = maxW > 0 && maxW <= 16;
        const bool notStrokeSized = span >= 150 && maxW * 4 < span;
        ST_BEGIN("pp_dab_region");
        ST_PASS("pp_dab_region dabs=%d blits=%d maxW=%d span=%d", dabs, blits, maxW, span);
        if (!begun || !ended || !regionPerDab || !dabSized || !notStrokeSized) {
            frame.closeDocument(doc, false);
            return pictura::selfTest().fail(344, "dab region not incremental");
        }
        frame.closeDocument(doc, false);
    }

    // pp_present_cache (345): after a region blit, painting from the patched
    // scaled present cache is byte-identical to the direct draw (cache off), so
    // the optimization cannot change the rendered result.
    {
        const bool created = frame.newDocument(QStringLiteral("PaintCache"), 96, 96,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
        PictureView* view = frame.activeView();
        ImageView* canvas = frame.imageView();
        if (!created || !view || !canvas) {
            return pictura::selfTest().fail(345, "present cache fixture");
        }
        const int doc = frame.activeDocumentIndex();
        view->set_active_layer(QStringLiteral("0"));
        frame.resize(420, 360);
        QApplication::processEvents();
        canvas->actualPixels();
        QApplication::processEvents();
        // Warm the present cache (two paints at the same zoom reuse it).
        QImage warm(canvas->size(), QImage::Format_ARGB32);
        canvas->render(&warm);
        canvas->render(&warm);

        const bool begun = view->begin_paint(0xFFFF0000u, 0xFFFFFFFFu, 8, 100, 100, 0, 100, 100,
                                             25, QStringLiteral("normal"), false, false);
        const bool dabbed = view->paint_dab(48.0, 48.0, 1.0);
        const QImage patched = canvas->grab().toImage();
        canvas->setPresentCacheEnabledForTest(false);
        const QImage direct = canvas->grab().toImage();
        canvas->setPresentCacheEnabledForTest(true);
        view->end_paint();
        const bool same = sameImage(patched, direct);
        ST_BEGIN("pp_present_cache");
        ST_PASS("pp_present_cache begun=%d dab=%d same=%d", begun ? 1 : 0, dabbed ? 1 : 0,
                same ? 1 : 0);
        if (!begun || !dabbed || !same) {
            frame.closeDocument(doc, false);
            return pictura::selfTest().fail(345, "present cache patch changed the render");
        }
        frame.closeDocument(doc, false);
    }

    // pp_no_registry_refresh (394): while a stroke is active the paint region
    // handler must not re-run the command registry per dab; releasing the stroke
    // emits `changed`, whose full refresh runs it exactly once. Krita's
    // "unnecessary objects per event" hot spot. No wall-clock budget.
    {
        const bool created = frame.newDocument(QStringLiteral("PaintRefresh"), 128, 128,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
        PictureView* view = frame.activeView();
        CommandRegistry* registry = frame.registry();
        if (!created || !view || !registry) {
            return pictura::selfTest().fail(394, "registry refresh fixture");
        }
        const int doc = frame.activeDocumentIndex();
        view->set_active_layer(QStringLiteral("0"));
        const bool begun = view->begin_paint(0xFFFF0000u, 0xFFFFFFFFu, 8, 100, 100, 0, 100, 100,
                                             12, QStringLiteral("normal"), false, false);
        const int before = registry->refreshCount();
        int dabs = 0;
        for (int i = 0; i < 16; ++i) {
            if (view->paint_dab(32.0 + i * 4.0, 64.0, 1.0)) {
                ++dabs;
            }
        }
        const int mid = registry->refreshCount();
        const bool ended = view->end_paint();
        const int after = registry->refreshCount();
        const bool noPerDab = mid == before;
        const bool onceOnCommit = after == mid + 1;
        ST_BEGIN("pp_no_registry_refresh");
        ST_PASS("pp_no_registry_refresh dabs=%d before=%d mid=%d after=%d", dabs, before, mid,
                after);
        if (!begun || !ended || dabs < 16 || !noPerDab || !onceOnCommit) {
            frame.closeDocument(doc, false);
            return pictura::selfTest().fail(394, "registry refreshed per dab");
        }
        frame.closeDocument(doc, false);
    }

    return 0;
}
