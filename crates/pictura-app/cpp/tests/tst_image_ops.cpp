// Image > Duplicate / Trim (#84), Image Size / Canvas Size: ported from
// photorust.

#include <QtTest/QtTest>

#include "canvas_size_dialog.h"
#include "frame.h"
#include "image_size_dialog.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/image_adjust/image_ops.cxxqt.h"
#include "pictura_app/src/cxxqt_object/image_adjust/image_size.cxxqt.h"
#include "pictura_app/src/cxxqt_object/image_adjust/new_document.cxxqt.h"
#include "pictura_app/src/cxxqt_object/paint_tools/fills.cxxqt.h"

#include "qt_test_support.h"

#include <QtCore/QTemporaryDir>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDoubleSpinBox>
#include <QtWidgets/QToolButton>

class ImageOpsTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void duplicateDocument();
    void trimDocument();
    void imageSizeResamplesAndSetsResolution();
    void imageSizePhysicalUnitsUseInches();
    void imageSizeEstimatesTheStoredMode();
    void canvasSizeGrowsAroundTheAnchor();

private:
    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void ImageOpsTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void ImageOpsTest::duplicateDocument()
{
    pictura::PicturaMainWindow& frame = *window_;
    QVERIFY(frame.newDocument(QStringLiteral("Dup"), 20, 16, QStringLiteral("rgb"), 8,
                              QStringLiteral("white")));
    pictura::PictureView* source = frame.activeView();
    QVERIFY(source);
    source->select_all();
    QVERIFY(pictura::edit_fill(*source, QColor(30, 60, 90).rgba(), -1, QStringLiteral("normal"),
                              100, false));
    source->deselect();
    const int doc = frame.activeDocumentIndex();

    const int before = frame.documentCount();
    auto* target = new pictura::PictureView(&frame);
    QVERIFY(pictura::duplicate_into(*source, *target, QStringLiteral("copy"), false));
    QCOMPARE(target->document_width(), 20);
    QCOMPARE(target->document_height(), 16);
    QCOMPARE(target->sample_argb(5, 5), QColor(30, 60, 90).rgba());
    QCOMPARE(target->history_count(), 1);
    QCOMPARE(target->history_label(0), QStringLiteral("Duplicate"));
    delete target;
    QCOMPARE(frame.documentCount(), before);

    QVERIFY(pictura::duplicate_name(*source).endsWith(QStringLiteral(" copy")));
    frame.closeDocument(doc, false);
}

void ImageOpsTest::trimDocument()
{
    pictura::PicturaMainWindow& frame = *window_;
    // A transparent 24×24 canvas; fill a 5×4 block at (3, 2), then trim.
    QVERIFY(frame.newDocument(QStringLiteral("Trim"), 24, 24, QStringLiteral("rgb"), 8,
                              QStringLiteral("transparent")));
    pictura::PictureView* view = frame.activeView();
    QVERIFY(view);
    const int doc = frame.activeDocumentIndex();

    QVERIFY(view->select_rect(3, 2, 5, 4, QStringLiteral("new"), 0.0));
    QVERIFY(pictura::edit_fill(*view, QColor(200, 0, 0).rgba(), -1, QStringLiteral("normal"), 100,
                               false));
    view->deselect();

    QVERIFY(pictura::trim_image(*view));
    QCOMPARE(view->history_count(), 1);
    QCOMPARE(view->history_label(0), QStringLiteral("Trim"));
    QCOMPARE(view->document_width(), 5);
    QCOMPARE(view->document_height(), 4);
    QCOMPARE(view->sample_argb(0, 0), QColor(200, 0, 0).rgba());

    // A second trim has nothing to remove.
    QVERIFY(!pictura::trim_image(*view));
    frame.closeDocument(doc, false);
}

// Image Size: the chained Width keeps the proportions, the Fit To presets
// and the summary follow, and Resample off changes only the resolution.
void ImageOpsTest::imageSizeResamplesAndSetsResolution()
{
    pictura::PicturaMainWindow& frame = *window_;
    QVERIFY(frame.newDocument(QStringLiteral("Size"), 40, 20, QStringLiteral("rgb"), 8,
                              QStringLiteral("white")));
    pictura::PictureView* view = frame.activeView();
    QVERIFY(view);
    const int doc = frame.activeDocumentIndex();
    QCOMPARE(pictura::document_ppi(*view), 72.0);

    {
        pictura::ImageSizeDialog dialog(view);
        auto* width = qobject_cast<QDoubleSpinBox*>(dialog.controlForTest(QStringLiteral("imageSizeWidth")));
        auto* fit = qobject_cast<QComboBox*>(dialog.controlForTest(QStringLiteral("imageSizeFitTo")));
        QVERIFY(width && fit);
        width->setValue(20);
        QCOMPARE(dialog.resultHeight(), 10);
        QCOMPARE(fit->currentText(), QStringLiteral("Custom"));
        // A landscape image takes the 4 x 6 in preset on its side.
        fit->setCurrentIndex(fit->findText(QStringLiteral("4 x 6 in 300 dpi")));
        QCOMPARE(dialog.resultWidth(), 1800);
        QCOMPARE(dialog.resultHeight(), 1200);
        QCOMPARE(dialog.resultPpi(), 300.0);
        fit->setCurrentIndex(fit->findText(QStringLiteral("Original Size")));
        width->setValue(20);
        QCOMPARE(dialog.resampleKind(), QStringLiteral("bicubic"));
        QVERIFY(pictura::image_size_apply(*view, dialog.resampleKind(), dialog.resultWidth(),
                                          dialog.resultHeight(), dialog.resultPpi(),
                                          dialog.resultPerCm()));
    }
    QCOMPARE(view->document_width(), 20);
    QCOMPARE(view->document_height(), 10);
    QCOMPARE(view->history_label(view->history_index()), QStringLiteral("Image Size"));

    {
        pictura::ImageSizeDialog dialog(view);
        auto* resample = qobject_cast<QCheckBox*>(dialog.controlForTest(QStringLiteral("imageSizeResample")));
        auto* resolution =
            qobject_cast<QDoubleSpinBox*>(dialog.controlForTest(QStringLiteral("imageSizeResolution")));
        QVERIFY(resample && resolution);
        resample->setChecked(false);
        resolution->setValue(300);
        QCOMPARE(dialog.resultWidth(), 20);
        QVERIFY(dialog.resampleKind().isEmpty());
        QVERIFY(pictura::image_size_apply(*view, dialog.resampleKind(), dialog.resultWidth(),
                                          dialog.resultHeight(), dialog.resultPpi(),
                                          dialog.resultPerCm()));
    }
    QCOMPARE(view->document_width(), 20);
    QCOMPARE(pictura::document_ppi(*view), 300.0);
    QVERIFY(!pictura::image_size_apply(*view, QString(), 20, 10, 300.0, false));
    view->undo();
    QCOMPARE(pictura::document_ppi(*view), 72.0);
    frame.closeDocument(doc, false);
}

// Image Size: a printed width in cm/mm/points sets pixels per inch, and a
// resolution edit resamples whichever axis is physical, both when chained.
void ImageOpsTest::imageSizePhysicalUnitsUseInches()
{
    pictura::PicturaMainWindow& frame = *window_;
    QVERIFY(frame.newDocument(QStringLiteral("Units"), 3000, 1500, QStringLiteral("rgb"), 8,
                              QStringLiteral("white")));
    pictura::PictureView* view = frame.activeView();
    QVERIFY(view);
    const int doc = frame.activeDocumentIndex();
    const auto find = [](pictura::ImageSizeDialog& dialog, const char* name) {
        return dialog.controlForTest(QLatin1String(name));
    };
    {
        pictura::ImageSizeDialog dialog(view);
        auto* resample = qobject_cast<QCheckBox*>(find(dialog, "imageSizeResample"));
        auto* width = qobject_cast<QDoubleSpinBox*>(find(dialog, "imageSizeWidth"));
        auto* unit = qobject_cast<QComboBox*>(find(dialog, "imageSizeWidthUnit"));
        QVERIFY(resample && width && unit);
        resample->setChecked(false);
        unit->setCurrentIndex(unit->findText(QStringLiteral("Centimeters")));
        width->setValue(10.0);
        QCOMPARE(qRound(dialog.resultPpi()), 762);
        unit->setCurrentIndex(unit->findText(QStringLiteral("Millimeters")));
        width->setValue(254.0);
        QCOMPARE(qRound(dialog.resultPpi()), 300);
        unit->setCurrentIndex(unit->findText(QStringLiteral("Points")));
        width->setValue(360.0);
        QCOMPARE(qRound(dialog.resultPpi()), 600);
    }
    {
        // Width in pixels, Height in inches: the Height drives the resample.
        pictura::ImageSizeDialog dialog(view);
        auto* height = qobject_cast<QComboBox*>(find(dialog, "imageSizeHeightUnit"));
        auto* chain = qobject_cast<QToolButton*>(find(dialog, "imageSizeConstrain"));
        auto* resolution = qobject_cast<QDoubleSpinBox*>(find(dialog, "imageSizeResolution"));
        QVERIFY(height && chain && resolution);
        height->setCurrentIndex(height->findText(QStringLiteral("Inches")));
        resolution->setValue(144.0);
        QCOMPARE(dialog.resultWidth(), 6000);
        QCOMPARE(dialog.resultHeight(), 3000);
        chain->setChecked(false);
        resolution->setValue(72.0);
        QCOMPARE(dialog.resultWidth(), 6000);
        QCOMPARE(dialog.resultHeight(), 1500);
    }
    frame.closeDocument(doc, false);
}

// Image Size's estimate counts the stored mode's planes and bits.
void ImageOpsTest::imageSizeEstimatesTheStoredMode()
{
    const auto bytes = [](const QString& mode, int bits) {
        pictura::PictureView view;
        if (!pictura::create_document(view, 8, 8, mode, bits, 0xffffffu, false, 72.0, false)) {
            return -1.0;
        }
        return pictura::bytesPerPixel(&view);
    };
    QCOMPARE(bytes(QStringLiteral("rgb"), 8), 3.0);
    QCOMPARE(bytes(QStringLiteral("cmyk"), 8), 4.0);
    QCOMPARE(bytes(QStringLiteral("cmyk"), 16), 8.0);
    QCOMPARE(bytes(QStringLiteral("grayscale"), 8), 1.0);
    QCOMPARE(bytes(QStringLiteral("bitmap"), 1), 0.125);
}

// Canvas Size: Relative grows the canvas around the anchor and the
// Background's new area takes the extension colour.
void ImageOpsTest::canvasSizeGrowsAroundTheAnchor()
{
    pictura::PicturaMainWindow& frame = *window_;
    // An opaque image opens as a Background; a new white document's Layer 0
    // would keep the new area transparent, as CS6 does for a plain layer.
    QTemporaryDir dir;
    QVERIFY(dir.isValid());
    QImage image(20, 10, QImage::Format_RGB32);
    image.fill(Qt::white);
    const QString path = dir.filePath(QStringLiteral("canvas.png"));
    QVERIFY(image.save(path) && frame.openDocumentAtPath(path));
    QCoreApplication::processEvents();
    pictura::PictureView* view = frame.activeView();
    QVERIFY(view);
    const int doc = frame.activeDocumentIndex();

    pictura::CanvasSizeDialog dialog(view, Qt::black, Qt::white);
    auto* relative = qobject_cast<QCheckBox*>(dialog.controlForTest(QStringLiteral("canvasSizeRelative")));
    auto* width = qobject_cast<QDoubleSpinBox*>(dialog.controlForTest(QStringLiteral("canvasSizeWidth")));
    auto* unit = qobject_cast<QComboBox*>(dialog.controlForTest(QStringLiteral("canvasSizeWidthUnit")));
    auto* extension = qobject_cast<QComboBox*>(dialog.controlForTest(QStringLiteral("canvasSizeExtension")));
    QVERIFY(relative && width && unit && extension);
    QCOMPARE(dialog.resultWidth(), 20);
    unit->setCurrentIndex(unit->findText(QStringLiteral("Percent")));
    QCOMPARE(width->value(), 100.0);
    unit->setCurrentIndex(unit->findText(QStringLiteral("Pixels")));
    relative->setChecked(true);
    width->setValue(10);
    QCOMPARE(dialog.resultWidth(), 30);
    QCOMPARE(dialog.resultHeight(), 10);
    QCOMPARE(dialog.anchorName(), QStringLiteral("center"));
    extension->setCurrentIndex(extension->findText(QStringLiteral("Foreground")));
    QCOMPARE(dialog.extensionColor(), QColor(Qt::black));

    QVERIFY(pictura::canvas_size_apply(*view, dialog.anchorName(), dialog.resultWidth(),
                                       dialog.resultHeight(), dialog.extensionColor().rgb() & 0xffffffu));
    QCOMPARE(view->document_width(), 30);
    QCOMPARE(view->history_label(view->history_index()), QStringLiteral("Canvas Size"));
    QCOMPARE(QColor::fromRgba(view->composite_argb(2, 5)), QColor(Qt::black));
    QCOMPARE(QColor::fromRgba(view->composite_argb(15, 5)), QColor(Qt::white));
    QCOMPARE(QColor::fromRgba(view->composite_argb(27, 5)), QColor(Qt::black));
    frame.closeDocument(doc, false);
}

QTEST_MAIN(ImageOpsTest)
#include "tst_image_ops.moc"
