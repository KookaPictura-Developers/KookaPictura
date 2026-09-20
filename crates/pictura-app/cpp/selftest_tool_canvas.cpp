#include "selftest_tool_canvas.h"
#include "selftest_report.h"

#include "frame.h"
#include "image_view.h"
#include "tool_hint_bar.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QPointF>
#include <QtCore/Qt>
#include <QtGui/QColor>
#include <QtGui/QImage>
#include <QtGui/QKeyEvent>
#include <QtGui/QMouseEvent>
#include <QtWidgets/QApplication>

#include <algorithm>
#include <cmath>

namespace {

using pictura::ImageView;
using pictura::PictureView;
using pictura::ToolId;

// Send a mouse event to `canvas` through the real event handler.
void sendMouse(ImageView* canvas, QEvent::Type type, const QPointF& local, Qt::MouseButton button,
               Qt::MouseButtons buttons)
{
    QMouseEvent event(type, local, QPointF(canvas->mapToGlobal(local.toPoint())), button, buttons,
                      Qt::NoModifier);
    QApplication::sendEvent(canvas, &event);
}

} // namespace

int pictura::runToolCanvasChecks(pictura::PicturaMainWindow& frame)
{
    // tc_brush_cursor (340): Brush/Pencil show a blank cursor while the drawn
    // ring carries the brush size, and another tool restores a visible cursor.
    {
        const bool created = frame.newDocument(QStringLiteral("BrushCursor"), 16, 16,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
        PictureView* view = frame.activeView();
        ImageView* canvas = frame.imageView();
        auto* tools = frame.findChild<ToolController*>();
        if (!created || !view || !canvas || !tools) {
            return pictura::selfTest().fail(340, "brush cursor fixture");
        }
        const int doc = frame.activeDocumentIndex();
        frame.setActiveTool(ToolId::Brush);
        canvas->setBrushOutline(24.0, QPointF(8.0, 8.0));
        tools->refreshCursor();
        const bool blank = canvas->cursor().shape() == Qt::BlankCursor;
        const bool outline = canvas->hasBrushOutlineForTest();
        frame.setActiveTool(ToolId::Move);
        tools->refreshCursor();
        const bool restored = canvas->cursor().shape() != Qt::BlankCursor;
        ST_BEGIN("tc_brush_cursor");
        ST_PASS("tc_brush_cursor blank=%d outline=%d restored=%d", blank ? 1 : 0,
                outline ? 1 : 0, restored ? 1 : 0);
        if (!blank || !outline || !restored) {
            frame.closeDocument(doc, false);
            return pictura::selfTest().fail(340, "blank brush cursor");
        }
        frame.closeDocument(doc, false);
    }

    // tc_ring_outside (341): the brush ring is drawn outside the document rect
    // (bounded only by the widget), not clipped to it.
    {
        const bool created = frame.newDocument(QStringLiteral("RingEdge"), 100, 100,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
        ImageView* canvas = frame.imageView();
        if (!created || !canvas) {
            return pictura::selfTest().fail(341, "ring fixture");
        }
        const int doc = frame.activeDocumentIndex();
        frame.resize(800, 600);
        QApplication::processEvents();
        canvas->setCanvasColor(QColor(255, 255, 255));
        canvas->actualPixels();
        QApplication::processEvents();
        canvas->setBrushOutline(80.0, QPointF(100.0, 50.0));
        QApplication::processEvents();
        const QImage shot = canvas->grab().toImage();
        const double z = canvas->zoom();
        const QPointF o = canvas->offset();
        const int docRight = int(std::lround(o.x() + 100.0 * z));
        const int top = std::max(0, int(std::lround(o.y())));
        const int bottom = std::min(shot.height() - 1, int(std::lround(o.y() + 100.0 * z)));
        int outside = 0;
        int inside = 0;
        for (int y = top; y <= bottom; ++y) {
            for (int x = std::max(0, docRight - 40); x < docRight; ++x) {
                if (qGray(shot.pixel(x, y)) < 64) {
                    ++inside;
                }
            }
            for (int x = docRight + 3; x <= std::min(shot.width() - 1, docRight + 40); ++x) {
                if (qGray(shot.pixel(x, y)) < 64) {
                    ++outside;
                }
            }
        }
        const bool ringDrawn = inside > 0;
        const bool ringOutside = outside > 0;
        ST_BEGIN("tc_ring_outside");
        ST_PASS("tc_ring_outside inside=%d outside=%d", inside, outside);
        if (!ringDrawn || !ringOutside) {
            frame.closeDocument(doc, false);
            return pictura::selfTest().fail(341, "ring clipped to document");
        }
        frame.closeDocument(doc, false);
    }

    // tc_space_pan (342): holding Space pans a left drag without starting the
    // tool's paint, and release restores the previous pan/cursor state.
    {
        const bool created = frame.newDocument(QStringLiteral("SpacePan"), 2000, 2000,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
        PictureView* view = frame.activeView();
        ImageView* canvas = frame.imageView();
        if (!created || !view || !canvas) {
            return pictura::selfTest().fail(342, "space pan fixture");
        }
        const int doc = frame.activeDocumentIndex();
        frame.resize(800, 600);
        QApplication::processEvents();
        canvas->actualPixels();
        QApplication::processEvents();
        frame.setActiveTool(ToolId::Brush);
        const QPointF base = canvas->offset();
        const int historyBase = view->history_count();
        QKeyEvent spaceDown(QEvent::KeyPress, Qt::Key_Space, Qt::NoModifier);
        QApplication::sendEvent(&frame, &spaceDown);
        const bool panOn = canvas->spacePanForTest() && canvas->panEnabled();
        const QPointF local(canvas->width() / 2.0, canvas->height() / 2.0);
        sendMouse(canvas, QEvent::MouseButtonPress, local, Qt::LeftButton, Qt::LeftButton);
        sendMouse(canvas, QEvent::MouseMove, local + QPointF(40.0, 30.0), Qt::NoButton,
                  Qt::LeftButton);
        sendMouse(canvas, QEvent::MouseButtonRelease, local + QPointF(40.0, 30.0), Qt::LeftButton,
                  Qt::NoButton);
        const bool panned = canvas->offset() != base;
        const bool noPaint = !view->is_painting() && view->history_count() == historyBase;
        QKeyEvent spaceUp(QEvent::KeyRelease, Qt::Key_Space, Qt::NoModifier);
        QApplication::sendEvent(&frame, &spaceUp);
        const bool released = !canvas->spacePanForTest();
        const bool panRestored = !canvas->panEnabled();
        const bool toolSame = frame.activeTool() == ToolId::Brush;
        ST_BEGIN("tc_space_pan");
        ST_PASS("tc_space_pan panOn=%d panned=%d noPaint=%d released=%d restored=%d tool=%d",
                panOn ? 1 : 0, panned ? 1 : 0, noPaint ? 1 : 0, released ? 1 : 0,
                panRestored ? 1 : 0, toolSame ? 1 : 0);
        if (!panOn || !panned || !noPaint || !released || !panRestored || !toolSame) {
            frame.closeDocument(doc, false);
            return pictura::selfTest().fail(342, "space pan");
        }
        frame.closeDocument(doc, false);
    }

    // tc_hint_bar (343): the marquee tool's context shows Shift = Add to
    // selection, a held key highlights its keycap, and a tool with no keycaps
    // falls back to its text hint.
    {
        const bool created = frame.newDocument(QStringLiteral("HintBar"), 8, 8,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
        auto* hintBar = frame.findChild<ToolHintBar*>(QStringLiteral("toolHintBar"));
        if (!created || !hintBar) {
            return pictura::selfTest().fail(343, "hint bar fixture");
        }
        const int doc = frame.activeDocumentIndex();
        frame.setActiveTool(ToolId::Marquee);
        const bool shiftHint = hintBar->hintCountForTest() >= 2
            && hintBar->keyForTest(0) == QStringLiteral("Shift")
            && hintBar->textForTest(0) == QStringLiteral("Add to selection");
        QKeyEvent shiftDown(QEvent::KeyPress, Qt::Key_Shift, Qt::ShiftModifier);
        QApplication::sendEvent(&frame, &shiftDown);
        const bool highlighted = hintBar->highlightedForTest(0);
        QKeyEvent shiftUp(QEvent::KeyRelease, Qt::Key_Shift, Qt::NoModifier);
        QApplication::sendEvent(&frame, &shiftUp);
        const bool cleared = !hintBar->highlightedForTest(0);
        frame.setActiveTool(ToolId::Hand);
        const bool fallback =
            hintBar->hintCountForTest() == 0 && !hintBar->fallbackForTest().isEmpty();
        ST_BEGIN("tc_hint_bar");
        ST_PASS("tc_hint_bar shift=%d highlight=%d cleared=%d fallback=%d", shiftHint ? 1 : 0,
                highlighted ? 1 : 0, cleared ? 1 : 0, fallback ? 1 : 0);
        if (!shiftHint || !highlighted || !cleared || !fallback) {
            frame.closeDocument(doc, false);
            return pictura::selfTest().fail(343, "tool hint bar");
        }
        frame.closeDocument(doc, false);
    }

    return 0;
}
