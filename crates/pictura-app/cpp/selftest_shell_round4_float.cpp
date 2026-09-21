#include "selftest_shell_round4_float.h"
#include "selftest_report.h"

#include "frame.h"
#include "panels/panel_column.h"
#include "panels/panel_group.h"
#include "session.h"
#include "toolbox.h"

#include <QtCore/QCoreApplication>
#include <QtCore/QPoint>
#include <QtCore/QRect>
#include <QtWidgets/QMainWindow>
#include <QtWidgets/QSplitter>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QWidget>

namespace {

void pumpFloat(int count)
{
    for (int i = 0; i < count; ++i) {
        QCoreApplication::processEvents();
    }
}

// Dock the tools column, float it with no resolved target, and leave the
// overlay live. Returns the overlay (null when the float did not happen).
pictura::PanelFloat* floatToolsColumnForTest(pictura::PicturaMainWindow& frame,
                                             pictura::PanelColumn* tools)
{
    QWidget* tabs = frame.findChild<QWidget*>(QStringLiteral("documentTabs"));
    if (!tools || !tabs) {
        return nullptr;
    }
    const QPoint parked = tabs->mapToGlobal(tabs->rect().center());
    tools->beginColumnHeaderDragForTest(
        tools->mapToGlobal(QPoint(qMax(1, tools->width() / 2), 8)));
    tools->dragColumnHeaderToForTest(parked);
    tools->dropColumnHeaderForTest(parked);
    pumpFloat(6);
    return tools->columnFloatForTest();
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

    // docked_tools_fixed_width (441): while docked the Tools column is a
    // fixed-width splitter pane: the handle beside it is disabled, so dragging
    // it cannot resize the toolbar.
    {
        frame.applyPanelSessionForTest(pictura::SessionState{});
        pumpFloat(6);
        auto* splitter = frame.findChild<QSplitter*>(QStringLiteral("centerSplitter"));
        auto* tools = frame.toolsColumn();
        bool isPane = false;
        bool handleOff = false;
        if (splitter && tools) {
            const int idx = splitter->indexOf(tools);
            isPane = idx >= 0;
            QSplitterHandle* handle = nullptr;
            if (idx > 0) {
                handle = splitter->handle(idx - 1);
            } else if (idx == 0) {
                handle = splitter->handle(0);
            }
            handleOff = handle && !handle->isEnabled();
        }
        ST_BEGIN("docked_tools_fixed_width");
        ST_PASS("docked_tools_fixed_width pane=%d handle_off=%d", isPane ? 1 : 0,
                handleOff ? 1 : 0);
        if (!(isPane && handleOff)) {
            return pictura::selfTest().fail(441, "docked tools fixed width");
        }
        frame.applyPanelSessionForTest(pictura::SessionState{});
        pumpFloat(6);
    }

    // floating_tools_toggle_resize (442): toggling the floating tools overlay
    // between one and two tool columns re-fits the overlay to the new content on
    // both axes — wider and shorter — so a two-column grid is neither clipped
    // nor left at the one-column height.
    {
        frame.applyPanelSessionForTest(pictura::SessionState{});
        pumpFloat(6);
        auto* tools = frame.toolsColumn();
        auto* toolbox = frame.findChild<pictura::Toolbox*>(QStringLiteral("toolsPanel"));
        bool floated = false;
        bool grew = false;
        bool matches = false;
        bool fitsHeight = false;
        if (tools && toolbox) {
            pictura::PanelFloat* floatWindow = floatToolsColumnForTest(frame, tools);
            floated = floatWindow && floatWindow->isVisible();
            if (floated) {
                const int beforeWidth = floatWindow->width();
                const int beforeHeight = floatWindow->height();
                QToolButton* toggle = tools->panelColumnToggleForTest();
                if (toggle) {
                    toggle->click();
                    pumpFloat(6);
                }
                const int afterWidth = floatWindow->width();
                const int afterHeight = floatWindow->height();
                grew = toolbox->columns() == 2 && afterWidth > beforeWidth;
                matches = afterWidth == tools->minimumWidth();
                fitsHeight = afterHeight < beforeHeight
                             && afterHeight <= tools->minimumSizeHint().height() + 8;
                if (toggle) {
                    toggle->click();
                    pumpFloat(6);
                }
            }
        }
        ST_BEGIN("floating_tools_toggle_resize");
        ST_PASS("floating_tools_toggle_resize floated=%d grew=%d matches=%d fits_h=%d",
                floated ? 1 : 0, grew ? 1 : 0, matches ? 1 : 0, fitsHeight ? 1 : 0);
        if (!(floated && grew && matches && fitsHeight)) {
            return pictura::selfTest().fail(442, "floating tools toggle resize");
        }
        frame.applyPanelSessionForTest(pictura::SessionState{});
        pumpFloat(6);
    }

    // float_column_drag_indicator (443): dragging an already-floating column by
    // the same header path as a docked one resolves the target and draws the
    // edge indicator. In child (Wayland) mode the overlay follows the cursor
    // above the splitter and would cover a viewport-drawn line, so the mark is a
    // frame-level widget raised over the overlay; release re-docks the column.
    {
        frame.applyPanelSessionForTest(pictura::SessionState{});
        pumpFloat(6);
        auto* tools = frame.toolsColumn();
        auto* primary = frame.panelColumn();
        auto* splitter = frame.findChild<QSplitter*>(QStringLiteral("centerSplitter"));
        pictura::PanelFloat::setForceChildOverlayForTest(true);
        bool floated = false;
        bool indicator = false;
        bool onFrame = false;
        bool redocked = false;
        if (tools && primary && splitter) {
            pictura::PanelFloat* floatWindow = floatToolsColumnForTest(frame, tools);
            floated = floatWindow && floatWindow->isVisible();
            if (floated) {
                QWidget* central = frame.centralWidget();
                const QPoint over(central->mapToGlobal(QPoint(2, central->height() / 2)));
                tools->beginColumnHeaderDragForTest(
                    floatWindow->mapToGlobal(QPoint(floatWindow->width() / 2, 8)));
                tools->dragColumnHeaderToForTest(over);
                indicator = primary->dropIndicatorVisibleForTest();
                onFrame = primary->edgeIndicatorOnFrameForTest();
                redocked = tools->dropColumnHeaderForTest(over)
                           && tools->columnFloatForTest() == nullptr
                           && splitter->indexOf(tools) >= 0 && !tools->isWindow();
                pumpFloat(6);
            }
        }
        pictura::PanelFloat::setForceChildOverlayForTest(false);
        ST_BEGIN("float_column_drag_indicator");
        ST_PASS("float_column_drag_indicator floated=%d indicator=%d on_frame=%d "
                "redocked=%d",
                floated ? 1 : 0, indicator ? 1 : 0, onFrame ? 1 : 0, redocked ? 1 : 0);
        if (!(floated && indicator && onFrame && redocked)) {
            return pictura::selfTest().fail(443, "float column drag indicator");
        }
        frame.applyPanelSessionForTest(pictura::SessionState{});
        pumpFloat(6);
    }
    return 0;
}
