#include "selftest_fills.h"
#include "selftest_report.h"

#include "selftest_paint_fixture.h"

#include "panels/numeric_field.h"

#include "pictura_app/src/cxxqt_object/paint_tools/fills.cxxqt.h"

#include <QtCore/QCoreApplication>
#include <QtGui/QKeyEvent>
#include <QtGui/QPainter>
#include <QtWidgets/QApplication>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QMenu>
#include <QtWidgets/QToolButton>

namespace {
using paint_fixture::Fixture;

// The foreground and background colours a check changes, restored on scope exit.
struct Colors {
    pictura::ToolController* tools;
    QColor foreground;
    QColor background;

    Colors(pictura::ToolController* t, const QColor& fg, const QColor& bg)
        : tools(t)
        , foreground(t->foreground())
        , background(t->background())
    {
        tools->setForeground(fg);
        tools->setBackground(bg);
    }
    ~Colors()
    {
        tools->setForeground(foreground);
        tools->setBackground(background);
    }
};
} // namespace

int pictura::runGradientToolChecks(pictura::PicturaMainWindow& frame)
{
    QImage seed(40, 40, QImage::Format_RGB32);
    seed.fill(QColor(0, 128, 0));
    Fixture f(frame, seed, QStringLiteral("pictura_gradient_seed"));
    if (!f.ok()) {
        return pictura::selfTest().fail(552, "gradient fixture");
    }
    const Colors colors(f.tools, Qt::black, Qt::white);
    const QRgb black = QColor(Qt::black).rgba();
    const QRgb white = QColor(Qt::white).rgba();

    // The G group's keys, through the window's shortcut path.
    frame.activateWindow();
    QCoreApplication::processEvents();
    const auto sendKey = [&frame](Qt::KeyboardModifiers mods, const QString& text) {
        QKeyEvent event(QEvent::KeyPress, Qt::Key_G, mods, text);
        QApplication::sendEvent(&frame, &event);
    };
    frame.setActiveTool(pictura::ToolId::Gradient);
    frame.setActiveTool(pictura::ToolId::Move);
    sendKey(Qt::NoModifier, QStringLiteral("g"));
    bool keys = frame.activeTool() == pictura::ToolId::Gradient;
    sendKey(Qt::ShiftModifier, QStringLiteral("G"));
    keys = keys && frame.activeTool() == pictura::ToolId::PaintBucket;
    sendKey(Qt::ShiftModifier, QStringLiteral("G"));
    keys = keys && frame.activeTool() == pictura::ToolId::Gradient;

    auto* sample = frame.findChild<QToolButton*>(QStringLiteral("optionsGradientSample"));
    QList<QToolButton*> styles;
    for (int i = 0; i < 5; ++i) {
        styles << frame.findChild<QToolButton*>(QStringLiteral("optionsGradientStyle%1").arg(i));
    }
    auto* mode = frame.findChild<QComboBox*>(QStringLiteral("optionsGradientMode"));
    auto* opacity =
        frame.findChild<pictura::NumericField*>(QStringLiteral("optionsGradientOpacity"));
    auto* reverse = frame.findChild<QCheckBox*>(QStringLiteral("optionsGradientReverse"));
    auto* dither = frame.findChild<QCheckBox*>(QStringLiteral("optionsGradientDither"));
    auto* transparency =
        frame.findChild<QCheckBox*>(QStringLiteral("optionsGradientTransparency"));
    if (!sample || !sample->menu() || styles.contains(nullptr) || !mode || !opacity || !reverse
        || !dither || !transparency) {
        return pictura::selfTest().fail(552, "gradient bar");
    }
    const QList<QAction*> presets = sample->menu()->actions();
    const bool bar = presets.size() == pictura::gradient_preset_count() && presets.size() > 1
        && presets.first()->isChecked() && !sample->icon().isNull() && styles.at(0)->isChecked()
        && mode->count() == 3 && mode->currentIndex() == 0 && opacity->value() == 100
        && !reverse->isChecked() && !dither->isChecked() && transparency->isChecked()
        && !pictura::isBrushTool(pictura::ToolId::Gradient);

    // The axis follows the drag; the gradient is drawn on release.
    const int base = f.view->history_index();
    f.canvas->mousePressed(QPointF(0, 20), Qt::LeftButton, 0);
    f.canvas->mouseMoved(QPointF(39, 20));
    const bool axisShown =
        f.canvas->hasRulerLineForTest() && f.view->history_index() == base;
    f.canvas->mouseReleased(QPointF(39, 20));
    const bool axis = axisShown && !f.canvas->hasRulerLineForTest();
    const int mid = qRed(f.view->sample_argb(20, 20));
    const bool linear = f.committedOnce(base, "Gradient") && f.view->sample_argb(0, 20) == black
        && f.view->sample_argb(39, 5) == white && mid > 100 && mid < 160;
    const bool undone = f.view->undo() && f.view->sample_argb(0, 20) == QColor(0, 128, 0).rgba()
        && f.view->redo() && f.view->sample_argb(0, 20) == black;

    f.drag({QPointF(10, 10)});
    const bool click = f.view->history_index() == base + 1;

    reverse->setChecked(true);
    f.drag({QPointF(0, 20), QPointF(39, 20)});
    const bool reversed = f.tools->gradientOptions().reverse
        && f.view->history_index() == base + 2 && f.view->sample_argb(0, 20) == white
        && f.view->sample_argb(39, 20) == black;
    reverse->setChecked(false);

    // Radial, inside a selection of the left half.
    styles.at(1)->click();
    f.view->select_rect(0, 0, 20, 40, QStringLiteral("new"), 0.0);
    const int inside = f.view->history_index();
    f.drag({QPointF(10, 20), QPointF(15, 20)});
    const bool selected = f.tools->gradientOptions().style == 1
        && f.committedOnce(inside, "Gradient") && f.view->sample_argb(10, 20) == black
        && f.view->sample_argb(10, 38) == white && f.view->sample_argb(39, 20) == black;
    f.view->deselect();
    styles.at(0)->click();

    presets.at(1)->trigger();
    const bool preset = f.tools->gradientOptions().preset == 1
        && sample->toolTip() == pictura::gradient_preset_name(1);
    presets.at(0)->trigger();

    ST_BEGIN("gradient_tool");
    ST_PASS("gradient keys=%d bar=%d axis=%d linear=%d undo=%d click=%d reverse=%d selection=%d "
            "preset=%d",
            keys ? 1 : 0, bar ? 1 : 0, axis ? 1 : 0, linear ? 1 : 0, undone ? 1 : 0, click ? 1 : 0,
            reversed ? 1 : 0, selected ? 1 : 0, preset ? 1 : 0);
    if (!keys || !bar || !axis || !linear || !undone || !click || !reversed || !selected
        || !preset) {
        return pictura::selfTest().fail(552, "gradient tool");
    }
    return 0;
}

int pictura::runPaintBucketChecks(pictura::PicturaMainWindow& frame)
{
    // Two separate red squares on white.
    const QColor red(220, 20, 20);
    QImage seed(40, 40, QImage::Format_RGB32);
    seed.fill(Qt::white);
    {
        QPainter painter(&seed);
        painter.fillRect(5, 5, 10, 10, red);
        painter.fillRect(25, 25, 10, 10, red);
    }
    Fixture f(frame, seed, QStringLiteral("pictura_paint_bucket_seed"));
    if (!f.ok()) {
        return pictura::selfTest().fail(553, "paint bucket fixture");
    }
    const QColor blue(0, 0, 255);
    const QColor green(0, 200, 0);
    const Colors colors(f.tools, blue, Qt::white);

    frame.setActiveTool(pictura::ToolId::PaintBucket);
    auto* fill = frame.findChild<QComboBox*>(QStringLiteral("optionsBucketFill"));
    auto* pattern = frame.findChild<QComboBox*>(QStringLiteral("optionsBucketPattern"));
    auto* mode = frame.findChild<QComboBox*>(QStringLiteral("optionsBucketMode"));
    auto* opacity = frame.findChild<pictura::NumericField*>(QStringLiteral("optionsBucketOpacity"));
    auto* tolerance =
        frame.findChild<pictura::NumericField*>(QStringLiteral("optionsBucketTolerance"));
    auto* antialias = frame.findChild<QCheckBox*>(QStringLiteral("optionsBucketAntialias"));
    auto* contiguous = frame.findChild<QCheckBox*>(QStringLiteral("optionsBucketContiguous"));
    auto* allLayers = frame.findChild<QCheckBox*>(QStringLiteral("optionsBucketAllLayers"));
    if (!fill || !pattern || !mode || !opacity || !tolerance || !antialias || !contiguous
        || !allLayers) {
        return pictura::selfTest().fail(553, "paint bucket bar");
    }
    const bool bar = fill->currentIndex() == 0 && !pattern->isEnabled() && pattern->count() > 0
        && mode->count() == 4 && opacity->value() == 100 && tolerance->value() == 32
        && antialias->isChecked() && contiguous->isChecked() && !allLayers->isChecked()
        && !pictura::isBrushTool(pictura::ToolId::PaintBucket);

    const int base = f.view->history_index();
    f.drag({QPointF(1, 1)});
    const bool filled = f.committedOnce(base, "Paint Bucket")
        && f.view->sample_argb(1, 1) == blue.rgba() && f.view->sample_argb(20, 20) == blue.rgba()
        && f.view->sample_argb(9, 9) == red.rgba() && f.view->sample_argb(29, 29) == red.rgba();

    // All Layers: the composite's red square, filled on an empty layer above.
    const int layer = f.view->add_layer(0);
    f.view->set_active_layer(QString::number(layer));
    f.tools->setForeground(green);
    allLayers->setChecked(true);
    const int above = f.view->history_index();
    f.drag({QPointF(9, 9)});
    const bool composite = f.committedOnce(above, "Paint Bucket")
        && f.view->sample_argb(9, 9) == green.rgba() && f.view->sample_argb(29, 29) == red.rgba()
        && f.view->sample_argb(1, 1) == blue.rgba();
    allLayers->setChecked(false);

    f.view->set_layer_lock(layer, QStringLiteral("pixels"), true);
    const int locked = f.view->history_index();
    f.drag({QPointF(1, 1)});
    const bool refused = f.view->history_index() == locked;

    // Contiguous off: every red pixel of the Background, under the green one.
    f.view->set_active_layer(QStringLiteral("0"));
    f.tools->setForeground(blue);
    contiguous->setChecked(false);
    f.drag({QPointF(29, 29)});
    const bool global = !f.tools->bucketOptions().contiguous
        && f.view->sample_argb(29, 29) == blue.rgba() && f.view->sample_argb(9, 9) == green.rgba();

    // Pattern: the checkerboard tile from the document origin, over all the blue.
    fill->setCurrentIndex(1);
    const int plain = f.view->history_index();
    f.drag({QPointF(1, 1)});
    const bool patterned = pattern->isEnabled() && f.tools->bucketOptions().fill == 1
        && f.committedOnce(plain, "Paint Bucket")
        && f.view->sample_argb(1, 1) == QColor(215, 215, 215).rgba()
        && f.view->sample_argb(33, 1) == QColor(90, 90, 90).rgba();
    fill->setCurrentIndex(0);
    contiguous->setChecked(true);

    ST_BEGIN("paint_bucket_tool");
    ST_PASS("paint bucket bar=%d fill=%d composite=%d refused=%d global=%d pattern=%d", bar ? 1 : 0,
            filled ? 1 : 0, composite ? 1 : 0, refused ? 1 : 0, global ? 1 : 0, patterned ? 1 : 0);
    if (!bar || !filled || !composite || !refused || !global || !patterned) {
        return pictura::selfTest().fail(553, "paint bucket tool");
    }
    return 0;
}
