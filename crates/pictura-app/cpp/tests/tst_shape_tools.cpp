// The shape tools: Rectangle (#43), Rounded Rectangle (#44), Ellipse (#45),
// and Polygon (#46), in each of the Shape / Path / Pixels modes.

#include <QtTest/QtTest>

#include "panels/numeric_field.h"
#include "selftest_paint_fixture.h"

#include "pictura_app/src/cxxqt_object/paths.cxxqt.h"

#include <QtGui/QKeyEvent>
#include <QtWidgets/QComboBox>

#include "qt_test_support.h"

namespace {

using paint_fixture::Fixture;

void sendKey(pictura::PicturaMainWindow& frame, int key, Qt::KeyboardModifiers mods,
             const QString& text)
{
    QKeyEvent event(QEvent::KeyPress, key, mods, text);
    QApplication::sendEvent(&frame, &event);
}

// The active tool's own options-bar control: each tool has its own page.
template <typename T>
T* visibleChild(pictura::PicturaMainWindow& frame, const QString& name)
{
    for (T* child : frame.findChildren<T*>(name)) {
        if (child->isVisible()) {
            return child;
        }
    }
    return nullptr;
}

// The foreground colour a check changes, restored on scope exit; the shape
// options likewise.
struct State {
    pictura::ToolController* tools;
    QColor foreground;
    pictura::ShapeOptions shape;

    explicit State(pictura::ToolController* t)
        : tools(t)
        , foreground(t->foreground())
        , shape(t->shapeOptions())
    {
        tools->setForeground(Qt::red);
    }
    ~State()
    {
        tools->setForeground(foreground);
        tools->setShapeOptions(shape);
    }
};

const QRgb kRed = QColor(Qt::red).rgba();
const QRgb kWhite = QColor(Qt::white).rgba();

} // namespace

class ShapeToolsTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void shapeTools();

private:
    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void ShapeToolsTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void ShapeToolsTest::shapeTools()
{
    pictura::PicturaMainWindow& frame = *window_;
    QImage seed(100, 100, QImage::Format_RGB32);
    seed.fill(Qt::white);
    Fixture f(frame, seed, QStringLiteral("pictura_shape_seed"));
    QVERIFY2(f.ok(), "shape fixture");
    const State state(f.tools);

    // U and Shift+U cycle the four implemented tools of the group.
    frame.show();
    QVERIFY(QTest::qWaitForWindowExposed(&frame));
    frame.activateWindow();
    QCoreApplication::processEvents();
    frame.setActiveTool(pictura::ToolId::Move);
    sendKey(frame, Qt::Key_U, Qt::NoModifier, QStringLiteral("u"));
    QCOMPARE(frame.activeTool(), pictura::ToolId::Rectangle);
    const QList<pictura::ToolId> cycle = {pictura::ToolId::RoundedRectangle,
                                          pictura::ToolId::Ellipse, pictura::ToolId::Polygon,
                                          pictura::ToolId::Rectangle};
    for (pictura::ToolId next : cycle) {
        sendKey(frame, Qt::Key_U, Qt::ShiftModifier, QStringLiteral("U"));
        QCOMPARE(frame.activeTool(), next);
    }
    // The tools read Shift and Alt live; a plain key clears the tracked Shift.
    sendKey(frame, Qt::Key_U, Qt::NoModifier, QStringLiteral("u"));
    QCOMPARE(frame.activeTool(), pictura::ToolId::Rectangle);

    // Pixels: a hexagon from the centre out paints the foreground onto the
    // background, adding no layer.
    frame.setActiveTool(pictura::ToolId::Polygon);
    auto* sides = frame.findChild<pictura::NumericField*>(QStringLiteral("optionsShapeSides"));
    QVERIFY(sides != nullptr);
    emit sides->valueChanged(6);
    QCOMPARE(f.tools->shapeOptions().sides, 6);
    auto* mode = visibleChild<QComboBox>(frame, QStringLiteral("optionsShapeMode"));
    QVERIFY(mode != nullptr);
    mode->setCurrentIndex(2);
    QCOMPARE(f.tools->shapeOptions().mode, 2);
    const int layers = f.view->layer_count();
    int base = f.view->history_index();
    f.drag({QPointF(50, 50), QPointF(50, 30), QPointF(50, 20)});
    QVERIFY2(f.committedOnce(base, "Polygon Tool"), "polygon pixels");
    QCOMPARE(f.view->layer_count(), layers);
    QCOMPARE(f.view->sample_argb(50, 50), kRed);
    QCOMPARE(f.view->sample_argb(50, 22), kRed);
    QCOMPARE(f.view->sample_argb(50, 85), kWhite);

    // Path: an ellipse joins the Work Path as four anchors; no layer, no pixels.
    frame.setActiveTool(pictura::ToolId::Ellipse);
    mode->setCurrentIndex(1);
    for (auto* page : frame.findChildren<QComboBox*>(QStringLiteral("optionsShapeMode"))) {
        QCOMPARE(page->currentIndex(), 1);
    }
    base = f.view->history_index();
    f.drag({QPointF(60, 60), QPointF(80, 70), QPointF(90, 80)});
    QVERIFY2(f.committedOnce(base, "Ellipse Tool"), "ellipse path");
    QCOMPARE(pictura::path_subpath_count(*f.view), 1);
    QCOMPARE(pictura::path_point_count(*f.view, 0), 4);
    QVERIFY(pictura::path_subpath_closed(*f.view, 0));
    const ::rust::Vec<double> box = pictura::path_subpath_bounds(*f.view, 0);
    QCOMPARE(box.size(), std::size_t(4));
    QCOMPARE(QRectF(QPointF(box[0], box[1]), QPointF(box[2], box[3])),
             QRectF(60, 60, 30, 20));
    QCOMPARE(f.view->layer_count(), layers);
    QCOMPARE(f.view->sample_argb(75, 70), kWhite);

    // The Rounded Rectangle's Radius rounds its corners into eight anchors.
    frame.setActiveTool(pictura::ToolId::RoundedRectangle);
    auto* radius = frame.findChild<pictura::NumericField*>(QStringLiteral("optionsShapeRadius"));
    QVERIFY(radius != nullptr);
    QCOMPARE(radius->value(), 10.0);
    base = f.view->history_index();
    f.drag({QPointF(5, 60), QPointF(30, 80), QPointF(45, 95)});
    QVERIFY2(f.committedOnce(base, "Rounded Rectangle Tool"), "rounded rectangle path");
    QCOMPARE(pictura::path_point_count(*f.view, 1), 8);

    // Shape: a rectangle becomes a "Rectangle 1" fill layer cut to it. The
    // outline is previewed while dragging and cleared on release.
    frame.setActiveTool(pictura::ToolId::Rectangle);
    mode->setCurrentIndex(0);
    base = f.view->history_index();
    f.canvas->mousePressed(QPointF(10, 10), Qt::LeftButton, 0);
    f.canvas->mouseMoved(QPointF(30, 30));
    QVERIFY(f.canvas->pathOverlayHasPreviewForTest());
    f.canvas->mouseMoved(QPointF(40, 30));
    f.canvas->mouseReleased(QPointF(40, 30));
    QVERIFY(!f.canvas->pathOverlayHasPreviewForTest());
    QVERIFY2(f.committedOnce(base, "Rectangle Tool"), "rectangle shape");
    QCOMPARE(f.view->layer_count(), layers + 1);
    QCOMPARE(f.view->layer_name(layers), QStringLiteral("Rectangle 1"));
    QCOMPARE(f.view->sample_argb(25, 20), kRed);
    QCOMPARE(f.view->sample_argb(45, 20), kWhite);
    QCOMPARE(f.view->sample_argb(25, 32), kWhite);
    QCOMPARE(pictura::path_subpath_count(*f.view), 2);

    // A click draws nothing; undo removes the shape layer.
    base = f.view->history_index();
    f.drag({QPointF(70, 10)});
    QCOMPARE(f.view->history_index(), base);
    QVERIFY(f.view->undo());
    QCOMPARE(f.view->layer_count(), layers);
    QCOMPARE(f.view->sample_argb(25, 20), kWhite);
}

QTEST_MAIN(ShapeToolsTest)
#include "tst_shape_tools.moc"
