// Keyboard paths into Edit and View commands: Delete / Backspace clearing a
// selection (#186) and Ctrl+= / Ctrl+Shift+= zooming in (#187).

#include <QtTest/QtTest>

#include "commands.h"
#include "frame.h"
#include "image_view.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include "qt_test_support.h"

class EditShortcutsTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void deleteAndBackspaceClearTheSelection();
    void ctrlEqualsZoomsIn();

private:
    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void EditShortcutsTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
    // Menu shortcuts only route to an active window.
    window_->show();
    window_->activateWindow();
    QVERIFY(QTest::qWaitForWindowActive(window_.get()));
}

void EditShortcutsTest::deleteAndBackspaceClearTheSelection()
{
    pictura::PicturaMainWindow& frame = *window_;
    for (const Qt::Key key : {Qt::Key_Delete, Qt::Key_Backspace}) {
        QVERIFY(frame.newDocument(QStringLiteral("Clear"), 40, 30, QStringLiteral("rgb"), 8,
                                  QStringLiteral("white")));
        pictura::PictureView* view = frame.activeView();
        pictura::ImageView* canvas = frame.imageView();
        QVERIFY(view && canvas);
        const int doc = frame.activeDocumentIndex();
        const int history = view->history_count();

        // No selection: nothing to clear.
        QTest::keyClick(canvas, key);
        QCOMPARE(view->history_count(), history);

        QVERIFY(view->select_rect(5, 5, 20, 16, QStringLiteral("new"), 0.0));
        const int selected = view->history_count();
        const QRgb outside = view->composite_argb(30, 20);
        const QRgb inside = view->composite_argb(8, 8);
        QTest::keyClick(canvas, key);
        QCOMPARE(view->history_count(), selected + 1);
        QCOMPARE(view->history_label(view->history_index()), QStringLiteral("Clear"));
        QVERIFY(view->composite_argb(8, 8) != inside);
        QCOMPARE(view->composite_argb(30, 20), outside);
        frame.closeDocument(doc, false);
    }
}

void EditShortcutsTest::ctrlEqualsZoomsIn()
{
    pictura::PicturaMainWindow& frame = *window_;
    QVERIFY(frame.newDocument(QStringLiteral("Zoom"), 40, 30, QStringLiteral("rgb"), 8,
                              QStringLiteral("white")));
    pictura::ImageView* canvas = frame.imageView();
    QVERIFY(canvas);
    const int doc = frame.activeDocumentIndex();
    const struct {
        Qt::Key key;
        Qt::KeyboardModifiers modifiers;
    } presses[] = {
        {Qt::Key_Equal, Qt::ControlModifier},
        {Qt::Key_Plus, Qt::ControlModifier | Qt::ShiftModifier},
        {Qt::Key_Plus, Qt::ControlModifier | Qt::KeypadModifier},
        {Qt::Key_Plus, Qt::ControlModifier},
    };
    canvas->setFocus();
    for (const auto& press : presses) {
        const double before = canvas->zoom();
        QTest::keyClick(canvas, press.key, press.modifiers);
        // One step each: a press must not reach both the menu shortcut and
        // the fallback.
        QVERIFY2(canvas->zoom() > before && canvas->zoom() < before * 1.3,
                 qPrintable(QStringLiteral("key %1 mods %2")
                                .arg(press.key)
                                .arg(int(press.modifiers))));
    }
    const double zoomed = canvas->zoom();
    QTest::keyClick(canvas, Qt::Key_Minus, Qt::ControlModifier);
    QVERIFY(canvas->zoom() < zoomed);
    frame.closeDocument(doc, false);
}

QTEST_MAIN(EditShortcutsTest)
#include "tst_edit_shortcuts.moc"
