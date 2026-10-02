#include <QtTest/QtTest>

#include "frame.h"
#include "panels/layers_panel.h"
#include "panels/properties_panel.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include "qt_test_support.h"

class PropertiesPanelTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void cleanup();
    void plainDocumentShowsNoProperties();
    void adjustmentLayerShowsItsName();

private:
    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
    pictura::PictureView* view_ = nullptr;
    pictura::PropertiesPanel* panel_ = nullptr;
};

void PropertiesPanelTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void PropertiesPanelTest::cleanup()
{
    if (window_) {
        while (window_->activeDocumentIndex() >= 0) {
            window_->closeDocument(window_->activeDocumentIndex(), false);
        }
    }
    view_ = nullptr;
    panel_ = nullptr;
}

void PropertiesPanelTest::plainDocumentShowsNoProperties()
{
    const bool created = window_->newDocument(QStringLiteral("Props"), 16, 16,
                                              QStringLiteral("rgb"), 8, QStringLiteral("white"));
    view_ = window_->activeView();
    panel_ = window_->findChild<pictura::PropertiesPanel*>(QStringLiteral("propertiesPanel"));
    QVERIFY2(created && view_ && panel_, "properties fixture");

    panel_->setView(view_);
    panel_->refresh();
    QCOMPARE(panel_->messageForTest(), QStringLiteral("No Properties"));
}

void PropertiesPanelTest::adjustmentLayerShowsItsName()
{
    const bool created = window_->newDocument(QStringLiteral("Adjust"), 16, 16,
                                              QStringLiteral("rgb"), 8, QStringLiteral("white"));
    view_ = window_->activeView();
    panel_ = window_->findChild<pictura::PropertiesPanel*>(QStringLiteral("propertiesPanel"));
    auto* layers =
        window_->findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
    QVERIFY2(created && view_ && panel_ && layers, "adjustment fixture");
    QVERIFY2(view_->add_adjustment(QStringLiteral("brightness-contrast")), "add adjustment");

    QString path;
    for (int i = 0; i < view_->layer_row_count(); ++i) {
        if (view_->layer_row_has_adjustment(i)) {
            path = view_->layer_row_path(i);
            break;
        }
    }
    QVERIFY2(!path.isEmpty(), "adjustment row exists");

    panel_->setView(view_);
    layers->setView(view_);
    QVERIFY2(layers->selectRowForTest(path), "adjustment row selected");
    QVERIFY2(panel_->messageForTest().contains(QStringLiteral("Brightness/Contrast")),
             "adjustment kind named");
}

QTEST_MAIN(PropertiesPanelTest)
#include "tst_properties_panel.moc"
