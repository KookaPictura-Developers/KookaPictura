// File > Save for Web & Devices (#61).

#include <QtTest/QtTest>

#include "commands.h"
#include "frame.h"
#include "save_for_web_dialog.h"

#include <QtCore/QTemporaryDir>
#include <QtGui/QAction>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QLabel>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSpinBox>

#include "qt_test_support.h"

namespace {

// A 64 x 48 colour field with a clear 8 x 8 top-left corner.
QImage picture()
{
    QImage image(64, 48, QImage::Format_RGBA8888);
    for (int y = 0; y < 48; ++y) {
        for (int x = 0; x < 64; ++x) {
            const int alpha = (x < 8 && y < 8) ? 0 : 255;
            image.setPixel(x, y, qRgba(x * 4, y * 5, (x + y) * 2, alpha));
        }
    }
    return image;
}

template <typename T>
T* child(QObject& parent, const char* name)
{
    return parent.findChild<T*>(QString::fromLatin1(name));
}

int distinctColors(const QImage& image)
{
    QSet<QRgb> seen;
    for (int y = 0; y < image.height(); ++y) {
        for (int x = 0; x < image.width(); ++x) {
            seen.insert(image.pixel(x, y));
        }
    }
    return int(seen.size());
}

} // namespace

class SaveForWebTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void formats();
    void sizePresetsSaveAndDone();
    void menuCommand();

private:
    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void SaveForWebTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void SaveForWebTest::formats()
{
    pictura::SaveForWebDialog dialog(picture());
    auto* format = child<QComboBox>(dialog, "saveForWebFormat");
    QVERIFY(format);

    // The first open is GIF 128 Dithered: a GIF that decodes to the image's
    // size, its clear corner still clear.
    QCOMPARE(child<QComboBox>(dialog, "saveForWebPreset")->currentText(),
             QStringLiteral("GIF 128 Dithered"));
    QByteArray gif = dialog.encoded();
    QVERIFY(gif.startsWith("GIF89a"));
    QImage decoded = QImage::fromData(gif).convertToFormat(QImage::Format_RGBA8888);
    QCOMPARE(decoded.size(), QSize(64, 48));
    QCOMPARE(qAlpha(decoded.pixel(2, 2)), 0);
    QCOMPARE(qAlpha(decoded.pixel(30, 30)), 255);
    QVERIFY(child<QLabel>(dialog, "saveForWebOptimizedInfo")->text().contains(
        QStringLiteral("sec @ 56.6 Kbps")));
    child<QCheckBox>(dialog, "saveForWebInterlaced")->setChecked(true);
    QVERIFY(dialog.encoded() != gif);

    // JPEG: quality trades size.
    format->setCurrentIndex(pictura::SaveForWebDialog::Jpeg);
    auto* quality = child<QSpinBox>(dialog, "saveForWebQuality");
    quality->setValue(10);
    const QByteArray low = dialog.encoded();
    quality->setValue(100);
    const QByteArray high = dialog.encoded();
    QVERIFY(low.startsWith("\xff\xd8") && high.startsWith("\xff\xd8"));
    QVERIFY(low.size() < high.size());
    QCOMPARE(dialog.suffix(), QStringLiteral("jpg"));

    // PNG-8 keeps to its colour table (plus the clear entry).
    format->setCurrentIndex(pictura::SaveForWebDialog::Png8);
    child<QSpinBox>(dialog, "saveForWebColors")->setValue(16);
    const QImage png8 = QImage::fromData(dialog.encoded());
    QCOMPARE(png8.size(), QSize(64, 48));
    QVERIFY2(distinctColors(png8) <= 17, qPrintable(QString::number(distinctColors(png8))));

    // PNG-24 without transparency lays the corner over the white matte.
    format->setCurrentIndex(pictura::SaveForWebDialog::Png24);
    child<QCheckBox>(dialog, "saveForWebPngTransparency")->setChecked(false);
    const QImage png24 = QImage::fromData(dialog.encoded());
    QCOMPARE(png24.pixel(2, 2), qRgb(255, 255, 255));

    // WBMP: a type-0 header and one bit per pixel.
    format->setCurrentIndex(pictura::SaveForWebDialog::Wbmp);
    const QByteArray wbmp = dialog.encoded();
    QCOMPARE(QByteArray(wbmp.constData(), 4), QByteArray("\x00\x00\x40\x30", 4));
    QCOMPARE(wbmp.size(), 4 + 8 * 48);
    QCOMPARE(dialog.suffix(), QStringLiteral("wbmp"));
}

void SaveForWebTest::sizePresetsSaveAndDone()
{
    QTemporaryDir dir;
    QVERIFY(dir.isValid());
    {
        pictura::SaveForWebDialog dialog(picture());
        // Image Size: 50 % halves both sides, proportions kept.
        child<QSpinBox>(dialog, "saveForWebPercent")->setValue(50);
        QCOMPARE(child<QSpinBox>(dialog, "saveForWebWidth")->value(), 32);
        QCOMPARE(dialog.sized().size(), QSize(32, 24));
        QCOMPARE(QImage::fromData(dialog.encoded()).size(), QSize(32, 24));

        // A preset sets the format and its settings.
        child<QComboBox>(dialog, "saveForWebPreset")->setCurrentText(QStringLiteral("JPEG Low"));
        QCOMPARE(child<QComboBox>(dialog, "saveForWebFormat")->currentIndex(),
                 int(pictura::SaveForWebDialog::Jpeg));
        QCOMPARE(child<QSpinBox>(dialog, "saveForWebQuality")->value(), 10);

        // Save writes exactly the optimised file.
        const QString path = dir.filePath(QStringLiteral("web.jpg"));
        QVERIFY(dialog.saveTo(path));
        QFile file(path);
        QVERIFY(file.open(QIODevice::ReadOnly));
        QCOMPARE(file.readAll(), dialog.encoded());

        // Done keeps the settings for the next open.
        child<QComboBox>(dialog, "saveForWebFormat")->setCurrentIndex(pictura::SaveForWebDialog::Png24);
        child<QPushButton>(dialog, "saveForWebDone")->click();
    }
    pictura::SaveForWebDialog again(picture());
    QCOMPARE(child<QComboBox>(again, "saveForWebFormat")->currentIndex(),
             int(pictura::SaveForWebDialog::Png24));
    QCOMPARE(child<QComboBox>(again, "saveForWebPreset")->currentIndex(), 0);
}

void SaveForWebTest::menuCommand()
{
    pictura::CommandRegistry* registry = window_->registry();
    QAction* action = registry->action(QString::fromLatin1(pictura::command_ids::FileSaveForWeb));
    QVERIFY(action);
    QCOMPARE(action->shortcut(), QKeySequence(QStringLiteral("Ctrl+Alt+Shift+S")));
    registry->refresh();
    QVERIFY(!action->isEnabled());
    QVERIFY(window_->newDocument(QStringLiteral("Web"), 16, 16, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    registry->refresh();
    QVERIFY(action->isEnabled());
}

QTEST_MAIN(SaveForWebTest)
#include "tst_save_for_web.moc"
