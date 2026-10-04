// Image > Adjustments (#83): the menu is live on an opened image; a dialog
// previews on the canvas, Cancel restores the layer exactly, OK applies one
// state named for the adjustment; Invert / Desaturate / Equalize / Auto apply
// at once, inside the selection.

#include <QtTest/QtTest>

#include "adjustment_dialog.h"
#include "commands.h"
#include "frame.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/image_adjust.cxxqt.h"

#include <QtCore/QTemporaryDir>
#include <QtGui/QAction>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QSlider>

#include "qt_test_support.h"

namespace {

QByteArray block(const char* kind)
{
    const ::rust::Vec<std::uint8_t> v =
        pictura::image_adjustment_default(QString::fromLatin1(kind), 0x000000u, 0xffffffu);
    return QByteArray(reinterpret_cast<const char*>(v.data()), qsizetype(v.size()));
}

} // namespace

class ImageAdjustmentsTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void cleanup();
    void menuIsLiveOnAnOpenedImage();
    void dialogPreviewsCancelsAndApplies();
    void colorLookupPresetRebuildsTheLook();
    void directCommandsRespectTheSelection();

private:
    // Open a 20x20 opaque image of `color` (it becomes the Background).
    bool openImage(const QColor& color);
    QAction* leaf(const QString& name) const;

    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
    QTemporaryDir dir_;
    pictura::PictureView* view_ = nullptr;
};

void ImageAdjustmentsTest::initTestCase()
{
    QVERIFY(stateHome_.isValid() && dir_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void ImageAdjustmentsTest::cleanup()
{
    while (window_->activeDocumentIndex() >= 0) {
        window_->closeDocument(window_->activeDocumentIndex(), false);
    }
    view_ = nullptr;
}

bool ImageAdjustmentsTest::openImage(const QColor& color)
{
    QImage image(20, 20, QImage::Format_RGB32);
    image.fill(color);
    const QString path = dir_.filePath(QStringLiteral("photo.png"));
    if (!image.save(path) || !window_->openDocumentAtPath(path)) {
        return false;
    }
    QCoreApplication::processEvents();
    view_ = window_->activeView();
    window_->registry()->refresh();
    return view_ != nullptr;
}

QAction* ImageAdjustmentsTest::leaf(const QString& name) const
{
    return window_->registry()->action(
        pictura::commandIdForPath({QStringLiteral("Image"), QStringLiteral("Adjustments"), name}));
}

void ImageAdjustmentsTest::menuIsLiveOnAnOpenedImage()
{
    QAction* levels = leaf(QStringLiteral("Levels"));
    QVERIFY(levels);
    window_->registry()->refresh();
    QVERIFY(!levels->isEnabled());
    QVERIFY(openImage(QColor(100, 150, 200)));
    for (const char* name :
         {"Brightness/Contrast", "Levels", "Curves", "Exposure", "Vibrance", "Hue/Saturation",
          "Color Balance", "Black & White", "Photo Filter", "Channel Mixer", "Invert",
          "Posterize", "Threshold", "Gradient Map", "Selective Color", "Shadows/Highlights",
          "Color Lookup", "Desaturate", "Equalize", "Auto Tone", "Auto Contrast", "Auto Color"}) {
        QAction* action = leaf(QString::fromUtf8(name));
        QVERIFY2(action && action->isEnabled(), name);
    }
    QCOMPARE(levels->text(), QStringLiteral("Levels…"));
    QCOMPARE(levels->shortcut(), QKeySequence(QStringLiteral("Ctrl+L")));
    QCOMPARE(leaf(QStringLiteral("Invert"))->text(), QStringLiteral("Invert"));
    QVERIFY(!leaf(QStringLiteral("Match Color"))->isEnabled());
    QAction* autoTone = window_->registry()->action(
        pictura::commandIdForPath({QStringLiteral("Image"), QStringLiteral("Auto Tone")}));
    QVERIFY(autoTone && autoTone->isEnabled());
}

void ImageAdjustmentsTest::dialogPreviewsCancelsAndApplies()
{
    QVERIFY(openImage(QColor(100, 150, 200)));
    const QRgb before = view_->composite_argb(5, 5);
    const int history = view_->history_count();
    {
        pictura::AdjustmentDialog dialog(view_, block("hue-saturation"), QRect());
        auto* lightness =
            qobject_cast<QSlider*>(dialog.controlForTest(QStringLiteral("lightness")));
        QVERIFY(lightness);
        lightness->setValue(-100);
        QCOMPARE(qGray(view_->composite_argb(5, 5)), 0);
        dialog.reject();
    }
    QCOMPARE(view_->composite_argb(5, 5), before);
    QCOMPARE(view_->history_count(), history);

    pictura::AdjustmentDialog dialog(view_, block("hue-saturation"), QRect());
    qobject_cast<QSlider*>(dialog.controlForTest(QStringLiteral("lightness")))->setValue(-100);
    dialog.accept();
    QCOMPARE(qGray(view_->composite_argb(5, 5)), 0);
    QCOMPARE(view_->history_count(), history + 1);
    QCOMPARE(view_->history_label(view_->history_index()), QStringLiteral("Hue/Saturation"));
    QVERIFY(view_->undo());
    QCOMPARE(view_->composite_argb(5, 5), before);
}

void ImageAdjustmentsTest::colorLookupPresetRebuildsTheLook()
{
    QVERIFY(openImage(QColor(100, 150, 200)));
    const QRgb before = view_->composite_argb(5, 5);
    {
        pictura::AdjustmentDialog dialog(view_, block("color-lookup"), QRect());
        auto* preset = qobject_cast<QComboBox*>(dialog.controlForTest(QStringLiteral("preset")));
        QVERIFY(preset);
        QCOMPARE(preset->count(), 7);
        preset->setCurrentIndex(1); // Warm Contrast, not the identity
        QVERIFY(view_->composite_argb(5, 5) != before);
        dialog.reject();
    }
    QCOMPARE(view_->composite_argb(5, 5), before);
}

void ImageAdjustmentsTest::directCommandsRespectTheSelection()
{
    QVERIFY(openImage(QColor(100, 150, 200)));
    QVERIFY(view_->select_rect(0, 0, 10, 20, QStringLiteral("new"), 0.0));
    leaf(QStringLiteral("Invert"))->trigger();
    QCOMPARE(view_->history_label(view_->history_index()), QStringLiteral("Invert"));
    QCOMPARE(QColor(view_->composite_argb(5, 5)), QColor(155, 105, 55));
    QCOMPARE(QColor(view_->composite_argb(15, 5)), QColor(100, 150, 200));

    view_->deselect();
    leaf(QStringLiteral("Desaturate"))->trigger();
    const QColor grey(view_->composite_argb(15, 5));
    QVERIFY(grey.red() == grey.green() && grey.green() == grey.blue());
    QCOMPARE(view_->history_label(view_->history_index()), QStringLiteral("Desaturate"));
    leaf(QStringLiteral("Equalize"))->trigger();
    QCOMPARE(view_->history_label(view_->history_index()), QStringLiteral("Equalize"));
    // Two greys spread to the ends of the range.
    QCOMPARE(qGray(view_->composite_argb(15, 5)), 255);
}

QTEST_MAIN(ImageAdjustmentsTest)
#include "tst_image_adjustments.moc"
