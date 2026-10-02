// The shape tools: Rectangle (#43), Rounded Rectangle (#44), Ellipse (#45),
// Polygon (#46), Line (#47), and Custom Shape (#48), in each of the Shape / Path / Pixels modes; live shapes
// and their conversion prompt; the Create dialogs; the Layers shape badge.

#include <QtTest/QtTest>

#include "panels/layers_panel.h"
#include "panels/numeric_field.h"
#include "selftest_paint_fixture.h"
#include "session.h"

#include "pictura_app/src/cxxqt_object/paths.cxxqt.h"

#include <QtCore/QTimer>
#include <QtGui/QKeyEvent>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDialog>
#include <QtWidgets/QDoubleSpinBox>
#include <QtWidgets/QMessageBox>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSpinBox>

#include <functional>
#include <memory>

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

// Answers the next modal dialog (polled from the dialog's own event loop)
// with `answer` and counts it in `*seen`. Destroy the timer once the action
// that may open the dialog has returned.
std::unique_ptr<QTimer> answerNextModal(std::function<void(QWidget*)> answer, int* seen)
{
    auto timer = std::make_unique<QTimer>();
    timer->setInterval(5);
    QTimer* raw = timer.get();
    QObject::connect(raw, &QTimer::timeout, [raw, answer, seen]() {
        if (QWidget* modal = QApplication::activeModalWidget()) {
            raw->stop();
            ++*seen;
            answer(modal);
        }
    });
    timer->start();
    return timer;
}

void pressButton(QWidget* modal, QMessageBox::StandardButton which, bool dontShowAgain = false)
{
    auto* box = qobject_cast<QMessageBox*>(modal);
    QVERIFY(box && box->objectName() == QStringLiteral("liveShapeToPathPrompt"));
    QVERIFY(box->text().contains(QStringLiteral("live shape into a regular path")));
    if (dontShowAgain) {
        box->checkBox()->setChecked(true);
    }
    box->button(which)->click();
}

// Fill in a Create dialog's numeric fields by object name, then OK it.
void fillCreateDialog(QWidget* modal, const QList<QPair<QString, double>>& fields,
                      const QStringList& checks = {})
{
    auto* dialog = qobject_cast<QDialog*>(modal);
    QVERIFY(dialog && dialog->objectName() == QStringLiteral("createShapeDialog"));
    for (const auto& [name, value] : fields) {
        if (auto* d = dialog->findChild<QDoubleSpinBox*>(name)) {
            d->setValue(value);
        } else if (auto* i = dialog->findChild<QSpinBox*>(name)) {
            i->setValue(int(value));
        } else {
            QFAIL(qPrintable(QStringLiteral("no field ") + name));
        }
    }
    for (const QString& name : checks) {
        auto* box = dialog->findChild<QCheckBox*>(name);
        QVERIFY(box);
        box->setChecked(true);
    }
    dialog->accept();
}

QRectF subpathBounds(const Fixture& f, int sp)
{
    const ::rust::Vec<double> b = pictura::path_subpath_bounds(*f.view, sp);
    return b.size() == 4 ? QRectF(QPointF(b[0], b[1]), QPointF(b[2], b[3])) : QRectF();
}

const QRgb kRed = QColor(Qt::red).rgba();
const QRgb kWhite = QColor(Qt::white).rgba();

} // namespace

class ShapeToolsTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void shapeTools();
    void liveShapes();
    void createDialogs();
    void lineAndCustomShape();

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

    // U and Shift+U cycle the six tools of the group.
    frame.show();
    QVERIFY(QTest::qWaitForWindowExposed(&frame));
    frame.activateWindow();
    QCoreApplication::processEvents();
    frame.setActiveTool(pictura::ToolId::Move);
    sendKey(frame, Qt::Key_U, Qt::NoModifier, QStringLiteral("u"));
    QCOMPARE(frame.activeTool(), pictura::ToolId::Rectangle);
    const QList<pictura::ToolId> cycle = {pictura::ToolId::RoundedRectangle,
                                          pictura::ToolId::Ellipse, pictura::ToolId::Polygon,
                                          pictura::ToolId::Line, pictura::ToolId::CustomShape,
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

    // The new layer's outline is drawn with its four corners, and the path
    // calls now read it; the Work Path keeps its two components. The Layers
    // panel marks the row as a shape layer.
    QCOMPARE(f.canvas->pathOverlayAnchorCountForTest(), 4);
    QCOMPARE(pictura::path_subpath_count(*f.view), 1);
    QVERIFY(pictura::path_target_is_live_shape(*f.view));
    pictura::path_set_layer_target(*f.view, false);
    QCOMPARE(pictura::path_subpath_count(*f.view), 2);
    pictura::path_set_layer_target(*f.view, true);
    auto* panel = frame.findChild<pictura::LayersPanel*>();
    QVERIFY(panel != nullptr);
    QVERIFY(panel->rowShapeForTest(f.view->active_layer_path()));
    QVERIFY(!panel->rowThumbnailForTest(f.view->active_layer_path()).isNull());

    // A click opens the Create dialog; cancelling it draws nothing. Undo
    // removes the shape layer.
    base = f.view->history_index();
    int seen = 0;
    {
        const auto cancel = answerNextModal(
            [](QWidget* modal) { qobject_cast<QDialog*>(modal)->reject(); }, &seen);
        f.drag({QPointF(70, 10)});
    }
    QCOMPARE(seen, 1);
    QCOMPARE(f.view->history_index(), base);
    QVERIFY(f.view->undo());
    QCOMPARE(f.view->layer_count(), layers);
    QCOMPARE(f.view->sample_argb(25, 20), kWhite);
}

void ShapeToolsTest::liveShapes()
{
    pictura::PicturaMainWindow& frame = *window_;
    QImage seed(100, 100, QImage::Format_RGB32);
    seed.fill(Qt::white);
    Fixture f(frame, seed, QStringLiteral("pictura_live_shape_seed"));
    QVERIFY2(f.ok(), "live shape fixture");
    const State state(f.tools);
    f.tools->setShapeOptions({0, 10.0, 5});

    // A Shape-mode rectangle is a live shape.
    frame.setActiveTool(pictura::ToolId::Rectangle);
    f.drag({QPointF(20, 20), QPointF(40, 40), QPointF(60, 50)});
    QVERIFY(pictura::path_target_is_live_shape(*f.view));

    // Path Selection moves it whole and it stays live.
    frame.setActiveTool(pictura::ToolId::PathSelection);
    int base = f.view->history_index();
    f.drag({QPointF(40, 35), QPointF(45, 35), QPointF(50, 40)});
    QVERIFY2(f.committedOnce(base, "Drag Path"), "move the live shape");
    QVERIFY(pictura::path_target_is_live_shape(*f.view));
    QCOMPARE(f.view->sample_argb(65, 52), kRed);
    QCOMPARE(f.view->sample_argb(80, 70), kWhite);

    // Direct Selection on a corner asks first; No leaves the shape alone.
    frame.setActiveTool(pictura::ToolId::DirectSelection);
    base = f.view->history_index();
    int seen = 0;
    {
        const auto no = answerNextModal(
            [](QWidget* m) { pressButton(m, QMessageBox::No); }, &seen);
        f.drag({QPointF(70, 55), QPointF(75, 60), QPointF(90, 80)});
    }
    QCOMPARE(seen, 1);
    QCOMPARE(f.view->history_index(), base);
    QVERIFY(pictura::path_target_is_live_shape(*f.view));
    QCOMPARE(f.view->sample_argb(80, 70), kWhite);

    // Yes turns it into a regular path. The prompt took the button release,
    // so that drag ends there; the next one reshapes it without asking.
    {
        const auto yes = answerNextModal(
            [](QWidget* m) { pressButton(m, QMessageBox::Yes); }, &seen);
        f.drag({QPointF(70, 55), QPointF(75, 60), QPointF(90, 80)});
    }
    QCOMPARE(seen, 2);
    QVERIFY(!pictura::path_target_is_live_shape(*f.view));
    {
        const auto unexpected = answerNextModal(
            [](QWidget* m) { pressButton(m, QMessageBox::No); }, &seen);
        f.drag({QPointF(70, 55), QPointF(75, 60), QPointF(90, 80)});
    }
    QCOMPARE(seen, 2);
    QVERIFY2(f.committedOnce(base, "Drag Anchor Point"), "reshape the converted shape");
    QCOMPARE(f.view->sample_argb(80, 70), kRed);

    // Undoing the reshape brings back the live shape.
    QVERIFY(f.view->undo());
    QVERIFY(pictura::path_target_is_live_shape(*f.view));
    QCOMPARE(f.view->sample_argb(80, 70), kWhite);

    // "Don't show again" is remembered: the next live shape converts and
    // reshapes in one drag, without asking.
    {
        const auto yes = answerNextModal(
            [](QWidget* m) { pressButton(m, QMessageBox::Yes, true); }, &seen);
        f.drag({QPointF(70, 55), QPointF(75, 60), QPointF(90, 80)});
    }
    QCOMPARE(seen, 3);
    QVERIFY(!pictura::loadSession().confirmLiveShapeToPath);
    frame.setActiveTool(pictura::ToolId::Ellipse);
    f.drag({QPointF(10, 60), QPointF(30, 80), QPointF(50, 90)});
    QVERIFY(pictura::path_target_is_live_shape(*f.view));
    frame.setActiveTool(pictura::ToolId::DirectSelection);
    base = f.view->history_index();
    {
        const auto unexpected = answerNextModal(
            [](QWidget* m) { pressButton(m, QMessageBox::No); }, &seen);
        f.drag({QPointF(50, 75), QPointF(55, 75), QPointF(60, 75)});
    }
    QCOMPARE(seen, 3);
    QVERIFY2(f.committedOnce(base, "Drag Anchor Point"), "reshape without asking");
    QVERIFY(!pictura::path_target_is_live_shape(*f.view));
    QCOMPARE(f.view->sample_argb(55, 75), kRed);

    pictura::SessionState session = pictura::loadSession();
    session.confirmLiveShapeToPath = true;
    QVERIFY(pictura::saveSession(session));
}

void ShapeToolsTest::createDialogs()
{
    pictura::PicturaMainWindow& frame = *window_;
    QImage seed(100, 100, QImage::Format_RGB32);
    seed.fill(Qt::white);
    Fixture f(frame, seed, QStringLiteral("pictura_create_shape_seed"));
    QVERIFY2(f.ok(), "create dialog fixture");
    const State state(f.tools);
    int seen = 0;

    // Shape mode: a 30 x 20 rectangle with its top-left corner at the click.
    f.tools->setShapeOptions({0, 10.0, 5});
    frame.setActiveTool(pictura::ToolId::Rectangle);
    int base = f.view->history_index();
    {
        const auto ok = answerNextModal(
            [](QWidget* m) {
                QCOMPARE(m->windowTitle(), QStringLiteral("Create Rectangle"));
                fillCreateDialog(m, {{QStringLiteral("createShapeWidth"), 30},
                                     {QStringLiteral("createShapeHeight"), 20}});
            },
            &seen);
        f.drag({QPointF(10, 10)});
    }
    QCOMPARE(seen, 1);
    QVERIFY2(f.committedOnce(base, "Rectangle Tool"), "create rectangle");
    QCOMPARE(f.view->sample_argb(35, 25), kRed);
    QCOMPARE(f.view->sample_argb(45, 25), kWhite);
    QCOMPARE(f.view->sample_argb(35, 35), kWhite);

    // Path mode from here on, so the Work Path shows what each dialog drew.
    f.tools->setShapeOptions({1, 10.0, 5});

    // From Center puts the ellipse's centre at the click.
    frame.setActiveTool(pictura::ToolId::Ellipse);
    {
        const auto ok = answerNextModal(
            [](QWidget* m) {
                QCOMPARE(m->windowTitle(), QStringLiteral("Create Ellipse"));
                fillCreateDialog(m,
                                 {{QStringLiteral("createShapeWidth"), 40},
                                  {QStringLiteral("createShapeHeight"), 20}},
                                 {QStringLiteral("createShapeFromCenter")});
            },
            &seen);
        f.drag({QPointF(50, 50)});
    }
    QCOMPARE(pictura::path_subpath_count(*f.view), 1);
    QCOMPARE(subpathBounds(f, 0), QRectF(30, 40, 40, 20));

    // Each corner takes its own radius: a square top-left corner is one anchor.
    frame.setActiveTool(pictura::ToolId::RoundedRectangle);
    {
        const auto ok = answerNextModal(
            [](QWidget* m) {
                QCOMPARE(m->windowTitle(), QStringLiteral("Create Rounded Rectangle"));
                fillCreateDialog(m, {{QStringLiteral("createShapeWidth"), 40},
                                     {QStringLiteral("createShapeHeight"), 40},
                                     {QStringLiteral("createShapeRadiusTopLeft"), 0},
                                     {QStringLiteral("createShapeRadiusBottomRight"), 15}});
            },
            &seen);
        f.drag({QPointF(5, 55)});
    }
    QCOMPARE(pictura::path_point_count(*f.view, 1), 7);
    QCOMPARE(subpathBounds(f, 1), QRectF(5, 55, 40, 40));

    // A five-pointed star fills its box with ten anchors.
    frame.setActiveTool(pictura::ToolId::Polygon);
    {
        const auto ok = answerNextModal(
            [](QWidget* m) {
                QCOMPARE(m->windowTitle(), QStringLiteral("Create Polygon"));
                auto* indent = m->findChild<QDoubleSpinBox*>(QStringLiteral("createShapeIndent"));
                QVERIFY(indent && !indent->isEnabled());
                fillCreateDialog(m,
                                 {{QStringLiteral("createShapeWidth"), 30},
                                  {QStringLiteral("createShapeHeight"), 30},
                                  {QStringLiteral("createShapeSides"), 5}},
                                 {QStringLiteral("createShapeStar")});
            },
            &seen);
        f.drag({QPointF(60, 10)});
    }
    QCOMPARE(seen, 4);
    QCOMPARE(pictura::path_point_count(*f.view, 2), 10);
    const QRectF star = subpathBounds(f, 2);
    QVERIFY2(qAbs(star.left() - 60) < 0.01 && qAbs(star.top() - 10) < 0.01
                 && qAbs(star.width() - 30) < 0.01 && qAbs(star.height() - 30) < 0.01,
             "the star fills its box");
}

void ShapeToolsTest::lineAndCustomShape()
{
    pictura::PicturaMainWindow& frame = *window_;
    QImage seed(100, 100, QImage::Format_RGB32);
    seed.fill(Qt::white);
    Fixture f(frame, seed, QStringLiteral("pictura_line_custom_seed"));
    QVERIFY2(f.ok(), "line and custom shape fixture");
    const State state(f.tools);
    int seen = 0;

    // Pixels: a 4 px line paints a band 4 px tall.
    frame.setActiveTool(pictura::ToolId::Line);
    auto* weight = visibleChild<pictura::NumericField>(frame, QStringLiteral("optionsShapeWeight"));
    QVERIFY(weight != nullptr);
    QCOMPARE(weight->value(), 1.0);
    emit weight->valueChanged(4);
    pictura::ShapeOptions o = f.tools->shapeOptions();
    QCOMPARE(o.weight, 4.0);
    o.mode = 2;
    f.tools->setShapeOptions(o);
    int base = f.view->history_index();
    f.drag({QPointF(10, 50), QPointF(50, 50), QPointF(90, 50)});
    QVERIFY2(f.committedOnce(base, "Line Tool"), "line pixels");
    QCOMPARE(f.view->sample_argb(50, 49), kRed);
    QCOMPARE(f.view->sample_argb(50, 51), kRed);
    QCOMPARE(f.view->sample_argb(50, 46), kWhite);

    // The Line has no Create dialog: a click draws nothing.
    base = f.view->history_index();
    {
        const auto unexpected = answerNextModal(
            [](QWidget* modal) { qobject_cast<QDialog*>(modal)->reject(); }, &seen);
        f.drag({QPointF(30, 80)});
    }
    QCOMPARE(seen, 0);
    QCOMPARE(f.view->history_index(), base);

    // An end arrowhead, switched on in the pop-up, adds a tip: seven anchors.
    auto* end = frame.findChild<QCheckBox*>(QStringLiteral("optionsShapeArrowEnd"));
    QVERIFY(end != nullptr);
    end->setChecked(true);
    QVERIFY(f.tools->shapeOptions().arrowEnd);
    o = f.tools->shapeOptions();
    o.mode = 1;
    f.tools->setShapeOptions(o);
    f.drag({QPointF(10, 20), QPointF(50, 20), QPointF(90, 20)});
    QCOMPARE(pictura::path_subpath_count(*f.view), 1);
    QCOMPARE(pictura::path_point_count(*f.view, 0), 7);
    const QRectF line = subpathBounds(f, 0);
    QVERIFY2(qAbs(line.right() - 90) < 1e-6 && qAbs(line.height() - 20) < 1e-6,
             "the tip reaches the release point; the head is 500 % of 4 px wide");

    // The Custom Shape picker lists the built-in shapes with silhouettes.
    frame.setActiveTool(pictura::ToolId::CustomShape);
    auto* picker = visibleChild<QComboBox>(frame, QStringLiteral("optionsShapeCustom"));
    QVERIFY(picker != nullptr);
    QCOMPARE(picker->count(), 6);
    QCOMPARE(picker->itemText(1), QStringLiteral("Heart"));
    QVERIFY(!picker->itemIcon(1).isNull());
    picker->setCurrentIndex(1);
    QCOMPARE(f.tools->shapeOptions().custom, 1);

    // Shape mode: the heart lands on a "Shape 1" layer, not a live shape.
    o = f.tools->shapeOptions();
    o.mode = 0;
    f.tools->setShapeOptions(o);
    base = f.view->history_index();
    f.drag({QPointF(20, 60), QPointF(50, 80), QPointF(80, 95)});
    QVERIFY2(f.committedOnce(base, "Shape Tool"), "custom shape layer");
    QCOMPARE(f.view->layer_name(f.view->layer_count() - 1), QStringLiteral("Shape 1"));
    QCOMPARE(f.view->sample_argb(50, 80), kRed);
    QCOMPARE(f.view->sample_argb(22, 62), kWhite);
    QVERIFY(!pictura::path_target_is_live_shape(*f.view));

    // A click opens Create Custom Shape.
    {
        const auto ok = answerNextModal(
            [](QWidget* m) {
                QCOMPARE(m->windowTitle(), QStringLiteral("Create Custom Shape"));
                fillCreateDialog(m, {{QStringLiteral("createShapeWidth"), 20},
                                     {QStringLiteral("createShapeHeight"), 20}});
            },
            &seen);
        f.drag({QPointF(5, 5)});
    }
    QCOMPARE(seen, 1);
    QCOMPARE(f.view->layer_name(f.view->layer_count() - 1), QStringLiteral("Shape 2"));
    QCOMPARE(f.view->sample_argb(15, 16), kRed);
}

QTEST_MAIN(ShapeToolsTest)
#include "tst_shape_tools.moc"
