#include "selftest_shell_round4_float.h"
#include "selftest_report.h"

#include "frame.h"
#include "panels/panel_column.h"
#include "panels/panel_group.h"
#include "session.h"

#include <QtCore/QCoreApplication>
#include <QtCore/QPoint>
#include <QtCore/QRect>
#include <QtWidgets/QMainWindow>
#include <QtWidgets/QWidget>

namespace {

void pumpFloat(int count)
{
    for (int i = 0; i < count; ++i) {
        QCoreApplication::processEvents();
    }
}

} // namespace

int pictura::runShellRound4FloatCheck(pictura::PicturaMainWindow& frame)
{
    // float_child_overlay (440): on a platform that forbids a client from
    // positioning its own top-level windows (Wayland), or when forced for test,
    // a dragged float is an in-window child of the frame rather than a
    // top-level, and it follows the cursor parent-relative, staying clamped to
    // the frame rect. The auto (top-level) branch is asserted first so the
    // platform decision itself is covered.
    frame.applyPanelSessionForTest(pictura::SessionState{});
    pumpFloat(6);
    const bool autoTopLevel = pictura::PanelFloat::overlayUsesTopLevel();
    pictura::PanelFloat::setForceChildOverlayForTest(true);
    const bool forcedChild = !pictura::PanelFloat::overlayUsesTopLevel();
    bool torn = false;
    bool child = false;
    bool follows = false;
    bool clamped = false;
    pictura::PanelColumn* column = frame.panelColumn();
    if (column) {
        pictura::PanelGroup* group = nullptr;
        for (pictura::PanelGroup* candidate : column->groups()) {
            if (candidate && candidate->titleCountForTest() > 0) {
                group = candidate;
                break;
            }
        }
        if (group && group->visiblePanels().isEmpty() && !group->panels().isEmpty()
            && group->panels().first()) {
            column->showPanel(group->panels().first()->objectName(), true);
            pumpFloat(4);
        }
        const QString panel = group && !group->visiblePanels().isEmpty()
                                  && group->visiblePanels().first()
                              ? group->visiblePanels().first()->objectName()
                              : QString();
        if (!panel.isEmpty()) {
            torn = column->tearOffForTest(panel);
            pumpFloat(8);
            const int index = column->floatCountForTest() - 1;
            pictura::PanelFloat* floatWindow = column->floatForTest(index);
            child = torn && floatWindow && !floatWindow->isWindow()
                    && qobject_cast<QMainWindow*>(floatWindow->parentWidget()) == &frame;
            if (child) {
                const QPoint a(
                    frame.mapToGlobal(QPoint(frame.width() / 2, frame.height() / 4)));
                const QPoint b = a + QPoint(24, 18);
                const bool movedA = column->floatClampedForTest(index, a);
                const QPoint posA = floatWindow->pos();
                const bool movedB = column->floatClampedForTest(index, b);
                const QPoint posB = floatWindow->pos();
                // Parent-relative `pos()` advances exactly with the cursor.
                follows = movedA && movedB && (posB - posA) == (b - a);
                const QRect frameRect(frame.mapToGlobal(QPoint(0, 0)), frame.size());
                clamped = frameRect.contains(column->floatGeometryForTest(index));
            }
            column->closeFloatForTest(index);
            pumpFloat(6);
        }
    }
    pictura::PanelFloat::setForceChildOverlayForTest(false);
    const bool restored = pictura::PanelFloat::overlayUsesTopLevel() == autoTopLevel;
    ST_BEGIN("float_child_overlay");
    ST_PASS("float_child_overlay auto_tl=%d forced=%d torn=%d child=%d follows=%d "
            "clamped=%d restored=%d",
            autoTopLevel ? 1 : 0, forcedChild ? 1 : 0, torn ? 1 : 0, child ? 1 : 0,
            follows ? 1 : 0, clamped ? 1 : 0, restored ? 1 : 0);
    if (!(autoTopLevel && forcedChild && torn && child && follows && clamped && restored)) {
        return pictura::selfTest().fail(440, "child float overlay");
    }
    frame.applyPanelSessionForTest(pictura::SessionState{});
    pumpFloat(6);
    return 0;
}
