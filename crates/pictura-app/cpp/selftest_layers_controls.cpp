#include "selftest_layers_controls.h"
#include "selftest_report.h"

#include "frame.h"
#include "panels/layers_panel.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

int pictura::runLayersControlsChecks(pictura::PicturaMainWindow& frame)
{
        // lpc_percent (199): the Opacity control carries 0..100 % and converts
        // to the stored 0..255 byte through the panel, a stored byte displays as
        // its percentage, a programmatic sync adds no history, and the lock
        // strip is five toggles.
        const bool lpcCreated = frame.newDocument(QStringLiteral("PercentCtl"), 16, 16,
                                                  QStringLiteral("rgb"), 8,
                                                  QStringLiteral("white"));
        pictura::PictureView* lpcView = frame.activeView();
        auto* lpcPanel = frame.findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
        if (!lpcCreated || !lpcView || !lpcPanel) {
            return pictura::selfTest().fail(199, "percent fixture");
        }
        const int lpcDoc = frame.activeDocumentIndex();
        lpcPanel->setView(lpcView);
        lpcView->select_all();
        lpcPanel->refresh();
        const int lpcBase = lpcView->history_count();
        lpcPanel->setOpacityPercentForTest(50);
        const bool lpcByteOk =
            lpcView->layer_opacity(0) == 128 && lpcView->history_count() == lpcBase + 1;
        lpcView->set_layers_opacity(QStringList{QStringLiteral("0")}, 128);
        lpcPanel->refresh();
        const bool lpcDisplayOk = lpcPanel->opacityPercentForTest() == 50
            && lpcPanel->lockButtonCountForTest() == 5;
        const int lpcSyncBase = lpcView->history_count();
        lpcPanel->refresh();
        const bool lpcSyncOk = lpcView->history_count() == lpcSyncBase
            && lpcPanel->opacityPercentForTest() == 50;
        ST_BEGIN("lpc_percent");
        ST_PASS("lpc_percent byte=%d display=%d sync=%d", lpcByteOk ? 1 : 0,
                     lpcDisplayOk ? 1 : 0, lpcSyncOk ? 1 : 0);
        if (!lpcByteOk || !lpcDisplayOk || !lpcSyncOk) {
            return pictura::selfTest().fail(199, "percent controls");
        }
        frame.closeDocument(lpcDoc, false);

        // lpc_nesting (200): the nesting flag reaches the row projection, "all"
        // is the four-bit set, Group Layers refuses a nesting-locked node, and
        // Move Up within the container still reorders.
        const bool lpcNestCreated = frame.newDocument(QStringLiteral("NestLock"), 16, 16,
                                                      QStringLiteral("rgb"), 8,
                                                      QStringLiteral("white"));
        pictura::PictureView* lpcNestView = frame.activeView();
        if (!lpcNestCreated || !lpcNestView) {
            return pictura::selfTest().fail(200, "nesting fixture");
        }
        const int lpcNestDoc = frame.activeDocumentIndex();
        const QString lpcGroup = lpcNestView->add_group_in(QString());
        const QString lpcA = lpcNestView->add_layer_in(lpcGroup);
        const QString lpcB = lpcNestView->add_layer_in(lpcGroup);
        lpcNestView->set_layer_name_path(lpcA, QStringLiteral("A"));
        lpcNestView->set_layer_name_path(lpcB, QStringLiteral("B"));
        const auto lpcRow = [lpcNestView](const QString& path) {
            for (int i = 0; i < lpcNestView->layer_row_count(); ++i) {
                if (lpcNestView->layer_row_path(i) == path) {
                    return i;
                }
            }
            return -1;
        };
        const int lpcLockBase = lpcNestView->history_count();
        const int lpcLocked =
            lpcNestView->set_layers_lock(QStringList{lpcA}, QStringLiteral("nesting"), true);
        const bool lpcBitOk = lpcLocked == 1
            && (lpcNestView->layer_row_lock(lpcRow(lpcA)) & 0x08) != 0
            && lpcNestView->history_count() == lpcLockBase + 1;
        const int lpcGroupBase = lpcNestView->history_count();
        const QString lpcWrapped = lpcNestView->group_layers(QStringList{lpcA});
        const bool lpcRefuseOk = lpcWrapped.isEmpty()
            && lpcNestView->history_count() == lpcGroupBase
            && lpcNestView->layer_row_path(lpcRow(lpcA)) == lpcA;
        const int lpcMoveBase = lpcNestView->history_count();
        const bool lpcMoved = lpcNestView->move_layer_path(lpcA, 1);
        const bool lpcMoveOk = lpcMoved
            && lpcNestView->layer_row_name(lpcRow(QStringLiteral("1/1"))) == QStringLiteral("A")
            && lpcNestView->history_count() == lpcMoveBase + 1;
        const int lpcAll = lpcNestView->set_layers_lock(
            QStringList{QStringLiteral("1/1")}, QStringLiteral("all"), true);
        const bool lpcAllOk = lpcAll == 1
            && (lpcNestView->layer_row_lock(lpcRow(QStringLiteral("1/1"))) & 0x0F) == 0x0F;
        ST_BEGIN("lpc_nesting");
        ST_PASS("lpc_nesting bit=%d refuse=%d move=%d all=%d", lpcBitOk ? 1 : 0,
                     lpcRefuseOk ? 1 : 0, lpcMoveOk ? 1 : 0, lpcAllOk ? 1 : 0);
        if (!lpcBitOk || !lpcRefuseOk || !lpcMoveOk || !lpcAllOk) {
            return pictura::selfTest().fail(200, "nesting lock");
        }

        // lpc_chrome (210): header order and labels, no panel menu button, the
        // filter lightswitch is on with an icon, the eye is left-anchored for a
        // nested row, and clicking a group chevron expands it.
        lpcPanel->setView(lpcNestView);
        lpcPanel->refresh();
        const bool chromeOrder = lpcPanel->headerOrderOkForTest();
        const bool chromeLabels =
            lpcPanel->opacityLabelPresentForTest() && lpcPanel->fillLabelPresentForTest();
        const bool chromeNoMenu = !lpcPanel->hasPanelMenuButtonForTest();
        const bool chromeSwitch =
            lpcPanel->filterToggleOnForTest() && lpcPanel->filterToggleHasIconForTest();
        const int eyeGroup = lpcPanel->eyeLeftForTest(lpcGroup);
        const int eyeChild = lpcPanel->eyeLeftForTest(QStringLiteral("1/0"));
        const bool chromeEye = eyeGroup >= 0 && eyeGroup == eyeChild;
        const bool chromeChevron = lpcPanel->chevronClickExpandsForTest(lpcGroup);
        ST_BEGIN("lpc_chrome");
        ST_PASS("lpc_chrome order=%d labels=%d nomenu=%d toggle=%d eye=%d chevron=%d",
                chromeOrder ? 1 : 0, chromeLabels ? 1 : 0, chromeNoMenu ? 1 : 0,
                chromeSwitch ? 1 : 0, chromeEye ? 1 : 0, chromeChevron ? 1 : 0);
        if (!chromeOrder || !chromeLabels || !chromeNoMenu || !chromeSwitch || !chromeEye
            || !chromeChevron) {
            return pictura::selfTest().fail(210, "panel chrome");
        }

        // lpr_rows (211): the percent label and % suffix, the semantic lock
        // icons, and drag enabled on the tree.
        const bool rowsLabel = lpcPanel->opacityLabelTextForTest() == QStringLiteral("Opacity");
        const bool rowsPercent = lpcPanel->opacitySuffixPresentForTest();
        const bool rowsLocks = lpcPanel->lockIconsPresentForTest();
        const bool rowsDrag = lpcPanel->treeDragEnabledForTest();
        ST_BEGIN("lpr_rows");
        ST_PASS("lpr_rows label=%d percent=%d locks=%d drag=%d", rowsLabel ? 1 : 0,
                rowsPercent ? 1 : 0, rowsLocks ? 1 : 0, rowsDrag ? 1 : 0);
        if (!rowsLabel || !rowsPercent || !rowsLocks || !rowsDrag) {
            return pictura::selfTest().fail(211, "row widgets");
        }

        // lpr_drag (212): a reorder is one undo step.
        const int dragBase = lpcNestView->history_count();
        const bool dragged =
            lpcPanel->moveForTest(QStringLiteral("1/0"), QStringLiteral("1/1"), 0);
        const bool dragOk = dragged && lpcNestView->history_count() == dragBase + 1;
        ST_BEGIN("lpr_drag");
        ST_PASS("lpr_drag moved=%d", dragOk ? 1 : 0);
        if (!dragOk) {
            return pictura::selfTest().fail(212, "drag reorder");
        }
        frame.closeDocument(lpcNestDoc, false);

        // lpr_drop (213): dropping a row on Delete removes it in one step.
        const bool dropCreated = frame.newDocument(QStringLiteral("DropCtl"), 16, 16,
                                                   QStringLiteral("rgb"), 8,
                                                   QStringLiteral("white"));
        pictura::PictureView* dropView = frame.activeView();
        if (!dropCreated || !dropView) {
            return pictura::selfTest().fail(213, "drop fixture");
        }
        const int dropDoc = frame.activeDocumentIndex();
        lpcPanel->setView(dropView);
        lpcPanel->refresh();
        const int dropRows = dropView->layer_row_count();
        const int dropBase = dropView->history_count();
        const bool dropped = lpcPanel->dropOnStripButtonForTest(QStringLiteral("layersStripDelete"),
                                                                {QStringLiteral("0")});
        const bool dropOk = dropped && dropView->layer_row_count() == dropRows - 1
            && dropView->history_count() == dropBase + 1;
        ST_BEGIN("lpr_drop");
        ST_PASS("lpr_drop dropped=%d", dropOk ? 1 : 0);
        if (!dropOk) {
            return pictura::selfTest().fail(213, "drop on delete");
        }
        frame.closeDocument(dropDoc, false);

    return 0;
}
