// The Type tools: Horizontal Type (#37), Vertical Type (#38), Horizontal Type
// Mask (#39), and Vertical Type Mask (#40).

#include <QtTest/QtTest>

#include "panels/layers_panel.h"
#include "type_fonts.h"
#include "type_text_edit.h"
#include "panels/numeric_field.h"
#include "selftest_paint_fixture.h"

#include "pictura_app/src/cxxqt_object/type_tools.cxxqt.h"

#include <QtGui/QClipboard>
#include <QtGui/QFontDatabase>
#include <QtGui/QFontInfo>
#include <QtGui/QKeyEvent>
#include <QtWidgets/QToolButton>

#include "qt_test_support.h"

namespace {

using paint_fixture::Fixture;

QRect layerRect(const Fixture& f, const QString& path)
{
    const QStringList parts = f.view->layer_rect(path).split(QLatin1Char(' '));
    if (parts.size() != 4) {
        return QRect();
    }
    return QRect(QPoint(parts[0].toInt(), parts[1].toInt()),
                 QPoint(parts[2].toInt() - 1, parts[3].toInt() - 1));
}

// The first type layer's path among the top-level layers, or empty.
QString typeLayer(const Fixture& f)
{
    for (int i = 0; i < f.view->layer_count(); ++i) {
        if (f.view->layer_is_type(QString::number(i))) {
            return QString::number(i);
        }
    }
    return QString();
}

QImage seed()
{
    QImage image(200, 160, QImage::Format_RGB32);
    image.fill(Qt::white);
    return image;
}

} // namespace

class TypeToolsTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void horizontalTypeTool();
    void typeKeysAndCancel();
    void reopenTypeLayer();
    void freeTransformKeepsType();
    void textEditModel();
    void editLikeATextField();
    void barRestylesTheSelectedLayer();
    void verticalTypeTool();
    void typeMaskTools();
    void typeOptionsBar();

private:
    void showFrame();

    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void TypeToolsTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void TypeToolsTest::showFrame()
{
    window_->show();
    QVERIFY(QTest::qWaitForWindowExposed(window_.get()));
    window_->activateWindow();
    QCoreApplication::processEvents();
}

void TypeToolsTest::horizontalTypeTool()
{
    pictura::PicturaMainWindow& frame = *window_;
    Fixture f(frame, seed(), QStringLiteral("pictura_type_seed"));
    QVERIFY2(f.ok(), "type fixture");
    showFrame();

    // T and Shift+T cycle all four Type tools.
    const auto sendKey = [&frame](int key, Qt::KeyboardModifiers mods, const QString& text) {
        QKeyEvent event(QEvent::KeyPress, key, mods, text);
        QApplication::sendEvent(&frame, &event);
    };
    frame.setActiveTool(pictura::ToolId::Move);
    sendKey(Qt::Key_T, Qt::NoModifier, QStringLiteral("t"));
    QCOMPARE(frame.activeTool(), pictura::ToolId::HorizontalType);
    for (pictura::ToolId next : {pictura::ToolId::VerticalType,
                                 pictura::ToolId::HorizontalTypeMask,
                                 pictura::ToolId::VerticalTypeMask,
                                 pictura::ToolId::HorizontalType}) {
        sendKey(Qt::Key_T, Qt::ShiftModifier, QStringLiteral("T"));
        QCOMPARE(frame.activeTool(), next);
    }

    // A click opens a session; typing previews without touching history, and
    // the window's letter shortcuts (B for the Brush) go quiet.
    const int layers = f.view->layer_count();
    const int base = f.view->history_index();
    f.drag({QPointF(20, 60)});
    QVERIFY(f.tools->textActive());
    QVERIFY(f.canvas->typeOverlayActiveForTest());
    QVERIFY(!f.canvas->typeOverlayHasTextForTest());
    QTest::keyClicks(f.canvas, QStringLiteral("Big"));
    QCOMPARE(frame.activeTool(), pictura::ToolId::HorizontalType);
    QTest::keyClick(f.canvas, Qt::Key_Return);
    QTest::keyClicks(f.canvas, QStringLiteral("type"));
    QVERIFY(f.canvas->typeOverlayHasTextForTest());
    QCOMPARE(f.view->history_index(), base);
    QCOMPARE(f.view->layer_count(), layers);

    // Ctrl+Enter commits one type layer, selected, with the text as typed.
    QTest::keyClick(f.canvas, Qt::Key_Return, Qt::ControlModifier);
    QVERIFY2(f.committedOnce(base, "Horizontal Type"), "one Horizontal Type state");
    QVERIFY(!f.tools->textActive());
    QVERIFY(!f.canvas->typeOverlayActiveForTest());
    QCOMPARE(f.view->layer_count(), layers + 1);
    const QString path = typeLayer(f);
    QVERIFY2(!path.isEmpty(), "a type layer");
    QCOMPARE(f.view->active_layer_path(), path);
    const QRect rect = layerRect(f, path);
    QVERIFY2(rect.left() == 20 && rect.top() < 60 && rect.bottom() > 60 + 24, "two lines down");
    QVERIFY2(rect.width() > 30 && rect.width() < 80, "as wide as \"type\"");

    // The Layers panel shows it as a type layer: a dark T on a white card, not
    // its pixels.
    auto* panel = frame.findChild<pictura::LayersPanel*>();
    QVERIFY(panel);
    const QImage thumb = panel->rowThumbnailForTest(path);
    QVERIFY2(!thumb.isNull(), "type row thumbnail");
    int white = 0;
    int dark = 0;
    for (int y = 0; y < thumb.height(); ++y) {
        for (int x = 0; x < thumb.width(); ++x) {
            const int v = qGray(thumb.pixel(x, y));
            white += v > 240 ? 1 : 0;
            dark += v < 80 ? 1 : 0;
        }
    }
    const int area = thumb.width() * thumb.height();
    QVERIFY2(white > area / 2 && dark > area / 30, "a T on white");
    const QColor stem = thumb.pixelColor(thumb.width() / 2, thumb.height() / 2);
    QVERIFY2(qGray(stem.rgb()) < 128, "the T's stem at the centre");
}

void TypeToolsTest::typeKeysAndCancel()
{
    pictura::PicturaMainWindow& frame = *window_;
    Fixture f(frame, seed(), QStringLiteral("pictura_type_keys"));
    QVERIFY2(f.ok(), "type fixture");
    showFrame();
    frame.setActiveTool(pictura::ToolId::HorizontalType);
    const int layers = f.view->layer_count();
    const int base = f.view->history_index();

    // Esc drops the text without a state.
    f.drag({QPointF(20, 60)});
    QTest::keyClicks(f.canvas, QStringLiteral("gone"));
    QTest::keyClick(f.canvas, Qt::Key_Escape);
    QVERIFY(!f.tools->textActive());
    QCOMPARE(f.view->history_index(), base);

    // Backspace back to nothing commits nothing.
    f.drag({QPointF(20, 60)});
    QTest::keyClicks(f.canvas, QStringLiteral("ab"));
    QTest::keyClick(f.canvas, Qt::Key_Backspace);
    QTest::keyClick(f.canvas, Qt::Key_Backspace);
    QVERIFY(!f.canvas->typeOverlayHasTextForTest());
    QTest::keyClick(f.canvas, Qt::Key_Return, Qt::ControlModifier);
    QCOMPARE(f.view->history_index(), base);

    // A click away commits; so does switching tools.
    f.drag({QPointF(20, 60)});
    QTest::keyClicks(f.canvas, QStringLiteral("one"));
    f.drag({QPointF(150, 150)});
    QVERIFY2(f.committedOnce(base, "Horizontal Type"), "click away commits");
    QVERIFY(!f.tools->textActive());
    f.drag({QPointF(20, 120)});
    QTest::keyClicks(f.canvas, QStringLiteral("two"));
    frame.setActiveTool(pictura::ToolId::Move);
    QVERIFY2(f.committedOnce(base + 1, "Horizontal Type"), "a tool switch commits");
    QCOMPARE(f.view->layer_count(), layers + 2);
}

void TypeToolsTest::reopenTypeLayer()
{
    pictura::PicturaMainWindow& frame = *window_;
    Fixture f(frame, seed(), QStringLiteral("pictura_type_reopen"));
    QVERIFY2(f.ok(), "type fixture");
    showFrame();
    frame.setActiveTool(pictura::ToolId::HorizontalType);
    const pictura::TypeOptions saved = f.tools->typeOptions();
    pictura::TypeOptions o = saved;
    o.size = 30.0;
    f.tools->setTypeOptions(o);
    f.drag({QPointF(20, 60)});
    QTest::keyClicks(f.canvas, QStringLiteral("Hello"));
    QTest::keyClick(f.canvas, Qt::Key_Return, Qt::ControlModifier);
    const QString path = typeLayer(f);
    const int index = path.toInt();
    const QRect before = layerRect(f, path);
    const int layers = f.view->layer_count();
    const int base = f.view->history_index();
    // Another tool's bar change leaves the layer alone; coming back to the
    // Type tool shows the selected layer's size.
    frame.setActiveTool(pictura::ToolId::Move);
    o.size = 12.0;
    f.tools->setTypeOptions(o);
    QCOMPARE(layerRect(f, path), before);
    frame.setActiveTool(pictura::ToolId::HorizontalType);
    QCOMPARE(f.tools->typeOptions().size, 30.0);
    o.size = 12.0;

    // A click on the type reopens it: hidden under its own text, nothing
    // recorded, and the bar shows the layer's size.
    f.drag({QPointF(before.center())});
    QVERIFY(f.tools->textActive());
    QVERIFY(!f.view->layer_visible(index));
    QVERIFY(f.canvas->typeOverlayHasTextForTest());
    QCOMPARE(f.tools->typeOptions().size, 30.0);
    QCOMPARE(f.view->history_index(), base);
    QTest::keyClick(f.canvas, Qt::Key_End);
    QTest::keyClicks(f.canvas, QStringLiteral(" there"));
    QTest::keyClick(f.canvas, Qt::Key_Return, Qt::ControlModifier);
    QVERIFY2(f.committedOnce(base, "Edit Type Layer"), "one Edit Type Layer state");
    QCOMPARE(f.view->layer_count(), layers);
    QVERIFY(f.view->layer_visible(index));
    QCOMPARE(f.view->layer_name(index), QStringLiteral("Hello there"));
    const QRect after = layerRect(f, path);
    QCOMPARE(after.left(), before.left());
    QCOMPARE(after.top(), before.top());
    QVERIFY(after.width() > before.width());

    // Esc leaves it as it was; Backspacing it blank and committing does too.
    f.drag({QPointF(after.center())});
    QTest::keyClicks(f.canvas, QStringLiteral("!!"));
    QTest::keyClick(f.canvas, Qt::Key_Escape);
    QVERIFY(f.view->layer_visible(index));
    QCOMPARE(f.view->history_index(), base + 1);
    f.drag({QPointF(after.center())});
    QTest::keyClick(f.canvas, Qt::Key_A, Qt::ControlModifier);
    QTest::keyClick(f.canvas, Qt::Key_Backspace);
    QTest::keyClick(f.canvas, Qt::Key_Return, Qt::ControlModifier);
    QVERIFY(f.view->layer_visible(index));
    QCOMPARE(f.view->history_index(), base + 1);
    QCOMPARE(layerRect(f, path), after);

    // The text keeps its orientation under the Vertical Type tool; the mask
    // tools start new type over it.
    frame.setActiveTool(pictura::ToolId::VerticalType);
    f.drag({QPointF(after.center())});
    QTest::keyClicks(f.canvas, QStringLiteral("!"));
    QTest::keyClick(f.canvas, Qt::Key_Return, Qt::ControlModifier);
    QVERIFY2(f.committedOnce(base + 1, "Edit Type Layer"), "reopened by Vertical Type");
    QVERIFY(layerRect(f, path).width() > layerRect(f, path).height());
    frame.setActiveTool(pictura::ToolId::HorizontalTypeMask);
    f.drag({QPointF(after.center())});
    QVERIFY(f.view->layer_visible(index));
    QTest::keyClicks(f.canvas, QStringLiteral("M"));
    QTest::keyClick(f.canvas, Qt::Key_Return, Qt::ControlModifier);
    QVERIFY2(f.committedOnce(base + 2, "Horizontal Type Mask"), "the mask starts new type");
    f.tools->setTypeOptions(saved);
}

void TypeToolsTest::freeTransformKeepsType()
{
    pictura::PicturaMainWindow& frame = *window_;
    Fixture f(frame, seed(), QStringLiteral("pictura_type_transform"));
    QVERIFY2(f.ok(), "type fixture");
    showFrame();
    frame.setActiveTool(pictura::ToolId::HorizontalType);
    const pictura::TypeOptions saved = f.tools->typeOptions();
    pictura::TypeOptions o = saved;
    o.size = 30.0;
    f.tools->setTypeOptions(o);
    f.drag({QPointF(10, 60)});
    QTest::keyClicks(f.canvas, QStringLiteral("Wide"));
    QTest::keyClick(f.canvas, Qt::Key_Return, Qt::ControlModifier);
    const QString path = typeLayer(f);
    const QRect before = layerRect(f, path);
    frame.setActiveTool(pictura::ToolId::Move);

    // Drag the right-middle handle out to twice the width and commit.
    QVERIFY(f.view->begin_free_transform(path));
    const double y = before.center().y();
    QVERIFY(f.view->transform_press(before.right() + 1, y, 1.0, false, false) >= 0);
    f.view->transform_move(before.right() + 1 + before.width(), y, 1.0, false, false);
    f.view->transform_release();
    const int base = f.view->history_index();
    QVERIFY(f.view->commit_transform());
    QVERIFY2(f.committedOnce(base, "Free Transform"), "one Free Transform state");

    // Still type, twice as wide, and reopening it keeps the stretch.
    QVERIFY(f.view->layer_is_type(path));
    const QRect stretched = layerRect(f, path);
    QVERIFY2(qAbs(stretched.width() - 2 * before.width()) <= 4, "twice as wide");
    frame.setActiveTool(pictura::ToolId::HorizontalType);
    f.drag({QPointF(stretched.center())});
    QVERIFY(f.tools->textActive());
    QTest::keyClicks(f.canvas, QStringLiteral("r"));
    QTest::keyClick(f.canvas, Qt::Key_Return, Qt::ControlModifier);
    QVERIFY2(f.committedOnce(base + 1, "Edit Type Layer"), "edited after the transform");
    QVERIFY(layerRect(f, path).width() > stretched.width() + 10);
    QCOMPARE(layerRect(f, path).height(), stretched.height());
    f.tools->setTypeOptions(saved);
}

void TypeToolsTest::textEditModel()
{
    pictura::TypeTextEdit e;
    e.reset(QStringLiteral("hello world\nnext"), 0);
    QCOMPARE(e.text(), QStringLiteral("hello world\rnext"));
    e.step(1, true, false);
    QCOMPARE(e.caret(), 5);
    e.step(1, true, true);
    QCOMPARE(e.selectedText(), QStringLiteral(" world"));
    e.insert(QStringLiteral(" there"));
    QCOMPARE(e.text(), QStringLiteral("hello there\rnext"));
    e.stepLine(1, false);
    QCOMPARE(e.caret(), 16);
    e.home(false, true);
    QCOMPARE(e.selectedText(), QStringLiteral("next"));
    e.step(-1, false, false);
    QCOMPARE(e.caret(), 12);
    e.backspace();
    QCOMPARE(e.text(), QStringLiteral("hello therenext"));
    e.deleteForward();
    QCOMPARE(e.text(), QStringLiteral("hello thereext"));
    e.selectWord(2);
    QCOMPARE(e.selectedText(), QStringLiteral("hello"));
    e.selectWord(5);
    QCOMPARE(e.selectedText(), QStringLiteral(" "));
    e.selectAll();
    e.end(true, false);
    QCOMPARE(e.caret(), int(e.text().size()));
    e.reset(QStringLiteral("a\U0001F600b"), 3);
    QCOMPARE(e.caret(), 3);
    e.step(-1, false, false);
    QCOMPARE(e.caret(), 1);
    e.moveTo(2, false);
    QCOMPARE(e.caret(), 1);
    e.step(1, false, false);
    e.backspace();
    QCOMPARE(e.text(), QStringLiteral("ab"));
}

void TypeToolsTest::editLikeATextField()
{
    pictura::PicturaMainWindow& frame = *window_;
    Fixture f(frame, seed(), QStringLiteral("pictura_type_field"));
    QVERIFY2(f.ok(), "type fixture");
    showFrame();
    frame.setActiveTool(pictura::ToolId::HorizontalType);
    const pictura::TypeOptions saved = f.tools->typeOptions();
    pictura::TypeOptions o = saved;
    o.size = 30.0;
    o.justification = 0;
    f.tools->setTypeOptions(o);
    f.drag({QPointF(10, 60)});
    QTest::keyClicks(f.canvas, QStringLiteral("one two"));
    const QLineF end = f.canvas->typeOverlayCaretForTest();

    // Clicking into the text puts the caret there; typing inserts at it.
    f.drag({QPointF(11, 50)});
    QVERIFY(f.tools->textActive());
    QVERIFY(f.canvas->typeOverlayCaretForTest().p1().x() < end.p1().x() - 40);
    QTest::keyClicks(f.canvas, QStringLiteral(">"));
    // Shift+arrows select, which typing replaces; Delete removes forward.
    QTest::keyClick(f.canvas, Qt::Key_End);
    QTest::keyClick(f.canvas, Qt::Key_Left, Qt::ShiftModifier | Qt::ControlModifier);
    QCOMPARE(f.canvas->typeOverlaySelectionForTest(), 3);
    QTest::keyClicks(f.canvas, QStringLiteral("2"));
    QTest::keyClick(f.canvas, Qt::Key_Home);
    QTest::keyClick(f.canvas, Qt::Key_Delete);

    // Ctrl+A, Ctrl+C copies with real newlines; Ctrl+V pastes them back.
    QTest::keyClick(f.canvas, Qt::Key_A, Qt::ControlModifier);
    QTest::keyClick(f.canvas, Qt::Key_C, Qt::ControlModifier);
    QCOMPARE(QGuiApplication::clipboard()->text(), QStringLiteral("one 2"));
    QTest::keyClick(f.canvas, Qt::Key_End);
    QGuiApplication::clipboard()->setText(QStringLiteral("\nthree"));
    QTest::keyClick(f.canvas, Qt::Key_V, Qt::ControlModifier);
    QCOMPARE(frame.activeTool(), pictura::ToolId::HorizontalType);

    // Double-click a word selects it; a drag across the text selects too.
    const QLineF caret = f.canvas->typeOverlayCaretForTest();
    const QPointF onThree(caret.p1().x() - 10, caret.center().y());
    f.drag({onThree});
    f.drag({onThree});
    QCOMPARE(f.canvas->typeOverlaySelectionForTest(), 5);
    QTest::keyClicks(f.canvas, QStringLiteral("3"));
    f.drag({QPointF(11, 50), QPointF(40, 50)});
    QVERIFY(f.canvas->typeOverlaySelectionForTest() >= 2);
    QTest::keyClick(f.canvas, Qt::Key_Right);
    QCOMPARE(f.canvas->typeOverlaySelectionForTest(), 0);

    const int base = f.view->history_index();
    QTest::keyClick(f.canvas, Qt::Key_Return, Qt::ControlModifier);
    QVERIFY2(f.committedOnce(base, "Horizontal Type"), "commit");
    QCOMPARE(QString(pictura::type_layer_text(*f.view, typeLayer(f))),
             QStringLiteral("one 2\r3"));
    f.tools->setTypeOptions(saved);
}

void TypeToolsTest::barRestylesTheSelectedLayer()
{
    pictura::PicturaMainWindow& frame = *window_;
    Fixture f(frame, seed(), QStringLiteral("pictura_type_restyle"));
    QVERIFY2(f.ok(), "type fixture");
    showFrame();
    frame.setActiveTool(pictura::ToolId::HorizontalType);
    const pictura::TypeOptions saved = f.tools->typeOptions();
    pictura::TypeOptions o = saved;
    o.family = QStringLiteral("Liberation Sans");
    o.size = 24.0;
    f.tools->setTypeOptions(o);
    f.drag({QPointF(10, 60)});
    QTest::keyClicks(f.canvas, QStringLiteral("iiiiiiii"));
    QTest::keyClick(f.canvas, Qt::Key_Return, Qt::ControlModifier);
    const QString path = typeLayer(f);
    const QRect small = layerRect(f, path);
    const int layers = f.view->layer_count();
    const int base = f.view->history_index();

    // Size: one state, the same text twice as tall.
    o.size = 48.0;
    f.tools->setTypeOptions(o);
    QVERIFY2(f.committedOnce(base, "Edit Type Layer"), "size restyles the layer");
    QCOMPARE(f.view->layer_count(), layers);
    const QRect big = layerRect(f, path);
    QVERIFY(qAbs(big.height() - 2 * small.height()) <= 3);
    QCOMPARE(QString(pictura::type_layer_text(*f.view, path)), QStringLiteral("iiiiiiii"));

    // Family: the layer is re-set in that face — monospace i's are wider —
    // and records it, keeping the size.
    const QString mono = QFontInfo(QFontDatabase::systemFont(QFontDatabase::FixedFont)).family();
    QVERIFY2(!pictura::typeFontBytes(mono).isEmpty(), "Qt hands over the face");
    o.family = mono;
    f.tools->setTypeOptions(o);
    QVERIFY2(f.committedOnce(base + 1, "Edit Type Layer"), "family restyles the layer");
    const QRect monoRect = layerRect(f, path);
    QVERIFY2(monoRect.width() > big.width() * 3 / 2, "set in the monospace face");
    // Its own ascent and descent, at the same size.
    QVERIFY(monoRect.height() > big.height() * 3 / 4 && monoRect.height() < big.height() * 3 / 2);
    QCOMPARE(QString(pictura::type_layer_font(*f.view, path)), mono);

    // Selecting the layer again shows its family and size in the bar.
    frame.setActiveTool(pictura::ToolId::Move);
    f.tools->setTypeOptions(saved);
    frame.setActiveTool(pictura::ToolId::HorizontalType);
    QCOMPARE(f.tools->typeOptions().family, mono);
    QCOMPARE(f.tools->typeOptions().size, 48.0);
    QCOMPARE(f.view->history_index(), base + 2);

    // A registered generic alias reads back as itself, not as whichever
    // family its spaceless name happens to match.
    pictura::registerTypeFont(QStringLiteral("monospace"));
    QCOMPARE(pictura::familyForFontName(QStringLiteral("monospace")), QStringLiteral("monospace"));
    f.tools->setTypeOptions(saved);
}

void TypeToolsTest::verticalTypeTool()
{
    pictura::PicturaMainWindow& frame = *window_;
    Fixture f(frame, seed(), QStringLiteral("pictura_type_vertical"));
    QVERIFY2(f.ok(), "type fixture");
    showFrame();
    frame.setActiveTool(pictura::ToolId::VerticalType);
    const int base = f.view->history_index();
    f.drag({QPointF(100, 10)});
    QTest::keyClicks(f.canvas, QStringLiteral("TALL"));
    QTest::keyClick(f.canvas, Qt::Key_Enter, Qt::KeypadModifier);
    QVERIFY2(f.committedOnce(base, "Vertical Type"), "keypad Enter commits");
    const QRect rect = layerRect(f, typeLayer(f));
    QCOMPARE(rect.top(), 10);
    QVERIFY2(rect.height() > 3 * rect.width(), "one column, top to bottom");
    QVERIFY(rect.left() < 100 && rect.right() > 100);
}

void TypeToolsTest::typeMaskTools()
{
    pictura::PicturaMainWindow& frame = *window_;
    Fixture f(frame, seed(), QStringLiteral("pictura_type_mask"));
    QVERIFY2(f.ok(), "type fixture");
    showFrame();
    const int layers = f.view->layer_count();
    const int base = f.view->history_index();

    frame.setActiveTool(pictura::ToolId::HorizontalTypeMask);
    f.drag({QPointF(20, 60)});
    QVERIFY(f.canvas->typeOverlayActiveForTest());
    QTest::keyClicks(f.canvas, QStringLiteral("MASK"));
    QTest::keyClick(f.canvas, Qt::Key_Return, Qt::ControlModifier);
    QVERIFY2(f.committedOnce(base, "Horizontal Type Mask"), "one mask state");
    QCOMPARE(f.view->layer_count(), layers);
    QVERIFY(f.view->has_selection());
    const int horizontal = f.view->selection_count();
    QVERIFY(horizontal > 50);
    QCOMPARE(f.view->selection_coverage(5, 5), 0);

    // Shift at the click adds the Vertical Type Mask's column to it.
    frame.setActiveTool(pictura::ToolId::VerticalTypeMask);
    f.drag({QPointF(170, 10)}, Qt::ShiftModifier);
    QTest::keyClicks(f.canvas, QStringLiteral("IO"));
    QTest::keyClick(f.canvas, Qt::Key_Return, Qt::ControlModifier);
    QVERIFY2(f.committedOnce(base + 1, "Vertical Type Mask"), "one vertical mask state");
    QVERIFY(f.view->selection_count() > horizontal);
}

void TypeToolsTest::typeOptionsBar()
{
    pictura::PicturaMainWindow& frame = *window_;
    Fixture f(frame, seed(), QStringLiteral("pictura_type_bar"));
    QVERIFY2(f.ok(), "type fixture");
    showFrame();
    const pictura::TypeOptions saved = f.tools->typeOptions();
    frame.setActiveTool(pictura::ToolId::HorizontalType);

    auto visible = [&frame](const QString& name) -> QToolButton* {
        for (auto* button : frame.findChildren<QToolButton*>(name)) {
            if (button->isVisible()) {
                return button;
            }
        }
        return nullptr;
    };
    QToolButton* commit = visible(QStringLiteral("optionsTypeCommit"));
    QVERIFY2(commit, "type bar");
    QVERIFY(!commit->isEnabled());

    // Size and alignment feed the next commit: right-aligned type ends at the click.
    pictura::TypeOptions o = f.tools->typeOptions();
    o.size = 40.0;
    f.tools->setTypeOptions(o);
    visible(QStringLiteral("optionsTypeAlignRight"))->click();
    QCOMPARE(f.tools->typeOptions().justification, 1);
    const int base = f.view->history_index();
    f.drag({QPointF(180, 80)});
    QVERIFY(commit->isEnabled());
    QTest::keyClicks(f.canvas, QStringLiteral("Right"));
    commit->click();
    QVERIFY2(f.committedOnce(base, "Horizontal Type"), "the bar's Commit");
    QVERIFY(!commit->isEnabled());
    const QRect rect = layerRect(f, typeLayer(f));
    QVERIFY2(rect.right() <= 181 && rect.right() >= 178, "ends at the click");
    QVERIFY(rect.height() >= 40);

    // Toggle Text Orientation swaps to the Vertical Type tool.
    visible(QStringLiteral("optionsTypeOrientation"))->click();
    QCOMPARE(frame.activeTool(), pictura::ToolId::VerticalType);
    f.tools->setTypeOptions(saved);
}

QTEST_MAIN(TypeToolsTest)
#include "tst_type_tools.moc"
