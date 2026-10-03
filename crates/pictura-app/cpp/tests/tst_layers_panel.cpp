#include <QtTest/QtTest>

#include <QtCore/QStringList>

#include "frame.h"
#include "panels/layers_panel.h"
#include "commands.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/shapes.cxxqt.h"

#include <QtGui/QAction>

#include "qt_test_support.h"

class LayersPanelTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void cleanup();
    void nestingLockRefusalAndReorder();
    void panelChrome();
    void rowWidgets();
    void dragReorder();
    void dropOnDelete();
    void clippingMasks();

private:
    bool setupNest();
    int rowOf(const QString& path);

    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
    pictura::PictureView* view_ = nullptr;
    pictura::LayersPanel* panel_ = nullptr;
    QString group_;
    QString a_;
    QString b_;
};

void LayersPanelTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void LayersPanelTest::cleanup()
{
    if (window_) {
        while (window_->activeDocumentIndex() >= 0) {
            window_->closeDocument(window_->activeDocumentIndex(), false);
        }
    }
    view_ = nullptr;
    panel_ = nullptr;
    group_.clear();
    a_.clear();
    b_.clear();
}

bool LayersPanelTest::setupNest()
{
    const bool created = window_->newDocument(QStringLiteral("NestLock"), 16, 16,
                                              QStringLiteral("rgb"), 8, QStringLiteral("white"));
    view_ = window_->activeView();
    panel_ = window_->findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
    if (!created || !view_ || !panel_) {
        return false;
    }
    group_ = view_->add_group_in(QString());
    a_ = view_->add_layer_in(group_);
    b_ = view_->add_layer_in(group_);
    view_->set_layer_name_path(a_, QStringLiteral("A"));
    view_->set_layer_name_path(b_, QStringLiteral("B"));
    return true;
}

int LayersPanelTest::rowOf(const QString& path)
{
    for (int i = 0; i < view_->layer_row_count(); ++i) {
        if (view_->layer_row_path(i) == path) {
            return i;
        }
    }
    return -1;
}

void LayersPanelTest::nestingLockRefusalAndReorder()
{
    QVERIFY2(setupNest(), "nesting fixture");
    const int doc = window_->activeDocumentIndex();
    const int lockBase = view_->history_count();
    const int locked =
        view_->set_layers_lock(QStringList{a_}, QStringLiteral("nesting"), true);
    QVERIFY2(locked == 1 && (view_->layer_row_lock(rowOf(a_)) & 0x08) != 0
                 && view_->history_count() == lockBase + 1,
             "nesting lock bit");
    const int groupBase = view_->history_count();
    const QString wrapped = view_->group_layers(QStringList{a_});
    QVERIFY2(wrapped.isEmpty() && view_->history_count() == groupBase
                 && view_->layer_row_path(rowOf(a_)) == a_,
             "group layers refused");
    const int moveBase = view_->history_count();
    const bool moved = view_->move_layer_path(a_, 1);
    QVERIFY2(moved && view_->layer_row_name(rowOf(QStringLiteral("1/1"))) == QStringLiteral("A")
                 && view_->history_count() == moveBase + 1,
             "move in container");
    const int all = view_->set_layers_lock(QStringList{QStringLiteral("1/1")},
                                           QStringLiteral("all"), true);
    QVERIFY2(all == 1
                 && (view_->layer_row_lock(rowOf(QStringLiteral("1/1"))) & 0x0F) == 0x0F,
             "all locks");
    window_->closeDocument(doc, false);
}

void LayersPanelTest::panelChrome()
{
    QVERIFY2(setupNest(), "chrome fixture");
    const int doc = window_->activeDocumentIndex();
    panel_->setView(view_);
    panel_->refresh();
    QVERIFY2(panel_->headerOrderOkForTest(), "header order");
    QVERIFY2(panel_->opacityLabelPresentForTest() && panel_->fillLabelPresentForTest(),
             "header labels");
    QVERIFY2(!panel_->hasPanelMenuButtonForTest(), "no panel menu button");
    QVERIFY2(panel_->filterToggleOnForTest() && panel_->filterToggleHasIconForTest(),
             "filter lightswitch");
    panel_->expandForTest(group_);
    const int eyeGroup = panel_->eyeLeftForTest(group_);
    const int eyeChild = panel_->eyeLeftForTest(QStringLiteral("1/0"));
    QVERIFY2(eyeGroup >= 0 && eyeGroup == eyeChild, "nested eye left-anchored");
    QVERIFY2(panel_->chevronClickExpandsForTest(group_), "chevron expands group");
    window_->closeDocument(doc, false);
}

void LayersPanelTest::rowWidgets()
{
    QVERIFY2(setupNest(), "rows fixture");
    const int doc = window_->activeDocumentIndex();
    panel_->setView(view_);
    panel_->refresh();
    QVERIFY2(panel_->opacityLabelTextForTest() == QStringLiteral("Opacity"), "opacity label");
    QVERIFY2(panel_->opacitySuffixPresentForTest(), "percent suffix");
    QVERIFY2(panel_->lockIconsPresentForTest(), "semantic lock icons");
    QVERIFY2(panel_->treeDragEnabledForTest(), "tree drag enabled");
    window_->closeDocument(doc, false);
}

void LayersPanelTest::dragReorder()
{
    QVERIFY2(setupNest(), "drag fixture");
    const int doc = window_->activeDocumentIndex();
    panel_->setView(view_);
    panel_->refresh();
    const int base = view_->history_count();
    const bool dragged =
        panel_->moveForTest(QStringLiteral("1/0"), QStringLiteral("1/1"), 0);
    QVERIFY2(dragged && view_->history_count() == base + 1, "drag reorder is one undo step");
    window_->closeDocument(doc, false);
}

void LayersPanelTest::dropOnDelete()
{
    const bool created = window_->newDocument(QStringLiteral("DropCtl"), 16, 16,
                                              QStringLiteral("rgb"), 8, QStringLiteral("white"));
    pictura::PictureView* view = window_->activeView();
    pictura::LayersPanel* panel =
        window_->findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
    QVERIFY2(created && view && panel, "drop fixture");
    const int doc = window_->activeDocumentIndex();
    panel->setView(view);
    panel->refresh();
    const int rows = view->layer_row_count();
    const int base = view->history_count();
    const bool dropped = panel->dropOnStripButtonForTest(
        QStringLiteral("layersStripDelete"), {QStringLiteral("0")});
    QVERIFY2(dropped && view->layer_row_count() == rows - 1
                 && view->history_count() == base + 1,
             "drop on delete is one undo step");
    window_->closeDocument(doc, false);
}

// Create Clipping Mask clips a full-canvas green fill to a red square below
// it; Release from the base frees it; Alt-clicking the line between the rows
// toggles it back and forth (#63).
void LayersPanelTest::clippingMasks()
{
    QVERIFY(window_->newDocument(QStringLiteral("Clip"), 16, 16, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    view_ = window_->activeView();
    panel_ = window_->findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
    QVERIFY(view_ && panel_);
    pictura::ShapeSpec square{};
    square.kind = 0;
    square.boxed = true;
    square.x0 = 2;
    square.y0 = 2;
    square.x1 = 8;
    square.y1 = 8;
    const QString base = pictura::shape_add_layer(*view_, square, 0xffff0000u);
    const QString fill = view_->add_solid_fill(0xff00ff00u);
    QVERIFY(!base.isEmpty() && !fill.isEmpty());
    const QRgb green = 0xff00ff00u;
    const QRgb white = 0xffffffffu;
    QCOMPARE(view_->sample_argb(12, 12), green);
    // The panel picks the new rows up on its queued refresh.
    QCoreApplication::processEvents();

    pictura::CommandRegistry* registry = window_->registry();
    QAction* create = registry->action(QString::fromLatin1(pictura::command_ids::LayerCreateClippingMask));
    QAction* release =
        registry->action(QString::fromLatin1(pictura::command_ids::LayerReleaseClippingMask));
    QVERIFY(create && release);
    QCOMPARE(create->shortcut(), QKeySequence(QStringLiteral("Ctrl+Alt+G")));

    panel_->selectPaths({fill}, fill);
    registry->refresh();
    QVERIFY(create->isEnabled());
    QVERIFY(!release->isEnabled());
    int before = view_->history_count();
    create->trigger();
    QCOMPARE(view_->history_count(), before + 1);
    QCOMPARE(view_->history_label(view_->history_index()), QStringLiteral("Create Clipping Mask"));
    QVERIFY(view_->layer_row_clipping(rowOf(fill)));
    QCOMPARE(view_->sample_argb(4, 4), green);
    QCOMPARE(view_->sample_argb(12, 12), white);

    // Release from the base frees the layer clipped to it.
    QCoreApplication::processEvents();
    panel_->selectPaths({base}, base);
    registry->refresh();
    QVERIFY(release->isEnabled());
    release->trigger();
    QCOMPARE(view_->history_label(view_->history_index()), QStringLiteral("Release Clipping Mask"));
    QVERIFY(!view_->layer_row_clipping(rowOf(fill)));
    QCOMPARE(view_->sample_argb(12, 12), green);

    // Alt-click on the line between the rows clips, and again releases.
    QCoreApplication::processEvents();
    panel_->altClickBelowRowForTest(fill);
    QVERIFY(view_->layer_row_clipping(rowOf(fill)));
    QCOMPARE(view_->sample_argb(12, 12), white);
    QCoreApplication::processEvents();
    panel_->altClickBelowRowForTest(fill);
    QVERIFY(!view_->layer_row_clipping(rowOf(fill)));
}

QTEST_MAIN(LayersPanelTest)
#include "tst_layers_panel.moc"
