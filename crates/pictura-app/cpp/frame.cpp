#include "frame_includes.h"

#include <QtWidgets/QTabBar>

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
    // docs_ is indexed in lockstep with the tab order (activeDocumentIndex,
    // viewAt, removeDocument), so a dragged tab must move its DocEntry too.
    // QTabBar::tabMoved(from, to) mirrors QList::move(from, to), and Qt keeps
    // the current tab current across the move.
    connect(tabs_->tabBar(), &QTabBar::tabMoved, this, [this](int from, int to) {
        if (from >= 0 && from < docs_.size() && to >= 0 && to < docs_.size()) {
            docs_.move(from, to);
        }
    });
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
    centerSplitter_->setChildrenCollapsible(false);
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
    refreshSelectionOverlay();
}

void PicturaMainWindow::refreshSelectionOverlay()
{
    PictureView* view = activeView();
    ImageView* canvas = imageView();
    if (!view || !canvas) {
        return;
    }
    if (view->has_selection()) {
        canvas->setSelectionContour(view->selection_contour());
    } else {
        canvas->clearSelectionContour();
    }
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
    if (!event->isAutoRepeat()
        && (event->key() == Qt::Key_Return || event->key() == Qt::Key_Enter)
        && tools_ && tools_->activeTool() == ToolId::PolygonalLasso) {
        tools_->commitPolygonLasso();
        return;
    }
    if (!event->isAutoRepeat() && event->key() == Qt::Key_Escape && tools_
        && tools_->cancelPolygonLasso()) {
        return;
    }
    if (!event->isAutoRepeat() && event->key() == Qt::Key_F) {
        cycleScreenMode(!(event->modifiers() & Qt::ShiftModifier));
        return;
    }
    QMainWindow::keyPressEvent(event);
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
            const int width = view->document_width();
            const int height = view->document_height();
            if (statusReadout_ == QStringLiteral("dimensions")) {
                text = QStringLiteral("W %1  H %2").arg(width).arg(height);
            } else {
                text = QStringLiteral("%1 × %2 px").arg(width).arg(height);
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

bool PicturaMainWindow::reorderDocumentsForTest()
{
    if (!tabs_ || !tabs_->tabBar()) {
        return false;
    }
    newDocument(QStringLiteral("ReorderA"), 4, 3, QStringLiteral("rgb"), 8,
                QStringLiteral("white"));
    newDocument(QStringLiteral("ReorderB"), 4, 3, QStringLiteral("rgb"), 8,
                QStringLiteral("white"));
    const int from = docs_.size() - 2;
    const int to = docs_.size() - 1;
    if (from < 0) {
        return false;
    }
    PictureView* a = viewAt(from);
    PictureView* b = viewAt(to);
    const QString nameA = documentName(from);
    const bool activeBefore = viewAt(activeDocumentIndex()) == b;
    tabs_->tabBar()->moveTab(from, to);
    QCoreApplication::processEvents();
    return activeBefore && a && b && viewAt(from) == b && viewAt(to) == a
           && documentName(to) == nameA && viewAt(activeDocumentIndex()) == b;
}

} // namespace pictura
