#include "selftest_shell_round3.h"
#include "selftest_report.h"

#include "frame.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/Qt>
#include <QtCore/QCoreApplication>
#include <QtWidgets/QApplication>
#include <QtWidgets/QTabBar>
#include <QtWidgets/QTabWidget>

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

    return 0;
}
