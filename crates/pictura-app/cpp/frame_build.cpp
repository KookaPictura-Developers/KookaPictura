#include "frame_includes.h"

namespace pictura {

void PicturaMainWindow::buildPanels()
{
    colorState_ = new ColorState(this);

    layersPanel_ = new LayersPanel(this);
    layersPanel_->setObjectName(QStringLiteral("layersPanel"));

    historyPanel_ = new HistoryPanel(this);
    historyPanel_->setObjectName(QStringLiteral("historyPanel"));

    navigatorPanel_ = new NavigatorPanel(this);
    navigatorPanel_->setObjectName(QStringLiteral("navigatorPanel"));

    colorPanel_ = new ColorPanel(colorState_, this);
    colorPanel_->setObjectName(QStringLiteral("colorPanel"));

    swatchesPanel_ = new SwatchesPanel(colorState_, this);
    swatchesPanel_->setObjectName(QStringLiteral("swatchesPanel"));

    infoPanel_ = new InfoPanel(this);
    infoPanel_->setObjectName(QStringLiteral("infoPanel"));

    histogramPanel_ = new HistogramPanel(this);
    histogramPanel_->setObjectName(QStringLiteral("histogramPanel"));

    gradientsPanel_ = new PlaceholderPanel(QStringLiteral("Gradients"), QString(), this);
    gradientsPanel_->setObjectName(QStringLiteral("gradientsPanel"));

    patternsPanel_ = new PlaceholderPanel(QStringLiteral("Patterns"), QString(), this);
    patternsPanel_->setObjectName(QStringLiteral("patternsPanel"));

    propertiesPanel_ =
        new PlaceholderPanel(QStringLiteral("Properties"), QStringLiteral("No Properties"), this);
    propertiesPanel_->setObjectName(QStringLiteral("propertiesPanel"));

    adjustmentsPanel_ = new PlaceholderPanel(QStringLiteral("Adjustments"), QString(), this);
    adjustmentsPanel_->setObjectName(QStringLiteral("adjustmentsPanel"));

    librariesPanel_ = new PlaceholderPanel(QStringLiteral("Libraries"), QString(), this);
    librariesPanel_->setObjectName(QStringLiteral("librariesPanel"));

    channelsPanel_ = new PlaceholderPanel(QStringLiteral("Channels"), QString(), this);
    channelsPanel_->setObjectName(QStringLiteral("channelsPanel"));

    pathsPanel_ = new PlaceholderPanel(QStringLiteral("Paths"), QString(), this);
    pathsPanel_->setObjectName(QStringLiteral("pathsPanel"));

    actionsPanel_ = new PlaceholderPanel(QStringLiteral("Actions"), QString(), this);
    actionsPanel_->setObjectName(QStringLiteral("actionsPanel"));

    // `Styles` takes the former Gradients/Patterns tab slot in the default set.
    stylesPanel_ = new PlaceholderPanel(QStringLiteral("Styles"), QString(), this);
    stylesPanel_->setObjectName(QStringLiteral("stylesPanel"));

    // Content widgets, not docks: the PanelColumn hosts them. Registering the
    // objectNames keeps the Window menu and the duplicate-name guard working.
    registerPanel(layersPanel_, Qt::RightDockWidgetArea);
    registerPanel(historyPanel_, Qt::RightDockWidgetArea);
    registerPanel(navigatorPanel_, Qt::RightDockWidgetArea);
    registerPanel(colorPanel_, Qt::RightDockWidgetArea);
    registerPanel(swatchesPanel_, Qt::RightDockWidgetArea);
    registerPanel(stylesPanel_, Qt::RightDockWidgetArea);
    registerPanel(infoPanel_, Qt::RightDockWidgetArea);
    registerPanel(histogramPanel_, Qt::RightDockWidgetArea);
    registerPanel(gradientsPanel_, Qt::RightDockWidgetArea);
    registerPanel(patternsPanel_, Qt::RightDockWidgetArea);
    registerPanel(propertiesPanel_, Qt::RightDockWidgetArea);
    registerPanel(adjustmentsPanel_, Qt::RightDockWidgetArea);
    registerPanel(librariesPanel_, Qt::RightDockWidgetArea);
    registerPanel(channelsPanel_, Qt::RightDockWidgetArea);
    registerPanel(pathsPanel_, Qt::RightDockWidgetArea);
    registerPanel(actionsPanel_, Qt::RightDockWidgetArea);

    // CS6 Essentials groups and order. The tab text is the panel title and the
    // tab icon reuses the panel's `window.panels.<name>` asset.
    auto addPanel = [](PanelGroup* group, QWidget* panel, const QString& title,
                       const QString& iconId) {
        group->addPanel(panel, title, pictura::icon(iconId));
    };

    auto* colorGroup = new PanelGroup(this);
    addPanel(colorGroup, colorPanel_, tr("Color"), QString::fromLatin1(command_ids::WindowPanelsColor));
    addPanel(colorGroup, swatchesPanel_, tr("Swatches"),
             QString::fromLatin1(command_ids::WindowPanelsSwatches));
    addPanel(colorGroup, stylesPanel_, tr("Styles"), QStringLiteral("window.panels.styles"));
    panelColumn_->addGroup(colorGroup);

    // Properties folds into the Adjustments group; its tab starts hidden.
    auto* adjustmentsGroup = new PanelGroup(this);
    addPanel(adjustmentsGroup, adjustmentsPanel_, tr("Adjustments"),
             QString::fromLatin1(command_ids::WindowPanelsAdjustments));
    addPanel(adjustmentsGroup, propertiesPanel_, tr("Properties"),
             QString::fromLatin1(command_ids::WindowPanelsProperties));
    panelColumn_->addGroup(adjustmentsGroup);

    auto* layersGroup = new PanelGroup(this);
    addPanel(layersGroup, layersPanel_, tr("Layers"),
             QString::fromLatin1(command_ids::WindowPanelsLayers));
    addPanel(layersGroup, channelsPanel_, tr("Channels"),
             QString::fromLatin1(command_ids::WindowPanelsChannels));
    addPanel(layersGroup, pathsPanel_, tr("Paths"),
             QString::fromLatin1(command_ids::WindowPanelsPaths));
    panelColumn_->addGroup(layersGroup);

    auto* navigatorGroup = new PanelGroup(this);
    addPanel(navigatorGroup, navigatorPanel_, tr("Navigator"),
             QString::fromLatin1(command_ids::WindowPanelsNavigator));
    addPanel(navigatorGroup, histogramPanel_, tr("Histogram"),
             QString::fromLatin1(command_ids::WindowPanelsHistogram));
    addPanel(navigatorGroup, infoPanel_, tr("Info"),
             QString::fromLatin1(command_ids::WindowPanelsInfo));
    panelColumn_->addGroup(navigatorGroup);

    // Iconic single-panel groups.
    auto* historyGroup = new PanelGroup(this);
    addPanel(historyGroup, historyPanel_, tr("History"),
             QString::fromLatin1(command_ids::WindowPanelsHistory));
    panelColumn_->addGroup(historyGroup);

    auto* actionsGroup = new PanelGroup(this);
    addPanel(actionsGroup, actionsPanel_, tr("Actions"),
             QString::fromLatin1(command_ids::WindowPanelsActions));
    panelColumn_->addGroup(actionsGroup);

    // Kept registered and Window-menu reachable, but out of the default groups.
    auto* overflowGroup = new PanelGroup(this);
    addPanel(overflowGroup, gradientsPanel_, tr("Gradients"),
             QString::fromLatin1(command_ids::WindowPanelsGradients));
    addPanel(overflowGroup, patternsPanel_, tr("Patterns"),
             QString::fromLatin1(command_ids::WindowPanelsPatterns));
    addPanel(overflowGroup, librariesPanel_, tr("Libraries"),
             QString::fromLatin1(command_ids::WindowPanelsLibraries));
    panelColumn_->addGroup(overflowGroup);

    // Default visibility matches the pre-M41 layout: Color/Swatches/Styles,
    // Adjustments, Layers/Channels/Paths and Navigator/Histogram/Info visible;
    // History, Actions and the overflow panels hidden.
    panelColumn_->showPanel(QStringLiteral("historyPanel"), false);
    panelColumn_->showPanel(QStringLiteral("actionsPanel"), false);
    panelColumn_->showPanel(QStringLiteral("propertiesPanel"), false);
    panelColumn_->showPanel(QStringLiteral("gradientsPanel"), false);
    panelColumn_->showPanel(QStringLiteral("patternsPanel"), false);
    panelColumn_->showPanel(QStringLiteral("librariesPanel"), false);
}

void PicturaMainWindow::buildTools(int toolsColumns, bool useShiftKeyForToolSwitch)
{
    tools_ = new ToolController(this);
    tools_->setViewProvider([this]() { return activeView(); });

    if (colorState_) {
        tools_->setForeground(colorState_->foreground());
        tools_->setBackground(colorState_->background());
        connect(colorState_, &ColorState::foregroundChanged, tools_,
                &ToolController::setForeground);
        connect(colorState_, &ColorState::backgroundChanged, tools_,
                &ToolController::setBackground);
    }

    auto* toolbox = new Toolbox(tools_, colorState_, this);
    toolbox_ = toolbox;
    toolbox->setShiftKeyForToolSwitch(useShiftKeyForToolSwitch);
    toolbox->setColumns(toolsColumns);

    // One plain and one Shift shortcut per distinct slot letter; the toolbox
    // resolves the letter to its group and honours `Use Shift Key For Tool
    // Switch`. refreshSlot() no longer owns any letter.
    for (const QChar key : toolShortcutKeys()) {
        connect(new QShortcut(QKeySequence(QString(key)), this), &QShortcut::activated, this,
                [this, key]() {
                    if (toolbox_) {
                        toolbox_->handleToolKey(key, false);
                    }
                });
        connect(new QShortcut(QKeySequence(Qt::SHIFT | Qt::Key(key.toUpper().unicode())), this),
                &QShortcut::activated, this, [this, key]() {
                    if (toolbox_) {
                        toolbox_->handleToolKey(key, true);
                    }
                });
    }

    auto brushActive = [this]() {
        return tools_ && (tools_->activeTool() == ToolId::Brush
                          || tools_->activeTool() == ToolId::Pencil);
    };
    connect(new QShortcut(QKeySequence(Qt::Key_BracketLeft), this), &QShortcut::activated, this,
            [this, brushActive]() {
                if (brushActive()) {
                    tools_->adjustBrushSize(-1);
                }
            });
    connect(new QShortcut(QKeySequence(Qt::Key_BracketRight), this), &QShortcut::activated, this,
            [this, brushActive]() {
                if (brushActive()) {
                    tools_->adjustBrushSize(+1);
                }
            });
    connect(new QShortcut(QKeySequence(Qt::SHIFT | Qt::Key_BracketLeft), this),
            &QShortcut::activated, this, [this, brushActive]() {
                if (brushActive()) {
                    tools_->adjustBrushHardness(-5);
                }
            });
    connect(new QShortcut(QKeySequence(Qt::SHIFT | Qt::Key_BracketRight), this),
            &QShortcut::activated, this, [this, brushActive]() {
                if (brushActive()) {
                    tools_->adjustBrushHardness(+5);
                }
            });

    // CS6 foreground/background swap. `X` is unassigned in the tool letter
    // catalogue, so it does not collide with the tool-switch shortcuts.
    connect(new QShortcut(QKeySequence(Qt::Key_X), this), &QShortcut::activated, this,
            [this]() {
                if (toolbox_) {
                    toolbox_->swapForegroundBackground();
                }
            });

    // M43: `D` resets fg/bg to the CS6 defaults (black/white). `D` is also
    // unassigned in the tool letter catalogue (see tools.cpp `kToolTable`).
    connect(new QShortcut(QKeySequence(Qt::Key_D), this), &QShortcut::activated, this,
            [this]() {
                if (toolbox_) {
                    toolbox_->resetForegroundBackground();
                }
            });

    toolsDock_ = toolbox;
    registerPanel(toolbox, Qt::LeftDockWidgetArea);
    connect(toolbox, &Toolbox::screenModeRequested, this,
            [this]() { cycleScreenMode(true); });
    connect(toolbox, &Toolbox::columnsChanged, this, [this](int) { saveSession(); });
    connect(toolbox, &QDockWidget::dockLocationChanged, this, [this](Qt::DockWidgetArea area) {
        // M46: only left/right persist; a stray top/bottom area must not be
        // re-applied by `ensureToolsNotTabified`.
        if (area == Qt::LeftDockWidgetArea || area == Qt::RightDockWidgetArea) {
            toolsArea_ = area;
        }
        // M47: a real dock is no longer a central-splitter pane.
        toolbox_->setSplitterPane(false);
        ensureToolsNotTabified();
    });
    connect(toolbox, &QDockWidget::topLevelChanged, this, [this](bool) {
        toolbox_->setSplitterPane(false);
        ensureToolsNotTabified();
    });

    // M45 T3: while the floating Tools panel is dragged, resolve the drop through
    // the column grammar and show the single indicator; on release host it at
    // that boundary. The commit is deferred past Qt's own dock-drag handling.
    connect(toolbox, &Toolbox::toolbarDragMoved, this, [this](const QPoint& pos) {
        // M47 D8: show the indicator for a docked/pane title-bar drag too.
        if (toolbox_) {
            resolveToolboxDrop(pos, nullptr, nullptr);
        }
    });
    connect(toolbox, &Toolbox::toolbarDragFinished, this, [this](const QPoint& pos) {
        // M47 D8: commit a docked or pane-hosted drag-out too; `commitToolboxDrop`
        // handles the reparenting, so the float state is irrelevant here.
        if (!toolbox_) {
            return;
        }
        QTimer::singleShot(0, this, [this, pos]() {
            if (toolbox_) {
                commitToolboxDrop(pos);
            }
        });
    });

    optionsBar_ = new OptionsBar(tools_, this);
    addToolBar(optionsBar_);

    connect(tools_, &ToolController::activeToolChanged, this, [this](ToolId id) {
        if (optionsBar_) {
            optionsBar_->showTool(id);
        }
        updateToolHint();
    });
    connect(tools_, &ToolController::foregroundSampled, this, [this](const QColor& color) {
        foreground_ = color;
        if (colorState_) {
            colorState_->setForeground(color);
        }
        updateToolHint();
    });

    if (optionsBar_) {
        optionsBar_->showTool(tools_->activeTool());
    }
}

void PicturaMainWindow::ensureToolsNotTabified()
{
    if (!toolsDock_ || toolsDock_->isFloating()) {
        return;
    }
    if (tabifiedDockWidgets(toolsDock_).isEmpty()) {
        return;
    }
    toolsDock_->setFloating(true);
    addDockWidget(toolsArea_, toolsDock_);
    toolsDock_->show();
}

void PicturaMainWindow::buildStatusBar()
{
    QStatusBar* bar = statusBar();
    zoomLabel_ = new QLabel(QStringLiteral("100%"), bar);
    sizeLabel_ = new QLabel(QStringLiteral("—"), bar);
    backendLabel_ = new QLabel(QStringLiteral("—"), bar);
    hintLabel_ = new QLabel(QStringLiteral("Ready"), bar);
    bar->addWidget(zoomLabel_);
    bar->addWidget(sizeLabel_);
    bar->addWidget(backendLabel_);
    bar->addWidget(hintLabel_);

    auto* optionsButton = new QToolButton(bar);
    optionsButton->setArrowType(Qt::DownArrow);
    optionsButton->setPopupMode(QToolButton::InstantPopup);
    auto* optionsMenu = new QMenu(optionsButton);
    auto* group = new QActionGroup(optionsMenu);
    for (const QString& key : {QStringLiteral("sizes"), QStringLiteral("dimensions")}) {
        QAction* option = optionsMenu->addAction(
            key == QStringLiteral("sizes") ? QStringLiteral("Document Sizes")
                                           : QStringLiteral("Document Dimensions"));
        option->setCheckable(true);
        option->setData(key);
        option->setChecked(key == QStringLiteral("sizes"));
        group->addAction(option);
        connect(option, &QAction::triggered, this, [this, key]() {
            statusReadout_ = key;
            updateStatus();
        });
    }
    optionsButton->setMenu(optionsMenu);
    bar->addPermanentWidget(optionsButton);
}


} // namespace pictura
