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
#include "panels/panel_rail.h"
#include "panels/placeholder_panel.h"
#include "panels/swatches_panel.h"
#include "session.h"
#include "theme.h"
#include "toolbox.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QFileInfo>
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
    tabs_->setTabsClosable(true);
    tabs_->setMovable(true);
    tabs_->setDocumentMode(true);
    setCentralWidget(tabs_);
    setDockOptions(QMainWindow::AnimatedDocks | QMainWindow::AllowTabbedDocks);

    registry_ = new CommandRegistry(this);
    addDefaultCommands(*registry_);

    const SessionState state = pictura::loadSession();
    recent_ = state.recent;
    gpuCompute_ = state.gpuCompute;
    // Probe the adapter once so the toggle can be offered without a document.
    {
        PictureView probe;
        gpuAvailable_ = probe.gpu_available();
    }
    rebuildRecentMenu();

    registerHandlers();
    buildMenus();
    buildPanels();
    buildTools();
    buildStatusBar();
    applyBrightness(state.brightnessLevel);
    if (!state.layout.isEmpty()) {
        restoreState(state.layout);
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

bool PicturaMainWindow::registerPanel(QDockWidget* dock, Qt::DockWidgetArea area)
{
    if (!dock || panelNames_.contains(dock->objectName())) {
        return false;
    }
    panelNames_.insert(dock->objectName());
    addDockWidget(area, dock);
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
    const QList<QDockWidget*> docks = findChildren<QDockWidget*>();
    auto setCanvasColor = [this](const QColor& color) {
        for (const DocEntry& entry : docs_) {
            if (entry.canvas) {
                entry.canvas->setCanvasColor(color);
            }
        }
    };
    switch (mode) {
    case ScreenMode::Standard: {
        Qt::WindowStates state = windowState();
        state.setFlag(Qt::WindowFullScreen, false);
        setWindowState(state);
        menuBar()->setVisible(true);
        statusBar()->setVisible(true);
        for (QDockWidget* dock : docks) {
            dock->setVisible(!panelsHidden_);
        }
        setCanvasColor(kCanvasColors[canvasColorIndex_]);
        break;
    }
    case ScreenMode::FullWithMenuBar:
        setWindowState(Qt::WindowFullScreen);
        menuBar()->setVisible(true);
        statusBar()->setVisible(false);
        for (QDockWidget* dock : docks) {
            dock->setVisible(false);
        }
        setCanvasColor(QColor(128, 128, 128));
        break;
    case ScreenMode::Full:
        setWindowState(Qt::WindowFullScreen);
        menuBar()->setVisible(false);
        statusBar()->setVisible(false);
        for (QDockWidget* dock : docks) {
            dock->setVisible(false);
        }
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
    const QList<QDockWidget*> docks = findChildren<QDockWidget*>();
    for (QDockWidget* dock : docks) {
        dock->setVisible(!hidden);
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
    SessionState state;
    state.layout = saveState();
    state.brightnessLevel = brightnessLevel_;
    state.gpuCompute = gpuCompute_;
    state.schemaVersion = 2;
    state.recent = recent_;
    pictura::saveSession(state);
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

    registerPanel(layersPanel_, Qt::RightDockWidgetArea);
    registerPanel(historyPanel_, Qt::RightDockWidgetArea);
    registerPanel(navigatorPanel_, Qt::RightDockWidgetArea);
    registerPanel(colorPanel_, Qt::RightDockWidgetArea);
    registerPanel(swatchesPanel_, Qt::RightDockWidgetArea);
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

    tabifyDockWidget(colorPanel_, swatchesPanel_);
    tabifyDockWidget(colorPanel_, gradientsPanel_);
    tabifyDockWidget(colorPanel_, patternsPanel_);
    colorPanel_->raise();

    tabifyDockWidget(propertiesPanel_, adjustmentsPanel_);
    tabifyDockWidget(propertiesPanel_, librariesPanel_);
    propertiesPanel_->raise();

    tabifyDockWidget(layersPanel_, channelsPanel_);
    tabifyDockWidget(layersPanel_, pathsPanel_);
    layersPanel_->raise();

    // The rail panels start collapsed; a restored session may override this.
    historyPanel_->hide();
    actionsPanel_->hide();
    infoPanel_->hide();
    navigatorPanel_->hide();
    histogramPanel_->hide();

    panelRail_ = new PanelRail(this);
    addToolBar(Qt::RightToolBarArea, panelRail_);

    // Each rail command id is the panel's `window.panels.<name>` asset id, so the
    // rail button, the dock tab, and the Window menu resolve the same icon.
    auto addRailPanel = [this](const char* commandId, const QString& tooltip) {
        const QString id = QString::fromLatin1(commandId);
        panelRail_->addPanel(id, pictura::icon(id), tooltip);
    };
    addRailPanel(command_ids::WindowPanelsHistory, tr("History"));
    addRailPanel(command_ids::WindowPanelsActions, tr("Actions"));
    addRailPanel(command_ids::WindowPanelsInfo, tr("Info"));
    addRailPanel(command_ids::WindowPanelsNavigator, tr("Navigator"));
    addRailPanel(command_ids::WindowPanelsHistogram, tr("Histogram"));

    connect(panelRail_, &PanelRail::commandTriggered, this, [this](const QString& id) {
        if (QAction* action = registry_->action(id)) {
            action->setChecked(!action->isChecked());
            registry_->dispatch(id);
        }
    });
    connect(historyPanel_, &QDockWidget::visibilityChanged, this, [this](bool visible) {
        panelRail_->setPanelChecked(command_ids::WindowPanelsHistory, visible);
    });
    connect(actionsPanel_, &QDockWidget::visibilityChanged, this, [this](bool visible) {
        panelRail_->setPanelChecked(command_ids::WindowPanelsActions, visible);
    });
    connect(infoPanel_, &QDockWidget::visibilityChanged, this, [this](bool visible) {
        panelRail_->setPanelChecked(command_ids::WindowPanelsInfo, visible);
    });
    connect(navigatorPanel_, &QDockWidget::visibilityChanged, this, [this](bool visible) {
        panelRail_->setPanelChecked(command_ids::WindowPanelsNavigator, visible);
    });
    connect(histogramPanel_, &QDockWidget::visibilityChanged, this, [this](bool visible) {
        panelRail_->setPanelChecked(command_ids::WindowPanelsHistogram, visible);
    });
}

void PicturaMainWindow::buildTools()
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

    auto cyclePaintTool = [this]() {
        if (tools_) {
            tools_->setActiveTool(tools_->activeTool() == ToolId::Brush ? ToolId::Pencil
                                                                        : ToolId::Brush);
        }
    };
    connect(new QShortcut(QKeySequence(Qt::Key_B), this), &QShortcut::activated, this,
            cyclePaintTool);
    connect(new QShortcut(QKeySequence(Qt::SHIFT | Qt::Key_B), this), &QShortcut::activated,
            this, cyclePaintTool);

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

    auto* toolbox = new Toolbox(tools_, colorState_, this);
    toolsDock_ = toolbox;
    registerPanel(toolbox, Qt::LeftDockWidgetArea);
    connect(toolbox, &Toolbox::screenModeRequested, this,
            [this]() { cycleScreenMode(true); });

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

    registry_->setHandler(command_ids::WindowPanelsLayers, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsLayers);
        if (layersPanel_ && action) {
            layersPanel_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsLayers,
                                  [this]() { return layersPanel_ && layersPanel_->isVisible(); });

    registry_->setHandler(command_ids::WindowPanelsTools, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsTools);
        if (toolsDock_ && action) {
            toolsDock_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsTools,
                                  [this]() { return toolsDock_ && toolsDock_->isVisible(); });

    registry_->setHandler(command_ids::WindowPanelsNavigator, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsNavigator);
        if (navigatorPanel_ && action) {
            navigatorPanel_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsNavigator,
                                  [this]() { return navigatorPanel_ && navigatorPanel_->isVisible(); });

    registry_->setHandler(command_ids::WindowPanelsHistory, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsHistory);
        if (historyPanel_ && action) {
            historyPanel_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsHistory,
                                  [this]() { return historyPanel_ && historyPanel_->isVisible(); });

    registry_->setHandler(command_ids::WindowPanelsColor, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsColor);
        if (colorPanel_ && action) {
            colorPanel_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsColor,
                                  [this]() { return colorPanel_ && colorPanel_->isVisible(); });

    registry_->setHandler(command_ids::WindowPanelsSwatches, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsSwatches);
        if (swatchesPanel_ && action) {
            swatchesPanel_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsSwatches,
                                  [this]() { return swatchesPanel_ && swatchesPanel_->isVisible(); });

    registry_->setHandler(command_ids::WindowPanelsInfo, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsInfo);
        if (infoPanel_ && action) {
            infoPanel_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsInfo,
                                  [this]() { return infoPanel_ && infoPanel_->isVisible(); });

    registry_->setHandler(command_ids::WindowPanelsHistogram, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsHistogram);
        if (histogramPanel_ && action) {
            histogramPanel_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsHistogram,
                                  [this]() { return histogramPanel_ && histogramPanel_->isVisible(); });

    registry_->setHandler(command_ids::WindowPanelsGradients, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsGradients);
        if (gradientsPanel_ && action) {
            gradientsPanel_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsGradients,
                                  [this]() { return gradientsPanel_ && gradientsPanel_->isVisible(); });

    registry_->setHandler(command_ids::WindowPanelsPatterns, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsPatterns);
        if (patternsPanel_ && action) {
            patternsPanel_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsPatterns,
                                  [this]() { return patternsPanel_ && patternsPanel_->isVisible(); });

    registry_->setHandler(command_ids::WindowPanelsProperties, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsProperties);
        if (propertiesPanel_ && action) {
            propertiesPanel_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsProperties,
                                  [this]() { return propertiesPanel_ && propertiesPanel_->isVisible(); });

    registry_->setHandler(command_ids::WindowPanelsAdjustments, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsAdjustments);
        if (adjustmentsPanel_ && action) {
            adjustmentsPanel_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsAdjustments,
                                  [this]() { return adjustmentsPanel_ && adjustmentsPanel_->isVisible(); });

    registry_->setHandler(command_ids::WindowPanelsLibraries, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsLibraries);
        if (librariesPanel_ && action) {
            librariesPanel_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsLibraries,
                                  [this]() { return librariesPanel_ && librariesPanel_->isVisible(); });

    registry_->setHandler(command_ids::WindowPanelsChannels, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsChannels);
        if (channelsPanel_ && action) {
            channelsPanel_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsChannels,
                                  [this]() { return channelsPanel_ && channelsPanel_->isVisible(); });

    registry_->setHandler(command_ids::WindowPanelsPaths, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsPaths);
        if (pathsPanel_ && action) {
            pathsPanel_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsPaths,
                                  [this]() { return pathsPanel_ && pathsPanel_->isVisible(); });

    registry_->setHandler(command_ids::WindowPanelsActions, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsActions);
        if (actionsPanel_ && action) {
            actionsPanel_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsActions,
                                  [this]() { return actionsPanel_ && actionsPanel_->isVisible(); });

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
