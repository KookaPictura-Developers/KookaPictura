#include <QtTest/QtTest>

#include <QtCore/QAbstractItemModel>
#include <QtCore/QMimeData>
#include <QtCore/QModelIndex>
#include <QtCore/QPoint>
#include <QtCore/QRect>
#include <QtCore/QStringList>
#include <QtCore/QTimer>
#include <QtGui/QAction>
#include <QtGui/QColor>
#include <QtGui/QDragEnterEvent>
#include <QtGui/QDragMoveEvent>
#include <QtGui/QDropEvent>
#include <QtGui/QImage>
#include <QtGui/QPalette>
#include <QtWidgets/QTabBar>

#include "file_drop_router.h"
#include "frame.h"
#include "image_view.h"
#include "panels/layers_panel.h"
#include "panels/layers_panel_internal.h"
#include "theme.h"
#include "commands.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/shapes.cxxqt.h"

#include "qt_test_support.h"

namespace {

QModelIndex indexForPath(QAbstractItemModel* model, const QString& path,
                         const QModelIndex& parent = QModelIndex())
{
    if (!model) {
        return {};
    }
    for (int row = 0; row < model->rowCount(parent); ++row) {
        const QModelIndex index = model->index(row, 0, parent);
        if (index.data(pictura::PathRole).toString() == path) {
            return index;
        }
        const QModelIndex child = indexForPath(model, path, index);
        if (child.isValid()) {
            return child;
        }
    }
    return {};
}

bool dropOnViewport(QWidget* viewport, const void* document, const QString& source,
                    const QPoint& pos)
{
    QMimeData mime;
    mime.setData(pictura::kLayerMimeType, source.toUtf8());
    mime.setData(pictura::kLayerSourceMimeType,
                 QByteArray::number(reinterpret_cast<quintptr>(document)));
    QDragEnterEvent enter(pos, Qt::MoveAction, &mime, Qt::LeftButton, Qt::NoModifier);
    QCoreApplication::sendEvent(viewport, &enter);
    QDragMoveEvent move(pos, Qt::MoveAction, &mime, Qt::LeftButton, Qt::NoModifier);
    QCoreApplication::sendEvent(viewport, &move);
    QDropEvent drop(QPointF(pos), Qt::MoveAction, &mime, Qt::LeftButton, Qt::NoModifier);
    QCoreApplication::sendEvent(viewport, &drop);
    return drop.isAccepted();
}

// Send a layer drag from `document` over `target` at `pos`; with `drop`, release
// it there. Returns whether the last event was accepted.
bool dragLayerOver(QWidget* target, const void* document, const QString& path,
                   const QPoint& pos, bool drop)
{
    QMimeData mime;
    mime.setData(pictura::kLayerMimeType, path.toUtf8());
    mime.setData(pictura::kLayerSourceMimeType,
                 QByteArray::number(reinterpret_cast<quintptr>(document)));
    const auto actions = Qt::MoveAction | Qt::CopyAction;
    QDragEnterEvent enter(pos, actions, &mime, Qt::LeftButton, Qt::NoModifier);
    QCoreApplication::sendEvent(target, &enter);
    QDragMoveEvent move(pos, actions, &mime, Qt::LeftButton, Qt::NoModifier);
    QCoreApplication::sendEvent(target, &move);
    if (!drop) {
        return move.isAccepted();
    }
    QDropEvent release(QPointF(pos), actions, &mime, Qt::LeftButton, Qt::NoModifier);
    QCoreApplication::sendEvent(target, &release);
    return release.isAccepted();
}

int countNamed(pictura::PictureView* view, const QString& name)
{
    int count = 0;
    for (int i = 0; i < view->layer_row_count(); ++i) {
        count += view->layer_row_name(i) == name ? 1 : 0;
    }
    return count;
}

} // namespace

class LayersPanelTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void cleanup();
    void nestingLockRefusalAndReorder();
    void panelChrome();
    void rowWidgets();
    void layerRowSurface();
    void gutterClickTogglesWithoutSelecting();
    void dragReorder();
    void dropAnywhereOnLayerRowReorders();
    void dropOnGroupRowEdgesPlacesSibling();
    void dragLayerOntoAnotherDocument();
    void moveToolFindsAnotherDocumentTab();
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

void LayersPanelTest::layerRowSurface()
{
    QVERIFY2(setupNest(), "row surface fixture");
    const int doc = window_->activeDocumentIndex();
    // The row image seams read the painted row, so the tree must be laid out.
    window_->show();
    panel_->setView(view_);
    panel_->refresh();
    panel_->expandForTest(group_);
    QTest::qWait(50);

    // The paint helper seeds the palette with a fixed neutral grey, so the
    // exact surfaces are deterministic without depending on the live palette.
    const QColor canvasBase(128, 128, 128);
    const QColor rowSurface = pictura::Theme::shade(canvasBase, 1);

    panel_->selectPaths({}, QString());
    const QImage unselected = panel_->rowImageForTest(a_);
    const QColor bg = unselected.pixelColor(unselected.width() - 4, 1);
    QCOMPARE(bg.rgba(), rowSurface.rgba());
    QVERIFY2(bg.rgba() != canvasBase.rgba(), "row surface steps lighter than the list");

    // The content column begins kContentPad past the gutter; the name shares
    // that left, and the 4 px name gap (kNameGap) follows the thumbnail.
    const QRect thumb = panel_->rowThumbRectForTest(a_);
    const QRect name = panel_->rowNameRectForTest(a_);
    QCOMPARE(thumb.left(), pictura::LayerRowDelegate::kEyeColumn
                               + pictura::LayerRowDelegate::kContentPad
                               + pictura::LayerRowDelegate::kIndent);
    QCOMPARE(name.left(), thumb.left() + thumb.width()
                              + pictura::LayerRowDelegate::kNameGap);

    // The paint helper seeds Highlight with blue; the delegate must paint the
    // neutral grey list-selection fill instead, and keep the eye gutter on the
    // row surface so a selected row's gutter matches an unselected one.
    panel_->selectPaths({a_}, a_);
    const QImage selected = panel_->rowImageForTest(a_);
    const QColor expectedSelection = pictura::Theme::shade(canvasBase, 3);
    const QColor selPixel = selected.pixelColor(selected.width() - 4, 1);
    QCOMPARE(selPixel.rgba(), expectedSelection.rgba());
    QVERIFY2(selPixel.rgba() != bg.rgba(), "selection changes the row surface");
    QVERIFY2(selPixel.red() == selPixel.green() && selPixel.green() == selPixel.blue(),
             "selection uses grey, not the palette blue");
    const QRect eye = panel_->rowEyeRectForTest(a_);
    QCOMPARE(selected.pixel(eye.left() + 2, eye.top() + 1),
             unselected.pixel(eye.left() + 2, eye.top() + 1));

    window_->closeDocument(doc, false);
}

void LayersPanelTest::gutterClickTogglesWithoutSelecting()
{
    QVERIFY2(setupNest(), "gutter fixture");
    const int doc = window_->activeDocumentIndex();
    panel_->setView(view_);
    panel_->refresh();
    panel_->expandForTest(group_);
    window_->show();
    QTest::qWait(50);

    QTreeView* tree = panel_->findChild<QTreeView*>();
    QVERIFY(tree != nullptr);
    QWidget* viewport = tree->viewport();
    QVERIFY(viewport != nullptr);
    const auto rowRect = [&]() -> QRect { return panel_->rowViewportRectForTest(a_); };

    const int kEyeColumn = pictura::LayerRowDelegate::kEyeColumn;
    QRect vr = rowRect();
    QVERIFY2(vr.isValid() && vr.width() > kEyeColumn, "layer A row is laid out");

    panel_->selectPaths({b_}, b_);
    const bool before = view_->layer_row_visible(rowOf(a_));
    QCOMPARE(panel_->selectedPaths(), QStringList{b_});

    // Re-read the row geometry: selecting b_ may scroll the view, which moves
    // layer A's rect.
    vr = rowRect();
    QVERIFY2(vr.isValid() && vr.width() > kEyeColumn, "layer A row is laid out after select");

    // The whole left gutter toggles visibility and never selects: the first
    // column pixel and the last gutter pixel before the padded content both
    // work.
    QTest::mouseClick(viewport, Qt::LeftButton, Qt::NoModifier,
                      QPoint(vr.left(), vr.center().y()));
    QCoreApplication::processEvents();
    QVERIFY2(view_->layer_row_visible(rowOf(a_)) != before, "gutter left edge toggles visibility");
    QCOMPARE(panel_->selectedPaths(), QStringList{b_});

    // Restore, then the last gutter pixel toggles identically.
    view_->set_layers_visible({a_}, before);
    QCoreApplication::processEvents();
    vr = rowRect();
    QVERIFY2(vr.isValid(), "layer A row still laid out");
    QTest::mouseClick(viewport, Qt::LeftButton, Qt::NoModifier,
                      QPoint(vr.left() + kEyeColumn - 1, vr.center().y()));
    QCoreApplication::processEvents();
    QVERIFY2(view_->layer_row_visible(rowOf(a_)) != before, "gutter last column toggles visibility");
    QCOMPARE(panel_->selectedPaths(), QStringList{b_});

    // The padded content column just past the gutter selects instead of toggling.
    vr = rowRect();
    QVERIFY2(vr.isValid(), "layer A row still laid out");
    const bool visibleBeforeContent = view_->layer_row_visible(rowOf(a_));
    QTest::mouseClick(viewport, Qt::LeftButton, Qt::NoModifier,
                      QPoint(vr.left() + kEyeColumn, vr.center().y()));
    QCoreApplication::processEvents();
    QVERIFY2(view_->layer_row_visible(rowOf(a_)) == visibleBeforeContent,
             "content click leaves visibility alone");
    QVERIFY2(panel_->selectedPaths().contains(a_), "content click selects the row");

    // Right-clicking the gutter opens the eye menu but must not disturb the
    // selection. The old glyph-only guard fell through to the row menu and
    // re-selected the row; the widened gutter guard does not. Emit the
    // custom-context-menu signal the tree emits on a real right-click and close
    // the menu at once so its nested event loop returns headlessly.
    panel_->selectPaths({b_}, b_);
    QTimer::singleShot(0, qApp, []() {
        if (QWidget* popup = QApplication::activePopupWidget()) {
            popup->close();
        }
    });
    vr = rowRect();
    QVERIFY2(vr.isValid(), "layer A row still laid out");
    QMetaObject::invokeMethod(tree, "customContextMenuRequested", Qt::DirectConnection,
                              Q_ARG(QPoint, QPoint(vr.left() + 1, vr.center().y())));
    QCoreApplication::processEvents();
    QCOMPARE(panel_->selectedPaths(), QStringList{b_});

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

// A real drag lets go somewhere inside a row, not on its 2 px edge. A plain
// layer row is a sibling target across its whole height: the upper half drops
// above it, the lower half below it.
void LayersPanelTest::dropAnywhereOnLayerRowReorders()
{
    const bool created = window_->newDocument(QStringLiteral("DropRow"), 16, 16,
                                              QStringLiteral("rgb"), 8, QStringLiteral("white"));
    view_ = window_->activeView();
    panel_ = window_->findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
    QVERIFY2(created && view_ && panel_, "drop row fixture");
    const int doc = window_->activeDocumentIndex();
    const QString a = view_->add_layer_in(QString());
    const QString b = view_->add_layer_in(QString());
    view_->set_layer_name_path(a, QStringLiteral("A"));
    view_->set_layer_name_path(b, QStringLiteral("B"));
    panel_->setView(view_);
    panel_->refresh();
    window_->show();
    QTest::qWait(50);
    QTreeView* tree = panel_->findChild<QTreeView*>();
    QVERIFY(tree != nullptr);

    QRect row = panel_->rowViewportRectForTest(b);
    QVERIFY2(row.height() > 8, "layer B row is laid out");
    const int base = view_->history_count();
    QVERIFY2(dropOnViewport(tree->viewport(), view_, a,
                            QPoint(row.center().x(), row.top() + row.height() / 4)),
             "upper-half drop accepted");
    QCOMPARE(view_->layer_row_name(rowOf(QStringLiteral("2"))), QStringLiteral("A"));
    QCOMPARE(view_->history_count(), base + 1);

    panel_->refresh();
    QCoreApplication::processEvents();
    row = panel_->rowViewportRectForTest(QStringLiteral("1"));
    QVERIFY2(dropOnViewport(tree->viewport(), view_, QStringLiteral("2"),
                            QPoint(row.center().x(), row.bottom() - row.height() / 4)),
             "lower-half drop accepted");
    QCOMPARE(view_->layer_row_name(rowOf(QStringLiteral("1"))), QStringLiteral("A"));
    QCOMPARE(view_->layer_row_name(rowOf(QStringLiteral("2"))), QStringLiteral("B"));
    window_->closeDocument(doc, false);
}

// A group row keeps its centre for drop-into; its upper and lower quarters are
// sibling targets. Drops well inside those quarters (past the old 2 px edge)
// must place the layer above or below the group, never inside it.
void LayersPanelTest::dropOnGroupRowEdgesPlacesSibling()
{
    const bool created = window_->newDocument(QStringLiteral("DropGroup"), 16, 16,
                                              QStringLiteral("rgb"), 8, QStringLiteral("white"));
    view_ = window_->activeView();
    panel_ = window_->findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
    QVERIFY2(created && view_ && panel_, "drop group fixture");
    const int doc = window_->activeDocumentIndex();
    const QString layer = view_->add_layer_in(QString());
    const QString group = view_->add_group_in(QString());
    view_->set_layer_name_path(layer, QStringLiteral("L"));
    view_->set_layer_name_path(group, QStringLiteral("G"));
    panel_->setView(view_);
    panel_->refresh();
    window_->show();
    QTest::qWait(50);
    QTreeView* tree = panel_->findChild<QTreeView*>();
    QVERIFY(tree != nullptr);

    QRect row = panel_->rowViewportRectForTest(QStringLiteral("2"));
    QVERIFY2(row.height() >= 16, "group row is laid out");
    const int base = view_->history_count();
    QVERIFY2(dropOnViewport(tree->viewport(), view_, QStringLiteral("1"),
                            QPoint(row.center().x(), row.top() + row.height() / 8)),
             "upper-quarter drop accepted");
    QCOMPARE(view_->layer_row_name(rowOf(QStringLiteral("2"))), QStringLiteral("L"));
    QCOMPARE(view_->layer_row_name(rowOf(QStringLiteral("1"))), QStringLiteral("G"));
    QCOMPARE(view_->history_count(), base + 1);

    panel_->refresh();
    QCoreApplication::processEvents();
    row = panel_->rowViewportRectForTest(QStringLiteral("1"));
    QVERIFY2(dropOnViewport(tree->viewport(), view_, QStringLiteral("2"),
                            QPoint(row.center().x(), row.bottom() - row.height() / 8)),
             "lower-quarter drop accepted");
    QCOMPARE(view_->layer_row_name(rowOf(QStringLiteral("1"))), QStringLiteral("L"));
    QCOMPARE(view_->layer_row_name(rowOf(QStringLiteral("2"))), QStringLiteral("G"));
    QCOMPARE(view_->history_count(), base + 2);
    window_->closeDocument(doc, false);
}

// Issue #231: a layer dragged onto another document's tab brings that document
// forward and copies the layer in; its canvas takes the drop too. The source is
// untouched, and the other document's panel never treats the drag as its own.
void LayersPanelTest::dragLayerOntoAnotherDocument()
{
    QVERIFY(window_->newDocument(QStringLiteral("Src"), 16, 16, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    pictura::PictureView* src = window_->activeView();
    const QString sky = src->add_layer_in(QString());
    src->set_layer_name_path(sky, QStringLiteral("Sky"));
    src->set_layers_opacity(QStringList{sky}, 40);
    QVERIFY(window_->newDocument(QStringLiteral("Dst"), 16, 16, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    pictura::PictureView* dst = window_->activeView();
    QVERIFY(window_->newDocument(QStringLiteral("Gray"), 16, 16, QStringLiteral("grayscale"), 8,
                                 QStringLiteral("white")));
    pictura::PictureView* gray = window_->activeView();
    window_->setActiveDocumentIndex(0);
    window_->show();
    QTest::qWait(50);
    auto* bar = window_->findChild<QTabBar*>(QStringLiteral("documentTabBar"));
    QVERIFY(bar != nullptr && bar->count() == 3);
    const int srcRows = src->layer_row_count();
    const int base = dst->history_count();

    QVERIFY2(!dragLayerOver(bar, src, sky, bar->tabRect(0).center(), true),
             "a drop on its own document's tab copies nothing");
    QCOMPARE(src->layer_row_count(), srcRows);

    QVERIFY2(dragLayerOver(bar, src, sky, bar->tabRect(1).center(), false),
             "hovering another tab accepts the drag");
    QCOMPARE(window_->activeView(), dst);
    QVERIFY2(dragLayerOver(bar, src, sky, bar->tabRect(1).center(), true), "tab drop accepted");
    QCOMPARE(countNamed(dst, QStringLiteral("Sky")), 1);
    QCOMPARE(dst->history_count(), base + 1);
    view_ = dst;
    panel_ = window_->findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
    QVERIFY(panel_ != nullptr);
    const QString copied = panel_->currentPath();
    QCOMPARE(dst->layer_row_name(rowOf(copied)), QStringLiteral("Sky"));
    view_ = src;
    const int srcOpacity = src->layer_row_opacity(rowOf(sky));
    view_ = dst;
    QVERIFY(srcOpacity < 100);
    QCOMPARE(dst->layer_row_opacity(rowOf(copied)), srcOpacity);
    QCOMPARE(src->layer_row_count(), srcRows);

    QWidget* canvas = window_->canvasAt(1);
    QVERIFY2(dragLayerOver(canvas, src, sky, canvas->rect().center(), true),
             "canvas drop accepted");
    QCOMPARE(countNamed(dst, QStringLiteral("Sky")), 2);

    const int dstRows = dst->layer_row_count();
    QTreeView* tree = panel_->findChild<QTreeView*>();
    QVERIFY(tree != nullptr);
    QVERIFY2(!dropOnViewport(tree->viewport(), src, QStringLiteral("1"),
                             tree->viewport()->rect().center()),
             "another document's drag is not a drop on this panel");
    QCOMPARE(dst->layer_row_count(), dstRows);

    // Back over the source tab after another document came forward: the drop
    // is the own-document no-op, not a copy into the last-hovered document.
    QVERIFY(dragLayerOver(bar, src, sky, bar->tabRect(1).center(), false));
    QVERIFY2(!dragLayerOver(bar, src, sky, bar->tabRect(0).center(), true),
             "returning to the source tab copies nothing");
    QCOMPARE(window_->activeView(), src);
    QCOMPARE(countNamed(dst, QStringLiteral("Sky")), 2);
    QCOMPARE(src->layer_row_count(), srcRows);

    // Empty tab-bar space is no target, whatever document is active.
    QVERIFY(dragLayerOver(bar, src, sky, bar->tabRect(1).center(), false));
    const QPoint pastTabs(bar->tabRect(bar->count() - 1).right() + 4, bar->height() / 2);
    if (bar->tabAt(pastTabs) < 0 && bar->rect().contains(pastTabs)) {
        QVERIFY2(!dragLayerOver(bar, src, sky, pastTabs, true),
                 "a drop past the last tab copies nothing");
        QCOMPARE(countNamed(dst, QStringLiteral("Sky")), 2);
    }

    QVERIFY2(!dragLayerOver(bar, src, sky, bar->tabRect(2).center(), true),
             "a Grayscale document refuses an RGB layer");
    QCOMPARE(window_->activeView(), gray);
    QCOMPARE(countNamed(gray, QStringLiteral("Sky")), 0);
}

// Issue #231: a Move-tool drag hands off to the layer drag once the pointer is
// over another document's tab; the current tab and the canvas keep the move.
void LayersPanelTest::moveToolFindsAnotherDocumentTab()
{
    QVERIFY(window_->newDocument(QStringLiteral("MoveSrc"), 16, 16, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    const int src = window_->activeDocumentIndex();
    QVERIFY(window_->newDocument(QStringLiteral("MoveDst"), 16, 16, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    const int dst = window_->activeDocumentIndex();
    window_->setActiveDocumentIndex(src);
    window_->show();
    QVERIFY(QTest::qWaitForWindowExposed(window_.get()));
    auto* bar = window_->findChild<QTabBar*>(QStringLiteral("documentTabBar"));
    QVERIFY(bar != nullptr);

    using pictura::FileDropRouter;
    QCOMPARE(FileDropRouter::otherDocumentTabAt(bar->mapToGlobal(bar->tabRect(dst).center())),
             dst);
    QCOMPARE(FileDropRouter::otherDocumentTabAt(bar->mapToGlobal(bar->tabRect(src).center())),
             -1);
    QWidget* canvas = window_->canvasAt(src);
    QCOMPARE(FileDropRouter::otherDocumentTabAt(canvas->mapToGlobal(canvas->rect().center())), -1);
    window_->closeDocument(dst, false);
    window_->closeDocument(src, false);
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
