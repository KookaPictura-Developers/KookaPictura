#include "selftest_shell_round3.h"
#include "selftest_report.h"

#include "frame.h"
#include "panels/panel_column.h"
#include "panels/panel_group.h"
#include "session.h"
#include "toolbox.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/Qt>
#include <QtCore/QCoreApplication>
#include <QtGui/QMouseEvent>
#include <QtWidgets/QApplication>
#include <QtWidgets/QSplitter>
#include <QtWidgets/QTabBar>
#include <QtWidgets/QTabWidget>
#include <QtWidgets/QToolButton>

int pictura::runShellRound3Checks(pictura::PicturaMainWindow& frame)
{
    // lss_empty_pane (349): the document pane stays in the workspace with no
    // document (so the widget columns cannot absorb it) while its empty tab strip
    // is hidden and reappears on create; closing the last document hides the
    // strip again.
    {
        while (frame.documentCount() > 0) {
            frame.closeDocument(0, false);
        }
        auto* tabs = frame.findChild<QTabWidget*>(QStringLiteral("documentTabs"));
        if (!tabs) {
            return pictura::selfTest().fail(349, "document tab pane missing");
        }
        QCoreApplication::processEvents();
        const bool spaceEmpty = tabs->isVisible() && tabs->width() > 0
                                && tabs->tabBar() && !tabs->tabBar()->isVisible();
        const bool created = frame.newDocument(QStringLiteral("Round3"), 32, 32,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
        const bool stripWithDoc = tabs->isVisible() && tabs->tabBar()->isVisible();
        const int doc = frame.activeDocumentIndex();
        frame.closeDocument(doc, false);
        QCoreApplication::processEvents();
        const bool spaceAgain = tabs->isVisible() && !tabs->tabBar()->isVisible();

        ST_BEGIN("lss_empty_pane");
        ST_PASS("lss_empty_pane space=%d strip=%d space2=%d", spaceEmpty ? 1 : 0,
                stripWithDoc ? 1 : 0, spaceAgain ? 1 : 0);
        if (!spaceEmpty || !created || !stripWithDoc || !spaceAgain) {
            return pictura::selfTest().fail(349, "empty workspace did not keep its space");
        }
    }

    // lss_fresh_pane (380): a freshly constructed window keeps the empty document
    // pane in the workspace (with visible space reserved for it) while hiding the
    // empty tab strip; creating a document shows the strip, and closing the last
    // one hides it again.
    {
        pictura::PicturaMainWindow fresh;
        auto* freshTabs = fresh.findChild<QTabWidget*>(QStringLiteral("documentTabs"));
        if (!freshTabs) {
            return pictura::selfTest().fail(380, "fresh document tab pane missing");
        }
        const bool freshSpace = freshTabs->isVisibleTo(&fresh)
                                && !freshTabs->tabBar()->isVisibleTo(&fresh);
        const bool freshCreated = fresh.newDocument(QStringLiteral("Fresh"), 32, 32,
                                                    QStringLiteral("rgb"), 8,
                                                    QStringLiteral("white"));
        const bool freshStrip = freshTabs->isVisibleTo(&fresh)
                                && freshTabs->tabBar()->isVisibleTo(&fresh);
        while (fresh.documentCount() > 0) {
            fresh.closeDocument(0, false);
        }
        const bool freshSpaceAgain = freshTabs->isVisibleTo(&fresh)
                                     && !freshTabs->tabBar()->isVisibleTo(&fresh);
        ST_BEGIN("lss_fresh_pane");
        ST_PASS("lss_fresh_pane space=%d strip=%d space2=%d", freshSpace ? 1 : 0,
                freshStrip ? 1 : 0, freshSpaceAgain ? 1 : 0);
        if (!freshSpace || !freshCreated || !freshStrip || !freshSpaceAgain) {
            return pictura::selfTest().fail(380, "fresh window ghost pane");
        }
    }

    // lss_tab_weight (350): the scoped document-tab rule carries medium weight
    // and extra right padding without touching the unscoped panel rules.
    {
        const QString sheet = qApp->styleSheet();
        const bool scoped = sheet.contains(QStringLiteral("QTabBar#documentTabBar::tab {"));
        const bool weight = sheet.contains(QStringLiteral("font-weight: 500"));
        const bool padding = sheet.contains(QStringLiteral("padding-right: 12px"));

        ST_BEGIN("lss_tab_weight");
        ST_PASS("lss_tab_weight scoped=%d weight=%d padding=%d", scoped ? 1 : 0, weight ? 1 : 0,
                padding ? 1 : 0);
        if (!scoped || !weight || !padding) {
            return pictura::selfTest().fail(350, "document tab rule missing");
        }
    }

    // lss_brush_keys (351): the pure helper maps US keys, evdev scan codes, and
    // their Shift variants, and returns 0 for non-paint tools and other keys.
    {
        const bool created = frame.newDocument(QStringLiteral("Round3Keys"), 32, 32,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
        PictureView* view = frame.activeView();
        if (!created || !view) {
            return pictura::selfTest().fail(351, "brush key fixture");
        }
        const int doc = frame.activeDocumentIndex();
        const bool usKeys =
            view->brush_shortcut_delta(Qt::Key_BracketLeft, 0, false, true) == -1
            && view->brush_shortcut_delta(Qt::Key_BracketRight, 0, false, true) == 1
            && view->brush_shortcut_delta(Qt::Key_BraceLeft, 0, true, true) == -5
            && view->brush_shortcut_delta(Qt::Key_BraceRight, 0, true, true) == 5;
        const bool nativeKeys =
            view->brush_shortcut_delta(0, 34, false, true) == -1
            && view->brush_shortcut_delta(0, 35, false, true) == 1
            && view->brush_shortcut_delta(0, 34, true, true) == -5
            && view->brush_shortcut_delta(0, 35, true, true) == 5;
        const bool guard =
            view->brush_shortcut_delta(Qt::Key_BracketRight, 35, false, false) == 0
            && view->brush_shortcut_delta(Qt::Key_A, 0, false, true) == 0;

        ST_BEGIN("lss_brush_keys");
        ST_PASS("lss_brush_keys us=%d native=%d guard=%d", usKeys ? 1 : 0, nativeKeys ? 1 : 0,
                guard ? 1 : 0);
        const bool ok = usKeys && nativeKeys && guard;
        if (!ok) {
            frame.closeDocument(doc, false);
            return pictura::selfTest().fail(351, "brush key mapping wrong");
        }
    }

    auto pump = [](int count) {
        for (int i = 0; i < count; ++i) {
            QCoreApplication::processEvents();
        }
    };

    // lss_col_header_move (401): a press-drag-release on a column's top header
    // moves the whole column onto a side of another column through the real
    // resolve/move path, and the frame's edge indicator shows while dragging.
    {
        frame.applyPanelSessionForTest(pictura::SessionState{});
        pump(6);
        pictura::PanelColumn* primary = frame.panelColumn();
        const bool made =
            frame.newColumnDropForTest(QStringLiteral("stylesPanel"), QStringLiteral("left"));
        pump(6);
        pictura::PanelColumn* left = frame.columnForPanel(QStringLiteral("stylesPanel"));
        bool indicator = false;
        bool moved = false;
        if (primary && left && primary != left) {
            const QRect lr(left->mapToGlobal(QPoint(0, 0)), left->size());
            const QPoint target(lr.left() + qMax(1, lr.width() / 4), lr.center().y());
            QWidget* header = primary->columnHeaderForTest();
            const bool began = primary->beginColumnHeaderDragForTest(target);
            primary->dragColumnHeaderToForTest(target);
            indicator = left->dropIndicatorVisibleForTest();
            const bool dropped = primary->dropColumnHeaderForTest(target);
            pump(6);
            const QList<pictura::PanelColumn*> order = frame.panelColumns();
            moved = header && began && dropped && !order.isEmpty() && order.first() == primary
                    && primary->isVisible() && primary->width() > 0;
        }
        ST_BEGIN("lss_col_header_move");
        ST_PASS("lss_col_header_move made=%d indicator=%d moved=%d", made ? 1 : 0,
                indicator ? 1 : 0, moved ? 1 : 0);
        if (!(made && indicator && moved)) {
            return pictura::selfTest().fail(401, "column header move");
        }
        frame.applyPanelSessionForTest(pictura::SessionState{});
        pump(6);
    }

    // lss_tab_compress (402): many-tab groups squeeze/elide instead of showing
    // scroll arrows (usesScrollButtons off), and the bar still does not expand.
    {
        pictura::PanelColumn* column = frame.panelColumn();
        pictura::PanelGroup* group =
            column && !column->groups().isEmpty() ? column->groups().first() : nullptr;
        const bool noScroll = group && !group->tabUsesScrollButtonsForTest();
        const bool notExpanding = group && group->tabBar() && !group->tabBar()->expanding();
        ST_BEGIN("lss_tab_compress");
        ST_PASS("lss_tab_compress noScroll=%d expanding=%d", noScroll ? 1 : 0,
                notExpanding ? 1 : 0);
        if (!(group && noScroll && notExpanding)) {
            return pictura::selfTest().fail(402, "tab compression");
        }
    }

    // lss_grip_drag (403): the group corner's reserved grip is present, draggable
    // and drives a whole-group drag through the existing signal path.
    {
        pictura::PanelColumn* column = frame.panelColumn();
        pictura::PanelGroup* group =
            column && !column->groups().isEmpty() ? column->groups().first() : nullptr;
        bool hasGrip = false;
        bool dragStarted = false;
        if (group) {
            QWidget* grip = group->headerGripForTest();
            hasGrip = grip && grip->objectName() == QStringLiteral("panelGroupDragGrip")
                      && grip->cursor().shape() == Qt::SizeAllCursor && grip->isVisible();
            const QMetaObject::Connection conn =
                QObject::connect(group, &pictura::PanelGroup::groupDragStarted,
                                 [&dragStarted](const QPoint&) { dragStarted = true; });
            if (grip) {
                const QPoint start = grip->mapToGlobal(QPoint(2, 2));
                QMouseEvent press(QEvent::MouseButtonPress, QPointF(2, 2), QPointF(start),
                                  Qt::LeftButton, Qt::LeftButton, Qt::NoModifier);
                QApplication::sendEvent(grip, &press);
                const QPoint moved = start + QPoint(24, 0);
                QMouseEvent move(QEvent::MouseMove, QPointF(26, 2), QPointF(moved), Qt::NoButton,
                                 Qt::LeftButton, Qt::NoModifier);
                QApplication::sendEvent(grip, &move);
                column->cancelDragForTest();
                QMouseEvent release(QEvent::MouseButtonRelease, QPointF(26, 2), QPointF(moved),
                                    Qt::LeftButton, Qt::NoButton, Qt::NoModifier);
                QApplication::sendEvent(grip, &release);
                pump(4);
            }
            QObject::disconnect(conn);
        }
        ST_BEGIN("lss_grip_drag");
        ST_PASS("lss_grip_drag grip=%d started=%d", hasGrip ? 1 : 0, dragStarted ? 1 : 0);
        if (!(hasGrip && dragStarted)) {
            return pictura::selfTest().fail(403, "header grip drag");
        }
    }

    // lss_float_header (404): a floating group shows its top bar with a working
    // collapse toggle and the close control (moved out of the corner), and the
    // bar itself drags the overlay.
    {
        frame.applyPanelSessionForTest(pictura::SessionState{});
        pump(6);
        pictura::PanelColumn* column = frame.panelColumn();
        pictura::PanelGroup* group =
            column && !column->groups().isEmpty() ? column->groups().first() : nullptr;
        bool torn = false;
        bool headerShown = false;
        bool toggleWorks = false;
        bool closeInHeader = false;
        bool dragMoved = false;
        if (column && group) {
            torn = column->tearOffForTest(group->objectName());
            pump(8);
            headerShown = group->floatHeaderVisibleForTest();
            QToolButton* toggle = group->floatToggleForTest();
            const bool wasCollapsed = group->isCollapsedToIcons();
            if (toggle) {
                toggle->click();
                pump(4);
                toggleWorks = group->isCollapsedToIcons() != wasCollapsed;
            }
            QWidget* floatHeader = group->floatHeaderForTest();
            QToolButton* close = group->floatCloseButton();
            closeInHeader = close && floatHeader && close->parentWidget() == floatHeader;
            const QRect before = column->floatGeometryForTest(0);
            if (floatHeader) {
                const QPoint start = floatHeader->mapToGlobal(QPoint(4, 4));
                QMouseEvent press(QEvent::MouseButtonPress, QPointF(4, 4), QPointF(start),
                                  Qt::LeftButton, Qt::LeftButton, Qt::NoModifier);
                QApplication::sendEvent(floatHeader, &press);
                const QPoint moved = start + QPoint(40, 25);
                QMouseEvent move(QEvent::MouseMove, QPointF(44, 29), QPointF(moved), Qt::NoButton,
                                 Qt::LeftButton, Qt::NoModifier);
                QApplication::sendEvent(floatHeader, &move);
                QMouseEvent release(QEvent::MouseButtonRelease, QPointF(44, 29), QPointF(moved),
                                    Qt::LeftButton, Qt::NoButton, Qt::NoModifier);
                QApplication::sendEvent(floatHeader, &release);
                pump(8);
                const QRect after = column->floatGeometryForTest(0);
                dragMoved = after.topLeft() != before.topLeft();
            }
            column->closeFloatForTest(0);
            pump(6);
        }
        ST_BEGIN("lss_float_header");
        ST_PASS("lss_float_header torn=%d header=%d toggle=%d close=%d drag=%d", torn ? 1 : 0,
                headerShown ? 1 : 0, toggleWorks ? 1 : 0, closeInHeader ? 1 : 0,
                dragMoved ? 1 : 0);
        if (!(torn && headerShown && toggleWorks && closeInHeader && dragMoved)) {
            return pictura::selfTest().fail(404, "float header");
        }
        frame.applyPanelSessionForTest(pictura::SessionState{});
        pump(6);
    }

    // lss_float_icons (405): collapsing a floating group to icons grows the
    // overlay to at least the icon-row minimum and the iconic row is styled as a
    // panel surface.
    {
        frame.applyPanelSessionForTest(pictura::SessionState{});
        pump(6);
        pictura::PanelColumn* column = frame.panelColumn();
        pictura::PanelGroup* group =
            column && !column->groups().isEmpty() ? column->groups().first() : nullptr;
        bool torn = false;
        bool minHeight = false;
        bool themed = false;
        if (column && group) {
            torn = column->tearOffForTest(group->objectName());
            pump(8);
            group->setCollapsedToIcons(true);
            pump(8);
            minHeight = group->isCollapsedToIcons()
                        && column->floatGeometryForTest(0).height()
                               >= pictura::PanelFloat::kFloatIconMinHeight;
            themed = qApp->styleSheet().contains(QStringLiteral("QWidget#panelGroupIconRow {"));
            column->closeFloatForTest(0);
            pump(6);
        }
        ST_BEGIN("lss_float_icons");
        ST_PASS("lss_float_icons torn=%d minHeight=%d themed=%d", torn ? 1 : 0,
                minHeight ? 1 : 0, themed ? 1 : 0);
        if (!(torn && minHeight && themed)) {
            return pictura::selfTest().fail(405, "float icon mode");
        }
        frame.applyPanelSessionForTest(pictura::SessionState{});
        pump(6);
    }

    // lss_toolbox_column_drop (406): the Tools panel can land on either side of a
    // widget column from a point in that column's interior (left half before it,
    // right half after it), and both workspace outer bands land it at the
    // splitter head/tail, through the same resolve/commit path the title-bar drag
    // drives.
    {
        frame.applyPanelSessionForTest(pictura::SessionState{});
        pump(6);
        const bool leftHalf =
            frame.toolboxOnColumnForTest(QStringLiteral("layersPanel"), false);
        const bool rightHalf =
            frame.toolboxOnColumnForTest(QStringLiteral("layersPanel"), true);
        // Leave the default chrome for the following suites: re-dock the tools
        // and drop the throwaway left column.
        if (auto* toolbox =
                frame.findChild<pictura::Toolbox*>(QStringLiteral("toolsPanel"))) {
            auto* cs = frame.findChild<QSplitter*>(QStringLiteral("centerSplitter"));
            if (cs && cs->indexOf(toolbox) >= 0) {
                frame.addDockWidget(Qt::LeftDockWidgetArea, toolbox);
                toolbox->setSplitterPane(false);
            }
        }
        frame.applyPanelSessionForTest(pictura::SessionState{});
        pump(6);
        ST_BEGIN("lss_toolbox_column_drop");
        ST_PASS("lss_toolbox_column_drop left=%d right=%d", leftHalf ? 1 : 0,
                rightHalf ? 1 : 0);
        if (!(leftHalf && rightHalf)) {
            return pictura::selfTest().fail(406, "toolbox column drop");
        }
    }

    return 0;
}
