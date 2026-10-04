#include <QtTest/QtTest>

#include <QtCore/QCoreApplication>
#include <QtCore/QStringList>
#include <QtGui/QAction>
#include <QtWidgets/QMenuBar>
#include <QtWidgets/QToolButton>

#include "commands.h"
#include "frame.h"
#include "panels/layers_panel.h"
#include "panels/panel_column.h"
#include "panels/panel_group.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "session.h"
#include "toolbox.h"

#include "qt_test_support.h"

class CommandTreeTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void menus();
    void dispatch();
    void menusPanel();
    void menuCount();
    void menubarClear();
    void panelMenusToolsIcons();
    void widgetmenuButton();
    void layerAdjustmentMenu();

private:
    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void CommandTreeTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
    QVERIFY(window_->newDocument(QStringLiteral("CommandTree"), 512, 512, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    window_->resize(1100, 700);
    window_->show();
    QCoreApplication::processEvents();
}

void CommandTreeTest::menus()
{
    const QStringList expectedMenus = {
        QStringLiteral("File"),  QStringLiteral("Edit"),   QStringLiteral("Image"),
        QStringLiteral("Layer"), QStringLiteral("Type"),   QStringLiteral("Select"),
        QStringLiteral("Filter"), QStringLiteral("View"),  QStringLiteral("Window"),
        QStringLiteral("Help")};
    QCOMPARE(window_->topLevelMenuTitles(), expectedMenus);
}

void CommandTreeTest::dispatch()
{
    pictura::PicturaMainWindow& frame = *window_;
    pictura::PictureView* view = frame.activeView();
    QVERIFY(view != nullptr);
    pictura::CommandRegistry* registry = frame.registry();
    QVERIFY(registry != nullptr);
    QVERIFY2(registry->dispatch(QString::fromLatin1(pictura::command_ids::SelectAll)),
             "dispatch select all");
    QVERIFY2(view->has_selection(), "select all selects");
    QVERIFY2(!registry->dispatch(QStringLiteral("no.such.command")), "unknown id inert");
    view->deselect();
}

void CommandTreeTest::menusPanel()
{
    auto* panel = window_->findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
    QVERIFY(panel != nullptr);
    const QStringList expectedRow = {
        QStringLiteral("Rename"),           QStringLiteral("New Layer"),
        QStringLiteral("New Group"),        QStringLiteral("Duplicate Layer(s)"),
        QStringLiteral("Delete Layer(s)"),  QStringLiteral("Group Layers"),
        QStringLiteral("Ungroup Layers"),   QStringLiteral("Move Layer Up"),
        QStringLiteral("Move Layer Down"),  QStringLiteral("Export As…"),
        QStringLiteral("Quick Export as PNG"), QStringLiteral("Color Label")};
    const QStringList expectedColor = {
        QStringLiteral("None"),  QStringLiteral("Red"),  QStringLiteral("Orange"),
        QStringLiteral("Yellow"), QStringLiteral("Green"), QStringLiteral("Blue"),
        QStringLiteral("Violet"), QStringLiteral("Gray")};
    const QStringList groupRow = panel->rowMenuTextsForTest(QStringLiteral("group"));
    const QStringList adjustmentRow = panel->rowMenuTextsForTest(QStringLiteral("adjustment"));
    const QStringList typeRow = panel->rowMenuTextsForTest(QStringLiteral("type"));
    const bool groupHidden = !groupRow.contains(QStringLiteral("Export As…"))
        && !groupRow.contains(QStringLiteral("Quick Export as PNG"));
    const bool adjustmentHidden = !adjustmentRow.contains(QStringLiteral("Export As…"))
        && !adjustmentRow.contains(QStringLiteral("Quick Export as PNG"));
    const bool typeHidden = !typeRow.contains(QStringLiteral("Export As…"))
        && !typeRow.contains(QStringLiteral("Quick Export as PNG"));
    QVERIFY2(panel->rowMenuTextsForTest() == expectedRow && groupHidden && adjustmentHidden
                 && typeHidden,
             "row menu");
    QVERIFY2(panel->colorLabelTextsForTest() == expectedColor, "color label menu");
}

void CommandTreeTest::menuCount()
{
    pictura::PanelColumn* column = window_->panelColumn();
    QVERIFY(column != nullptr);
    const QStringList expectedMenu = {
        QStringLiteral("Close"),
        QStringLiteral("Close Panel Group"),
        QStringLiteral("Minimize"),
        QStringLiteral("Collapse to Icons"),
        QStringLiteral("Auto-Collapse Iconic Panels"),
        QStringLiteral("Auto-Show Hidden Panels"),
        QStringLiteral("Interface Options\u2026"),
    };
    QCOMPARE(column->tabMenuActionsForTest(), expectedMenu);
    const bool a0 = column->autoCollapseIconicForTest();
    column->triggerTabMenuForTest(QStringLiteral("Auto-Collapse Iconic Panels"));
    const bool a1 = column->autoCollapseIconicForTest();
    column->triggerTabMenuForTest(QStringLiteral("Auto-Collapse Iconic Panels"));
    const bool a2 = column->autoCollapseIconicForTest();
    const bool b0 = column->autoShowHiddenForTest();
    column->triggerTabMenuForTest(QStringLiteral("Auto-Show Hidden Panels"));
    const bool b1 = column->autoShowHiddenForTest();
    column->triggerTabMenuForTest(QStringLiteral("Auto-Show Hidden Panels"));
    const bool b2 = column->autoShowHiddenForTest();
    QVERIFY2(a1 != a0 && a2 == a0 && b1 != b0 && b2 == b0, "tab menu checkables toggle");
    bool optionsFired = false;
    QObject::connect(column, &pictura::PanelColumn::interfaceOptionsRequested, column,
                     [&optionsFired]() { optionsFired = true; });
    column->triggerTabMenuForTest(QStringLiteral("Interface Options\u2026"));
    QVERIFY2(optionsFired, "interface options signal");
}

void CommandTreeTest::menubarClear()
{
    QMenuBar* bar = window_->menuBar();
    QVERIFY(bar != nullptr);
    QCoreApplication::processEvents();
    bool menuClear = true;
    QString menuOffender;
    const QRect barRect(bar->mapToGlobal(QPoint(0, 0)), bar->size());
    for (QWidget* child : window_->findChildren<QWidget*>()) {
        if (!child || child == bar || !child->isVisible() || bar->isAncestorOf(child)
            || child->window() != window_.get()) {
            continue;
        }
        const QRect childRect(child->mapToGlobal(QPoint(0, 0)), child->size());
        if (childRect.intersects(barRect)) {
            menuClear = false;
            if (menuOffender.isEmpty()) {
                menuOffender = child->objectName().isEmpty()
                                   ? QString::fromLatin1(child->metaObject()->className())
                                   : child->objectName();
            }
        }
    }
    QVERIFY2(menuClear, qPrintable(QStringLiteral("menu-bar overlay=") + menuOffender));
    const bool staleDiscarded = !window_->restoreStoredLayout(
        QByteArrayLiteral("stale-layout"), window_->layoutRevisionForTest() - 1);
    QVERIFY2(staleDiscarded, "stale layout discarded");
    window_->saveSession();
    QVERIFY2(pictura::loadSession().layoutRevision == window_->layoutRevisionForTest(),
             "layout revision saved");
}

void CommandTreeTest::panelMenusToolsIcons()
{
    auto* toolbox = window_->findChild<pictura::Toolbox*>(QStringLiteral("toolsPanel"));
    QVERIFY(toolbox != nullptr);
    toolbox->setColumns(1);
    QCoreApplication::processEvents();
    const QList<QToolButton*> slotButtons = toolbox->slotButtons();
    bool toolIcons = !slotButtons.isEmpty();
    for (QToolButton* button : slotButtons) {
        if (button->minimumWidth() < 32 || button->iconSize().width() < 22) {
            toolIcons = false;
        }
    }
    auto* screenMode = toolbox->findChild<QToolButton*>(QStringLiteral("screenModeButton"));
    if (screenMode && !screenMode->icon().isNull()
        && (screenMode->minimumWidth() < 32 || screenMode->iconSize().width() < 22)) {
        toolIcons = false;
    }
    const int min1 = toolbox->minimumWidth();
    const int content1 = toolbox->contentWidthForTest();
    toolbox->setColumns(2);
    QCoreApplication::processEvents();
    const int min2 = toolbox->minimumWidth();
    const int content2 = toolbox->contentWidthForTest();
    const bool toolTight =
        min1 > 0 && min1 == content1 && min2 == content2 && min2 > min1;
    toolbox->setColumns(1);
    QCoreApplication::processEvents();
    QVERIFY2(toolIcons, "slot/screen-mode icons are large enough");
    QVERIFY2(toolTight, "one/two-column widths equal content width");
}

void CommandTreeTest::widgetmenuButton()
{
    pictura::PanelColumn* column = window_->panelColumn();
    QVERIFY(column != nullptr);
    column->setRailMode(false);
    column->ensureGroupVisibleForTest(QStringLiteral("layersPanel"));
    QCoreApplication::processEvents();
    pictura::PanelGroup* layerGroup = column->groupForPanel(QStringLiteral("layersPanel"));
    QVERIFY(layerGroup != nullptr);
    column->showPanel(QStringLiteral("layersPanel"), true);
    QCoreApplication::processEvents();
    QToolButton* btnPtr = layerGroup->headerMenuButtonForTest();
    QToolButton* hooked = column->widgetMenuButtonForTest(layerGroup->objectName());
    QVERIFY2(btnPtr && hooked == btnPtr && btnPtr->isVisible()
                 && layerGroup->headerMenuAtRightForTest(),
             "widget menu button at header right");

    const QStringList layerTexts = layerGroup->panelMenuTextsForTest();
    column->showPanel(QStringLiteral("channelsPanel"), true);
    QCoreApplication::processEvents();
    const QStringList channelTexts = layerGroup->panelMenuTextsForTest();
    const bool follows = layerGroup->headerMenuButtonForTest()
        && layerGroup->headerMenuButtonForTest()->objectName()
               == QStringLiteral("panelWidgetMenu_channelsPanel");
    const QStringList colorTexts =
        column->widgetMenuTextsForTest(QStringLiteral("colorPanel"));
    const bool firstsDiffer = !layerTexts.isEmpty() && !channelTexts.isEmpty()
        && !colorTexts.isEmpty() && layerTexts.first() != channelTexts.first()
        && layerTexts.first() != colorTexts.first();
    column->showPanel(QStringLiteral("layersPanel"), true);
    QCoreApplication::processEvents();
    const bool options = layerGroup->panelMenuEnabledForTest(QStringLiteral("Panel Options…"));
    QVERIFY2(follows && firstsDiffer && options, "per-panel widget menu");

    const bool copyDisabled = !layerGroup->panelMenuEnabledForTest(QStringLiteral("Copy CSS"));
    const bool blendDisabled =
        !layerGroup->panelMenuEnabledForTest(QStringLiteral("Blending Options…"));
    const bool triggerBlocked = !column->triggerWidgetMenuForTest(
        QStringLiteral("layersPanel"), QStringLiteral("Copy CSS"));
    QVERIFY2(copyDisabled && blendDisabled && triggerBlocked
                 && layerGroup->panelMenuToolTipForTest(QStringLiteral("Copy CSS"))
                        == QStringLiteral("Copy CSS — not implemented yet"),
             "disabled entries");
    QVERIFY2(column->widgetMenuHasCloseForTest(QStringLiteral("layersPanel"))
                 && column->widgetMenuHasCloseForTest(QStringLiteral("channelsPanel"))
                 && column->widgetMenuHasCloseForTest(QStringLiteral("colorPanel"))
                 && column->widgetMenuHasCloseForTest(QStringLiteral("historyPanel")),
             "close stays off the widget menu");
}

// Layer > New Adjustment Layer: every CS6 kind is implemented, enabled with a
// document open, and creates an adjustment layer named for its menu entry;
// Shadows/Highlights and HDR Toning are destructive Image commands, not kinds.
void CommandTreeTest::layerAdjustmentMenu()
{
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    pictura::CommandRegistry* registry = window_->registry();
    QVERIFY(registry != nullptr);

    const QStringList leaves = {
        QStringLiteral("Brightness/Contrast"), QStringLiteral("Levels"),
        QStringLiteral("Curves"),              QStringLiteral("Exposure"),
        QStringLiteral("Vibrance"),            QStringLiteral("Hue/Saturation"),
        QStringLiteral("Color Balance"),       QStringLiteral("Black & White"),
        QStringLiteral("Photo Filter"),        QStringLiteral("Channel Mixer"),
        QStringLiteral("Color Lookup"),        QStringLiteral("Invert"),
        QStringLiteral("Posterize"),           QStringLiteral("Threshold"),
        QStringLiteral("Gradient Map"),        QStringLiteral("Selective Color")};
    for (const QString& name : leaves) {
        QAction* action = registry->action(pictura::commandIdForPath(
            {QStringLiteral("Layer"), QStringLiteral("New Adjustment Layer"), name}));
        QVERIFY2(action, qPrintable(name));
        registry->refresh();
        QVERIFY2(action->isEnabled(), qPrintable(name));
        const QString expected =
            name == QStringLiteral("Invert") ? name : name + QStringLiteral("\u2026");
        QCOMPARE(action->text(), expected);

        const int before = view->layer_row_count();
        action->trigger();
        QCOMPARE(view->layer_row_count(), before + 1);
        bool named = false;
        for (int i = 0; i < view->layer_row_count(); ++i) {
            if (view->layer_row_name(i) == name) {
                named = true;
            }
        }
        QVERIFY2(named, qPrintable(name));
    }

    for (const QString& gone :
         {QStringLiteral("Shadows/Highlights"), QStringLiteral("HDR Toning")}) {
        QVERIFY2(!registry->action(pictura::commandIdForPath(
                     {QStringLiteral("Layer"), QStringLiteral("New Adjustment Layer"), gone})),
                 qPrintable(gone));
    }

    // The Adjustments panel lists the same sixteen kinds in CS6 panel order,
    // then the two trailing toggles.
    pictura::PanelColumn* column = window_->panelColumn();
    QVERIFY(column != nullptr);
    const QStringList expectedPanel = {
        QStringLiteral("Brightness-Contrast"), QStringLiteral("Levels"),
        QStringLiteral("Curves"),              QStringLiteral("Exposure"),
        QStringLiteral("Vibrance"),            QStringLiteral("Hue-Saturation"),
        QStringLiteral("Color Balance"),       QStringLiteral("Black & White"),
        QStringLiteral("Photo Filter"),        QStringLiteral("Channel Mixer"),
        QStringLiteral("Color Lookup"),        QStringLiteral("Invert"),
        QStringLiteral("Posterize"),           QStringLiteral("Threshold"),
        QStringLiteral("Gradient Map"),        QStringLiteral("Selective Color"),
        QStringLiteral("Add Mask by Default"), QStringLiteral("Clip to Layer")};
    QCOMPARE(column->widgetMenuTextsForTest(QStringLiteral("adjustmentsPanel")), expectedPanel);
}

QTEST_MAIN(CommandTreeTest)
#include "tst_command_tree.moc"
