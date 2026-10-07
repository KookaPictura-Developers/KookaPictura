// File > New: the dialog's presets, units, and mode/depth pairing, and the
// document create_document makes from them (one "New" state). Ported from
// photorust's NewDocumentDialog.

#include <QtTest/QtTest>

#include "frame.h"
#include "new_document_dialog.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/image_adjust/image_size.cxxqt.h"
#include "pictura_app/src/cxxqt_object/image_adjust/new_document.cxxqt.h"

#include <QtGui/QStandardItemModel>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDoubleSpinBox>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>

#include "qt_test_support.h"

namespace {

bool itemEnabled(QComboBox* combo, int index)
{
    auto* model = qobject_cast<QStandardItemModel*>(combo->model());
    return model && model->item(index) && model->item(index)->isEnabled();
}

} // namespace

class NewDocumentTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void presetsUnitsAndDepths();
    void createsEachModeWithOneState();

private:
    pictura::test::ScopedStateHome stateHome_;
};

void NewDocumentTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
}

void NewDocumentTest::presetsUnitsAndDepths()
{
    pictura::NewDocumentDialog dialog(QStringLiteral("Untitled-4"), QColor(10, 20, 30));
    auto* name = qobject_cast<QLineEdit*>(dialog.controlForTest(QStringLiteral("newDocName")));
    auto* type = qobject_cast<QComboBox*>(dialog.controlForTest(QStringLiteral("newDocType")));
    auto* size = qobject_cast<QComboBox*>(dialog.controlForTest(QStringLiteral("newDocSize")));
    auto* width = qobject_cast<QDoubleSpinBox*>(dialog.controlForTest(QStringLiteral("newDocWidth")));
    auto* unit = qobject_cast<QComboBox*>(dialog.controlForTest(QStringLiteral("newDocUnit")));
    auto* mode = qobject_cast<QComboBox*>(dialog.controlForTest(QStringLiteral("newDocMode")));
    auto* depth = qobject_cast<QComboBox*>(dialog.controlForTest(QStringLiteral("newDocDepth")));
    auto* contents = qobject_cast<QComboBox*>(dialog.controlForTest(QStringLiteral("newDocBackground")));
    auto* imageSize = qobject_cast<QLabel*>(dialog.controlForTest(QStringLiteral("newDocImageSize")));
    QVERIFY(name && type && size && width && unit && mode && depth && contents && imageSize);

    // Opens on Default Photoshop Size: 1280 x 800 px at 72 ppi, 2.93M as RGB.
    QCOMPARE(name->text(), QStringLiteral("Untitled-4"));
    QCOMPARE(type->currentText(), QStringLiteral("Default Photoshop Size"));
    QCOMPARE(dialog.widthPixels(), 1280);
    QCOMPARE(dialog.heightPixels(), 800);
    QCOMPARE(imageSize->text(), QStringLiteral("2.93M"));

    // International Paper lists its sizes; A4 at 300 ppi in millimeters.
    type->setCurrentIndex(type->findText(QStringLiteral("International Paper")));
    QCOMPARE(size->currentText(), QStringLiteral("A4"));
    QCOMPARE(unit->currentText(), QStringLiteral("Millimeters"));
    QCOMPARE(dialog.widthPixels(), 2480);
    QCOMPARE(dialog.heightPixels(), 3508);
    QCOMPARE(dialog.spec().ppi, 300.0);
    // Pixels/cm restates the resolution, not the size.
    auto* resolutionUnit =
        qobject_cast<QComboBox*>(dialog.controlForTest(QStringLiteral("newDocResolutionUnit")));
    QVERIFY(resolutionUnit);
    resolutionUnit->setCurrentIndex(1);
    QCOMPARE(dialog.widthPixels(), 2480);
    QVERIFY(dialog.spec().perCm);
    QVERIFY(qAbs(dialog.spec().ppi - 300.0) < 0.01);
    resolutionUnit->setCurrentIndex(0);

    // Restating in inches keeps the pixels.
    unit->setCurrentIndex(unit->findText(QStringLiteral("Inches")));
    QCOMPARE(dialog.widthPixels(), 2480);
    QCOMPARE(width->value(), 8.27);

    // A typed size turns the type to Custom.
    unit->setCurrentIndex(unit->findText(QStringLiteral("Pixels")));
    width->setValue(500);
    QCOMPARE(type->currentText(), QStringLiteral("Custom"));
    QCOMPARE(dialog.widthPixels(), 500);

    // Bitmap is 1-bit only; RGB cannot take 1 bit; Lab cannot take 32.
    mode->setCurrentIndex(mode->findText(QStringLiteral("Bitmap")));
    QCOMPARE(depth->currentData().toInt(), 1);
    QVERIFY(!itemEnabled(depth, depth->findData(8)));
    mode->setCurrentIndex(mode->findText(QStringLiteral("Lab Color")));
    QCOMPARE(depth->currentData().toInt(), 8);
    QVERIFY(!itemEnabled(depth, depth->findData(1)));
    QVERIFY(!itemEnabled(depth, depth->findData(32)));

    contents->setCurrentIndex(contents->findText(QStringLiteral("Background Color")));
    QCOMPARE(dialog.spec().fill, QColor(10, 20, 30));
    contents->setCurrentIndex(contents->findText(QStringLiteral("Transparent")));
    QVERIFY(dialog.spec().transparent);
}

void NewDocumentTest::createsEachModeWithOneState()
{
    {
        pictura::PictureView view;
        QVERIFY(pictura::create_document(view, 30, 20, QStringLiteral("rgb"), 8, 0x102030u, false,
                                         300.0, false));
        QCOMPARE(view.document_width(), 30);
        QCOMPARE(view.history_count(), 1);
        QCOMPARE(view.history_label(0), QStringLiteral("New"));
        QCOMPARE(view.composite_argb(5, 5), qRgb(0x10, 0x20, 0x30));
        QCOMPARE(pictura::document_ppi(view), 300.0);
    }
    {
        pictura::PictureView view;
        QVERIFY(pictura::create_document(view, 16, 16, QStringLiteral("cmyk"), 16, 0xffffffu,
                                         false, 72.0, false));
        QCOMPARE(view.history_count(), 1);
        QCOMPARE(view.document_depth_bits(), 16);
    }
    {
        pictura::PictureView view;
        QVERIFY(pictura::create_document(view, 16, 16, QStringLiteral("bitmap"), 1, 0xffffffu,
                                         false, 72.0, false));
        QCOMPARE(view.history_count(), 1);
    }
    pictura::PictureView refused;
    QVERIFY(!pictura::create_document(refused, 16, 16, QStringLiteral("rgb"), 1, 0, false, 72.0,
                                      false));
    QVERIFY(!refused.has_document());
}

QTEST_MAIN(NewDocumentTest)
#include "tst_new_document.moc"
