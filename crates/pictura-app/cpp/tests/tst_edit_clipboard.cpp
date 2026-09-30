#include <QtTest/QtTest>

#include <QtGui/QClipboard>
#include <QtGui/QGuiApplication>
#include <QtGui/QImage>

#include "commands.h"
#include "frame.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/clipboard.cxxqt.h"

#include "qt_test_support.h"

class EditClipboardTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void editClipboard();

private:
    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void EditClipboardTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void EditClipboardTest::editClipboard()
{
    pictura::PicturaMainWindow& frame = *window_;
    const bool created = frame.newDocument(QStringLiteral("ClipboardCtl"), 8, 8,
                                           QStringLiteral("rgb"), 8, QStringLiteral("white"));
    pictura::PictureView* view = frame.activeView();
    QVERIFY2(created && view, "clipboard fixture");
    const int doc = frame.activeDocumentIndex();
    pictura::CommandRegistry* registry = frame.registry();
    QClipboard* system = QGuiApplication::clipboard();
    const auto enabled = [registry](const char* id) {
        registry->refresh();
        QAction* action = registry->action(QString::fromLatin1(id));
        return action && action->isEnabled();
    };
    const auto step = [view, registry](const char* id, const char* label) {
        const int base = view->history_count();
        registry->dispatch(QString::fromLatin1(id));
        if (!label) {
            return view->history_count() == base;
        }
        return view->history_count() == base + 1
            && view->history_label(base) == QString::fromLatin1(label);
    };
    const auto hasMask = [view]() {
        for (int row = 0; row < view->layer_row_count(); ++row) {
            if (view->layer_row_has_mask(row)) {
                return true;
            }
        }
        return false;
    };

    system->clear();
    pictura::clipboard_purge();
    const bool pasteOffEmpty = !enabled(pictura::command_ids::EditPaste);
    const quint32 white = view->sample_argb(0, 0);
    const QString path = view->add_solid_fill(0xff2244aau);
    const bool raster = view->rasterize_fill_content(path);
    frame.selectLayerPath(path);
    const quint32 fill = view->sample_argb(3, 3);
    view->select_rect(2, 2, 3, 3, QStringLiteral("new"), 0.0);

    const bool copied = step(pictura::command_ids::EditCopy, nullptr) && pictura::clipboard_has_contents()
        && system->image().size() == QSize(3, 3);
    const bool cut = step(pictura::command_ids::EditCut, "Cut") && view->sample_argb(3, 3) == white
        && view->sample_argb(1, 1) == fill;
    const int layers = view->layer_count();
    const bool inPlace = step(pictura::command_ids::EditPasteInPlace, "Paste")
        && view->layer_count() == layers + 1 && view->sample_argb(3, 3) == fill
        && view->sample_argb(1, 1) == fill;
    const bool into = step(pictura::command_ids::EditPasteInto, "Paste Into")
        && view->layer_count() == layers + 2 && hasMask() && !view->has_selection();

    frame.selectLayerPath(path);
    view->select_rect(0, 0, 2, 2, QStringLiteral("new"), 0.0);
    const bool clear = step(pictura::command_ids::EditClear, "Clear") && view->sample_argb(0, 1) == white
        && view->sample_argb(6, 0) == fill;
    // A plain paste lands wherever the (headless) canvas centre maps.
    const bool paste = step(pictura::command_ids::EditPaste, "Paste") && view->layer_count() == layers + 3;

    // Another application's image replaces our copy and pastes in place at the
    // canvas origin.
    QImage foreign(2, 2, QImage::Format_ARGB32);
    foreign.fill(0xff00ff00u);
    system->setImage(foreign);
    const bool imported = step(pictura::command_ids::EditPasteInPlace, "Paste")
        && view->layer_count() == layers + 4 && view->sample_argb(0, 0) == 0xff00ff00u;

    // Purge drops our copy and our own export from the system clipboard.
    const bool merged = step(pictura::command_ids::EditCopyMerged, nullptr)
        && system->image().pixel(0, 0) == 0xff00ff00u;
    const bool purged = step(pictura::command_ids::EditPurgeClipboard, nullptr)
        && !pictura::clipboard_has_contents() && system->image().isNull()
        && !enabled(pictura::command_ids::EditPaste);

    QVERIFY2(raster, "rasterize fill content");
    QVERIFY2(pasteOffEmpty, "paste disabled on empty clipboard");
    QVERIFY2(copied, "copy");
    QVERIFY2(cut, "cut");
    QVERIFY2(inPlace, "paste in place");
    QVERIFY2(into, "paste into");
    QVERIFY2(clear, "clear");
    QVERIFY2(paste, "paste");
    QVERIFY2(imported, "import foreign image");
    QVERIFY2(merged, "copy merged");
    QVERIFY2(purged, "purge clipboard");
    frame.closeDocument(doc, false);
}

QTEST_MAIN(EditClipboardTest)
#include "tst_edit_clipboard.moc"
