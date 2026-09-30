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
    void cursorColorAndCmyk();
    void cmykOffCanvas();
    void rulerModeShowsAngleLengthAndRulerSize();

private:
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

void InfoPanelTest::cursorColorAndCmyk()
{
    const bool created = window_->newDocument(QStringLiteral("Info"), 16, 16,
                                              QStringLiteral("rgb"), 8, QStringLiteral("white"));
    view_ = window_->activeView();
    panel_ = window_->findChild<pictura::InfoPanel*>(QStringLiteral("infoPanel"));
    QVERIFY2(created && view_ && panel_, "info fixture");

    panel_->setView(view_);
    panel_->setCursorPosition(QPointF(2, 2));
    QVERIFY2(panel_->colorTextForTest().contains(QStringLiteral("255")), "rgb readout");
    QCOMPARE(panel_->cmykTextForTest(), QStringLiteral("C 0  M 0  Y 0  K 0"));
    QCOMPARE(panel_->sizeTextForTest(), QStringLiteral("16 × 16"));
}

void InfoPanelTest::cmykOffCanvas()
{
    const bool created = window_->newDocument(QStringLiteral("OffCanvas"), 16, 16,
                                              QStringLiteral("rgb"), 8, QStringLiteral("white"));
    view_ = window_->activeView();
    panel_ = window_->findChild<pictura::InfoPanel*>(QStringLiteral("infoPanel"));
    QVERIFY2(created && view_ && panel_, "off-canvas fixture");

    panel_->setView(view_);
    panel_->setCursorPosition(QPointF(100, 100));
    QVERIFY2(panel_->cmykTextForTest().isEmpty(), "off-canvas cmyk blank");
}

void InfoPanelTest::rulerModeShowsAngleLengthAndRulerSize()
{
    const bool created = window_->newDocument(QStringLiteral("Ruler"), 16, 16,
                                              QStringLiteral("rgb"), 8, QStringLiteral("white"));
    view_ = window_->activeView();
    panel_ = window_->findChild<pictura::InfoPanel*>(QStringLiteral("infoPanel"));
    QVERIFY2(created && view_ && panel_, "ruler fixture");

    panel_->setView(view_);
    panel_->setRulerMode(true);
    QVERIFY2(panel_->rulerMode(), "ruler mode on");
    QVERIFY2(panel_->rulerTextForTest().isEmpty(), "no line -> blank A/L");
    QVERIFY2(panel_->sizeTextForTest().isEmpty(), "no line -> blank W/H");

    pictura::set_ruler(*view_, 2.0, 3.0, 2.0, 13.0, false);
    panel_->refresh();
    QVERIFY2(panel_->rulerTextForTest().contains(QStringLiteral("-90.0")), "angle readout");
    QVERIFY2(panel_->rulerTextForTest().contains(QStringLiteral("10.0")), "length readout");
    QCOMPARE(panel_->sizeTextForTest(), QStringLiteral("0.0 × 10.0"));

    panel_->setRulerMode(false);
    QCOMPARE(panel_->sizeTextForTest(), QStringLiteral("16 × 16"));
}

QTEST_MAIN(InfoPanelTest)
#include "tst_info_panel.moc"
