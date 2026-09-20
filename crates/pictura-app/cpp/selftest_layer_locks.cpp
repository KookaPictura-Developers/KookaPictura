#include "selftest_layer_locks.h"
#include "selftest_report.h"

#include "frame.h"
#include "image_view.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QPointF>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtCore/Qt>

int pictura::runLayerLocksChecks(pictura::PicturaMainWindow& frame)
{
    // llk_move_refusal (313): a Move-tool drag over a position-locked layer is
    // refused at the shared translate entry, so the rect, the composite, and
    // the history are all unchanged.
    {
        const bool created = frame.newDocument(QStringLiteral("LayerLockMove"), 8, 8,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
        pictura::PictureView* view = frame.activeView();
        pictura::ImageView* canvas = frame.imageView();
        auto* tools = frame.findChild<pictura::ToolController*>();
        if (!created || !view || !canvas || !tools) {
            return pictura::selfTest().fail(313, "move refusal fixture");
        }
        const int doc = frame.activeDocumentIndex();
        const QString locked = view->add_layer_in(QString());
        view->set_layer_name_path(locked, QStringLiteral("LockedMove"));
        view->set_layers_lock(QStringList{locked}, QStringLiteral("position"), true);
        const QString rectBefore = view->layer_rect(locked);
        const unsigned int pixelBefore = view->composite_argb(2, 2);
        const int historyBefore = view->history_count();
        frame.setActiveTool(pictura::ToolId::Move);
        canvas->mousePressed(QPointF(2, 2), Qt::LeftButton, int(Qt::NoModifier));
        canvas->mouseMoved(QPointF(7, 7));
        canvas->mouseReleased(QPointF(7, 7));
        const bool rectSame = view->layer_rect(locked) == rectBefore;
        const bool compositeSame = view->composite_argb(2, 2) == pixelBefore;
        const bool noHistory = view->history_count() == historyBefore;
        ST_BEGIN("llk_move_refusal");
        ST_PASS("llk_move_refusal rect=%d composite=%d history=%d", rectSame ? 1 : 0,
                compositeSame ? 1 : 0, view->history_count() - historyBefore);
        if (!rectSame || !compositeSame || !noHistory) {
            frame.closeDocument(doc, false);
            return pictura::selfTest().fail(313, "move-tool lock refusal");
        }
        frame.closeDocument(doc, false);
    }

    // llk_filter_refusal (314): a filter on a pixel-locked layer returns false
    // and leaves the composite and history untouched.
    {
        const bool created = frame.newDocument(QStringLiteral("LayerLockFilter"), 8, 8,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
        pictura::PictureView* view = frame.activeView();
        if (!created || !view) {
            return pictura::selfTest().fail(314, "filter refusal fixture");
        }
        const int doc = frame.activeDocumentIndex();
        const QString locked = view->add_layer_in(QString());
        view->set_layer_name_path(locked, QStringLiteral("LockedFilter"));
        view->set_layers_lock(QStringList{locked}, QStringLiteral("pixels"), true);
        const unsigned int pixelBefore = view->composite_argb(4, 4);
        const int historyBefore = view->history_count();
        const bool filtered = view->apply_filter(QStringLiteral("add-noise"));
        const bool compositeSame = view->composite_argb(4, 4) == pixelBefore;
        const bool noHistory = view->history_count() == historyBefore;
        ST_BEGIN("llk_filter_refusal");
        ST_PASS("llk_filter_refusal applied=%d composite=%d history=%d", filtered ? 1 : 0,
                compositeSame ? 1 : 0, view->history_count() - historyBefore);
        if (filtered || !compositeSame || !noHistory) {
            frame.closeDocument(doc, false);
            return pictura::selfTest().fail(314, "filter lock refusal");
        }
        frame.closeDocument(doc, false);
    }

    // llk_locked_cursor (315): a paint tool over a pixel-locked layer shows the
    // forbidden cursor; unlocking restores the normal tool cursor.
    {
        const bool created = frame.newDocument(QStringLiteral("LayerLockCursor"), 8, 8,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
        pictura::PictureView* view = frame.activeView();
        auto* tools = frame.findChild<pictura::ToolController*>();
        if (!created || !view || !tools) {
            return pictura::selfTest().fail(315, "locked cursor fixture");
        }
        const int doc = frame.activeDocumentIndex();
        const QString locked = view->add_layer_in(QString());
        view->set_layer_name_path(locked, QStringLiteral("LockedCursor"));
        view->set_layers_lock(QStringList{locked}, QStringLiteral("pixels"), true);
        frame.setActiveTool(pictura::ToolId::Brush);
        tools->refreshCursor();
        const bool forbidden =
            frame.imageView()->cursor().shape() == Qt::ForbiddenCursor;
        view->set_layers_lock(QStringList{locked}, QStringLiteral("pixels"), false);
        tools->refreshCursor();
        const bool restored =
            frame.imageView()->cursor().shape() != Qt::ForbiddenCursor;
        ST_BEGIN("llk_locked_cursor");
        ST_PASS("llk_locked_cursor locked=%d unlocked=%d", forbidden ? 1 : 0,
                restored ? 1 : 0);
        if (!forbidden || !restored) {
            frame.closeDocument(doc, false);
            return pictura::selfTest().fail(315, "locked pixel cursor");
        }
        frame.closeDocument(doc, false);
    }

    // llk_paint_refusal (316): a brush stroke on a pixel-locked layer begins no
    // stroke, writes no pixels, adds no history, and does not dirty the
    // document.
    {
        const bool created = frame.newDocument(QStringLiteral("LayerLockPaint"), 8, 8,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
        pictura::PictureView* view = frame.activeView();
        if (!created || !view) {
            return pictura::selfTest().fail(316, "paint refusal fixture");
        }
        const int doc = frame.activeDocumentIndex();
        const QString locked = view->add_layer_in(QString());
        view->set_layer_name_path(locked, QStringLiteral("LockedPaint"));
        view->set_layers_lock(QStringList{locked}, QStringLiteral("pixels"), true);
        const unsigned int pixelBefore = view->composite_argb(4, 4);
        const int historyBefore = view->history_count();
        const bool begun = view->begin_paint(0xFFFF0000u, 0xFFFFFFFFu, 8, 100, 100, 0, 100, 100,
                                             25, QStringLiteral("normal"), false, false);
        const bool noStroke = !view->is_painting();
        const bool compositeSame = view->composite_argb(4, 4) == pixelBefore;
        const bool noHistory = view->history_count() == historyBefore;
        ST_BEGIN("llk_paint_refusal");
        ST_PASS("llk_paint_refusal begun=%d stroke=%d composite=%d history=%d", begun ? 1 : 0,
                noStroke ? 1 : 0, compositeSame ? 1 : 0, view->history_count() - historyBefore);
        if (begun || !noStroke || !compositeSame || !noHistory) {
            frame.closeDocument(doc, false);
            return pictura::selfTest().fail(316, "paint lock refusal");
        }
        frame.closeDocument(doc, false);
    }

    return 0;
}
