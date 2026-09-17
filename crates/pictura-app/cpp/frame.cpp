#include "frame.h"

#include "commands.h"
#include "dialogs.h"
#include "icons.h"
#include "image_view.h"
#include "new_document_dialog.h"
#include "options_bar.h"
#include "panels/color_panel.h"
#include "panels/histogram_panel.h"
#include "panels/history_panel.h"
#include "panels/info_panel.h"
#include "panels/layers_panel.h"
#include "panels/navigator_panel.h"
#include "panels/panel_column.h"
#include "panels/panel_group.h"
#include "panels/placeholder_panel.h"
#include "panels/swatches_panel.h"
#include "preferences_dialog.h"
#include "session.h"
#include "theme.h"
#include "toolbox.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QFileInfo>
#include <QtCore/QCoreApplication>
#include <QtCore/QJsonArray>
#include <QtCore/QJsonObject>
#include <QtCore/QRect>
#include <QtCore/QSignalBlocker>
#include <QtCore/QTimer>
#include <QtGui/QAction>
#include <QtGui/QActionGroup>
#include <QtGui/QCloseEvent>
#include <QtGui/QColor>
#include <QtGui/QKeyEvent>
#include <QtGui/QKeySequence>
#include <QtGui/QShortcut>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDockWidget>
#include <QtWidgets/QFileDialog>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QListWidget>
#include <QtWidgets/QMenu>
#include <QtWidgets/QMenuBar>
#include <QtWidgets/QMessageBox>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSpinBox>
#include <QtWidgets/QSplitter>
#include <QtWidgets/QStatusBar>
#include <QtWidgets/QTabWidget>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QWidget>

namespace pictura {

namespace {

constexpr int kCanvasColorCount = 4;
const QColor kCanvasColors[kCanvasColorCount] = {
    QColor(37, 37, 37), QColor(82, 82, 82), QColor(0, 0, 0), QColor(255, 255, 255)};

constexpr int kRecentLimit = 20;

} // namespace

PicturaMainWindow::PicturaMainWindow(QWidget* parent)
    : QMainWindow(parent)
{
    tabs_ = new QTabWidget(this);
    tabs_->setObjectName(QStringLiteral("documentTabs"));
    tabs_->tabBar()->setObjectName(QStringLiteral("documentTabBar"));
    tabs_->setTabsClosable(true);
    tabs_->setMovable(true);
    tabs_->setDocumentMode(true);

    // The document area and the panel columns share the central widget through
    // a splitter; the columns are the only host for the panels. M43: the
    // splitter is an ordered set of left columns, the document tabs, then right
    // columns, with the tabs keeping the stretch.
    panelColumn_ = new PanelColumn(this);
    centerSplitter_ = new QSplitter(Qt::Horizontal, this);
    centerSplitter_->setObjectName(QStringLiteral("centerSplitter"));
    centerSplitter_->addWidget(tabs_);
    centerSplitter_->addWidget(panelColumn_);
    centerSplitter_->setStretchFactor(0, 1);
    centerSplitter_->setStretchFactor(1, 0);
    setCentralWidget(centerSplitter_);
    setDockOptions(QMainWindow::AnimatedDocks | QMainWindow::AllowTabbedDocks);

    registry_ = new CommandRegistry(this);
    addDefaultCommands(*registry_);

    const SessionState state = pictura::loadSession();
    recent_ = state.recent;
    gpuCompute_ = state.gpuCompute;
    useShiftKeyForToolSwitch_ = state.useShiftKeyForToolSwitch;
    // Probe the adapter once so the toggle can be offered without a document.
    {
        PictureView probe;
        gpuAvailable_ = probe.gpu_available();
    }
    rebuildRecentMenu();

    registerHandlers();
    buildMenus();
    buildPanels();
    buildTools(state.toolsColumns, state.useShiftKeyForToolSwitch);
    buildStatusBar();
    applyBrightness(state.brightnessLevel);
    applyPanelSession(state);
    // Persist every column change through the same path as the Window toggles,
    // and route the tab menu's `Interface Options…` to the Interface pane.
    wirePanelColumn(panelColumn_);
    if (!state.layout.isEmpty()) {
        restoreStoredLayout(state.layout, state.layoutRevision);
    }

    // Expensive panel work (layer thumbnails, histogram) is coalesced off the
    // paint-dab/commit path so the canvas can repaint first.
    panelRefreshTimer_ = new QTimer(this);
    panelRefreshTimer_->setSingleShot(true);
    panelRefreshTimer_->setInterval(120);
    connect(panelRefreshTimer_, &QTimer::timeout, this, &PicturaMainWindow::refreshPanels);

    connect(tabs_, &QTabWidget::currentChanged, this, [this](int) {
        refresh();
        panelRefreshTimer_->stop();
        refreshPanels();
    });
    connect(tabs_, &QTabWidget::tabCloseRequested, this,
            [this](int index) { closeDocument(index, true); });

    auto* hidePanels = new QShortcut(QKeySequence(Qt::Key_Tab), this);
    connect(hidePanels, &QShortcut::activated, this, [this]() { setPanelsHidden(!panelsHidden_); });
    auto* hidePanelsBack = new QShortcut(QKeySequence(Qt::SHIFT | Qt::Key_Tab), this);
    connect(hidePanelsBack, &QShortcut::activated, this,
            [this]() { setPanelsHidden(!panelsHidden_); });
    auto* cycleCanvas = new QShortcut(QKeySequence(Qt::Key_Space, Qt::Key_F), this);
    connect(cycleCanvas, &QShortcut::activated, this, [this]() { cycleCanvasColor(true); });
    auto* brightnessDown = new QShortcut(QKeySequence(Qt::SHIFT | Qt::Key_F1), this);
    connect(brightnessDown, &QShortcut::activated, this,
            [this]() { setBrightnessLevel(brightnessLevel_ - 1); });
    auto* brightnessUp = new QShortcut(QKeySequence(Qt::SHIFT | Qt::Key_F2), this);
    connect(brightnessUp, &QShortcut::activated, this,
            [this]() { setBrightnessLevel(brightnessLevel_ + 1); });

    resize(1100, 700);
    setWindowTitle(QStringLiteral("Kooka Pictura"));
}

PicturaMainWindow::~PicturaMainWindow() = default;

QStringList PicturaMainWindow::topLevelMenuTitles() const
{
    return registry_->topLevelTitles();
}

ToolId PicturaMainWindow::activeTool() const
{
    return tools_ ? tools_->activeTool() : ToolId::Move;
}

void PicturaMainWindow::setActiveTool(ToolId id)
{
    if (tools_) {
        tools_->setActiveTool(id);
    }
}

bool PicturaMainWindow::hasPendingCrop() const
{
    return tools_ && tools_->hasPendingCrop();
}

bool PicturaMainWindow::commitCrop()
{
    if (!tools_ || !tools_->commitCrop()) {
        return false;
    }
    refresh();
    return true;
}

bool PicturaMainWindow::registerPanel(QWidget* panel, Qt::DockWidgetArea area)
{
    if (!panel || panelNames_.contains(panel->objectName())) {
        return false;
    }
    panelNames_.insert(panel->objectName());
    if (auto* dock = qobject_cast<QDockWidget*>(panel)) {
        addDockWidget(area, dock);
    }
    return true;
}

ImageView* PicturaMainWindow::imageView() const
{
    return canvasAt(activeDocumentIndex());
}

PictureView* PicturaMainWindow::activeView() const
{
    return viewAt(activeDocumentIndex());
}

int PicturaMainWindow::activeDocumentIndex() const
{
    if (!tabs_ || tabs_->count() == 0 || docs_.isEmpty()) {
        return -1;
    }
    return tabs_->currentIndex();
}

void PicturaMainWindow::setActiveDocumentIndex(int index)
{
    if (tabs_ && index >= 0 && index < tabs_->count()) {
        tabs_->setCurrentIndex(index);
    }
}

PictureView* PicturaMainWindow::viewAt(int index) const
{
    if (index < 0 || index >= docs_.size()) {
        return nullptr;
    }
    return docs_.at(index).view;
}

ImageView* PicturaMainWindow::canvasAt(int index) const
{
    if (index < 0 || index >= docs_.size()) {
        return nullptr;
    }
    return docs_.at(index).canvas;
}

QString PicturaMainWindow::documentPath(int index) const
{
    if (index < 0 || index >= docs_.size()) {
        return QString();
    }
    return docs_.at(index).path;
}

QString PicturaMainWindow::documentName(int index) const
{
    if (index < 0 || index >= docs_.size()) {
        return QStringLiteral("Untitled");
    }
    const DocEntry& entry = docs_.at(index);
    if (!entry.path.isEmpty()) {
        return QFileInfo(entry.path).fileName();
    }
    return QStringLiteral("Untitled-%1").arg(entry.untitledNumber);
}

bool PicturaMainWindow::isDocumentDirty(int index) const
{
    PictureView* view = viewAt(index);
    return view && view->is_dirty();
}

QString PicturaMainWindow::activeFilePath() const
{
    return documentPath(activeDocumentIndex());
}

QString PicturaMainWindow::activeDocumentName() const
{
    return documentName(activeDocumentIndex());
}

bool PicturaMainWindow::isActiveDirty() const
{
    return isDocumentDirty(activeDocumentIndex());
}

int PicturaMainWindow::addDocument(PictureView* view, const QString& path)
{
    if (!view) {
        return -1;
    }
    // Apply the persisted preference to every newly created/opened tab.
    if (view->gpu_compute() != gpuCompute_) {
        view->set_gpu_compute(gpuCompute_);
    }
    gpuAvailable_ = view->gpu_available();
    DocEntry entry;
    entry.view = view;
    if (path.isEmpty()) {
        entry.untitledNumber = ++untitledCounter_;
    } else {
        entry.path = path;
    }
    entry.canvas = new ImageView(this);
    if (view->has_document()) {
        entry.canvas->setImage(view->image());
    } else {
        entry.canvas->replaceImage(view->image());
    }
    entry.canvas->setCanvasColor(kCanvasColors[canvasColorIndex_]);

    connect(view, &PictureView::changed, this, &PicturaMainWindow::refresh);
    connect(view, &PictureView::regionBlitted, this,
            [this, canvas = entry.canvas](const QImage& region, int x, int y) {
                if (canvas) {
                    canvas->blitRegion(region, x, y);
                }
                // The cheapest sync only: restart the same 120 ms panel timer the
                // `changed` path uses, and never call image()/replaceImage(), so
                // the region path cannot refresh panels faster than a full
                // recomposite.
                panelRefreshTimer_->start();
                updateTabTitle(activeDocumentIndex());
                updateWindowTitle();
                if (registry_) {
                    registry_->refresh();
                }
            });
    connect(entry.canvas, &ImageView::zoomChanged, this, [this](double) { updateStatus(); });
    connect(entry.canvas, &ImageView::mouseMoved, this, [this](const QPointF& p) {
        if (infoPanel_) {
            infoPanel_->setCursorPosition(p);
        }
    });

    docs_.append(entry);
    const int index = docs_.size() - 1;
    tabs_->addTab(entry.canvas, documentName(index));
    tabs_->setCurrentIndex(index);
    if (!path.isEmpty()) {
        rememberRecent(path);
    }
    refresh();
    panelRefreshTimer_->stop();
    refreshPanels();
    return index;
}

bool PicturaMainWindow::newDocument(const QString& name, int width, int height,
                                    const QString& mode, int depth, const QString& background)
{
    // `name` is accepted for the frozen API; DocEntry carries no name field, so
    // the tab always shows the generated Untitled-<n>.
    Q_UNUSED(name);
    auto* view = new PictureView(this);
    if (!view->new_document(width, height, mode, depth, background)) {
        delete view;
        return false;
    }
    addDocument(view, QString());
    return true;
}

bool PicturaMainWindow::openPath(const QString& path)
{
    auto* view = new PictureView(this);
    if (!view->open(path)) {
        delete view;
        return false;
    }
    addDocument(view, path);
    return true;
}

bool PicturaMainWindow::saveActive()
{
    const int index = activeDocumentIndex();
    if (index < 0) {
        return false;
    }
    QString path = docs_.at(index).path;
    if (path.isEmpty()) {
        path = QFileDialog::getSaveFileName(this, tr("Save As"), QString(),
                                            QStringLiteral("Photoshop files (*.psd *.psb)"));
        if (path.isEmpty()) {
            return false;
        }
        if (QFileInfo(path).suffix().isEmpty()) {
            path += QStringLiteral(".psd");
        }
    }
    return saveActiveAs(path);
}

bool PicturaMainWindow::saveActiveAs(const QString& path)
{
    const int index = activeDocumentIndex();
    PictureView* view = viewAt(index);
    if (!view || path.isEmpty() || !view->save(path)) {
        return false;
    }
    docs_[index].path = path;
    updateTabTitle(index);
    rememberRecent(path);
    updateWindowTitle();
    return true;
}

bool PicturaMainWindow::revertActive()
{
    const int index = activeDocumentIndex();
    PictureView* view = viewAt(index);
    const QString path = documentPath(index);
    if (!view || path.isEmpty() || !view->open(path)) {
        return false;
    }
    updateTabTitle(index);
    refresh();
    return true;
}

bool PicturaMainWindow::closeDocument(int index, bool interactive)
{
    if (index < 0 || index >= docs_.size()) {
        return false;
    }
    if (interactive && isDocumentDirty(index)) {
        switch (askUnsaved(this, documentName(index))) {
        case UnsavedChoice::Cancel:
            return false;
        case UnsavedChoice::Save:
            setActiveDocumentIndex(index);
            if (!saveActive()) {
                return false;
            }
            break;
        case UnsavedChoice::Discard:
            break;
        }
    }
    removeDocument(index);
    return true;
}

bool PicturaMainWindow::closeActiveDocument(bool interactive)
{
    return closeDocument(activeDocumentIndex(), interactive);
}

void PicturaMainWindow::removeDocument(int index)
{
    if (index < 0 || index >= docs_.size()) {
        return;
    }
    const DocEntry entry = docs_.takeAt(index);
    tabs_->removeTab(index);
    delete entry.canvas;
    delete entry.view;
    refresh();
    panelRefreshTimer_->stop();
    refreshPanels();
}

void PicturaMainWindow::showNewDocumentDialog()
{
    NewDocumentSpec spec;
    if (!NewDocumentDialog::get(this, &spec)) {
        return;
    }
    newDocument(spec.name, spec.width, spec.height, spec.mode, spec.depth, spec.background);
}

void PicturaMainWindow::showOpenDialog()
{
    const QString path = QFileDialog::getOpenFileName(
        this, tr("Open"), QString(), QStringLiteral("Photoshop files (*.psd *.psb)"));
    if (!path.isEmpty()) {
        openPath(path);
    }
}

void PicturaMainWindow::rememberRecent(const QString& path)
{
    if (path.isEmpty()) {
        return;
    }
    recent_.removeAll(path);
    recent_.prepend(path);
    while (recent_.size() > kRecentLimit) {
        recent_.removeLast();
    }
    saveSession();
}

void PicturaMainWindow::rebuildRecentMenu()
{
    QStringList valid;
    for (const QString& path : recent_) {
        if (QFileInfo::exists(path)) {
            valid.append(path);
        }
    }

    if (valid.isEmpty()) {
        registry_->add(QStringLiteral("file.openRecent.none"),
                       {QStringLiteral("File"), QStringLiteral("Open Recent")},
                       QStringLiteral("No Recent Files"), QKeySequence(), false);
        return;
    }

    for (int i = 0; i < valid.size(); ++i) {
        const QString path = valid.at(i);
        const QString label = QFileInfo(path).fileName();
        const QString id = QStringLiteral("file.openRecent.%1").arg(i);
        registry_->add(id,
                       {QStringLiteral("File"), QStringLiteral("Open Recent"), label},
                       label,
                       QKeySequence(),
                       true);
        registry_->setHandler(id, [this, path]() { openPath(path); });
    }
}

void PicturaMainWindow::setBrightnessLevel(int level)
{
    applyBrightness(level);
}

void PicturaMainWindow::setScreenMode(ScreenMode mode)
{
    screenMode_ = mode;
    auto setCanvasColor = [this](const QColor& color) {
        for (const DocEntry& entry : docs_) {
            if (entry.canvas) {
                entry.canvas->setCanvasColor(color);
            }
        }
    };
    auto setChromeVisible = [this](bool visible) {
        if (toolsDock_) {
            toolsDock_->setVisible(visible);
        }
        for (PanelColumn* column : panelColumns()) {
            column->setVisible(visible);
        }
    };
    switch (mode) {
    case ScreenMode::Standard: {
        Qt::WindowStates state = windowState();
        state.setFlag(Qt::WindowFullScreen, false);
        setWindowState(state);
        menuBar()->setVisible(true);
        statusBar()->setVisible(true);
        setChromeVisible(!panelsHidden_);
        setCanvasColor(kCanvasColors[canvasColorIndex_]);
        break;
    }
    case ScreenMode::FullWithMenuBar:
        setWindowState(Qt::WindowFullScreen);
        menuBar()->setVisible(true);
        statusBar()->setVisible(false);
        setChromeVisible(false);
        setCanvasColor(QColor(128, 128, 128));
        break;
    case ScreenMode::Full:
        setWindowState(Qt::WindowFullScreen);
        menuBar()->setVisible(false);
        statusBar()->setVisible(false);
        setChromeVisible(false);
        setCanvasColor(Qt::black);
        break;
    }
    registry_->refresh();
}

void PicturaMainWindow::cycleScreenMode(bool forward)
{
    int index = static_cast<int>(screenMode_);
    index += forward ? 1 : -1;
    index = (index % 3 + 3) % 3;
    setScreenMode(static_cast<ScreenMode>(index));
}

void PicturaMainWindow::cycleCanvasColor(bool forward)
{
    canvasColorIndex_ = (canvasColorIndex_ + (forward ? 1 : -1) + kCanvasColorCount)
                        % kCanvasColorCount;
    for (const DocEntry& entry : docs_) {
        if (entry.canvas) {
            entry.canvas->setCanvasColor(kCanvasColors[canvasColorIndex_]);
        }
    }
}

void PicturaMainWindow::setPanelsHidden(bool hidden)
{
    panelsHidden_ = hidden;
    if (toolsDock_) {
        toolsDock_->setVisible(!hidden);
    }
    for (PanelColumn* column : panelColumns()) {
        column->setVisible(!hidden);
    }
}

void PicturaMainWindow::retargetDock()
{
    PictureView* view = activeView();
    ImageView* canvas = canvasAt(activeDocumentIndex());

    if (layersPanel_) {
        layersPanel_->setView(view);
        layersPanel_->refresh();
    }
    if (historyPanel_) {
        historyPanel_->setView(view);
        historyPanel_->refresh();
    }
    if (navigatorPanel_) {
        navigatorPanel_->setCanvas(canvas);
        navigatorPanel_->refresh();
    }
    if (infoPanel_) {
        infoPanel_->setView(view);
        infoPanel_->refresh();
    }
    if (histogramPanel_) {
        histogramPanel_->setView(view);
        histogramPanel_->refresh();
    }
}

void PicturaMainWindow::refreshPanels()
{
    retargetDock();
}

void PicturaMainWindow::refresh()
{
    const int index = activeDocumentIndex();
    PictureView* view = activeView();
    ImageView* canvas = canvasAt(index);

    if (view && canvas) {
        if (view->has_document()) {
            canvas->replaceImage(view->image());
        } else {
            canvas->setImage(view->image());
        }
    }
    if (tools_) {
        tools_->bindCanvas(canvas);
    }

    panelRefreshTimer_->start();
    updateStatus();
    if (registry_) {
        registry_->refresh();
    }
    updateTabTitle(index);
    updateWindowTitle();
}

void PicturaMainWindow::updateTabTitle(int index)
{
    if (!tabs_ || index < 0 || index >= docs_.size()) {
        return;
    }
    QString title = documentName(index);
    if (isDocumentDirty(index)) {
        title += QStringLiteral(" *");
    }
    tabs_->setTabText(index, title);
    tabs_->setTabToolTip(index, docs_.at(index).path);
}

void PicturaMainWindow::updateWindowTitle()
{
    const int index = activeDocumentIndex();
    const QString name = index >= 0 ? documentName(index) : QStringLiteral("Untitled");
    setWindowTitle(QStringLiteral("%1 — Kooka Pictura").arg(name));
}

void PicturaMainWindow::saveSession()
{
    // A session restore rebuilds the columns from the store; re-saving mid
    // rebuild would persist a partial layout.
    if (restoringPanelSession_) {
        return;
    }
    SessionState state = pictura::loadSession();
    state.layout = saveState();
    state.layoutRevision = kLayoutRevision;
    state.brightnessLevel = brightnessLevel_;
    state.gpuCompute = gpuCompute_;
    state.toolsColumns = toolbox_ ? toolbox_->columns() : 1;
    state.useShiftKeyForToolSwitch = useShiftKeyForToolSwitch_;
    if (panelColumn_) {
        const bool iconic = panelColumn_->railMode();
        state.panelRailMode = iconic ? QStringLiteral("iconic") : QStringLiteral("normal");
        // Keep the last normal-mode width; an iconic column has no width of its
        // own, so the loaded value survives unchanged.
        if (!iconic && panelColumn_->width() > 0) {
            state.railWidth = panelColumn_->width();
        }
        state.autoCollapseIconic = panelColumn_->autoCollapseIconic();
        state.autoShowHidden = panelColumn_->autoShowHidden();
        // Legacy flat mirror of the primary column kept for older stores.
        state.panelGroups = panelColumn_->savePanelState();
    }
    // v6: the ordered per-column layout, in central-splitter order.
    QJsonArray columns;
    int order = 0;
    for (PanelColumn* column : panelColumns()) {
        QJsonObject entry;
        entry.insert(QStringLiteral("side"),
                     sideOf(column) == PanelSide::Left ? QStringLiteral("left")
                                                       : QStringLiteral("right"));
        entry.insert(QStringLiteral("order"), order++);
        entry.insert(QStringLiteral("groups"), column->savePanelState());
        columns.append(entry);
    }
    state.panelColumns = columns;
    state.schemaVersion = 6;
    state.recent = recent_;
    pictura::saveSession(state);
}

bool PicturaMainWindow::restoreStoredLayout(const QByteArray& layout, int revision)
{
    if (layout.isEmpty() || revision != kLayoutRevision) {
        return false;
    }
    return restoreState(layout);
}

void PicturaMainWindow::wirePanelColumn(PanelColumn* column)
{
    if (!column) {
        return;
    }
    connect(column, &PanelColumn::stateChanged, this, [this]() { saveSession(); });
    connect(column, &PanelColumn::interfaceOptionsRequested, this,
            [this]() { showPreferences(PreferencesDialog::kInterface); });
}

void PicturaMainWindow::reapplyColumnStretch()
{
    if (!centerSplitter_ || !tabs_) {
        return;
    }
    for (int i = 0; i < centerSplitter_->count(); ++i) {
        centerSplitter_->setStretchFactor(i, centerSplitter_->widget(i) == tabs_ ? 1 : 0);
    }
}

QList<PanelColumn*> PicturaMainWindow::panelColumns() const
{
    QList<PanelColumn*> out;
    if (!centerSplitter_) {
        return out;
    }
    for (int i = 0; i < centerSplitter_->count(); ++i) {
        if (auto* column = qobject_cast<PanelColumn*>(centerSplitter_->widget(i))) {
            out << column;
        }
    }
    return out;
}

int PicturaMainWindow::columnCount() const
{
    return panelColumns().size();
}

PanelSide PicturaMainWindow::sideOf(const PanelColumn* column) const
{
    if (!column || !centerSplitter_ || !tabs_) {
        return PanelSide::Right;
    }
    const int tabsIndex = centerSplitter_->indexOf(tabs_);
    const int index = centerSplitter_->indexOf(const_cast<PanelColumn*>(column));
    return index >= 0 && index < tabsIndex ? PanelSide::Left : PanelSide::Right;
}

PanelColumn* PicturaMainWindow::createPanelColumn(PanelSide side, PanelColumn* anchor)
{
    if (!centerSplitter_) {
        return nullptr;
    }
    auto* column = new PanelColumn(this);
    column->setDynamic(true);
    wirePanelColumn(column);
    int insertAt;
    const int anchorIndex = anchor ? centerSplitter_->indexOf(anchor) : -1;
    if (anchorIndex >= 0) {
        // M44 W5: place the new column immediately before/after its anchor.
        insertAt = side == PanelSide::Left ? anchorIndex : anchorIndex + 1;
    } else {
        insertAt = side == PanelSide::Left ? 0 : centerSplitter_->count();
    }
    centerSplitter_->insertWidget(insertAt, column);
    reapplyColumnStretch();
    column->setVisible(!panelsHidden_);
    return column;
}

void PicturaMainWindow::removeColumnIfEmpty(PanelColumn* column)
{
    // Only drop-created columns disappear; the primary column keeps its place
    // even when every group is closed.
    if (!column || column == panelColumn_ || !column->isDynamic()) {
        return;
    }
    if (!column->groups().isEmpty()) {
        return;
    }
    column->hide();
    column->setParent(nullptr);
    column->deleteLater();
    reapplyColumnStretch();
    saveSession();
}

PanelColumn* PicturaMainWindow::columnForPanel(const QString& objectName) const
{
    for (PanelColumn* column : panelColumns()) {
        if (column->groupForPanel(objectName)) {
            return column;
        }
    }
    return nullptr;
}

PanelColumn* PicturaMainWindow::columnAtGlobal(const QPoint& globalPos) const
{
    for (PanelColumn* column : panelColumns()) {
        if (column->isVisible()
            && QRect(column->mapToGlobal(QPoint(0, 0)), column->size()).contains(globalPos)) {
            return column;
        }
    }
    return nullptr;
}

int PicturaMainWindow::newColumnSideAt(const QPoint& globalPos) const
{
    // Dropping over the Tools dock allocates a column on the dock's side.
    if (toolsDock_ && toolsDock_->isVisible()) {
        const QRect dockRect(toolsDock_->mapToGlobal(QPoint(0, 0)), toolsDock_->size());
        if (dockRect.contains(globalPos)) {
            switch (toolsArea_) {
            case Qt::RightDockWidgetArea:
            case Qt::BottomDockWidgetArea:
                return 1;
            default:
                return 0;
            }
        }
    }
    QWidget* central = centralWidget();
    if (!central) {
        return -1;
    }
    // ponytail: chosen constant band at the central area's outer edges, not a
    // sourced CS6 metric. A screenshot can retune it.
    constexpr int kNewColumnMargin = 28;
    const int left = central->mapToGlobal(QPoint(0, 0)).x();
    const int right = left + central->width();
    if (globalPos.x() < left + kNewColumnMargin) {
        return 0;
    }
    if (globalPos.x() > right - kNewColumnMargin) {
        return 1;
    }
    return -1;
}

PanelColumn* PicturaMainWindow::columnEdgeAnchorAt(const QPoint& globalPos,
                                                   const PanelColumn* exclude,
                                                   int* side) const
{
    if (side) {
        *side = -1;
    }
    // ponytail: chosen proximity band beside a column, not a sourced CS6 metric.
    constexpr int kEdgeBand = 26;
    constexpr int kEdgeInside = 6;
    for (PanelColumn* column : panelColumns()) {
        if (!column || column == exclude || !column->isVisible()) {
            continue;
        }
        const QRect r(column->mapToGlobal(QPoint(0, 0)), column->size());
        if (!r.isValid() || r.width() <= kEdgeBand + kEdgeInside) {
            continue;
        }
        if (globalPos.y() < r.top() || globalPos.y() > r.bottom()) {
            continue;
        }
        if (globalPos.x() >= r.left() - kEdgeBand && globalPos.x() <= r.left() + kEdgeInside) {
            if (side) {
                *side = 0;
            }
            return column;
        }
        if (globalPos.x() >= r.right() - kEdgeInside && globalPos.x() <= r.right() + kEdgeBand) {
            if (side) {
                *side = 1;
            }
            return column;
        }
    }
    return nullptr;
}

QString PicturaMainWindow::panelColumnSideForTest(int index) const
{    const QList<PanelColumn*> columns = panelColumns();
    if (index < 0 || index >= columns.size()) {
        return QString();
    }
    return sideOf(columns.at(index)) == PanelSide::Left ? QStringLiteral("left")
                                                        : QStringLiteral("right");
}

bool PicturaMainWindow::newColumnDropForTest(const QString& panelName, const QString& side)
{
    PanelColumn* source = columnForPanel(panelName);
    if (!source || !centralWidget()) {
        return false;
    }
    const int before = columnCount();
    const QRect central(centralWidget()->mapToGlobal(QPoint(0, 0)), centralWidget()->size());
    QPoint point;
    int expected = -1;
    if (side == QStringLiteral("left")) {
        point = QPoint(central.left() + 2, central.center().y());
        expected = 0;
    } else if (side == QStringLiteral("right")) {
        point = QPoint(central.right() - 2, central.center().y());
        expected = 1;
    } else if (side == QStringLiteral("tools")) {
        if (!toolsDock_ || !toolsDock_->isVisible()) {
            return false;
        }
        point = toolsDock_->mapToGlobal(toolsDock_->rect().center());
        expected = toolsArea_ == Qt::LeftDockWidgetArea ? 0 : 1;
    } else {
        return false;
    }
    if (!source->beginTabDragForTest(panelName)) {
        return false;
    }
    source->dragToForTest(point);
    const bool dropped = source->dropForTest(point);
    QCoreApplication::processEvents();
    PanelColumn* destination = columnForPanel(panelName);
    if (!dropped || !destination || destination == source) {
        return false;
    }
    if (columnCount() != before + 1) {
        return false;
    }
    const PanelSide want = expected == 0 ? PanelSide::Left : PanelSide::Right;
    return sideOf(destination) == want;
}

bool PicturaMainWindow::newColumnBesideForTest(const QString& panelName,
                                               const QString& anchorPanel)
{
    PanelColumn* source = columnForPanel(panelName);
    PanelColumn* anchor = columnForPanel(anchorPanel);
    if (!source || !anchor || source == anchor) {
        return false;
    }
    const int before = columnCount();
    const QRect r(anchor->mapToGlobal(QPoint(0, 0)), anchor->size());
    // Just outside the anchor's left edge: the resolver treats this as a new
    // column anchored immediately before `anchor`.
    const QPoint point(r.left() - 8, r.center().y());
    if (!source->beginTabDragForTest(panelName)) {
        return false;
    }
    source->dragToForTest(point);
    const bool dropped = source->dropForTest(point);
    QCoreApplication::processEvents();
    PanelColumn* destination = columnForPanel(panelName);
    if (!dropped || !destination || destination == source || columnCount() != before + 1) {
        return false;
    }
    const int destIndex = centerSplitter_->indexOf(destination);
    const int anchorIndex = centerSplitter_->indexOf(anchor);
    return destIndex >= 0 && anchorIndex >= 0 && qAbs(destIndex - anchorIndex) == 1;
}

bool PicturaMainWindow::dropIntoGroupForTest(const QString& panelName, const QString& targetPanel,
                                             int index)
{
    PanelColumn* source = columnForPanel(panelName);
    PanelColumn* destination = columnForPanel(targetPanel);
    if (!source || !destination) {
        return false;
    }
    PanelGroup* targetGroup = destination->groupForPanel(targetPanel);
    if (!targetGroup) {
        return false;
    }
    QPoint point;
    if (destination->railMode()) {
        point = destination->stripEntryPointForTest(targetPanel, 0);
    } else {
        destination->ensureGroupVisibleForTest(targetPanel);
        for (int i = 0; i < 4; ++i) {
            QCoreApplication::processEvents();
        }
        const int at = index < 0 ? targetGroup->titleCountForTest()
                                 : qMin(index, targetGroup->titleCountForTest());
        point = targetGroup->tabInsertionGlobalPointForTest(at);
    }
    if (point.isNull() || !source->beginTabDragForTest(panelName)) {
        return false;
    }
    source->dragToForTest(point);
    const bool dropped = source->dropForTest(point);
    for (int i = 0; i < 4; ++i) {
        QCoreApplication::processEvents();
    }
    PanelColumn* owner = columnForPanel(panelName);
    return dropped && owner && owner->groupForPanel(panelName) == targetGroup;
}

bool PicturaMainWindow::dropBoundaryForTest(const QString& panelName, const QString& targetPanel,
                                            bool above)
{
    PanelColumn* source = columnForPanel(panelName);
    PanelColumn* destination = columnForPanel(targetPanel);
    if (!source || !destination) {
        return false;
    }
    PanelGroup* targetGroup = destination->groupForPanel(targetPanel);
    if (!targetGroup) {
        return false;
    }
    QPoint point;
    if (destination->railMode()) {
        // M44 C3: compact boundaries are the inter-group dividers / strip ends,
        // not a button edge (a button edge now inserts into its group).
        const int gi = destination->compactStripGroupIndexForTest(targetPanel);
        if (gi < 0) {
            return false;
        }
        point = destination->compactStripBoundaryPointForTest(above ? gi : gi + 1);
    } else {
        destination->ensureGroupVisibleForTest(targetPanel);
        for (int i = 0; i < 4; ++i) {
            QCoreApplication::processEvents();
        }
        const QRect groupRect(targetGroup->mapToGlobal(QPoint(0, 0)), targetGroup->size());
        // Stay clear of the tab bar so the drop resolves the group's top/bottom
        // half, not a tab insertion.
        const int barBottom = targetGroup->tabBarGlobalRect().bottom();
        const int top = qMax(groupRect.top(), barBottom + 1);
        const int height = qMax(1, groupRect.bottom() - top);
        point = QPoint(groupRect.center().x(),
                       above ? top + height / 4 : top + (height * 3) / 4);
    }
    PanelGroup* original = source->groupForPanel(panelName);
    if (point.isNull() || !source->beginTabDragForTest(panelName)) {
        return false;
    }
    source->dragToForTest(point);
    const bool dropped = source->dropForTest(point);
    for (int i = 0; i < 4; ++i) {
        QCoreApplication::processEvents();
    }
    // A panel boundary drop leaves the panel alone in a fresh group.
    PanelColumn* owner = columnForPanel(panelName);
    PanelGroup* landed = owner ? owner->groupForPanel(panelName) : nullptr;
    return dropped && landed && landed != original && landed->titleCountForTest() == 1;
}

void PicturaMainWindow::clearDynamicColumns()
{
    // Move every drop-created column's groups back into the primary column and
    // delete the column, without the `removeColumnIfEmpty` save side effect.
    const QList<PanelColumn*> columns = panelColumns();
    for (PanelColumn* column : columns) {
        if (!column || column == panelColumn_) {
            continue;
        }
        const QList<PanelGroup*> groups = column->groups();
        for (PanelGroup* group : groups) {
            if (!group) {
                continue;
            }
            if (PanelGroup* taken = column->takeGroup(group->objectName())) {
                panelColumn_->addGroup(taken);
            }
        }
        column->hide();
        column->setParent(nullptr);
        column->deleteLater();
    }
    reapplyColumnStretch();
}

void PicturaMainWindow::applyPanelSession(const SessionState& state)
{
    if (!panelColumn_) {
        return;
    }
    restoringPanelSession_ = true;
    clearDynamicColumns();

    // Order the stored columns by `order` (a stable selection sort: the list is
    // at most a handful of entries).
    QList<QJsonObject> entries;
    for (const QJsonValue& value : state.panelColumns) {
        if (value.isObject()) {
            entries.append(value.toObject());
        }
    }
    for (int i = 0; i < entries.size(); ++i) {
        for (int j = i + 1; j < entries.size(); ++j) {
            if (entries.at(j).value(QStringLiteral("order")).toInt()
                < entries.at(i).value(QStringLiteral("order")).toInt()) {
                entries.swapItemsAt(i, j);
            }
        }
    }

    // The primary column is the first stored right-hand column; its groups stay
    // in `panelColumn_`. Every other stored column adopts its groups from the
    // current column stack. A column whose groups are all unknown is skipped.
    int primary = -1;
    QList<int> leftIndices;
    QList<int> rightIndices;
    for (int i = 0; i < entries.size(); ++i) {
        if (entries.at(i).value(QStringLiteral("side")).toString()
            == QStringLiteral("left")) {
            leftIndices.append(i);
        } else {
            rightIndices.append(i);
        }
    }
    if (!rightIndices.isEmpty()) {
        primary = rightIndices.first();
    }

    auto buildColumn = [this](const QJsonObject& entry, PanelSide side) {
        const QJsonArray groups = entry.value(QStringLiteral("groups")).toArray();
        QList<PanelGroup*> moved;
        for (const QJsonValue& value : groups) {
            const QString name =
                value.toObject().value(QStringLiteral("name")).toString();
            if (PanelGroup* group = panelColumn_->takeGroup(name)) {
                moved.append(group);
            }
        }
        if (moved.isEmpty()) {
            return;
        }
        PanelColumn* column = createPanelColumn(side);
        if (!column) {
            for (PanelGroup* group : moved) {
                panelColumn_->addGroup(group);
            }
            return;
        }
        for (PanelGroup* group : moved) {
            column->addGroup(group);
        }
        column->restorePanelState(groups);
    };

    if (entries.isEmpty()) {
        // No v6 layout (or an explicit empty one): keep the legacy behaviour.
        panelColumn_->restorePanelState(state.panelGroups);
    } else {
        // Left columns are inserted at the splitter head, so adopt them
        // outermost-first to preserve their left-to-right order.
        for (int k = leftIndices.size() - 1; k >= 0; --k) {
            buildColumn(entries.at(leftIndices.at(k)), PanelSide::Left);
        }
        for (int i = 0; i < rightIndices.size(); ++i) {
            if (rightIndices.at(i) != primary) {
                buildColumn(entries.at(rightIndices.at(i)), PanelSide::Right);
            }
        }
        if (primary >= 0) {
            panelColumn_->restorePanelState(
                entries.at(primary).value(QStringLiteral("groups")).toArray());
        }
    }

    const bool iconic = state.panelRailMode == QStringLiteral("iconic");
    for (PanelColumn* column : panelColumns()) {
        column->setAutoCollapseIconic(state.autoCollapseIconic);
        column->setAutoShowHidden(state.autoShowHidden);
        column->setRailMode(iconic);
    }
    if (!iconic) {
        panelColumn_->setPreferredWidth(state.railWidth);
    }
    restoringPanelSession_ = false;
}

void PicturaMainWindow::showPreferences(const QString& page)
{
    if (!preferencesDialog_) {
        preferencesDialog_ = new PreferencesDialog(this);
        preferencesDialog_->setShiftKeyForToolSwitch(useShiftKeyForToolSwitch_);
        preferencesDialog_->setAutoCollapseIconic(panelColumn_->autoCollapseIconic());
        preferencesDialog_->setAutoShowHidden(panelColumn_->autoShowHidden());
        preferencesDialog_->setBrightnessLevel(brightnessLevel_);
        connect(preferencesDialog_, &PreferencesDialog::useShiftKeyForToolSwitchChanged,
                this, [this](bool on) {
                    useShiftKeyForToolSwitch_ = on;
                    if (toolbox_) {
                        toolbox_->setShiftKeyForToolSwitch(on);
                    }
                    saveSession();
                });
        connect(preferencesDialog_, &PreferencesDialog::autoCollapseIconicChanged, this,
                [this](bool on) { panelColumn_->setAutoCollapseIconic(on); });
        connect(preferencesDialog_, &PreferencesDialog::autoShowHiddenChanged, this,
                [this](bool on) { panelColumn_->setAutoShowHidden(on); });
        connect(preferencesDialog_, &PreferencesDialog::brightnessLevelChanged, this,
                [this](int level) {
                    setBrightnessLevel(level);
                    saveSession();
                });
    }
    preferencesDialog_->openOn(page);
}

void PicturaMainWindow::closeEvent(QCloseEvent* event)
{
    for (int i = 0; i < docs_.size(); ++i) {
        if (!isDocumentDirty(i)) {
            continue;
        }
        switch (askUnsaved(this, documentName(i))) {
        case UnsavedChoice::Cancel:
            event->ignore();
            return;
        case UnsavedChoice::Save:
            setActiveDocumentIndex(i);
            if (!saveActive()) {
                event->ignore();
                return;
            }
            break;
        case UnsavedChoice::Discard:
            break;
        }
    }
    saveSession();
    QMainWindow::closeEvent(event);
}

void PicturaMainWindow::keyPressEvent(QKeyEvent* event)
{
    if (!event->isAutoRepeat()
        && (event->key() == Qt::Key_Return || event->key() == Qt::Key_Enter)
        && tools_ && tools_->activeTool() == ToolId::Crop) {
        commitCrop();
        return;
    }
    if (!event->isAutoRepeat() && event->key() == Qt::Key_F) {
        cycleScreenMode(!(event->modifiers() & Qt::ShiftModifier));
        return;
    }
    QMainWindow::keyPressEvent(event);
}

void PicturaMainWindow::buildMenus()
{
    registry_->buildMenuBar(menuBar());

    // Icon for every implemented command; ids without an asset are skipped.
    static const char* const kIconCommands[] = {
        command_ids::FileNew,
        command_ids::FileOpen,
        command_ids::FileSave,
        command_ids::FileSaveAs,
        command_ids::FileRevert,
        command_ids::FileClose,
        command_ids::FileCloseAll,
        command_ids::FileExit,
        command_ids::EditUndo,
        command_ids::EditRedo,
        command_ids::EditStepForward,
        command_ids::EditStepBackward,
        command_ids::ImageRotate90Cw,
        command_ids::ImageRotate90Ccw,
        command_ids::ImageRotate180,
        command_ids::ImageFlipHorizontal,
        command_ids::ImageFlipVertical,
        command_ids::ImageCrop,
        command_ids::SelectAll,
        command_ids::SelectDeselect,
        command_ids::ViewZoomIn,
        command_ids::ViewZoomOut,
        command_ids::ViewFitOnScreen,
        command_ids::ViewActualPixels,
        command_ids::ViewScreenModeStandard,
        command_ids::ViewScreenModeFullWithMenuBar,
        command_ids::ViewScreenModeFull,
        command_ids::ViewOptions,
        command_ids::WindowPanelsLayers,
        command_ids::WindowPanelsTools,
        command_ids::HelpAbout,
    };
    for (const char* id : kIconCommands) {
        const QString commandId = QString::fromLatin1(id);
        if (QAction* action = registry_->action(commandId)) {
            action->setIcon(icon(commandId));
        }
    }
}

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

    // Iconic single-panel groups. Phase A only records `defaultIconic`; the
    // compact rendering lands in Phase B.
    auto* historyGroup = new PanelGroup(this);
    addPanel(historyGroup, historyPanel_, tr("History"),
             QString::fromLatin1(command_ids::WindowPanelsHistory));
    historyGroup->setDefaultIconic(true);
    panelColumn_->addGroup(historyGroup);

    auto* actionsGroup = new PanelGroup(this);
    addPanel(actionsGroup, actionsPanel_, tr("Actions"),
             QString::fromLatin1(command_ids::WindowPanelsActions));
    actionsGroup->setDefaultIconic(true);
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
        // M44 T2: the Tools dock can now sit on any side.
        if (area != Qt::NoDockWidgetArea) {
            toolsArea_ = area;
        }
        ensureToolsNotTabified();
    });
    connect(toolbox, &QDockWidget::topLevelChanged, this,
            [this](bool) { ensureToolsNotTabified(); });

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

    auto* readout = new QAction(this);
    readout->setObjectName(QStringLiteral("statusReadout"));
    readout->setData(QStringLiteral("sizes"));

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
        connect(option, &QAction::triggered, this, [this, readout, key]() {
            readout->setData(key);
            updateStatus();
        });
    }
    optionsButton->setMenu(optionsMenu);
    bar->addPermanentWidget(optionsButton);
}

void PicturaMainWindow::registerHandlers()
{
    registry_->setHandler(command_ids::FileNew, [this]() { showNewDocumentDialog(); });
    registry_->setHandler(command_ids::FileOpen, [this]() { showOpenDialog(); });
    registry_->setEnabledProvider(command_ids::FileOpen, []() { return true; });

    registry_->setHandler(command_ids::FileSave, [this]() { saveActive(); });
    registry_->setHandler(command_ids::FileSaveAs, [this]() {
        QString path = QFileDialog::getSaveFileName(this, tr("Save As"), activeFilePath(),
                                                    QStringLiteral("Photoshop files (*.psd *.psb)"));
        if (path.isEmpty()) {
            return;
        }
        if (QFileInfo(path).suffix().isEmpty()) {
            path += QStringLiteral(".psd");
        }
        saveActiveAs(path);
    });
    registry_->setHandler(command_ids::FileRevert, [this]() {
        const int index = activeDocumentIndex();
        if (index < 0 || documentPath(index).isEmpty()) {
            return;
        }
        if (isDocumentDirty(index)) {
            switch (askUnsaved(this, documentName(index))) {
            case UnsavedChoice::Cancel:
                return;
            case UnsavedChoice::Save:
                setActiveDocumentIndex(index);
                if (!saveActive()) {
                    return;
                }
                break;
            case UnsavedChoice::Discard:
                break;
            }
        }
        revertActive();
    });
    registry_->setHandler(command_ids::FileClose, [this]() { closeActiveDocument(true); });
    registry_->setHandler(command_ids::FileCloseAll, [this]() {
        for (int i = docs_.size() - 1; i >= 0; --i) {
            if (!closeDocument(i, true)) {
                break;
            }
        }
    });
    registry_->setHandler(command_ids::FileExit, [this]() {
        for (int i = docs_.size() - 1; i >= 0; --i) {
            if (!closeDocument(i, true)) {
                break;
            }
        }
        if (documentCount() == 0) {
            qApp->quit();
        }
    });

    auto hasDocument = [this]() { return documentCount() > 0; };
    for (const char* id : {command_ids::FileSave, command_ids::FileSaveAs, command_ids::FileClose,
                           command_ids::FileCloseAll}) {
        registry_->setEnabledProvider(id, hasDocument);
    }
    registry_->setEnabledProvider(command_ids::FileRevert,
                                  [this]() { return !activeFilePath().isEmpty(); });

    registry_->setHandler(command_ids::EditUndo, [this]() {
        PictureView* view = activeView();
        if (!view) {
            return;
        }
        const bool ok = view->can_undo() ? view->undo() : view->redo();
        if (ok) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::EditUndo, [this]() {
        PictureView* view = activeView();
        return view && (view->can_undo() || view->can_redo());
    });
    registry_->setLabelProvider(command_ids::EditUndo, [this]() {
        PictureView* view = activeView();
        return view && !view->can_undo() ? QStringLiteral("Redo") : QStringLiteral("Undo");
    });

    registry_->setHandler(command_ids::EditRedo, [this]() {
        if (PictureView* view = activeView(); view && view->redo()) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::EditRedo,
                                  [this]() { return activeView() && activeView()->can_redo(); });

    registry_->setHandler(command_ids::EditStepBackward, [this]() {
        if (PictureView* view = activeView(); view && view->undo()) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::EditStepBackward,
                                  [this]() { return activeView() && activeView()->can_undo(); });

    registry_->setHandler(command_ids::EditStepForward, [this]() {
        if (PictureView* view = activeView(); view && view->redo()) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::EditStepForward,
                                  [this]() { return activeView() && activeView()->can_redo(); });

    registry_->setHandler(command_ids::ImageRotate90Cw, [this]() {
        if (PictureView* view = activeView(); view && view->rotate_doc(1)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::ImageRotate90Ccw, [this]() {
        if (PictureView* view = activeView(); view && view->rotate_doc(3)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::ImageRotate180, [this]() {
        if (PictureView* view = activeView(); view && view->rotate_doc(2)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::ImageFlipHorizontal, [this]() {
        if (PictureView* view = activeView(); view && view->flip_doc(true)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::ImageFlipVertical, [this]() {
        if (PictureView* view = activeView(); view && view->flip_doc(false)) {
            refresh();
        }
    });
    for (const char* id : {command_ids::ImageRotate90Cw, command_ids::ImageRotate90Ccw,
                           command_ids::ImageRotate180, command_ids::ImageFlipHorizontal,
                           command_ids::ImageFlipVertical}) {
        registry_->setEnabledProvider(id,
                                      [this]() { return activeView() && activeView()->has_document(); });
    }

    registry_->setHandler(command_ids::ImageCrop, [this]() {
        if (tools_ && tools_->hasPendingCrop()) {
            commitCrop();
            return;
        }
        PictureView* view = activeView();
        if (!view) {
            return;
        }
        const QString bounds = view->selection_bounds();
        if (bounds.isEmpty()) {
            return;
        }
        const QStringList parts = bounds.split(QLatin1Char(' '), Qt::SkipEmptyParts);
        if (parts.size() != 4) {
            return;
        }
        if (view->crop(parts.at(0).toInt(), parts.at(1).toInt(), parts.at(2).toInt(),
                       parts.at(3).toInt())) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::ImageCrop,
                                  [this]() { return activeView() && activeView()->has_document(); });

    registry_->setHandler(command_ids::SelectAll, [this]() {
        if (PictureView* view = activeView()) {
            view->select_all();
        }
    });
    registry_->setEnabledProvider(command_ids::SelectAll,
                                  [this]() { return activeView() && activeView()->has_document(); });
    registry_->setHandler(command_ids::SelectDeselect, [this]() {
        if (PictureView* view = activeView()) {
            view->deselect();
        }
    });
    registry_->setEnabledProvider(command_ids::SelectDeselect,
                                  [this]() { return activeView() && activeView()->has_document(); });

    // M37: layer creation and grouping. The target is the active document's
    // current layer — the Layers panel's selected row, -1 when none.
    // ponytail: one current layer; M38's multi-selection upgrades these to
    // per-selection operations.
    registry_->setHandler(command_ids::LayerNewLayer, [this]() {
        if (PictureView* view = activeView()) {
            view->add_layer(layersPanel_ ? layersPanel_->currentLayer() : -1);
            refresh();
        }
    });
    registry_->setHandler(command_ids::LayerNewGroup, [this]() {
        if (PictureView* view = activeView()) {
            view->add_group(layersPanel_ ? layersPanel_->currentLayer() : -1);
            refresh();
        }
    });
    registry_->setHandler(command_ids::LayerDuplicateLayer, [this]() {
        if (PictureView* view = activeView()) {
            view->duplicate_layer(layersPanel_ ? layersPanel_->currentLayer() : -1);
            refresh();
        }
    });
    registry_->setHandler(command_ids::LayerGroupLayers, [this]() {
        if (PictureView* view = activeView()) {
            view->group_layer(layersPanel_ ? layersPanel_->currentLayer() : -1);
            refresh();
        }
    });
    registry_->setHandler(command_ids::LayerUngroupLayers, [this]() {
        if (PictureView* view = activeView()) {
            view->ungroup_layer(layersPanel_ ? layersPanel_->currentLayer() : -1);
            refresh();
        }
    });
    for (const char* id : {command_ids::LayerNewLayer, command_ids::LayerNewGroup,
                           command_ids::LayerDuplicateLayer, command_ids::LayerGroupLayers,
                           command_ids::LayerUngroupLayers}) {
        registry_->setEnabledProvider(id,
                                      [this]() { return activeView() && activeView()->has_document(); });
    }

    registry_->setHandler(command_ids::ViewZoomIn, [this]() {
        if (ImageView* canvas = imageView()) {
            canvas->zoomIn();
        }
    });
    registry_->setHandler(command_ids::ViewZoomOut, [this]() {
        if (ImageView* canvas = imageView()) {
            canvas->zoomOut();
        }
    });
    registry_->setHandler(command_ids::ViewFitOnScreen, [this]() {
        if (ImageView* canvas = imageView()) {
            canvas->fitOnScreen();
        }
    });
    registry_->setHandler(command_ids::ViewActualPixels, [this]() {
        if (ImageView* canvas = imageView()) {
            canvas->actualPixels();
        }
    });

    registry_->setHandler(command_ids::ViewScreenModeStandard,
                          [this]() { setScreenMode(ScreenMode::Standard); });
    registry_->setHandler(command_ids::ViewScreenModeFullWithMenuBar,
                          [this]() { setScreenMode(ScreenMode::FullWithMenuBar); });
    registry_->setHandler(command_ids::ViewScreenModeFull,
                          [this]() { setScreenMode(ScreenMode::Full); });
    registry_->setCheckedProvider(command_ids::ViewScreenModeStandard,
                                  [this]() { return screenMode_ == ScreenMode::Standard; });
    registry_->setCheckedProvider(command_ids::ViewScreenModeFullWithMenuBar, [this]() {
        return screenMode_ == ScreenMode::FullWithMenuBar;
    });
    registry_->setCheckedProvider(command_ids::ViewScreenModeFull,
                                  [this]() { return screenMode_ == ScreenMode::Full; });

    registry_->setHandler(command_ids::ViewOptions, [this]() {
        QAction* action = registry_->action(command_ids::ViewOptions);
        if (optionsBar_ && action) {
            optionsBar_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::ViewOptions,
                                  [this]() { return optionsBar_ && optionsBar_->isVisible(); });

    registry_->setHandler(command_ids::ViewGpuCompute, [this]() {
        const QAction* action = registry_->action(command_ids::ViewGpuCompute);
        gpuCompute_ = action && action->isChecked();
        for (const DocEntry& entry : docs_) {
            if (entry.view) {
                entry.view->set_gpu_compute(gpuCompute_);
            }
        }
        saveSession();
        registry_->refresh();
        refresh();
    });
    registry_->setEnabledProvider(command_ids::ViewGpuCompute, [this]() { return gpuAvailable_; });
    registry_->setCheckedProvider(command_ids::ViewGpuCompute, [this]() { return gpuCompute_; });

    registry_->setHandler(command_ids::WindowPanelsTools, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsTools);
        if (toolsDock_ && action) {
            toolsDock_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsTools,
                                  [this]() { return toolsDock_ && toolsDock_->isVisible(); });

    // The two implemented Preferences leaves; the other CS6 panes stay
    // disabled and are no-ops (their command-tree enablement is unchanged).
    registry_->setHandler(command_ids::EditPreferencesGeneral,
                          [this]() { showPreferences(PreferencesDialog::kGeneral); });
    registry_->setHandler(command_ids::EditPreferencesInterface,
                          [this]() { showPreferences(PreferencesDialog::kInterface); });

    // Every other panel is a content widget hosted by the PanelColumn; the
    // Window > Panels toggles drive and reflect the column's registry rather
    // than a dock's visibility.
    struct PanelToggle {
        const char* command;
        const char* panel;
    };
    static const PanelToggle kPanelToggles[] = {
        {command_ids::WindowPanelsLayers, "layersPanel"},
        {command_ids::WindowPanelsNavigator, "navigatorPanel"},
        {command_ids::WindowPanelsHistory, "historyPanel"},
        {command_ids::WindowPanelsColor, "colorPanel"},
        {command_ids::WindowPanelsSwatches, "swatchesPanel"},
        {command_ids::WindowPanelsInfo, "infoPanel"},
        {command_ids::WindowPanelsHistogram, "histogramPanel"},
        {command_ids::WindowPanelsGradients, "gradientsPanel"},
        {command_ids::WindowPanelsPatterns, "patternsPanel"},
        {command_ids::WindowPanelsProperties, "propertiesPanel"},
        {command_ids::WindowPanelsAdjustments, "adjustmentsPanel"},
        {command_ids::WindowPanelsLibraries, "librariesPanel"},
        {command_ids::WindowPanelsChannels, "channelsPanel"},
        {command_ids::WindowPanelsPaths, "pathsPanel"},
        {command_ids::WindowPanelsActions, "actionsPanel"},
    };
    for (const PanelToggle& toggle : kPanelToggles) {
        const QString command = QString::fromLatin1(toggle.command);
        const QString panel = QString::fromLatin1(toggle.panel);
        registry_->setHandler(command, [this, command, panel]() {
            QAction* action = registry_->action(command);
            PanelColumn* owner = columnForPanel(panel);
            if (!owner) {
                owner = panelColumn_;
            }
            if (owner && action) {
                owner->showPanel(panel, action->isChecked());
            }
        });
        registry_->setCheckedProvider(command, [this, panel]() {
            PanelColumn* owner = columnForPanel(panel);
            if (!owner) {
                owner = panelColumn_;
            }
            return owner && owner->isPanelVisible(panel);
        });
    }

    registry_->setHandler(command_ids::HelpAbout, [this]() {
        QMessageBox::about(this, tr("About Kooka Pictura"),
                           tr("Kooka Pictura — a Photoshop CS6 reimplementation in Rust and Qt."));
    });
}

void PicturaMainWindow::updateStatus()
{
    ImageView* canvas = imageView();
    if (zoomLabel_) {
        zoomLabel_->setText(
            canvas ? QStringLiteral("%1%").arg(qRound(canvas->zoom() * 100.0)) : QStringLiteral("—"));
    }
    if (sizeLabel_) {
        QString text = QStringLiteral("—");
        PictureView* view = activeView();
        if (view && view->has_document()) {
            const QImage image = view->image();
            const QAction* readout = findChild<QAction*>(QStringLiteral("statusReadout"));
            const QString mode = readout ? readout->data().toString() : QStringLiteral("sizes");
            if (mode == QStringLiteral("dimensions")) {
                text = QStringLiteral("W %1  H %2").arg(image.width()).arg(image.height());
            } else {
                text = QStringLiteral("%1 × %2 px").arg(image.width()).arg(image.height());
            }
        }
        sizeLabel_->setText(text);
    }
    if (backendLabel_) {
        PictureView* view = activeView();
        backendLabel_->setText(view ? view->active_backend() : QStringLiteral("—"));
    }
}

void PicturaMainWindow::updateToolHint()
{
    if (!hintLabel_) {
        return;
    }
    QString text = tools_ ? QString::fromLatin1(toolInfo(tools_->activeTool()).hint)
                          : QStringLiteral("Ready");
    if (foreground_.isValid()) {
        text += QStringLiteral("  ·  Foreground %1").arg(foreground_.name());
    }
    hintLabel_->setText(text);
}

void PicturaMainWindow::applyBrightness(int level)
{
    brightnessLevel_ = Theme::clampLevel(level);
    Theme::apply(brightnessLevel_);
}

} // namespace pictura
