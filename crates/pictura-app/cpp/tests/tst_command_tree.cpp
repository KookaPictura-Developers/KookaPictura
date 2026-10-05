#include <QtTest/QtTest>
#include <QtTest/QSignalSpy>

#include <QtCore/QCoreApplication>
#include <QtCore/QStringList>
#include <QtGui/QAction>
#include <QtGui/QImage>
#include <QtGui/QMouseEvent>
#include <QtGui/QPalette>
#include <QtWidgets/QApplication>
#include <QtWidgets/QMenu>
#include <QtWidgets/QMenuBar>
#include <QtWidgets/QToolButton>

#include "commands.h"
#include "frame.h"
#include "icons.h"
#include "panels/layers_panel.h"
#include "panels/panel_column.h"
#include "panels/panel_column_internal.h"
#include "panels/panel_group.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "preferences_dialog.h"
#include "session.h"
#include "theme.h"
#include "tool_hint_bar.h"
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
    void toolboxSlotMetrics();
    void toolboxInteractions();
    void widgetmenuButton();
    void panelGroupChrome();
    void selfAnchorDock();
    void layerAdjustmentMenu();
    void railModeResizeGrip();
    void menuMoves();
    void preferencesPages();
    void toolHintsToggle();

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

void CommandTreeTest::toolboxSlotMetrics()
{
    using pictura::Toolbox;
    QCOMPARE(Toolbox::slotSizeForDpi(96.0), QSize(36, 28));
    QCOMPARE(Toolbox::iconSizeForDpi(96.0), QSize(24, 20));
    QCOMPARE(Toolbox::slotSizeForDpi(192.0), QSize(72, 56));
    QCOMPARE(Toolbox::iconSizeForDpi(192.0), QSize(48, 40));

    auto* toolbox = window_->findChild<pictura::Toolbox*>(QStringLiteral("toolsPanel"));
    QVERIFY(toolbox != nullptr);
    const QList<QToolButton*> slotList = toolbox->slotButtons();
    QCOMPARE(int(slotList.size()), 21);
    QVERIFY(!slotList.isEmpty());
    const qreal dpi = toolbox->screen() ? toolbox->screen()->logicalDotsPerInch() : 96.0;
    // The fixed size follows logical DPI alone; Qt multiplies by the device
    // pixel ratio at paint time, so we must not fold that ratio in here.
    const QSize expected = Toolbox::slotSizeForDpi(dpi);
    for (QToolButton* button : slotList) {
        QCOMPARE(button->size(), expected);
        QCOMPARE(button->minimumWidth(), expected.width());
        QVERIFY(button->minimumWidth() >= 32);
        QVERIFY(button->iconSize().width() >= 22);
    }
}

void CommandTreeTest::toolboxInteractions()
{
    auto* toolbox = window_->findChild<pictura::Toolbox*>(QStringLiteral("toolsPanel"));
    QVERIFY(toolbox != nullptr);
    toolbox->setColumns(1);
    QCoreApplication::processEvents();

    // A flyout opens beside its button, never over it, and carries the icon
    // margin styling.
    toolbox->openSlotFlyoutForTest(2);
    QCoreApplication::processEvents();
    QMenu* flyout = toolbox->slotMenuForTest(2);
    QVERIFY(flyout != nullptr);
    QVERIFY(flyout->isVisible());
    QVERIFY(!flyout->styleSheet().isEmpty());
    QToolButton* button2 = toolbox->slotButtons().value(1);
    QVERIFY(button2 != nullptr);
    const QPoint btnTopLeft = button2->mapToGlobal(QPoint(0, 0));
    const int btnRight = btnTopLeft.x() + button2->width();
    QVERIFY2(flyout->pos().x() >= btnRight
                 || flyout->pos().x() + flyout->width() <= btnTopLeft.x(),
             "flyout does not cover the button");

    // A real click on another slot both closes the flyout and switches the
    // tool; the app-level filter forwards the press past the open popup.
    QToolButton* button1 = toolbox->slotButtons().value(0);
    QVERIFY(button1 != nullptr);
    window_->setActiveTool(pictura::ToolId::Brush);
    QCoreApplication::processEvents();
    QCOMPARE(window_->activeTool(), pictura::ToolId::Brush);
    QTest::mouseClick(button1, Qt::LeftButton);
    QCoreApplication::processEvents();
    QCOMPARE(window_->activeTool(), pictura::ToolId::Move);
    QVERIFY(!toolbox->slotMenuForTest(2)->isVisible());

    // The swap hit target is the top-left corner and a click exchanges fg/bg.
    auto* fgbg = toolbox->foregroundBackgroundForTest();
    QVERIFY(fgbg != nullptr);
    const QRect swap = fgbg->swapRectForTest();
    QVERIFY(swap.right() > fgbg->width() / 2);
    QVERIFY(swap.top() < fgbg->height() / 2);
    const int swapsBefore = fgbg->swapCountForTest();
    const QColor fgBefore = fgbg->foregroundForTest();
    const QColor bgBefore = fgbg->backgroundForTest();
    QMouseEvent swapPress(QEvent::MouseButtonPress, QPointF(swap.center()),
                          QPointF(fgbg->mapToGlobal(swap.center())), Qt::LeftButton,
                          Qt::LeftButton, Qt::NoModifier);
    QApplication::sendEvent(fgbg, &swapPress);
    QCoreApplication::processEvents();
    QCOMPARE(fgbg->swapCountForTest(), swapsBefore + 1);
    QCOMPARE(fgbg->foregroundForTest(), bgBefore);
    QCOMPARE(fgbg->backgroundForTest(), fgBefore);

    // The Paint Mask stub is checkable with a mapped icon, left of screen mode.
    auto* paintMask = toolbox->findChild<QToolButton*>(QStringLiteral("paintMaskButton"));
    auto* screenMode = toolbox->findChild<QToolButton*>(QStringLiteral("screenModeButton"));
    QVERIFY(paintMask != nullptr);
    QVERIFY(screenMode != nullptr);
    QVERIFY(paintMask->isCheckable());
    QVERIFY(!paintMask->icon().isNull());
    // One-column mode stacks the Quick Mask / Screen Mode pair vertically and
    // sizes them like the tool slots.
    QVERIFY(paintMask->y() < screenMode->y());
    QCOMPARE(paintMask->width(), screenMode->width());
    // Both footer buttons match a tool slot in either column mode.
    const QSize slotSize = toolbox->slotButtons().first()->size();
    const QSize slotIcon = toolbox->slotButtons().first()->iconSize();
    toolbox->setColumns(2);
    QCoreApplication::processEvents();
    QCOMPARE(paintMask->size(), slotSize);
    QCOMPARE(screenMode->size(), slotSize);
    QCOMPARE(paintMask->iconSize(), slotIcon);
    QCOMPARE(screenMode->iconSize(), slotIcon);
    QVERIFY2(toolbox->contentWidthForTest() >= paintMask->width() + screenMode->width(),
             "two-column footer fits the content width");
    toolbox->setColumns(1);
    QCoreApplication::processEvents();
    QSignalSpy maskSpy(paintMask, &QToolButton::toggled);
    paintMask->click();
    QCOMPARE(maskSpy.count(), 1);
    QVERIFY(paintMask->isChecked());

    // The screen-mode button opens an InstantPopup of the three registry
    // commands, and choosing one switches the frame.
    QMenu* modeMenu = screenMode->menu();
    QVERIFY(modeMenu != nullptr);
    QCOMPARE(screenMode->popupMode(), QToolButton::InstantPopup);
    QCOMPARE(int(modeMenu->actions().size()), 3);
    for (QAction* action : modeMenu->actions()) {
        QVERIFY(action->isCheckable());
        // Display-only F accelerator column; no key binding is added here.
        QVERIFY2(action->text().endsWith(QStringLiteral("\tF")),
                 "screen-mode items carry the F hint");
    }
    pictura::CommandRegistry* registry = window_->registry();
    QVERIFY(registry != nullptr);
    QAction* full =
        registry->action(QString::fromLatin1(pictura::command_ids::ViewScreenModeFull));
    QVERIFY(full != nullptr);
    QVERIFY(modeMenu->actions().contains(full));
    window_->setScreenMode(pictura::PicturaMainWindow::ScreenMode::Standard);
    const QImage standardIcon = screenMode->icon().pixmap(16, 16).toImage();
    full->trigger();
    QCoreApplication::processEvents();
    QCOMPARE(window_->screenMode(), pictura::PicturaMainWindow::ScreenMode::Full);
    QVERIFY(full->isChecked());
    // The footer button mirrors the active mode's registry icon.
    QVERIFY2(screenMode->icon().pixmap(16, 16).toImage() != standardIcon,
             "screen-mode button icon follows the mode");
    window_->setScreenMode(pictura::PicturaMainWindow::ScreenMode::Standard);
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

void CommandTreeTest::panelGroupChrome()
{
    pictura::PanelColumn* column = window_->panelColumn();
    QVERIFY(column != nullptr);
    column->setRailMode(false);
    column->showPanel(QStringLiteral("layersPanel"), true);
    QCoreApplication::processEvents();
    pictura::PanelGroup* group = column->groupForPanel(QStringLiteral("layersPanel"));
    QVERIFY(group != nullptr);
    const QString title = group->titleForPanel(QStringLiteral("layersPanel"));
    QVERIFY(!title.isEmpty());

    // Normal-mode tabs are iconless; the icon is retained for the iconic strip.
    QVERIFY2(!group->tabHasIconForTest(title), "normal tab carries no icon");
    QVERIFY2(!group->titleIconForTest(title).isNull(), "panel icon retained for iconic mode");
    group->setCollapsedToIconsForTest(true);
    QCoreApplication::processEvents();
    QVERIFY2(!group->iconicIconForTest(QStringLiteral("layersPanel")).isNull(),
             "iconic strip shows the panel icon");
    group->setCollapsedToIconsForTest(false);
    QCoreApplication::processEvents();

    // The corner action button draws the bundled `panel.menu` glyph, not text,
    // and keeps a right margin.
    QToolButton* corner = group->headerMenuButtonForTest();
    QVERIFY(corner != nullptr);
    QVERIFY2(!corner->icon().isNull(), "corner glyph resolves");
    QVERIFY2(corner->text().isEmpty(), "corner shows the glyph, not a character");
    QVERIFY2(corner->icon().pixmap(16, 16).toImage()
                 == pictura::icon(QStringLiteral("panel.menu")).pixmap(16, 16).toImage(),
             "corner uses panel.menu");
    QVERIFY2(group->headerCornerRightMarginForTest() > 0, "corner keeps a right margin");

    // The header band paints the inactive-tab surface behind the corner.
    QWidget* band = group->headerBandForTest();
    QWidget* cornerWidget = group->headerCornerForTest();
    QVERIFY(band != nullptr && cornerWidget != nullptr);
    QVERIFY(band->height() > 0);
    const QImage image = group->grab().toImage();
    const qreal dpr = image.devicePixelRatio();
    const QPoint gap = band->mapTo(group, QPoint(cornerWidget->x() + 2, 2));
    const QColor gapColour =
        image.pixelColor(int(gap.x() * dpr + 0.5), int(gap.y() * dpr + 0.5));
    QCOMPARE(gapColour.rgba(), QColor(QStringLiteral("#363636")).rgba());

    // The column header sits on the group-header shade with a one-step-darker
    // bottom rule; the container keeps side rules only.
    const QString sheet = qApp->styleSheet();
    const QString baseHex = QStringLiteral("#363636");
    const QString headerRuleHex =
        pictura::Theme::shade(QColor(baseHex), -1).name(QColor::HexRgb);
    QVERIFY2(sheet.contains(QStringLiteral("QWidget#panelColumnHeader { background: ")
                            + baseHex + QStringLiteral("; border: 0; border-bottom: 1px solid ")
                            + headerRuleHex + QStringLiteral("; }")),
             "column header uses the group-header shade and a darker bottom rule");
}

void CommandTreeTest::selfAnchorDock()
{
    pictura::PanelColumn* primary = window_->panelColumn();
    QVERIFY(primary != nullptr);
    primary->setRailMode(false);
    primary->showPanel(QStringLiteral("layersPanel"), true);
    QCoreApplication::processEvents();

    // A panel in the right-hand column docks to the document-facing (left) side
    // of its own column; the source keeps its other groups.
    QVERIFY2(primary->selfAnchorDockForTest(QStringLiteral("layersPanel"), false),
             "panel docks to the left of its own column");
    // And another panel docks to the opposite (right) side of its own column.
    QVERIFY2(primary->selfAnchorDockForTest(QStringLiteral("channelsPanel"), true),
             "panel docks to the right of its own column");
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

// An iconic rail column is resizable from its workspace-facing edge whether it
// is docked left or right; the grip drives the frame's widget-column resize.
void CommandTreeTest::railModeResizeGrip()
{
    pictura::PicturaMainWindow& frame = *window_;
    pictura::PanelColumn* right = frame.panelColumn();
    QVERIFY(right != nullptr);
    pictura::PanelColumn* left = frame.createPanelColumn(pictura::PanelSide::Left, nullptr);
    QVERIFY(left != nullptr);
    QCoreApplication::processEvents();

    struct Case {
        pictura::PanelColumn* column;
        bool leftDock;
    };
    const Case cases[] = {{right, false}, {left, true}};
    for (const Case& c : cases) {
        c.column->setRailMode(true);
        QCoreApplication::processEvents();
        QVERIFY2(c.column->railModeForTest(), "rail mode on");

        QWidget* grip = c.column->findChild<QWidget*>(QStringLiteral("panelResizeGrip"));
        QVERIFY2(grip != nullptr, "workspace-edge grip exists");
        QVERIFY2(grip->isVisible(), "grip stays visible in rail mode");
        // A right-docked column's workspace edge is its left side; a left-docked
        // column's is its right side.
        QCOMPARE(grip->x(), c.leftDock ? c.column->width() - grip->width() : 0);
        QVERIFY2(c.column->maximumWidth() > c.column->minimumWidth(),
                 "rail column is width-resizable");
        QCOMPARE(c.column->minimumWidth(), pictura::kIconStripMinWidth);

        const int before = c.column->width();
        const int globalX = c.column->mapToGlobal(QPoint(0, 0)).x();
        frame.beginWidgetColumnResize(c.column, globalX);
        // Grow by 30 px: drag a right-docked column's left edge left, a
        // left-docked column's right edge right.
        frame.updateWidgetColumnResize(globalX + (c.leftDock ? 30 : -30));
        frame.endWidgetColumnResize();
        QCoreApplication::processEvents();
        QCOMPARE(c.column->width(), before + 30);
    }
}

// The Window menu carries Options; the old View > Options and View > Use GPU
// Compute entries and the Window > 3D stub are gone, while the other 3D
// entries (Workspace/Panels/Edit ▸ Preferences) stay.
void CommandTreeTest::menuMoves()
{
    pictura::CommandRegistry* registry = window_->registry();
    QVERIFY(registry != nullptr);
    bool windowOptions = false;
    bool viewOptions = false;
    bool viewGpu = false;
    bool window3d = false;
    for (const pictura::CommandInfo& info : registry->describe()) {
        if (info.id == QString::fromLatin1(pictura::command_ids::ViewOptions)) {
            windowOptions = info.path
                == QStringList({QStringLiteral("Window"), QStringLiteral("Options")});
            viewOptions = info.path
                == QStringList({QStringLiteral("View"), QStringLiteral("Options")});
        }
        if (info.id == QStringLiteral("view.gpuCompute")) {
            viewGpu = true;
        }
        if (info.path == QStringList({QStringLiteral("Window"), QStringLiteral("3D")})) {
            window3d = true;
        }
    }
    QVERIFY2(windowOptions, "Window > Options exists");
    QVERIFY2(!viewOptions, "View > Options is gone");
    QVERIFY2(!viewGpu, "View > Use GPU Compute is gone");
    QVERIFY2(!window3d, "Window > 3D is gone");
    QVERIFY2(registry->action(pictura::commandIdForPath(
                 {QStringLiteral("Window"), QStringLiteral("Workspace"), QStringLiteral("3D")})),
             "Workspace > 3D stays");
    QVERIFY2(registry->action(pictura::commandIdForPath(
                 {QStringLiteral("Window"), QStringLiteral("Panels"), QStringLiteral("3D")})),
             "Window > Panels > 3D stays");
    QVERIFY2(registry->action(pictura::commandIdForPath(
                 {QStringLiteral("Edit"), QStringLiteral("Preferences"), QStringLiteral("3D")})),
             "Edit > Preferences > 3D stays");
}

// The Preferences Performance page is real: it exposes the GPU-compute
// checkbox and the renamed rows still land on their own stack page.
void CommandTreeTest::preferencesPages()
{
    pictura::PreferencesDialog* prefs = window_->preferencesDialog();
    if (!prefs) {
        window_->registry()->dispatch(
            QString::fromLatin1(pictura::command_ids::EditPreferencesGeneral));
        QCoreApplication::processEvents();
        prefs = window_->preferencesDialog();
    }
    QVERIFY(prefs != nullptr);
    QVERIFY2(prefs->pagesForTest().contains(QStringLiteral("Performance")),
             "Performance is a real page");

    prefs->openOn(QStringLiteral("Performance"));
    QCOMPARE(prefs->currentPageForTest(), QStringLiteral("Performance"));
    prefs->openOn(QStringLiteral("General"));
    QCOMPARE(prefs->currentPageForTest(), QStringLiteral("General"));
    prefs->openOn(QStringLiteral("Interface"));
    QCOMPARE(prefs->currentPageForTest(), QStringLiteral("Interface"));
    // A disabled pane never becomes the shown page.
    prefs->openOn(QStringLiteral("File Handling"));
    QCOMPARE(prefs->currentPageForTest(), QStringLiteral("Interface"));

    QSignalSpy gpuSpy(prefs, &pictura::PreferencesDialog::gpuComputeChanged);
    prefs->setGpuCompute(true);
    QVERIFY(prefs->checkboxForTest(QStringLiteral("useGpuCompute")));
    prefs->setCheckboxForTest(QStringLiteral("useGpuCompute"), false);
    QCOMPARE(gpuSpy.count(), 1);
    QCOMPARE(gpuSpy.at(0).at(0).toBool(), false);
    prefs->setGpuComputeEnabled(false);
    prefs->close();
}

// View > Tool Hints is a checkable toggle, on by default; it hides the footer
// hint strip (once a document owns the footer) and restores it.
void CommandTreeTest::toolHintsToggle()
{
    QAction* action = window_->registry()->action(
        QString::fromLatin1(pictura::command_ids::ViewToolHints));
    QVERIFY2(action != nullptr, "Tool Hints command exists");
    QVERIFY2(action->isCheckable(), "Tool Hints is checkable");
    QVERIFY2(action->isChecked(), "Tool Hints defaults on");

    auto* hintBar = window_->findChild<pictura::ToolHintBar*>(QStringLiteral("toolHintBar"));
    QVERIFY2(hintBar != nullptr, "hint bar exists");

    QVERIFY(window_->newDocument(QStringLiteral("Hints"), 4, 3, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    QCoreApplication::processEvents();
    QVERIFY2(hintBar->isVisible(), "hints shown with a document");

    action->trigger();
    QCoreApplication::processEvents();
    QVERIFY2(!action->isChecked(), "toggled off");
    QVERIFY2(!hintBar->isVisible(), "hints hidden");

    action->trigger();
    QCoreApplication::processEvents();
    QVERIFY2(action->isChecked() && hintBar->isVisible(), "hints restored");

    while (window_->activeDocumentIndex() >= 0) {
        window_->closeDocument(window_->activeDocumentIndex(), false);
    }
}

QTEST_MAIN(CommandTreeTest)
#include "tst_command_tree.moc"