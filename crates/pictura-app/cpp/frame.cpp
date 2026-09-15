#include "frame.h"

#include "commands.h"
#include "image_view.h"
#include "session.h"
#include "theme.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

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
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QWidget>

namespace pictura {

namespace {

constexpr int kCanvasColorCount = 4;
const QColor kCanvasColors[kCanvasColorCount] = {
    QColor(Qt::darkGray), QColor(Qt::gray), QColor(Qt::black), QColor(Qt::white)};

} // namespace

PicturaMainWindow::PicturaMainWindow(PictureView* view, QWidget* parent)
    : QMainWindow(parent)
    , view_(view)
{
    imageView_ = new ImageView(this);
    registry_ = new CommandRegistry(this);
    addDefaultCommands(*registry_);
    registerHandlers();
    buildMenus();
    buildPanels();
    buildStatusBar();
    setCentralWidget(imageView_);
    resize(1100, 700);
    setWindowTitle(QStringLiteral("Kooka Pictura"));
    setDockOptions(QMainWindow::AnimatedDocks | QMainWindow::AllowTabbedDocks);

    const SessionState session = pictura::loadSession();
    setBrightnessLevel(session.brightnessLevel);
    if (!session.layout.isEmpty()) {
        restoreState(session.layout);
    }

    auto* hidePanels = new QShortcut(QKeySequence(Qt::Key_Tab), this);
    connect(hidePanels, &QShortcut::activated, this, [this]() { setPanelsHidden(!panelsHidden_); });
    auto* hidePanelsBack = new QShortcut(QKeySequence(Qt::SHIFT | Qt::Key_Tab), this);
    connect(hidePanelsBack, &QShortcut::activated, this, [this]() { setPanelsHidden(!panelsHidden_); });
    auto* cycleCanvas = new QShortcut(QKeySequence(Qt::Key_Space, Qt::Key_F), this);
    connect(cycleCanvas, &QShortcut::activated, this, [this]() { cycleCanvasColor(true); });
    auto* brightnessDown = new QShortcut(QKeySequence(Qt::SHIFT | Qt::Key_F1), this);
    connect(brightnessDown, &QShortcut::activated, this,
            [this]() { setBrightnessLevel(brightnessLevel_ - 1); });
    auto* brightnessUp = new QShortcut(QKeySequence(Qt::SHIFT | Qt::Key_F2), this);
    connect(brightnessUp, &QShortcut::activated, this,
            [this]() { setBrightnessLevel(brightnessLevel_ + 1); });

    refresh();
    connect(view_, &PictureView::changed, this, &PicturaMainWindow::refresh);
    connect(imageView_, &ImageView::zoomChanged, this, [this](double) { updateStatus(); });
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

void PicturaMainWindow::setBrightnessLevel(int level)
{
    applyBrightness(level);
}

void PicturaMainWindow::setScreenMode(ScreenMode mode)
{
    screenMode_ = mode;
    const QList<QDockWidget*> docks = findChildren<QDockWidget*>();
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
        imageView_->setCanvasColor(kCanvasColors[canvasColorIndex_]);
        break;
    }
    case ScreenMode::FullWithMenuBar:
        setWindowState(Qt::WindowFullScreen);
        menuBar()->setVisible(true);
        statusBar()->setVisible(false);
        for (QDockWidget* dock : docks) {
            dock->setVisible(false);
        }
        imageView_->setCanvasColor(QColor(128, 128, 128));
        break;
    case ScreenMode::Full:
        setWindowState(Qt::WindowFullScreen);
        menuBar()->setVisible(false);
        statusBar()->setVisible(false);
        for (QDockWidget* dock : docks) {
            dock->setVisible(false);
        }
        imageView_->setCanvasColor(Qt::black);
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
    imageView_->setCanvasColor(kCanvasColors[canvasColorIndex_]);
}

void PicturaMainWindow::setPanelsHidden(bool hidden)
{
    panelsHidden_ = hidden;
    const QList<QDockWidget*> docks = findChildren<QDockWidget*>();
    for (QDockWidget* dock : docks) {
        dock->setVisible(!hidden);
    }
}

void PicturaMainWindow::refresh()
{
    if (view_->has_document()) {
        imageView_->replaceImage(view_->image());
    } else {
        imageView_->setImage(view_->image());
    }

    if (layerList_) {
        QSignalBlocker blocker(layerList_);
        layerList_->clear();
        const int count = view_->layer_count();
        for (int i = 0; i < count; ++i) {
            auto* item = new QListWidgetItem(QStringLiteral("%1  [%2]")
                                                 .arg(view_->layer_name(i), view_->layer_kind(i)));
            item->setFlags(item->flags() | Qt::ItemIsUserCheckable);
            item->setCheckState(view_->layer_visible(i) ? Qt::Checked : Qt::Unchecked);
            layerList_->addItem(item);
        }
    }

    if (auto* selectionLabel = findChild<QLabel*>(QStringLiteral("selectionLabel"))) {
        selectionLabel->setText(
            QStringLiteral("Selection: %1 px").arg(view_->selection_count()));
    }
    if (auto* undo = findChild<QPushButton*>(QStringLiteral("dockUndo"))) {
        undo->setEnabled(view_->can_undo());
    }
    if (auto* redo = findChild<QPushButton*>(QStringLiteral("dockRedo"))) {
        redo->setEnabled(view_->can_redo());
    }

    updateStatus();
    registry_->refresh();
}

void PicturaMainWindow::saveSession()
{
    SessionState state;
    state.layout = saveState();
    state.brightnessLevel = brightnessLevel_;
    state.schemaVersion = 1;
    pictura::saveSession(state);
}

void PicturaMainWindow::closeEvent(QCloseEvent* event)
{
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

    const QImage image = view_->image();
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
        view_->set_layer_visible(layerList_->row(item), item->checkState() == Qt::Checked);
    });
    connect(addButton, &QPushButton::clicked, this, [this, adjustmentCombo]() {
        if (view_->add_adjustment(adjustmentCombo->currentData().toString())) {
            refresh();
            layerList_->setCurrentRow(layerList_->count() - 1);
        }
    });
    connect(removeButton, &QPushButton::clicked, this, [this]() {
        view_->remove_layer(layerList_->currentRow());
        refresh();
    });
    connect(applyFilterButton, &QPushButton::clicked, this, [this, filterCombo]() {
        if (view_->apply_filter(filterCombo->currentData().toString())) {
            refresh();
        }
    });
    connect(selectAllButton, &QPushButton::clicked, this, [this]() { view_->select_all(); });
    connect(wandButton, &QPushButton::clicked, this, [this]() {
        const QImage current = view_->image();
        if (!current.isNull()) {
            view_->magic_wand(current.width() / 2, current.height() / 2, 32);
        }
    });
    connect(deselectButton, &QPushButton::clicked, this, [this]() { view_->deselect(); });
    connect(applyImageSizeButton, &QPushButton::clicked, this,
            [this, imageWidthSpin, imageHeightSpin, resizeCombo]() {
                if (view_->resize_image(resizeCombo->currentData().toString(),
                                        imageWidthSpin->value(), imageHeightSpin->value())) {
                    refresh();
                }
            });
    connect(applyCanvasSizeButton, &QPushButton::clicked, this,
            [this, canvasWidthSpin, canvasHeightSpin, anchorCombo]() {
                if (view_->resize_canvas(anchorCombo->currentData().toString(),
                                         canvasWidthSpin->value(), canvasHeightSpin->value())) {
                    refresh();
                }
            });
    connect(rotateCwButton, &QPushButton::clicked, this, [this]() {
        if (view_->rotate_doc(1)) {
            refresh();
        }
    });
    connect(rotateCcwButton, &QPushButton::clicked, this, [this]() {
        if (view_->rotate_doc(3)) {
            refresh();
        }
    });
    connect(rotate180Button, &QPushButton::clicked, this, [this]() {
        if (view_->rotate_doc(2)) {
            refresh();
        }
    });
    connect(flipHorizontalButton, &QPushButton::clicked, this, [this]() {
        if (view_->flip_doc(true)) {
            refresh();
        }
    });
    connect(flipVerticalButton, &QPushButton::clicked, this, [this]() {
        if (view_->flip_doc(false)) {
            refresh();
        }
    });
    connect(undoButton, &QPushButton::clicked, this, [this]() {
        if (view_->undo()) {
            refresh();
        }
    });
    connect(redoButton, &QPushButton::clicked, this, [this]() {
        if (view_->redo()) {
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
    registry_->setHandler(command_ids::FileOpen, [this]() {
        const QString path = QFileDialog::getOpenFileName(
            this, tr("Open"), QString(), QStringLiteral("Photoshop files (*.psd *.psb)"));
        if (path.isEmpty()) {
            return;
        }
        view_->open(path);
        imageView_->setImage(view_->image());
        refresh();
    });
    registry_->setEnabledProvider(command_ids::FileOpen, []() { return true; });

    registry_->setHandler(command_ids::EditUndo, [this]() {
        const bool ok = view_->can_undo() ? view_->undo() : view_->redo();
        if (ok) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::EditUndo,
                                  [this]() { return view_->can_undo() || view_->can_redo(); });
    registry_->setLabelProvider(command_ids::EditUndo, [this]() {
        return view_->can_undo() ? QStringLiteral("Undo") : QStringLiteral("Redo");
    });

    registry_->setHandler(command_ids::EditRedo, [this]() {
        if (view_->redo()) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::EditRedo, [this]() { return view_->can_redo(); });

    registry_->setHandler(command_ids::EditStepBackward, [this]() {
        if (view_->undo()) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::EditStepBackward,
                                  [this]() { return view_->can_undo(); });

    registry_->setHandler(command_ids::EditStepForward, [this]() {
        if (view_->redo()) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::EditStepForward,
                                  [this]() { return view_->can_redo(); });

    registry_->setHandler(command_ids::ImageRotate90Cw, [this]() {
        if (view_->rotate_doc(1)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::ImageRotate90Ccw, [this]() {
        if (view_->rotate_doc(3)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::ImageRotate180, [this]() {
        if (view_->rotate_doc(2)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::ImageFlipHorizontal, [this]() {
        if (view_->flip_doc(true)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::ImageFlipVertical, [this]() {
        if (view_->flip_doc(false)) {
            refresh();
        }
    });
    for (const char* id : {command_ids::ImageRotate90Cw, command_ids::ImageRotate90Ccw,
                           command_ids::ImageRotate180, command_ids::ImageFlipHorizontal,
                           command_ids::ImageFlipVertical}) {
        registry_->setEnabledProvider(id, [this]() { return view_->has_document(); });
    }

    registry_->setHandler(command_ids::SelectAll, [this]() { view_->select_all(); });
    registry_->setEnabledProvider(command_ids::SelectAll,
                                  [this]() { return view_->has_document(); });
    registry_->setHandler(command_ids::SelectDeselect, [this]() { view_->deselect(); });
    registry_->setEnabledProvider(command_ids::SelectDeselect,
                                  [this]() { return view_->has_document(); });

    registry_->setHandler(command_ids::ViewZoomIn, [this]() { imageView_->zoomIn(); });
    registry_->setHandler(command_ids::ViewZoomOut, [this]() { imageView_->zoomOut(); });
    registry_->setHandler(command_ids::ViewFitOnScreen, [this]() { imageView_->fitOnScreen(); });
    registry_->setHandler(command_ids::ViewActualPixels, [this]() { imageView_->actualPixels(); });

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
    if (zoomLabel_) {
        zoomLabel_->setText(QStringLiteral("%1%").arg(qRound(imageView_->zoom() * 100.0)));
    }
    if (sizeLabel_) {
        QString text = QStringLiteral("—");
        if (view_->has_document()) {
            const QImage image = view_->image();
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
