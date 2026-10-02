#include <QtTest/QtTest>

#include <QtCore/QPointF>

#include "frame.h"
#include "panels/info_panel.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/annotations.cxxqt.h"

#include "qt_test_support.h"

class InfoPanelTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void cleanup();
    void colorBlockReadsRgbAndSwitchesModes();
    void measurementUnitReformatsPosition();
    void selectionDrivesSizeAndBlanksWithoutOne();
    void rulerModeSwapsTopRightBlock();
    void offCanvasBlanksColorReadouts();
    void docLineIsPresent();

private:
    void openDocument(const QString& name);

    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
    pictura::PictureView* view_ = nullptr;
    pictura::InfoPanel* panel_ = nullptr;
};

void InfoPanelTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void InfoPanelTest::cleanup()
{
    if (window_) {
        while (window_->activeDocumentIndex() >= 0) {
            window_->closeDocument(window_->activeDocumentIndex(), false);
        }
    }
    view_ = nullptr;
    panel_ = nullptr;
}

void InfoPanelTest::openDocument(const QString& name)
{
    const bool created = window_->newDocument(name, 16, 16, QStringLiteral("rgb"), 8,
                                              QStringLiteral("white"));
    view_ = window_->activeView();
    panel_ = window_->findChild<pictura::InfoPanel*>(QStringLiteral("infoPanel"));
    QVERIFY2(created && view_ && panel_, "info fixture");
    panel_->setView(view_);
    // The panel outlives a document, so reset its readout state for isolation.
    panel_->setColorModeForTest(0, QStringLiteral("RGB"));
    panel_->setColorModeForTest(1, QStringLiteral("CMYK"));
    panel_->setMeasurementUnitForTest(0, QStringLiteral("Pixels"));
    panel_->setMeasurementUnitForTest(1, QStringLiteral("Pixels"));
}

void InfoPanelTest::colorBlockReadsRgbAndSwitchesModes()
{
    openDocument(QStringLiteral("Color"));
    panel_->setCursorPosition(QPointF(2, 2));

    QVERIFY2(panel_->colorBlockTextForTest(0).contains(QStringLiteral("R : 255")), "rgb readout");
    QVERIFY2(panel_->colorBlockTextForTest(1).contains(QStringLiteral("C : 0")),
             "top-right defaults to CMYK");

    panel_->setColorModeForTest(0, QStringLiteral("CMYK"));
    const QString cmyk = panel_->colorBlockTextForTest(0);
    QVERIFY2(cmyk.contains(QStringLiteral("C : 0")) && cmyk.contains(QStringLiteral("M : 0"))
                 && cmyk.contains(QStringLiteral("Y : 0")) && cmyk.contains(QStringLiteral("K : 0")),
             "top-left switches to CMYK rows");

    panel_->setColorModeForTest(0, QStringLiteral("Grayscale"));
    QVERIFY2(panel_->colorBlockTextForTest(0).contains(QStringLiteral("K : 255")),
             "grayscale uses qGray");
}

void InfoPanelTest::measurementUnitReformatsPosition()
{
    openDocument(QStringLiteral("Units"));
    panel_->setCursorPosition(QPointF(2, 2));
    QCOMPARE(panel_->positionTextForTest(), QStringLiteral("2, 2"));

    // 72 PPI: 2 px = 0.71 mm.
    panel_->setMeasurementUnitForTest(0, QStringLiteral("Millimeters"));
    QCOMPARE(panel_->positionTextForTest(), QStringLiteral("0.71, 0.71"));

    panel_->setMeasurementUnitForTest(0, QStringLiteral("Pixels"));
    QCOMPARE(panel_->positionTextForTest(), QStringLiteral("2, 2"));
}

void InfoPanelTest::selectionDrivesSizeAndBlanksWithoutOne()
{
    openDocument(QStringLiteral("Selection"));
    QVERIFY2(panel_->sizeTextForTest().isEmpty(), "no selection blanks W/H");

    QVERIFY2(view_->select_rect(2, 2, 4, 4, QStringLiteral("new"), 0.0), "select rect");
    panel_->refresh();
    QCOMPARE(panel_->sizeTextForTest(), QStringLiteral("4 × 4"));

    panel_->setMeasurementUnitForTest(1, QStringLiteral("Millimeters"));
    QCOMPARE(panel_->sizeTextForTest(), QStringLiteral("1.41 × 1.41"));
}

void InfoPanelTest::rulerModeSwapsTopRightBlock()
{
    openDocument(QStringLiteral("Ruler"));
    panel_->setRulerMode(true);
    QVERIFY2(panel_->rulerMode(), "ruler mode on");
    QVERIFY2(panel_->rulerTextForTest().isEmpty(), "no line -> blank A/L");
    QVERIFY2(panel_->sizeTextForTest().isEmpty(), "no line -> blank W/H");

    pictura::set_ruler(*view_, 2.0, 3.0, 2.0, 13.0, false);
    panel_->refresh();
    QVERIFY2(panel_->rulerTextForTest().contains(QStringLiteral("-90.0")), "angle readout");
    QVERIFY2(panel_->rulerTextForTest().contains(QStringLiteral("10.0")), "length readout");
    QCOMPARE(panel_->sizeTextForTest(), QStringLiteral("0 × 10"));

    panel_->setRulerMode(false);
    QVERIFY2(panel_->colorBlockTextForTest(1).contains(QStringLiteral("C :")),
             "CMYK block restored");
}

void InfoPanelTest::offCanvasBlanksColorReadouts()
{
    openDocument(QStringLiteral("OffCanvas"));
    panel_->setCursorPosition(QPointF(100, 100));
    QVERIFY2(!panel_->colorBlockTextForTest(0).contains(QStringLiteral("255")),
             "off-canvas RGB blank");
    QVERIFY2(!panel_->colorBlockTextForTest(1).contains(QStringLiteral("0")),
             "off-canvas CMYK blank");
}

void InfoPanelTest::docLineIsPresent()
{
    openDocument(QStringLiteral("Doc"));
    panel_->setCursorPosition(QPointF(2, 2));
    QVERIFY2(panel_->docTextForTest().startsWith(QStringLiteral("Doc: ")), "doc line present");
    // A 16x16 RGB document holds an 8-plane footprint = 2048 bytes = 2.0K.
    QVERIFY2(panel_->docTextForTest().contains(QStringLiteral("2.0K")), "memory footprint");
}

QTEST_MAIN(InfoPanelTest)
#include "tst_info_panel.moc"
