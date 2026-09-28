#include "selftest_healing.h"
#include "selftest_report.h"

#include "frame.h"
#include "image_view.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/annotations.cxxqt.h"

#include <QtCore/QDir>
#include <QtCore/QFile>
#include <QtGui/QImage>
#include <QtGui/QPainter>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QLabel>

int pictura::runHealingChecks(pictura::PicturaMainWindow& frame)
{
    // Seed: a white field with a black 6×6 blemish at (17, 17).
    QImage seed(40, 40, QImage::Format_RGB32);
    seed.fill(Qt::white);
    QPainter(&seed).fillRect(17, 17, 6, 6, Qt::black);
    const QString seedPath = QDir::temp().filePath(QStringLiteral("pictura_healing_seed.png"));

    const bool created = frame.newDocument(QStringLiteral("HealingCtl"), 40, 40,
                                           QStringLiteral("rgb"), 8, QStringLiteral("white"));
    pictura::PictureView* view = frame.activeView();
    pictura::ImageView* canvas = frame.imageView();
    auto* tools = frame.findChild<pictura::ToolController*>();
    if (!created || !view || !canvas || !tools || !seed.save(seedPath, "PNG")
        || !view->open_image(seedPath)) {
        return pictura::selfTest().fail(536, "healing fixture");
    }
    const int doc = frame.activeDocumentIndex();
    const auto press = [canvas](QPointF at, Qt::KeyboardModifiers mods = Qt::NoModifier) {
        canvas->mousePressed(at, Qt::LeftButton, int(mods));
        canvas->mouseReleased(at);
    };

    // Spot Healing Brush: one click on the blemish rebuilds it from the white
    // surroundings and records exactly one "Spot Healing Brush" state.
    frame.setActiveTool(pictura::ToolId::SpotHealingBrush);
    const bool spotActive = tools->activeTool() == pictura::ToolId::SpotHealingBrush;
    const int base = view->history_index();
    press(QPointF(20, 20));
    const bool committed = view->history_index() == base + 1
        && view->history_label(base + 1) == QStringLiteral("Spot Healing Brush");
    const quint32 healedArgb = view->sample_argb(20, 20);
    const bool healed = qRed(healedArgb) > 200 && qGreen(healedArgb) > 200
        && qBlue(healedArgb) > 200;

    // Healing Brush: painting before an Alt sample is refused (no state).
    frame.setActiveTool(pictura::ToolId::HealingBrush);
    const int refusedBase = view->history_index();
    press(QPointF(30, 30));
    const bool refused = view->history_index() == refusedBase;
    // After an Alt sample, a stroke commits one "Healing Brush" state.
    press(QPointF(30, 30), Qt::AltModifier);
    press(QPointF(20, 20));
    const bool transplanted = view->history_index() == refusedBase + 1
        && view->history_label(refusedBase + 1) == QStringLiteral("Healing Brush");

    // Count (Extended): numbered marks in groups, the overlay, and the bar.
    frame.setActiveTool(pictura::ToolId::Count);
    const bool countActive = tools->activeTool() == pictura::ToolId::Count;
    auto* groupCombo = frame.findChild<QComboBox*>(QStringLiteral("optionsCountGroup"));
    auto* totalLabel = frame.findChild<QLabel*>(QStringLiteral("optionsCountTotal"));
    const bool bar = groupCombo && totalLabel && groupCombo->count() == 1
        && totalLabel->text().startsWith(QStringLiteral("Count:"));
    const int cbase = view->history_index();
    press(QPointF(5, 5));
    press(QPointF(35, 35));
    const bool counted = pictura::count_active_total(*view) == 2
        && canvas->countOverlayCountForTest() == 2 && view->history_index() == cbase + 2;
    // A new group is independent; its marks overlay alongside group 1's.
    const int g2 = pictura::count_add_group(*view, QStringLiteral("Count Group 2"));
    press(QPointF(20, 20));
    const bool grouped = g2 == 1 && pictura::count_group_total(*view, 0) == 2
        && pictura::count_group_total(*view, 1) == 1 && pictura::count_active_total(*view) == 1
        && canvas->countOverlayCountForTest() == 3;
    // Hiding group 1 leaves only group 2's mark on the overlay.
    pictura::count_set_visible(*view, 0, false);
    const bool hidden = canvas->countOverlayCountForTest() == 1;
    pictura::count_set_visible(*view, 0, true);
    // Colour and sizes are stored on the group.
    pictura::count_set_color(*view, 1, 0xff0000);
    pictura::count_set_marker_size(*view, 1, 6);
    pictura::count_set_label_size(*view, 1, 20);
    const bool styled = pictura::count_group_color(*view, 1) == 0xff0000
        && pictura::count_group_marker_size(*view, 1) == 6
        && pictura::count_group_label_size(*view, 1) == 20;
    // Delete the second group, then Clear the first.
    const bool deletedGroup =
        pictura::count_remove_group(*view, 1) && pictura::count_group_count(*view) == 1;
    const bool cleared = pictura::count_clear(*view) && pictura::count_active_total(*view) == 0
        && canvas->countOverlayCountForTest() == 0;
    frame.setActiveTool(pictura::ToolId::Move);

    ST_BEGIN("healing_tools");
    ST_PASS("healing spot=%d/%d/%d heal=%d/%d count=%d/%d/%d/%d/%d/%d/%d", spotActive ? 1 : 0,
            committed ? 1 : 0, healed ? 1 : 0, refused ? 1 : 0, transplanted ? 1 : 0,
            countActive ? 1 : 0, bar ? 1 : 0, counted ? 1 : 0, grouped ? 1 : 0, hidden ? 1 : 0,
            styled ? 1 : 0, deletedGroup ? 1 : 0, cleared ? 1 : 0);
    frame.closeDocument(doc, false);
    QFile::remove(seedPath);
    if (!spotActive || !committed || !healed || !refused || !transplanted || !countActive || !bar
        || !counted || !grouped || !hidden || !styled || !deletedGroup || !cleared) {
        return pictura::selfTest().fail(536, "healing tools");
    }
    return 0;
}
