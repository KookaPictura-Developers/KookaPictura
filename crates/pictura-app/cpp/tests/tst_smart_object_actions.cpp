#include <QtTest/QtTest>

#include <QtCore/QStringList>

#include "commands.h"
#include "frame.h"
#include "panels/layers_panel.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include "qt_test_support.h"

namespace {

QString id(const char* command)
{
    return QString::fromLatin1(command);
}

} // namespace

// The advanced Smart Object actions: Reset Transform, Convert to Layers, and
// New Smart Object via Copy. The engine behavior is unit-tested in
// pictura-render; these cover the command/menu wiring and the one-undo-state
// contract.
class SmartObjectActionsTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void cleanup();
    void rowMenuOffersSmartRows();
    void commandsRequireASmartObject();
    void resetTransformRecordsOneState();
    void convertToLayersConsumesObject();
    void newViaCopyAddsIndependentLayer();

private:
    QString makeSmartLayer(pictura::PictureView* view, pictura::LayersPanel* panel);

    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void SmartObjectActionsTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
    window_->show();
    QCoreApplication::processEvents();
}

void SmartObjectActionsTest::cleanup()
{
    if (window_) {
        while (window_->activeDocumentIndex() >= 0) {
            window_->closeDocument(window_->activeDocumentIndex(), false);
        }
    }
}

QString SmartObjectActionsTest::makeSmartLayer(pictura::PictureView* view,
                                               pictura::LayersPanel* panel)
{
    view->add_layer(-1);
    panel->refresh();
    const QString path = view->layer_row_path(0);
    panel->selectPaths({path}, path);
    window_->registry()->refresh();
    if (!view->convert_to_smart_object(path)) {
        return QString();
    }
    panel->refresh();
    window_->registry()->refresh();
    return path;
}

void SmartObjectActionsTest::rowMenuOffersSmartRows()
{
    auto* panel = window_->findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
    QVERIFY(panel != nullptr);
    const QStringList smart = panel->smartRowMenuTextsForTest();
    QVERIFY2(smart.contains(QStringLiteral("Reset Transform")), "Reset Transform row");
    QVERIFY2(smart.contains(QStringLiteral("Convert to Layers")), "Convert to Layers row");
    QVERIFY2(smart.contains(QStringLiteral("New Smart Object via Copy")),
             "New Smart Object via Copy row");
    QVERIFY(panel->smartRowMenuEnabledForTest(QStringLiteral("Reset Transform")));
    QVERIFY(panel->smartRowMenuEnabledForTest(QStringLiteral("Convert to Layers")));
    QVERIFY(panel->smartRowMenuEnabledForTest(QStringLiteral("New Smart Object via Copy")));

    const QStringList pixel = panel->rowMenuTextsForTest(QStringLiteral("pixel"));
    QVERIFY2(!pixel.contains(QStringLiteral("Reset Transform")), "plain pixel omits Reset");
    QVERIFY2(!pixel.contains(QStringLiteral("Convert to Layers")), "plain pixel omits Convert");
    QVERIFY2(!pixel.contains(QStringLiteral("New Smart Object via Copy")),
             "plain pixel omits via Copy");
}

void SmartObjectActionsTest::commandsRequireASmartObject()
{
    QVERIFY(window_->newDocument(QStringLiteral("SmartCmds"), 16, 16, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    pictura::PictureView* view = window_->activeView();
    auto* panel = window_->findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
    pictura::CommandRegistry* registry = window_->registry();
    QVERIFY(view && panel && registry);

    // A transparent pixel layer is not yet a smart object, so reset/convert are
    // disabled while convert-to-smart-object is enabled.
    view->add_layer(-1);
    panel->refresh();
    const QString path = view->layer_row_path(0);
    panel->selectPaths({path}, path);
    registry->refresh();
    QVERIFY(registry->action(id(pictura::command_ids::LayerSmartObjectConvertTo))->isEnabled());
    QVERIFY(!registry->action(id(pictura::command_ids::LayerSmartObjectResetTransform))->isEnabled());
    QVERIFY(
        !registry->action(id(pictura::command_ids::LayerSmartObjectConvertToLayers))->isEnabled());
    QVERIFY(!registry->action(id(pictura::command_ids::LayerSmartObjectNewViaCopy))->isEnabled());

    QVERIFY(view->convert_to_smart_object(path));
    panel->refresh();
    registry->refresh();
    QVERIFY(registry->action(id(pictura::command_ids::LayerSmartObjectResetTransform))->isEnabled());
    QVERIFY(
        registry->action(id(pictura::command_ids::LayerSmartObjectConvertToLayers))->isEnabled());
    QVERIFY(
        registry->action(id(pictura::command_ids::LayerSmartObjectNewViaCopy))->isEnabled());
}

void SmartObjectActionsTest::resetTransformRecordsOneState()
{
    QVERIFY(window_->newDocument(QStringLiteral("SmartReset"), 16, 16, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    pictura::PictureView* view = window_->activeView();
    auto* panel = window_->findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
    pictura::CommandRegistry* registry = window_->registry();
    QVERIFY(view && panel && registry);
    const QString path = makeSmartLayer(view, panel);
    QVERIFY2(!path.isEmpty(), "smart layer prepared");

    const int before = view->history_count();
    QVERIFY(registry->dispatch(id(pictura::command_ids::LayerSmartObjectResetTransform)));
    QCOMPARE(view->history_count(), before + 1);
    QVERIFY2(!view->layer_smart_object_state(path).isEmpty(), "stays a smart object");
}

void SmartObjectActionsTest::convertToLayersConsumesObject()
{
    QVERIFY(window_->newDocument(QStringLiteral("SmartLayers"), 16, 16, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    pictura::PictureView* view = window_->activeView();
    auto* panel = window_->findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
    pictura::CommandRegistry* registry = window_->registry();
    QVERIFY(view && panel && registry);
    const QString path = makeSmartLayer(view, panel);
    QVERIFY2(!path.isEmpty(), "smart layer prepared");

    const int before = view->history_count();
    QVERIFY(registry->dispatch(id(pictura::command_ids::LayerSmartObjectConvertToLayers)));
    QCOMPARE(view->history_count(), before + 1);
    QVERIFY2(view->layer_smart_object_state(path).isEmpty(), "object consumed into layers");
    QVERIFY(!registry->action(id(pictura::command_ids::LayerSmartObjectConvertToLayers))
                 ->isEnabled());
}

void SmartObjectActionsTest::newViaCopyAddsIndependentLayer()
{
    QVERIFY(window_->newDocument(QStringLiteral("SmartCopy"), 16, 16, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    pictura::PictureView* view = window_->activeView();
    auto* panel = window_->findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
    pictura::CommandRegistry* registry = window_->registry();
    QVERIFY(view && panel && registry);
    const QString path = makeSmartLayer(view, panel);
    QVERIFY2(!path.isEmpty(), "smart layer prepared");

    const int layersBefore = view->layer_row_count();
    const int before = view->history_count();
    QVERIFY(registry->dispatch(id(pictura::command_ids::LayerSmartObjectNewViaCopy)));
    QCOMPARE(view->history_count(), before + 1);
    QCOMPARE(view->layer_row_count(), layersBefore + 1);
    QVERIFY2(!view->layer_smart_object_state(path).isEmpty(), "original keeps its source");
}

QTEST_MAIN(SmartObjectActionsTest)
#include "tst_smart_object_actions.moc"
