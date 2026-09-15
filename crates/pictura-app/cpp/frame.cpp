#include "frame.h"

#include "commands.h"
#include "dialogs.h"
#include "image_view.h"
#include "new_document_dialog.h"
#include "session.h"
#include "theme.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QFileInfo>
#include <QtCore/QSignalBlocker>
#include <QtGui/QAction>
#include <QtGui/QActionGroup>
#include <QtGui/QCloseEvent>
#include <QtGui/QColor>
#include <QtGui/QKeyEvent>
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
    QColor(Qt::darkGray), QColor(Qt::gray), QColor(Qt::black), QColor(Qt::white)};

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
    rebuildRecentMenu();

    registerHandlers();
    buildMenus();
    buildPanels();
    buildStatusBar();
    applyBrightness(state.brightnessLevel);
    if (!state.layout.isEmpty()) {
        restoreState(state.layout);
    }

    connect(tabs_, &QTabWidget::currentChanged, this, [this](int) { refresh(); });
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
    connect(entry.canvas, &ImageView::zoomChanged, this, [this](double) { updateStatus(); });

    docs_.append(entry);
    const int index = docs_.size() - 1;
    tabs_->addTab(entry.canvas, documentName(index));
    tabs_->setCurrentIndex(index);
    if (!path.isEmpty()) {
        rememberRecent(path);
    }
    refresh();
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

    if (layerList_) {
        QSignalBlocker blocker(layerList_);
        layerList_->clear();
        if (view) {
            const int count = view->layer_count();
            for (int i = 0; i < count; ++i) {
                auto* item = new QListWidgetItem(QStringLiteral("%1  [%2]")
                                                     .arg(view->layer_name(i), view->layer_kind(i)));
                item->setFlags(item->flags() | Qt::ItemIsUserCheckable);
                item->setCheckState(view->layer_visible(i) ? Qt::Checked : Qt::Unchecked);
                layerList_->addItem(item);
            }
        }
    }

    if (auto* selectionLabel = findChild<QLabel*>(QStringLiteral("selectionLabel"))) {
        selectionLabel->setText(
            QStringLiteral("Selection: %1 px").arg(view ? view->selection_count() : 0));
    }
    if (auto* undo = findChild<QPushButton*>(QStringLiteral("dockUndo"))) {
        undo->setEnabled(view && view->can_undo());
    }
    if (auto* redo = findChild<QPushButton*>(QStringLiteral("dockRedo"))) {
        redo->setEnabled(view && view->can_redo());
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

    retargetDock();
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
    state.schemaVersion = 1;
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
    if (!event->isAutoRepeat() && event->key() == Qt::Key_F) {
        cycleScreenMode(!(event->modifiers() & Qt::ShiftModifier));
        return;
    }
    QMainWindow::keyPressEvent(event);
}

void PicturaMainWindow::buildMenus()
{
    registry_->buildMenuBar(menuBar());
}

void PicturaMainWindow::buildPanels()
{
    layersDock_ = new QDockWidget(QStringLiteral("Layers"), this);
    layersDock_->setObjectName(QStringLiteral("layersPanel"));

    auto* panel = new QWidget(layersDock_);
    auto* panelLayout = new QVBoxLayout(panel);

    layerList_ = new QListWidget(panel);
    layerList_->setSelectionMode(QAbstractItemView::SingleSelection);
    panelLayout->addWidget(layerList_, 1);

    auto* adjustmentCombo = new QComboBox(panel);
    adjustmentCombo->addItem(QStringLiteral("Invert"), QStringLiteral("invert"));
    adjustmentCombo->addItem(QStringLiteral("Posterize"), QStringLiteral("posterize"));
    adjustmentCombo->addItem(QStringLiteral("Threshold"), QStringLiteral("threshold"));
    adjustmentCombo->addItem(QStringLiteral("Brightness/Contrast"),
                             QStringLiteral("brightness-contrast"));
    adjustmentCombo->addItem(QStringLiteral("Hue/Saturation"), QStringLiteral("hue-saturation"));
    panelLayout->addWidget(adjustmentCombo);

    auto* addButton = new QPushButton(QStringLiteral("Add Adjustment"), panel);
    auto* removeButton = new QPushButton(QStringLiteral("Remove Layer"), panel);
    panelLayout->addWidget(addButton);
    panelLayout->addWidget(removeButton);

    auto* filterCombo = new QComboBox(panel);
    filterCombo->addItem(QStringLiteral("Gaussian Blur"), QStringLiteral("gaussian-blur"));
    filterCombo->addItem(QStringLiteral("Box Blur"), QStringLiteral("box-blur"));
    filterCombo->addItem(QStringLiteral("Motion Blur"), QStringLiteral("motion-blur"));
    filterCombo->addItem(QStringLiteral("Median"), QStringLiteral("median"));
    filterCombo->addItem(QStringLiteral("Despeckle"), QStringLiteral("despeckle"));
    filterCombo->addItem(QStringLiteral("Sharpen"), QStringLiteral("sharpen"));
    filterCombo->addItem(QStringLiteral("Sharpen More"), QStringLiteral("sharpen-more"));
    filterCombo->addItem(QStringLiteral("Unsharp Mask"), QStringLiteral("unsharp-mask"));
    filterCombo->addItem(QStringLiteral("Add Noise"), QStringLiteral("add-noise"));
    filterCombo->addItem(QStringLiteral("Maximum"), QStringLiteral("maximum"));
    filterCombo->addItem(QStringLiteral("Minimum"), QStringLiteral("minimum"));
    filterCombo->addItem(QStringLiteral("Offset"), QStringLiteral("offset"));
    filterCombo->addItem(QStringLiteral("High Pass"), QStringLiteral("high-pass"));
    filterCombo->addItem(QStringLiteral("Emboss"), QStringLiteral("emboss"));
    filterCombo->addItem(QStringLiteral("Find Edges"), QStringLiteral("find-edges"));
    filterCombo->addItem(QStringLiteral("Solarize"), QStringLiteral("solarize"));
    filterCombo->addItem(QStringLiteral("Mosaic"), QStringLiteral("mosaic"));
    filterCombo->addItem(QStringLiteral("Crystallize"), QStringLiteral("crystallize"));
    filterCombo->addItem(QStringLiteral("Facet"), QStringLiteral("facet"));
    filterCombo->addItem(QStringLiteral("Fragment"), QStringLiteral("fragment"));
    filterCombo->addItem(QStringLiteral("Mezzotint"), QStringLiteral("mezzotint"));
    filterCombo->addItem(QStringLiteral("Pointillize"), QStringLiteral("pointillize"));
    filterCombo->addItem(QStringLiteral("Color Halftone"), QStringLiteral("color-halftone"));
    filterCombo->addItem(QStringLiteral("Twirl"), QStringLiteral("twirl"));
    filterCombo->addItem(QStringLiteral("Pinch"), QStringLiteral("pinch"));
    filterCombo->addItem(QStringLiteral("Spherize"), QStringLiteral("spherize"));
    filterCombo->addItem(QStringLiteral("Ripple"), QStringLiteral("ripple"));
    filterCombo->addItem(QStringLiteral("Wave"), QStringLiteral("wave"));
    filterCombo->addItem(QStringLiteral("Polar Coordinates"), QStringLiteral("polar-coordinates"));
    filterCombo->addItem(QStringLiteral("Shear"), QStringLiteral("shear"));
    filterCombo->addItem(QStringLiteral("ZigZag"), QStringLiteral("zigzag"));
    filterCombo->addItem(QStringLiteral("Ocean Ripple"), QStringLiteral("ocean-ripple"));
    filterCombo->addItem(QStringLiteral("Clouds"), QStringLiteral("clouds"));
    filterCombo->addItem(QStringLiteral("Difference Clouds"), QStringLiteral("difference-clouds"));
    filterCombo->addItem(QStringLiteral("Fibers"), QStringLiteral("fibers"));
    filterCombo->addItem(QStringLiteral("Lens Flare"), QStringLiteral("lens-flare"));
    panelLayout->addWidget(filterCombo);

    auto* applyFilterButton = new QPushButton(QStringLiteral("Apply Filter"), panel);
    panelLayout->addWidget(applyFilterButton);

    auto* selectAllButton = new QPushButton(QStringLiteral("Select all"), panel);
    auto* wandButton = new QPushButton(QStringLiteral("Magic wand (center)"), panel);
    auto* deselectButton = new QPushButton(QStringLiteral("Deselect"), panel);
    auto* selectionRow = new QHBoxLayout();
    selectionRow->addWidget(selectAllButton);
    selectionRow->addWidget(wandButton);
    selectionRow->addWidget(deselectButton);
    panelLayout->addLayout(selectionRow);
    auto* selectionLabel = new QLabel(panel);
    selectionLabel->setObjectName(QStringLiteral("selectionLabel"));
    panelLayout->addWidget(selectionLabel);

    auto* imageHeader = new QLabel(QStringLiteral("Image"), panel);
    panelLayout->addWidget(imageHeader);

    const QImage image = activeView() ? activeView()->image() : QImage();
    auto* imageWidthSpin = new QSpinBox(panel);
    auto* imageHeightSpin = new QSpinBox(panel);
    imageWidthSpin->setRange(1, 32767);
    imageHeightSpin->setRange(1, 32767);
    if (!image.isNull()) {
        imageWidthSpin->setValue(image.width());
        imageHeightSpin->setValue(image.height());
    }
    auto* resizeCombo = new QComboBox(panel);
    resizeCombo->addItem(QStringLiteral("Nearest"), QStringLiteral("nearest"));
    resizeCombo->addItem(QStringLiteral("Bilinear"), QStringLiteral("bilinear"));
    resizeCombo->addItem(QStringLiteral("Bicubic"), QStringLiteral("bicubic"));
    auto* imageSizeRow = new QHBoxLayout();
    imageSizeRow->addWidget(imageWidthSpin);
    imageSizeRow->addWidget(imageHeightSpin);
    imageSizeRow->addWidget(resizeCombo);
    panelLayout->addLayout(imageSizeRow);
    auto* applyImageSizeButton = new QPushButton(QStringLiteral("Apply Image Size"), panel);
    panelLayout->addWidget(applyImageSizeButton);

    auto* canvasWidthSpin = new QSpinBox(panel);
    auto* canvasHeightSpin = new QSpinBox(panel);
    canvasWidthSpin->setRange(1, 32767);
    canvasHeightSpin->setRange(1, 32767);
    if (!image.isNull()) {
        canvasWidthSpin->setValue(image.width());
        canvasHeightSpin->setValue(image.height());
    }
    auto* anchorCombo = new QComboBox(panel);
    anchorCombo->addItem(QStringLiteral("Top Left"), QStringLiteral("top-left"));
    anchorCombo->addItem(QStringLiteral("Top Center"), QStringLiteral("top-center"));
    anchorCombo->addItem(QStringLiteral("Top Right"), QStringLiteral("top-right"));
    anchorCombo->addItem(QStringLiteral("Center Left"), QStringLiteral("center-left"));
    anchorCombo->addItem(QStringLiteral("Center"), QStringLiteral("center"));
    anchorCombo->addItem(QStringLiteral("Center Right"), QStringLiteral("center-right"));
    anchorCombo->addItem(QStringLiteral("Bottom Left"), QStringLiteral("bottom-left"));
    anchorCombo->addItem(QStringLiteral("Bottom Center"), QStringLiteral("bottom-center"));
    anchorCombo->addItem(QStringLiteral("Bottom Right"), QStringLiteral("bottom-right"));
    auto* canvasSizeRow = new QHBoxLayout();
    canvasSizeRow->addWidget(canvasWidthSpin);
    canvasSizeRow->addWidget(canvasHeightSpin);
    canvasSizeRow->addWidget(anchorCombo);
    panelLayout->addLayout(canvasSizeRow);
    auto* applyCanvasSizeButton = new QPushButton(QStringLiteral("Apply Canvas Size"), panel);
    panelLayout->addWidget(applyCanvasSizeButton);

    auto* rotateCwButton = new QPushButton(QStringLiteral("Rotate 90° CW"), panel);
    auto* rotateCcwButton = new QPushButton(QStringLiteral("Rotate 90° CCW"), panel);
    auto* rotate180Button = new QPushButton(QStringLiteral("Rotate 180°"), panel);
    auto* flipHorizontalButton = new QPushButton(QStringLiteral("Flip Horizontal"), panel);
    auto* flipVerticalButton = new QPushButton(QStringLiteral("Flip Vertical"), panel);
    auto* orientationRow = new QHBoxLayout();
    orientationRow->addWidget(rotateCwButton);
    orientationRow->addWidget(rotateCcwButton);
    orientationRow->addWidget(rotate180Button);
    orientationRow->addWidget(flipHorizontalButton);
    orientationRow->addWidget(flipVerticalButton);
    panelLayout->addLayout(orientationRow);

    auto* undoButton = new QPushButton(QStringLiteral("Undo"), panel);
    auto* redoButton = new QPushButton(QStringLiteral("Redo"), panel);
    undoButton->setObjectName(QStringLiteral("dockUndo"));
    redoButton->setObjectName(QStringLiteral("dockRedo"));
    auto* historyRow = new QHBoxLayout();
    historyRow->addWidget(undoButton);
    historyRow->addWidget(redoButton);
    panelLayout->addLayout(historyRow);

    layersDock_->setWidget(panel);
    registerPanel(layersDock_, Qt::RightDockWidgetArea);

    connect(layerList_, &QListWidget::itemChanged, this, [this](QListWidgetItem* item) {
        PictureView* view = activeView();
        if (!view || !layerList_) {
            return;
        }
        view->set_layer_visible(layerList_->row(item), item->checkState() == Qt::Checked);
    });
    connect(addButton, &QPushButton::clicked, this, [this, adjustmentCombo]() {
        PictureView* view = activeView();
        if (!view) {
            return;
        }
        if (view->add_adjustment(adjustmentCombo->currentData().toString())) {
            refresh();
            if (layerList_) {
                layerList_->setCurrentRow(layerList_->count() - 1);
            }
        }
    });
    connect(removeButton, &QPushButton::clicked, this, [this]() {
        PictureView* view = activeView();
        if (!view || !layerList_) {
            return;
        }
        view->remove_layer(layerList_->currentRow());
        refresh();
    });
    connect(applyFilterButton, &QPushButton::clicked, this, [this, filterCombo]() {
        PictureView* view = activeView();
        if (view && view->apply_filter(filterCombo->currentData().toString())) {
            refresh();
        }
    });
    connect(selectAllButton, &QPushButton::clicked, this, [this]() {
        if (PictureView* view = activeView()) {
            view->select_all();
        }
    });
    connect(wandButton, &QPushButton::clicked, this, [this]() {
        PictureView* view = activeView();
        if (!view) {
            return;
        }
        const QImage current = view->image();
        if (!current.isNull()) {
            view->magic_wand(current.width() / 2, current.height() / 2, 32);
        }
    });
    connect(deselectButton, &QPushButton::clicked, this, [this]() {
        if (PictureView* view = activeView()) {
            view->deselect();
        }
    });
    connect(applyImageSizeButton, &QPushButton::clicked, this,
            [this, imageWidthSpin, imageHeightSpin, resizeCombo]() {
                PictureView* view = activeView();
                if (view && view->resize_image(resizeCombo->currentData().toString(),
                                               imageWidthSpin->value(), imageHeightSpin->value())) {
                    refresh();
                }
            });
    connect(applyCanvasSizeButton, &QPushButton::clicked, this,
            [this, canvasWidthSpin, canvasHeightSpin, anchorCombo]() {
                PictureView* view = activeView();
                if (view && view->resize_canvas(anchorCombo->currentData().toString(),
                                                canvasWidthSpin->value(),
                                                canvasHeightSpin->value())) {
                    refresh();
                }
            });
    connect(rotateCwButton, &QPushButton::clicked, this, [this]() {
        if (PictureView* view = activeView(); view && view->rotate_doc(1)) {
            refresh();
        }
    });
    connect(rotateCcwButton, &QPushButton::clicked, this, [this]() {
        if (PictureView* view = activeView(); view && view->rotate_doc(3)) {
            refresh();
        }
    });
    connect(rotate180Button, &QPushButton::clicked, this, [this]() {
        if (PictureView* view = activeView(); view && view->rotate_doc(2)) {
            refresh();
        }
    });
    connect(flipHorizontalButton, &QPushButton::clicked, this, [this]() {
        if (PictureView* view = activeView(); view && view->flip_doc(true)) {
            refresh();
        }
    });
    connect(flipVerticalButton, &QPushButton::clicked, this, [this]() {
        if (PictureView* view = activeView(); view && view->flip_doc(false)) {
            refresh();
        }
    });
    connect(undoButton, &QPushButton::clicked, this, [this]() {
        if (PictureView* view = activeView(); view && view->undo()) {
            refresh();
        }
    });
    connect(redoButton, &QPushButton::clicked, this, [this]() {
        if (PictureView* view = activeView(); view && view->redo()) {
            refresh();
        }
    });
}

void PicturaMainWindow::buildStatusBar()
{
    QStatusBar* bar = statusBar();
    zoomLabel_ = new QLabel(QStringLiteral("100%"), bar);
    sizeLabel_ = new QLabel(QStringLiteral("—"), bar);
    hintLabel_ = new QLabel(QStringLiteral("Ready"), bar);
    bar->addWidget(zoomLabel_);
    bar->addWidget(sizeLabel_);
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

    registry_->setHandler(command_ids::WindowPanelsLayers, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsLayers);
        if (layersDock_ && action) {
            layersDock_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsLayers,
                                  [this]() { return layersDock_ && layersDock_->isVisible(); });

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
}

void PicturaMainWindow::applyBrightness(int level)
{
    brightnessLevel_ = Theme::clampLevel(level);
    Theme::apply(brightnessLevel_);
}

} // namespace pictura
