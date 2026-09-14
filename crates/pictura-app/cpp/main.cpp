#include <QtCore/QCoreApplication>
#include <QtCore/QDebug>
#include <QtCore/QSignalBlocker>
#include <QtCore/QSet>
#include <QtCore/QTimer>
#include <QtGui/QImage>
#include <QtGui/QKeySequence>
#include <QtGui/QMouseEvent>
#include <QtGui/QPainter>
#include <QtGui/QShortcut>
#include <QtGui/QWheelEvent>
#include <QtWidgets/QApplication>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDockWidget>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QListWidget>
#include <QtWidgets/QMainWindow>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSpinBox>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QWidget>

#include <cmath>
#include <cstdint>
#include <cstdio>

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include "interop.h"

class ImageView : public QWidget
{
    Q_OBJECT

public:
    explicit ImageView(QWidget* parent = nullptr)
        : QWidget(parent)
    {
        setWindowTitle(QStringLiteral("Kooka Pictura - M0 walking skeleton"));
        setMinimumSize(640, 480);
    }

    void setImage(const QImage& image)
    {
        image_ = image;
        zoom_ = 1.0;
        offset_ = QPointF(0.0, 0.0);
        update();
    }

    // Zoom about a cursor position so the point under the cursor stays put.
    void zoomAt(const QPointF& cursor, int angleDelta)
    {
        const double factor = std::pow(1.0015, angleDelta);
        offset_ = cursor - (cursor - offset_) * factor;
        zoom_ *= factor;
        update();
    }

    void panBy(const QPointF& delta)
    {
        offset_ += delta;
        update();
    }

    double zoom() const { return zoom_; }
    QPointF offset() const { return offset_; }

protected:
    void paintEvent(QPaintEvent*) override
    {
        QPainter painter(this);
        painter.fillRect(rect(), Qt::darkGray);
        if (image_.isNull()) {
            return;
        }
        painter.translate(offset_);
        painter.scale(zoom_, zoom_);
        painter.drawImage(QPointF(0.0, 0.0), image_);
    }

    void wheelEvent(QWheelEvent* event) override
    {
        zoomAt(event->position(), event->angleDelta().y());
    }

    void mousePressEvent(QMouseEvent* event) override
    {
        if (event->button() == Qt::LeftButton) {
            last_ = event->position();
        }
    }

    void mouseMoveEvent(QMouseEvent* event) override
    {
        if (event->buttons() & Qt::LeftButton) {
            panBy(event->position() - last_);
            last_ = event->position();
        }
    }

private:
    QImage image_;
    double zoom_ = 1.0;
    QPointF offset_;
    QPointF last_;
};

int main(int argc, char* argv[])
{
    QApplication app(argc, argv);

    // Surface Qt's own diagnostics (QRhi logs through qWarning) on stderr so
    // the interop probe can capture them under xvfb/offscreen.
    qInstallMessageHandler([](QtMsgType type, const QMessageLogContext&, const QString& message) {
        std::fprintf(stderr, "qt[%d]: %s\n", static_cast<int>(type), message.toLocal8Bit().constData());
        std::fflush(stderr);
    });

    const QStringList args = app.arguments();
    bool selfTest = false;
    bool interopProbe = false;
    QString psdPath;
    for (int i = 1; i < args.size(); ++i) {
        if (args.at(i) == QStringLiteral("--self-test")) {
            selfTest = true;
        } else if (args.at(i) == QStringLiteral("--interop-probe")) {
            interopProbe = true;
        } else if (!args.at(i).startsWith(QLatin1Char('-'))) {
            psdPath = args.at(i);
        }
    }

    pictura::PictureView view;

    if (interopProbe) {
        const bool prepared = view.gpu_interop_prepare();
        if (!prepared) {
            std::fprintf(stderr, "pictura interop-probe: no Vulkan device\n");
            return 0;
        }
        const std::int32_t result = pictura_try_qrhi_import(view.gpu_vk_instance(),
                                                            view.gpu_vk_physical_device(),
                                                            view.gpu_vk_device(),
                                                            view.gpu_vk_queue_family(),
                                                            view.gpu_vk_image(),
                                                            view.gpu_image_width(),
                                                            view.gpu_image_height());
        std::fprintf(stderr, "pictura interop-probe: qrhi_import=%d\n", result);
        std::fflush(stderr);
        return 0;
    }

    const bool codecLoaded = view.open(psdPath);
    // M0.5: fall back to the offscreen GPU demo only when no document loaded
    // (a loaded PSD must keep its composited image). 0 = no GPU,
    // 1 = rendered non-blank, 2 = rendered blank.
    int gpu = 0;
    if (!codecLoaded) {
        gpu = view.render_gpu();
    }
    const QImage image = view.image();

    QMainWindow mainWindow;
    ImageView* window = new ImageView(&mainWindow);
    window->setImage(image);
    mainWindow.setCentralWidget(window);
    mainWindow.resize(1100, 700);

    // Layer dock: visibility checkboxes, adjustment add, remove.
    auto* dock = new QDockWidget(QStringLiteral("Layers"), &mainWindow);
    auto* panel = new QWidget(dock);
    auto* panelLayout = new QVBoxLayout(panel);
    auto* layerList = new QListWidget(panel);
    layerList->setSelectionMode(QAbstractItemView::SingleSelection);
    panelLayout->addWidget(layerList, 1);

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

    // Selection controls: the active selection masks any adjustment added next.
    auto* selectAllButton = new QPushButton(QStringLiteral("Select all"), panel);
    auto* wandButton = new QPushButton(QStringLiteral("Magic wand (center)"), panel);
    auto* deselectButton = new QPushButton(QStringLiteral("Deselect"), panel);
    auto* selectionRow = new QHBoxLayout();
    selectionRow->addWidget(selectAllButton);
    selectionRow->addWidget(wandButton);
    selectionRow->addWidget(deselectButton);
    panelLayout->addLayout(selectionRow);
    auto* selectionLabel = new QLabel(panel);
    panelLayout->addWidget(selectionLabel);

    // Image section: resample pixels, grow the canvas, rotate and flip.
    auto* imageHeader = new QLabel(QStringLiteral("Image"), panel);
    panelLayout->addWidget(imageHeader);

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

    // History: undo/redo step through the states captured by mutating ops.
    auto* undoButton = new QPushButton(QStringLiteral("Undo"), panel);
    auto* redoButton = new QPushButton(QStringLiteral("Redo"), panel);
    auto* historyRow = new QHBoxLayout();
    historyRow->addWidget(undoButton);
    historyRow->addWidget(redoButton);
    panelLayout->addLayout(historyRow);

    dock->setWidget(panel);
    mainWindow.addDockWidget(Qt::RightDockWidgetArea, dock);

    // Rebuild the list and re-display the composite after any layer change.
    auto refresh = [&]() {
        QSignalBlocker blocker(layerList);
        layerList->clear();
        const int count = view.layer_count();
        for (int i = 0; i < count; ++i) {
            auto* item = new QListWidgetItem(QStringLiteral("%1  [%2]")
                                                 .arg(view.layer_name(i), view.layer_kind(i)));
            item->setFlags(item->flags() | Qt::ItemIsUserCheckable);
            item->setCheckState(view.layer_visible(i) ? Qt::Checked : Qt::Unchecked);
            layerList->addItem(item);
        }
        window->setImage(view.image());
        selectionLabel->setText(
            QStringLiteral("Selection: %1 px").arg(view.selection_count()));
        undoButton->setEnabled(view.can_undo());
        redoButton->setEnabled(view.can_redo());
    };

    QObject::connect(layerList, &QListWidget::itemChanged, &mainWindow, [&](QListWidgetItem* item) {
        view.set_layer_visible(layerList->row(item), item->checkState() == Qt::Checked);
    });
    QObject::connect(addButton, &QPushButton::clicked, &mainWindow, [&]() {
        if (view.add_adjustment(adjustmentCombo->currentData().toString())) {
            refresh();
            layerList->setCurrentRow(layerList->count() - 1);
        }
    });
    QObject::connect(removeButton, &QPushButton::clicked, &mainWindow, [&]() {
        view.remove_layer(layerList->currentRow());
        refresh();
    });
    QObject::connect(applyFilterButton, &QPushButton::clicked, &mainWindow, [&]() {
        if (view.apply_filter(filterCombo->currentData().toString())) {
            refresh();
        }
    });
    QObject::connect(selectAllButton, &QPushButton::clicked, &mainWindow, [&]() {
        view.select_all();
    });
    QObject::connect(wandButton, &QPushButton::clicked, &mainWindow, [&]() {
        const QImage current = view.image();
        if (!current.isNull()) {
            view.magic_wand(current.width() / 2, current.height() / 2, 32);
        }
    });
    QObject::connect(deselectButton, &QPushButton::clicked, &mainWindow, [&]() {
        view.deselect();
    });
    QObject::connect(applyImageSizeButton, &QPushButton::clicked, &mainWindow, [&]() {
        if (view.resize_image(resizeCombo->currentData().toString(),
                              imageWidthSpin->value(),
                              imageHeightSpin->value())) {
            refresh();
        }
    });
    QObject::connect(applyCanvasSizeButton, &QPushButton::clicked, &mainWindow, [&]() {
        if (view.resize_canvas(anchorCombo->currentData().toString(),
                               canvasWidthSpin->value(),
                               canvasHeightSpin->value())) {
            refresh();
        }
    });
    QObject::connect(rotateCwButton, &QPushButton::clicked, &mainWindow, [&]() {
        if (view.rotate_doc(1)) {
            refresh();
        }
    });
    QObject::connect(rotateCcwButton, &QPushButton::clicked, &mainWindow, [&]() {
        if (view.rotate_doc(3)) {
            refresh();
        }
    });
    QObject::connect(rotate180Button, &QPushButton::clicked, &mainWindow, [&]() {
        if (view.rotate_doc(2)) {
            refresh();
        }
    });
    QObject::connect(flipHorizontalButton, &QPushButton::clicked, &mainWindow, [&]() {
        if (view.flip_doc(true)) {
            refresh();
        }
    });
    QObject::connect(flipVerticalButton, &QPushButton::clicked, &mainWindow, [&]() {
        if (view.flip_doc(false)) {
            refresh();
        }
    });
    const auto doUndo = [&]() {
        if (view.undo()) {
            refresh();
        }
    };
    const auto doRedo = [&]() {
        if (view.redo()) {
            refresh();
        }
    };
    QObject::connect(undoButton, &QPushButton::clicked, &mainWindow, doUndo);
    QObject::connect(redoButton, &QPushButton::clicked, &mainWindow, doRedo);
    auto* undoShortcut = new QShortcut(QKeySequence(Qt::CTRL | Qt::Key_Z), &mainWindow);
    QObject::connect(undoShortcut, &QShortcut::activated, &mainWindow, doUndo);
    auto* redoShortcut = new QShortcut(QKeySequence(Qt::CTRL | Qt::Key_Y), &mainWindow);
    QObject::connect(redoShortcut, &QShortcut::activated, &mainWindow, doRedo);
    QObject::connect(&view, &pictura::PictureView::changed, &mainWindow, [&]() { refresh(); });

    refresh();
    mainWindow.show();

    if (selfTest) {
        std::fprintf(stderr,
                     "pictura self-test: image=%dx%d codec_loaded=%d gpu=%d\n",
                     image.width(),
                     image.height(),
                     codecLoaded ? 1 : 0,
                     gpu);
        std::fflush(stderr);
        if (image.isNull()) {
            std::fprintf(stderr, "pictura self-test: FAIL: null image\n");
            return 2;
        }
        if (gpu == 2) {
            std::fprintf(stderr, "pictura self-test: FAIL: GPU render was blank\n");
            return 4;
        }
        if (!codecLoaded) {
            // The generated image (GPU or CPU fallback) must not be blank.
            QSet<QRgb> seen;
            for (int gy = 0; gy < 8; ++gy) {
                for (int gx = 0; gx < 8; ++gx) {
                    const int x = image.width() * gx / 8 + image.width() / 16;
                    const int y = image.height() * gy / 8 + image.height() / 16;
                    seen.insert(image.pixel(x, y));
                }
            }
            std::fprintf(stderr,
                         "pictura self-test: nonblank=%d distinct=%d\n",
                         seen.size() >= 2 ? 1 : 0,
                         seen.size());
            std::fflush(stderr);
            if (seen.size() < 2) {
                std::fprintf(stderr, "pictura self-test: FAIL: blank render\n");
                return 5;
            }
        }
        if (codecLoaded) {
            // two_layers.psd is an 8x8 layer stack: a red top-left quadrant and
            // a blue bottom-right quadrant; the other quadrants are uncovered.
            QSet<QRgb> seen;
            for (int y = 0; y < image.height(); ++y) {
                for (int x = 0; x < image.width(); ++x) {
                    seen.insert(image.pixel(x, y));
                }
            }
            std::fprintf(stderr, "pictura self-test: layered distinct=%d\n", seen.size());
            std::fflush(stderr);
            if (seen.size() < 2) {
                std::fprintf(stderr, "pictura self-test: FAIL: blank composition\n");
                return 6;
            }
            if (image.width() >= 8 && image.height() >= 8) {
                const QRgb tl = image.pixel(2, 2);
                const QRgb br = image.pixel(6, 6);
                const QRgb tr = image.pixel(6, 2);
                const QRgb bl = image.pixel(2, 6);
                std::fprintf(stderr,
                             "pictura self-test: tl=(%d,%d,%d,a%d) br=(%d,%d,%d,a%d) "
                             "tr_a=%d bl_a=%d\n",
                             qRed(tl),
                             qGreen(tl),
                             qBlue(tl),
                             qAlpha(tl),
                             qRed(br),
                             qGreen(br),
                             qBlue(br),
                             qAlpha(br),
                             qAlpha(tr),
                             qAlpha(bl));
                std::fflush(stderr);
                const bool red = qRed(tl) > 200 && qGreen(tl) < 60 && qBlue(tl) < 60
                                 && qAlpha(tl) == 255;
                const bool blue = qBlue(br) > 200 && qRed(br) < 60 && qGreen(br) < 60
                                  && qAlpha(br) == 255;
                if (!red || !blue) {
                    std::fprintf(stderr, "pictura self-test: FAIL: composited quadrants wrong\n");
                    return 7;
                }
                // Uncovered quadrants must be transparent: this proves the layer
                // stack was composited, not the opaque embedded PSD composite.
                if (qAlpha(tr) != 0 || qAlpha(bl) != 0) {
                    std::fprintf(stderr, "pictura self-test: FAIL: layer stack not composited\n");
                    return 8;
                }
            }

            // M5-C2: a wand selection must confine an adjustment to the
            // selected quadrant. Wand the red top-left, Invert it, and require
            // the blue bottom-right to be untouched. Clean up afterwards so the
            // full-frame checks below see the original stack.
            const bool wand = view.magic_wand(2, 2, 10);
            const bool hasSelection = view.has_selection();
            const int selectedPx = view.selection_count();
            std::fprintf(stderr,
                         "pictura self-test: magic_wand=%d has_selection=%d selected_px=%d\n",
                         wand ? 1 : 0,
                         hasSelection ? 1 : 0,
                         selectedPx);
            std::fflush(stderr);
            if (!wand || !hasSelection || selectedPx <= 0
                || selectedPx >= image.width() * image.height()) {
                std::fprintf(stderr, "pictura self-test: FAIL: wand selection wrong\n");
                return 14;
            }
            const bool maskedAdded = view.add_adjustment(QStringLiteral("invert"));
            const QImage masked = view.image();
            const QRgb mtl = masked.pixel(2, 2);
            const QRgb mbr = masked.pixel(6, 6);
            std::fprintf(stderr,
                         "pictura self-test: masked_adjustment=%d tl=(%d,%d,%d,a%d) "
                         "br=(%d,%d,%d,a%d)\n",
                         maskedAdded ? 1 : 0,
                         qRed(mtl),
                         qGreen(mtl),
                         qBlue(mtl),
                         qAlpha(mtl),
                         qRed(mbr),
                         qGreen(mbr),
                         qBlue(mbr),
                         qAlpha(mbr));
            std::fflush(stderr);
            const bool maskedCyan = qRed(mtl) < 60 && qGreen(mtl) > 200 && qBlue(mtl) > 200;
            const bool maskedBlue = qBlue(mbr) > 200 && qRed(mbr) < 60 && qGreen(mbr) < 60;
            if (!maskedAdded || !maskedCyan || !maskedBlue) {
                std::fprintf(stderr, "pictura self-test: FAIL: masked adjustment not confined\n");
                return 15;
            }
            view.remove_layer(view.layer_count() - 1);
            view.deselect();
            if (view.has_selection() || view.selection_count() != 0) {
                std::fprintf(stderr, "pictura self-test: FAIL: deselect left a selection\n");
                return 16;
            }

            // M4-C: add an Invert adjustment layer over the stack and verify the
            // composite changed as expected (red -> cyan, blue -> yellow).
            const QImage beforeAdjust = view.image();
            const bool added = view.add_adjustment(QStringLiteral("invert"));
            const QImage adjusted = view.image();
            const int layerCount = view.layer_count();
            std::fprintf(stderr,
                         "pictura self-test: add_adjustment(invert)=%d layers=%d last_kind=%s\n",
                         added ? 1 : 0,
                         layerCount,
                         view.layer_kind(layerCount - 1).toLocal8Bit().constData());
            std::fflush(stderr);
            if (!added || layerCount != 3
                || view.layer_kind(layerCount - 1) != QStringLiteral("adjustment")) {
                std::fprintf(stderr, "pictura self-test: FAIL: invert adjustment not added\n");
                return 9;
            }
            const QRgb atl = adjusted.pixel(2, 2);
            const QRgb abr = adjusted.pixel(6, 6);
            std::fprintf(stderr,
                         "pictura self-test: adjusted tl=(%d,%d,%d,a%d) br=(%d,%d,%d,a%d)\n",
                         qRed(atl),
                         qGreen(atl),
                         qBlue(atl),
                         qAlpha(atl),
                         qRed(abr),
                         qGreen(abr),
                         qBlue(abr),
                         qAlpha(abr));
            std::fflush(stderr);
            const bool cyan = qRed(atl) < 60 && qGreen(atl) > 200 && qBlue(atl) > 200
                              && qAlpha(atl) == 255;
            const bool yellow = qRed(abr) > 200 && qGreen(abr) > 200 && qBlue(abr) < 60
                                && qAlpha(abr) == 255;
            if (!cyan || !yellow) {
                std::fprintf(stderr, "pictura self-test: FAIL: invert composite wrong\n");
                return 10;
            }
            if (beforeAdjust == adjusted) {
                std::fprintf(stderr, "pictura self-test: FAIL: adjustment did not change image\n");
                return 11;
            }

            // Toggle the bottom pixel layer's visibility: the output must change.
            if (!view.layer_visible(0)) {
                std::fprintf(stderr, "pictura self-test: FAIL: base layer not visible\n");
                return 12;
            }
            view.set_layer_visible(0, false);
            const QImage hidden = view.image();
            bool differs = false;
            for (int y = 0; y < hidden.height() && !differs; ++y) {
                for (int x = 0; x < hidden.width(); ++x) {
                    if (hidden.pixel(x, y) != adjusted.pixel(x, y)) {
                        differs = true;
                        break;
                    }
                }
            }
            std::fprintf(stderr, "pictura self-test: visibility_change=%d\n", differs ? 1 : 0);
            std::fflush(stderr);
            if (!differs || view.layer_visible(0)) {
                std::fprintf(stderr,
                             "pictura self-test: FAIL: visibility toggle did not change output\n");
                return 13;
            }

            // M6-C: a filter must confine its change to the active selection.
            // The topmost pixel layer is the bottom-right blue quadrant; wand
            // that quadrant, apply the fixed-seed Add Noise, and require the
            // selected quadrant to change while the rest is bit-identical.
            view.deselect();
            const bool filterWand = view.magic_wand(6, 6, 10);
            const bool filterSelected = view.has_selection();
            const int filterSelectedPx = view.selection_count();
            std::fprintf(stderr,
                         "pictura self-test: filter_wand=%d selected_px=%d\n",
                         filterWand ? 1 : 0,
                         filterSelectedPx);
            std::fflush(stderr);
            if (!filterWand || !filterSelected || filterSelectedPx <= 0
                || filterSelectedPx >= view.image().width() * view.image().height()) {
                std::fprintf(stderr, "pictura self-test: FAIL: filter selection wrong\n");
                return 17;
            }
            const QImage filterBefore = view.image();
            const bool filtered = view.apply_filter(QStringLiteral("add-noise"));
            const QImage filterAfter = view.image();
            int insideChanged = 0;
            int outsideChanged = 0;
            for (int y = 0; y < filterAfter.height(); ++y) {
                for (int x = 0; x < filterAfter.width(); ++x) {
                    const bool inside = x >= 4 && x < 8 && y >= 4 && y < 8;
                    if (filterAfter.pixel(x, y) == filterBefore.pixel(x, y)) {
                        continue;
                    }
                    if (inside) {
                        ++insideChanged;
                    } else {
                        ++outsideChanged;
                    }
                }
            }
            std::fprintf(stderr,
                         "pictura self-test: filter_change=%d changed_inside=%d "
                         "changed_outside=%d\n",
                         filtered ? 1 : 0,
                         insideChanged,
                         outsideChanged);
            std::fflush(stderr);
            if (!filtered || insideChanged == 0 || outsideChanged != 0) {
                std::fprintf(stderr, "pictura self-test: FAIL: filter not confined\n");
                return 18;
            }
            view.deselect();

            // M13: document ops. Earlier checks mutated the stack (hidden
            // layer, active invert, noise), so assert the exact 90 deg CW
            // remap (x,y) -> (7-y,x) on captured pixels, plus that a
            // successful op clears the selection.
            const QImage preRotate = view.image();
            view.select_all();
            const bool rotated = view.rotate_doc(1);
            const QImage rotatedImg = view.image();
            const QRgb rotTr = rotatedImg.pixel(5, 2);
            const QRgb rotBl = rotatedImg.pixel(1, 6);
            std::fprintf(stderr,
                         "pictura self-test: rotate_cw=%d size=%dx%d "
                         "map_tl=%d map_br=%d corner_a=%d sel=%d\n",
                         rotated ? 1 : 0,
                         rotatedImg.width(),
                         rotatedImg.height(),
                         rotTr == preRotate.pixel(2, 2) ? 1 : 0,
                         rotBl == preRotate.pixel(6, 6) ? 1 : 0,
                         qAlpha(rotatedImg.pixel(5, 6)),
                         view.selection_count());
            std::fflush(stderr);
            if (!rotated || rotatedImg.width() != 8 || rotatedImg.height() != 8
                || rotTr != preRotate.pixel(2, 2) || rotBl != preRotate.pixel(6, 6)
                || qAlpha(rotatedImg.pixel(5, 6)) != 0 || view.has_selection()
                || view.selection_count() != 0) {
                std::fprintf(stderr, "pictura self-test: FAIL: rotate cw wrong\n");
                return 19;
            }

            // M13: invalid document ops must be rejected and leave pixels put.
            const bool badRotate = view.rotate_doc(0);
            const bool badRotateClean = view.image() == rotatedImg;
            const bool badResize = view.resize_image(QStringLiteral("bicubic"), 0, 8);
            const bool badResizeClean = view.image() == rotatedImg;
            const bool badCanvas = view.resize_canvas(QStringLiteral("nope"), 10, 10);
            const bool badCanvasClean = view.image() == rotatedImg;
            std::fprintf(stderr,
                         "pictura self-test: reject rotate0=%d resize_w0=%d "
                         "canvas_bad_anchor=%d unchanged=%d\n",
                         badRotate ? 1 : 0,
                         badResize ? 1 : 0,
                         badCanvas ? 1 : 0,
                         badRotateClean && badResizeClean && badCanvasClean ? 1 : 0);
            std::fflush(stderr);
            if (badRotate || badResize || badCanvas || !badRotateClean || !badResizeClean
                || !badCanvasClean) {
                std::fprintf(stderr, "pictura self-test: FAIL: invalid doc op accepted\n");
                return 20;
            }

            // M13: CCW must undo CW bit-exactly, then growing the canvas to
            // 10x12 with the bottom-right anchor maps old (x,y) to (x+2,y+4)
            // and leaves the new top-left area transparent.
            const bool restoredOk = view.rotate_doc(3);
            const QImage restored = view.image();
            if (!restoredOk || restored != preRotate) {
                std::fprintf(stderr, "pictura self-test: FAIL: rotate ccw did not restore\n");
                return 21;
            }
            const bool grown = view.resize_canvas(QStringLiteral("bottom-right"), 10, 12);
            const QImage grownImg = view.image();
            std::fprintf(stderr,
                         "pictura self-test: canvas_grow=%d size=%dx%d "
                         "map_tl=%d map_br=%d corner_a=%d\n",
                         grown ? 1 : 0,
                         grownImg.width(),
                         grownImg.height(),
                         grownImg.pixel(4, 6) == restored.pixel(2, 2) ? 1 : 0,
                         grownImg.pixel(8, 10) == restored.pixel(6, 6) ? 1 : 0,
                         qAlpha(grownImg.pixel(0, 0)));
            std::fflush(stderr);
            if (!grown || grownImg.width() != 10 || grownImg.height() != 12
                || grownImg.pixel(4, 6) != restored.pixel(2, 2)
                || grownImg.pixel(8, 10) != restored.pixel(6, 6)
                || qAlpha(grownImg.pixel(0, 0)) != 0 || qAlpha(grownImg.pixel(1, 1)) != 0) {
                std::fprintf(stderr, "pictura self-test: FAIL: canvas growth wrong\n");
                return 21;
            }

            // M14: mutating ops capture history; undo/redo must round-trip the
            // pixels bit-exactly.
            const QImage preUndo = view.image();
            const int depthBefore = view.history_depth();
            const bool historyRotated = view.rotate_doc(1);
            const int depthAfterRotate = view.history_depth();
            const bool rotatedCanUndo = view.can_undo();
            const QImage postRotate = view.image();
            const bool undone = view.undo();
            const bool undoIdentical = view.image() == preUndo;
            const bool redone = view.redo();
            const bool redoIdentical = view.image() == postRotate;
            std::fprintf(stderr,
                         "pictura self-test: history rotate=%d depth=%d undo=%d "
                         "undo_ident=%d redo=%d redo_ident=%d\n",
                         historyRotated ? 1 : 0,
                         depthAfterRotate,
                         undone ? 1 : 0,
                         undoIdentical ? 1 : 0,
                         redone ? 1 : 0,
                         redoIdentical ? 1 : 0);
            std::fflush(stderr);
            if (!historyRotated || depthAfterRotate != depthBefore + 1
                || !rotatedCanUndo || postRotate.width() != 12
                || postRotate.height() != 10 || !undone || !undoIdentical
                || !redone || !redoIdentical) {
                std::fprintf(stderr, "pictura self-test: FAIL: undo/redo round-trip wrong\n");
                return 22;
            }

            // M14: a fresh op invalidates redo; reopening the fixture resets
            // history and an empty-stack undo fails without touching pixels.
            const bool flipped = view.flip_doc(true);
            const bool redoInvalidated = flipped && !view.can_redo();
            const bool reopened = view.open(psdPath);
            const QImage reopenedImg = view.image();
            const bool noUndoAfterOpen = !view.can_undo();
            const bool boundaryUndone = view.undo();
            const bool boundaryIdentical = view.image() == reopenedImg;
            const bool openReset = reopened && noUndoAfterOpen && boundaryIdentical;
            std::fprintf(stderr,
                         "pictura self-test: history redo_invalid=%d open_reset=%d "
                         "boundary_undo=%d\n",
                         redoInvalidated ? 1 : 0,
                         openReset ? 1 : 0,
                         boundaryUndone ? 1 : 0);
            std::fflush(stderr);
            if (!redoInvalidated || !openReset || boundaryUndone) {
                std::fprintf(stderr, "pictura self-test: FAIL: history invalidation wrong\n");
                return 23;
            }

            // M15: render filters. The topmost pixel layer is the blue
            // quadrant, so a full-frame apply must change pixels only inside
            // the blue rect (4,4)-(8,8); the fixed seed must make a re-apply
            // bit-identical.
            const QImage preClouds = view.image();
            const bool clouded = view.apply_filter(QStringLiteral("clouds"));
            const QImage cloudedImg = view.image();
            int cloudsInside = 0;
            int cloudsOutside = 0;
            for (int y = 0; y < cloudedImg.height(); ++y) {
                for (int x = 0; x < cloudedImg.width(); ++x) {
                    if (cloudedImg.pixel(x, y) == preClouds.pixel(x, y)) {
                        continue;
                    }
                    if (x >= 4 && x < 8 && y >= 4 && y < 8) {
                        ++cloudsInside;
                    } else {
                        ++cloudsOutside;
                    }
                }
            }
            const bool reapplied = view.apply_filter(QStringLiteral("clouds"));
            const bool reapplyIdentical = reapplied && view.image() == cloudedImg;
            const bool flared = view.apply_filter(QStringLiteral("lens-flare"));
            const QImage flaredImg = view.image();
            int flareOutsideChanged = 0;
            for (int y = 0; y < flaredImg.height(); ++y) {
                for (int x = 0; x < flaredImg.width(); ++x) {
                    if (!(x >= 4 && x < 8 && y >= 4 && y < 8)
                        && flaredImg.pixel(x, y) != cloudedImg.pixel(x, y)) {
                        ++flareOutsideChanged;
                    }
                }
            }
            std::fprintf(stderr,
                         "pictura self-test: clouds=%d confined=%d inside=%d "
                         "reapply_ident=%d flare=%d\n",
                         clouded ? 1 : 0,
                         cloudsOutside == 0 && flareOutsideChanged == 0 ? 1 : 0,
                         cloudsInside,
                         reapplyIdentical ? 1 : 0,
                         flared && flaredImg != cloudedImg ? 1 : 0);
            std::fflush(stderr);
            if (!clouded || cloudedImg == preClouds || cloudsInside == 0
                || cloudsOutside != 0 || !reapplyIdentical || !flared
                || flaredImg == cloudedImg || flareOutsideChanged != 0) {
                std::fprintf(stderr, "pictura self-test: FAIL: clouds/flare wrong\n");
                return 24;
            }
        }
        const QPointF center(window->width() / 2.0, window->height() / 2.0);
        window->zoomAt(center, 120);
        const QPointF afterZoom = window->offset();
        window->panBy(QPointF(10.0, 5.0));
        if (window->zoom() <= 1.0 || window->offset() != afterZoom + QPointF(10.0, 5.0)) {
            std::fprintf(stderr, "pictura self-test: FAIL: zoom/pan transform wrong\n");
            return 3;
        }
        std::fprintf(stderr,
                     "pictura self-test: zoom=%.3f pan_ok=1\n",
                     window->zoom());
        std::fflush(stderr);
        QTimer::singleShot(2000, &app, &QCoreApplication::quit);
    }

    return app.exec();
}

#include "main.moc"
