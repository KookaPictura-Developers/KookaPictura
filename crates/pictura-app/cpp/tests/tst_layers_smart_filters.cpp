#include <QtTest/QtTest>

#include <QtCore/QStringList>
#include <QtWidgets/QTreeView>

#include <functional>

#include "frame.h"
#include "panels/layers_panel.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/layers_smart_filters.cxxqt.h"

#include "qt_test_support.h"

class LayersSmartFiltersTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void cleanup();
    void treeShowsGroupAndToggles();

private:
    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void LayersSmartFiltersTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
    window_->show();
    QCoreApplication::processEvents();
}

void LayersSmartFiltersTest::cleanup()
{
    if (window_) {
        while (window_->activeDocumentIndex() >= 0) {
            window_->closeDocument(window_->activeDocumentIndex(), false);
        }
    }
}

void LayersSmartFiltersTest::treeShowsGroupAndToggles()
{
    QVERIFY(window_->newDocument(QStringLiteral("SmartFilters"), 16, 16, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    pictura::PictureView* view = window_->activeView();
    pictura::LayersPanel* panel =
        window_->findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
    QVERIFY(view && panel);

    const QString path = view->add_solid_fill(0xff808080u);
    QVERIFY(view->rasterize_fill_content(path));
    QVERIFY(view->apply_pictura_raw_filter(path, 20.0, 0.0, 1.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                                           0.0, 0.0));

    const QString group = path + QStringLiteral("/@sf");
    const QString child = group + QStringLiteral("/0");

    panel->setView(view);
    panel->refresh();
    QCoreApplication::processEvents();
    panel->expandForTest(path);
    panel->expandForTest(group);
    QCoreApplication::processEvents();

    auto* tree = panel->findChild<QTreeView*>();
    QVERIFY(tree);
    QAbstractItemModel* model = tree->model();
    QVERIFY(model);

    std::function<QModelIndex(const QModelIndex&)> findText =
        [&](const QModelIndex& parent) -> QModelIndex {
        for (int r = 0; r < model->rowCount(parent); ++r) {
            const QModelIndex index = model->index(r, 0, parent);
            if (index.data(Qt::DisplayRole).toString() == QStringLiteral("Smart Filters")) {
                return index;
            }
            const QModelIndex found = findText(index);
            if (found.isValid()) {
                return found;
            }
        }
        return {};
    };
    const QModelIndex groupIndex = findText(QModelIndex());
    QVERIFY(groupIndex.isValid());
    QCOMPARE(model->rowCount(groupIndex), 1);
    QCOMPARE(model->index(0, 0, groupIndex).data(Qt::DisplayRole).toString(),
             QStringLiteral("Camera Raw Filter"));

    QVERIFY(pictura::layer_smart_filter_visible(*view, path, 0));
    QVERIFY(panel->clickSmartFilterEyeForTest(child));
    QCoreApplication::processEvents();
    QVERIFY(!pictura::layer_smart_filter_visible(*view, path, 0));

    QVERIFY(pictura::layer_smart_filters_enabled(*view, path));
    QVERIFY(panel->clickSmartFilterEyeForTest(group));
    QCoreApplication::processEvents();
    QVERIFY(!pictura::layer_smart_filters_enabled(*view, path));
}

QTEST_MAIN(LayersSmartFiltersTest)
#include "tst_layers_smart_filters.moc"
