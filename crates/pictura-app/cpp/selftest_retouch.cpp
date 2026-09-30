#include "selftest_retouch.h"
#include "selftest_report.h"

#include "selftest_paint_fixture.h"

#include "panels/numeric_field.h"

#include <QtGui/QPainter>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>

namespace {
using paint_fixture::BrushState;
using paint_fixture::Fixture;
} // namespace

int pictura::runBlurToolChecks(pictura::PicturaMainWindow& frame)
{
    // White left of x = 20, black right of it.
    QImage seed(40, 40, QImage::Format_RGB32);
    seed.fill(Qt::white);
    QPainter(&seed).fillRect(20, 0, 20, 40, Qt::black);
    Fixture f(frame, seed, QStringLiteral("pictura_blur_seed"));
    if (!f.ok()) {
        return pictura::selfTest().fail(554, "blur fixture");
    }
    const BrushState brush(f.tools);
    f.tools->setBrushSize(12);
    const pictura::BlurOptions options = f.tools->blurOptions();

    frame.setActiveTool(pictura::ToolId::Blur);
    auto* mode = frame.findChild<QComboBox*>(QStringLiteral("optionsBlurMode"));
    auto* strength = frame.findChild<pictura::NumericField*>(QStringLiteral("optionsBlurStrength"));
    auto* sampleAll = frame.findChild<QCheckBox*>(QStringLiteral("optionsBlurSampleAll"));
    if (!mode || !strength || !sampleAll) {
        return pictura::selfTest().fail(554, "blur bar");
    }
    const bool bar = mode->count() == 7 && mode->currentIndex() == 0 && strength->value() == 50
        && !sampleAll->isChecked() && pictura::isBrushTool(pictura::ToolId::Blur);

    strength->setValue(100);
    const QRgb white = QColor(Qt::white).rgba();
    const QRgb black = QColor(Qt::black).rgba();
    const int base = f.view->history_index();
    f.drag({QPointF(20, 5), QPointF(20, 20), QPointF(20, 35), QPointF(20, 20)});
    const bool softened = f.committedOnce(base, "Blur") && f.view->sample_argb(19, 20) != white
        && f.view->sample_argb(20, 20) != black && f.view->sample_argb(2, 2) == white
        && f.view->sample_argb(37, 37) == black;

    // An empty layer has nothing of its own to blur; Sample All Layers reads the
    // composite and lands the result on it.
    const int layer = f.view->add_layer(0);
    f.view->set_active_layer(QString::number(layer));
    const int empty = f.view->history_index();
    f.drag({QPointF(20, 10), QPointF(20, 30)});
    const bool nothing = f.view->history_index() == empty;
    sampleAll->setChecked(true);
    f.drag({QPointF(20, 10), QPointF(20, 30)});
    const bool sampled = f.tools->blurOptions().sampleAllLayers && f.committedOnce(empty, "Blur");
    sampleAll->setChecked(false);
    strength->setValue(options.strength);

    ST_BEGIN("blur_tool");
    ST_PASS("blur bar=%d soften=%d empty=%d sample_all=%d", bar ? 1 : 0, softened ? 1 : 0,
            nothing ? 1 : 0, sampled ? 1 : 0);
    if (!bar || !softened || !nothing || !sampled) {
        return pictura::selfTest().fail(554, "blur tool");
    }
    return 0;
}
