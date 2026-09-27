#include "selftest_magnetic_lasso.h"
#include "selftest_report.h"

#include "frame.h"
#include "image_view.h"
#include "panels/numeric_field.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QDir>
#include <QtCore/QFile>
#include <QtGui/QImage>
#include <QtGui/QKeyEvent>
#include <QtGui/QPainter>
#include <QtWidgets/QApplication>

int pictura::runMagneticLassoChecks(pictura::PicturaMainWindow& frame)
{
    QImage seed(40, 40, QImage::Format_RGB32);
    seed.fill(Qt::white);
    QPainter(&seed).fillRect(10, 10, 20, 20, Qt::black);
    const QString seedPath = QDir::temp().filePath(QStringLiteral("pictura_magnetic_seed.png"));
    const bool created = frame.newDocument(QStringLiteral("MagneticCtl"), 40, 40,
                                           QStringLiteral("rgb"), 8, QStringLiteral("white"));
    pictura::PictureView* view = frame.activeView();
    pictura::ImageView* canvas = frame.imageView();
    auto* tools = frame.findChild<pictura::ToolController*>();
    if (!created || !view || !canvas || !tools || !seed.save(seedPath, "PNG")
        || !view->open_image(seedPath)) {
        return pictura::selfTest().fail(530, "magnetic lasso fixture");
    }
    const int doc = frame.activeDocumentIndex();
    frame.setActiveTool(pictura::ToolId::MagneticLasso);
    const bool active = tools->activeTool() == pictura::ToolId::MagneticLasso;
    tools->setCombineMode(pictura::SelectionMode::New);
    tools->setFeather(0.0);
    tools->setMagneticWidth(6);
    tools->setMagneticContrast(10);
    tools->setMagneticFrequency(0);
    const auto click = [canvas](qreal x, qreal y) {
        canvas->mouseMoved(QPointF(x, y));
        canvas->mousePressed(QPointF(x, y), Qt::LeftButton, int(Qt::NoModifier));
        canvas->mouseReleased(QPointF(x, y));
    };
    const auto key = [&frame](Qt::Key k) {
        QKeyEvent press(QEvent::KeyPress, k, Qt::NoModifier);
        QApplication::sendEvent(&frame, &press);
    };

    // Snap: every click is two pixels inside the square, so only a wire that
    // bowed out to the edges selects column 10, column 28, and row 28 (a
    // straight outline through the clicks misses all three at pixel centres).
    view->deselect();
    const int base = view->history_count();
    click(12, 12);
    click(12, 27);
    click(27, 27);
    click(27, 12);
    click(12, 12);
    const bool snapped = view->history_count() == base + 1 && view->selection_coverage(20, 20) > 0
        && view->selection_coverage(10, 20) > 0 && view->selection_coverage(20, 28) > 0
        && view->selection_coverage(28, 20) > 0 && view->selection_coverage(5, 20) == 0
        && view->selection_coverage(20, 33) == 0 && view->selection_coverage(33, 20) == 0
        && !canvas->hasSelectionPreviewForTest();

    // Delete peels fastening points back; deleting the first abandons the trace.
    view->deselect();
    const int peelBase = view->history_count();
    click(12, 12);
    click(12, 27);
    click(27, 27);
    key(Qt::Key_Delete);
    key(Qt::Key_Backspace);
    const bool stillOpen = canvas->hasSelectionPreviewForTest();
    key(Qt::Key_Delete);
    const bool peeled = stillOpen && !tools->commitPolygonLasso()
        && view->history_count() == peelBase && !view->has_selection()
        && !canvas->hasSelectionPreviewForTest();

    // Escape leaves the selection and history exactly as they were.
    view->select_rect(1, 1, 4, 4, QStringLiteral("new"), 0.0);
    const int escBase = view->history_count();
    const int escCount = view->selection_count();
    click(12, 12);
    click(12, 27);
    key(Qt::Key_Escape);
    const bool escaped = view->history_count() == escBase && view->selection_count() == escCount
        && view->selection_coverage(2, 2) > 0 && !tools->commitPolygonLasso();
    view->deselect();

    // Frequency: at 100 a 15 px wire fastens itself (two deletes still leave the
    // trace open), at 0 it never does (the second delete finds nothing).
    const auto deletesUntilClosed = [&](int frequency) {
        tools->setMagneticFrequency(frequency);
        click(12, 12);
        canvas->mouseMoved(QPointF(12, 27));
        int deletes = 0;
        while (tools->removeLassoPoint() && deletes < 10) {
            ++deletes;
        }
        return deletes;
    };
    const int denseDeletes = deletesUntilClosed(100);
    const int sparseDeletes = deletesUntilClosed(0);
    const bool frequency = denseDeletes > sparseDeletes && sparseDeletes == 1;

    // `]` / `[` step Width by 1 px and the options-bar field follows.
    tools->setMagneticWidth(10);
    const bool up = tools->applyBrushShortcut(Qt::Key_BracketRight, 0, false)
        && tools->magneticWidth() == 11;
    auto* widthField = frame.findChild<pictura::NumericField*>(
        QStringLiteral("optionsMagneticWidth"));
    const bool fieldFollows = widthField && qRound(widthField->value()) == 11;
    const bool down = tools->applyBrushShortcut(Qt::Key_BracketLeft, 0, false)
        && tools->magneticWidth() == 10;
    const bool width = up && down && fieldFollows;

    ST_BEGIN("magnetic_lasso");
    ST_PASS("magnetic_lasso active=%d snapped=%d peeled=%d escaped=%d dense=%d sparse=%d "
            "width=%d",
            active ? 1 : 0, snapped ? 1 : 0, peeled ? 1 : 0, escaped ? 1 : 0, denseDeletes,
            sparseDeletes, width ? 1 : 0);
    tools->setMagneticFrequency(57);
    frame.setActiveTool(pictura::ToolId::Move);
    frame.closeDocument(doc, false);
    QFile::remove(seedPath);
    if (!active || !snapped || !peeled || !escaped || !frequency || !width) {
        return pictura::selfTest().fail(530, "magnetic lasso");
    }
    return 0;
}
