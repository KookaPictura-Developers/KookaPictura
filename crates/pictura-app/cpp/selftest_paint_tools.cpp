#include "selftest_paint_tools.h"
#include "selftest_report.h"

#include "frame.h"
#include "image_view.h"
#include "panels/numeric_field.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QDir>
#include <QtCore/QFile>
#include <QtGui/QImage>
#include <QtGui/QPainter>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QToolButton>

#include <functional>

namespace {

// A document opened from `seed`, closed (and its PNG removed) on scope exit.
struct Fixture {
    pictura::PicturaMainWindow& frame;
    QString path;
    pictura::PictureView* view = nullptr;
    pictura::ImageView* canvas = nullptr;
    pictura::ToolController* tools = nullptr;
    int doc = -1;

    Fixture(pictura::PicturaMainWindow& f, const QImage& seed, const QString& name)
        : frame(f)
        , path(QDir::temp().filePath(name + QStringLiteral(".png")))
    {
        if (!frame.newDocument(name, seed.width(), seed.height(), QStringLiteral("rgb"), 8,
                               QStringLiteral("white"))) {
            return;
        }
        doc = frame.activeDocumentIndex();
        pictura::PictureView* v = frame.activeView();
        if (v && seed.save(path, "PNG") && v->open_image(path)) {
            view = v;
            canvas = frame.imageView();
            tools = frame.findChild<pictura::ToolController*>();
        }
    }
    ~Fixture()
    {
        frame.setActiveTool(pictura::ToolId::Move);
        if (doc >= 0) {
            frame.closeDocument(doc, false);
        }
        QFile::remove(path);
    }
    bool ok() const { return view && canvas && tools; }

    void drag(const QList<QPointF>& points, Qt::KeyboardModifiers mods = Qt::NoModifier) const
    {
        canvas->mousePressed(points.first(), Qt::LeftButton, int(mods));
        for (const QPointF& p : points.mid(1)) {
            canvas->mouseMoved(p);
        }
        canvas->mouseReleased(points.last());
    }

    bool committedOnce(int base, const char* label) const
    {
        return view->history_index() == base + 1
            && view->history_label(base + 1) == QString::fromLatin1(label);
    }
};

} // namespace

int pictura::runRedEyeChecks(pictura::PicturaMainWindow& frame)
{
    // Seed: skin tone with a red 5×5 pupil at (18, 18).
    QImage seed(40, 40, QImage::Format_RGB32);
    seed.fill(QColor(215, 175, 150));
    QPainter(&seed).fillRect(18, 18, 5, 5, QColor(220, 30, 30));
    Fixture f(frame, seed, QStringLiteral("pictura_redeye_seed"));
    if (!f.ok()) {
        return pictura::selfTest().fail(539, "red eye fixture");
    }

    frame.setActiveTool(pictura::ToolId::RedEye);
    const bool active = f.tools->activeTool() == pictura::ToolId::RedEye;
    auto* pupil = frame.findChild<pictura::NumericField*>(QStringLiteral("optionsRedEyePupil"));
    auto* darken =
        frame.findChild<pictura::NumericField*>(QStringLiteral("optionsRedEyeDarken"));
    const bool bar = pupil && darken && pupil->value() == 50 && darken->value() == 50
        && f.tools->redEyePupil() == 50 && f.tools->redEyeDarken() == 50;

    const int base = f.view->history_index();
    f.drag({QPointF(14, 14), QPointF(20, 20), QPointF(26, 26)});
    const bool committed = f.committedOnce(base, "Red Eye Tool");
    const quint32 pupilArgb = f.view->sample_argb(20, 20);
    const bool fixed = qRed(pupilArgb) < 80;
    const bool skinKept = f.view->sample_argb(15, 15) == QColor(215, 175, 150).rgba();
    // A click on skin alone (no red in its box) records nothing.
    f.drag({QPointF(5, 5)});
    const bool clickNoop = f.view->history_index() == base + 1;

    ST_BEGIN("red_eye_tool");
    ST_PASS("red eye active=%d bar=%d commit=%d fixed=%d skin=%d click=%d", active ? 1 : 0,
            bar ? 1 : 0, committed ? 1 : 0, fixed ? 1 : 0, skinKept ? 1 : 0, clickNoop ? 1 : 0);
    if (!active || !bar || !committed || !fixed || !skinKept || !clickNoop) {
        return pictura::selfTest().fail(539, "red eye tool");
    }
    return 0;
}

int pictura::runColorReplacementChecks(pictura::PicturaMainWindow& frame)
{
    // Seed: a blue field (x < 30) beside a yellow one.
    QImage seed(60, 20, QImage::Format_RGB32);
    seed.fill(QColor(220, 220, 40));
    QPainter(&seed).fillRect(0, 0, 30, 20, QColor(40, 40, 200));
    Fixture f(frame, seed, QStringLiteral("pictura_replace_seed"));
    if (!f.ok()) {
        return pictura::selfTest().fail(540, "color replacement fixture");
    }

    frame.setActiveTool(pictura::ToolId::ColorReplacement);
    const bool active = f.tools->activeTool() == pictura::ToolId::ColorReplacement;
    auto combo = [&frame](const char* name) {
        return frame.findChild<QComboBox*>(QString::fromLatin1(name));
    };
    QComboBox* mode = combo("optionsColorReplaceMode");
    QComboBox* sampling = combo("optionsColorReplaceSampling");
    QComboBox* limits = combo("optionsColorReplaceLimits");
    bool bar = mode && sampling && limits && mode->currentIndex() == 2
        && sampling->currentIndex() == 0 && limits->currentIndex() == 1;
    if (bar) {
        mode->setCurrentIndex(3);
        bar = f.tools->colorReplaceOptions().mode == 3;
        mode->setCurrentIndex(2);
        bar = bar && f.tools->colorReplaceOptions().mode == 2;
    }

    const QColor foreground = f.tools->foreground();
    const int size = f.tools->brushSize();
    f.tools->setForeground(QColor(40, 200, 40));
    f.tools->setBrushSize(10);
    const int base = f.view->history_index();
    f.drag({QPointF(8, 10), QPointF(14, 10), QPointF(20, 10)});
    const bool committed = f.committedOnce(base, "Color Replacement Tool");
    const quint32 argb = f.view->sample_argb(14, 10);
    const bool recoloured = qGreen(argb) > qRed(argb) + 20 && qGreen(argb) > qBlue(argb);
    const bool farKept = f.view->sample_argb(45, 10) == QColor(220, 220, 40).rgba();
    f.tools->setForeground(foreground);
    f.tools->setBrushSize(size);

    ST_BEGIN("color_replacement_tool");
    ST_PASS("color replacement active=%d bar=%d commit=%d recoloured=%d far=%d",
            active ? 1 : 0, bar ? 1 : 0, committed ? 1 : 0, recoloured ? 1 : 0,
            farKept ? 1 : 0);
    if (!active || !bar || !committed || !recoloured || !farKept) {
        return pictura::selfTest().fail(540, "color replacement tool");
    }
    return 0;
}

int pictura::runMixerBrushChecks(pictura::PicturaMainWindow& frame)
{
    // Seed: a black band (x < 20) on white.
    QImage seed(64, 40, QImage::Format_RGB32);
    seed.fill(Qt::white);
    QPainter(&seed).fillRect(0, 0, 20, 40, Qt::black);
    Fixture f(frame, seed, QStringLiteral("pictura_mixer_seed"));
    if (!f.ok()) {
        return pictura::selfTest().fail(541, "mixer brush fixture");
    }

    frame.setActiveTool(pictura::ToolId::MixerBrush);
    const bool active = f.tools->activeTool() == pictura::ToolId::MixerBrush;
    const pictura::MixerOptions saved = f.tools->mixerOptions();
    const QColor savedPaint = f.tools->mixerReservoir();
    const int savedSize = f.tools->brushSize();

    // The preset menu sets Wet/Load/Mix; Load and Mix grey out while dry.
    auto* preset = frame.findChild<QComboBox*>(QStringLiteral("optionsMixerPreset"));
    auto* mix = frame.findChild<pictura::NumericField*>(QStringLiteral("optionsMixerMix"));
    bool bar = preset && mix && preset->currentText() == QStringLiteral("Dry")
        && !mix->isEnabled();
    if (bar) {
        const int wetHeavy = preset->findText(QStringLiteral("Wet, Heavy Mix"));
        preset->setCurrentIndex(wetHeavy);
        emit preset->activated(wetHeavy);
        const pictura::MixerOptions o = f.tools->mixerOptions();
        bar = o.wet == 50 && o.load == 50 && o.mix == 60 && mix->isEnabled()
            && mix->value() == 60;
    }

    // A fully wet smear from the black band into the white drags black along
    // and leaves the pickup on the brush.
    pictura::MixerOptions wet = saved;
    wet.wet = 100;
    wet.load = 100;
    wet.mix = 100;
    f.tools->setMixerOptions(wet);
    f.tools->setMixerReservoir(Qt::white);
    f.tools->setBrushSize(10);
    const int base = f.view->history_index();
    QList<QPointF> path;
    for (int x = 10; x <= 40; x += 2) {
        path << QPointF(x, 20);
    }
    f.drag(path);
    const bool committed = f.committedOnce(base, "Mixer Brush Tool");
    const bool smeared = qRed(f.view->sample_argb(26, 20)) < 250;
    const bool carried = f.tools->mixerReservoir().red() < 255;
    // Clean After Stroke empties the brush.
    wet.cleanAfterStroke = true;
    f.tools->setMixerOptions(wet);
    f.drag({QPointF(50, 30), QPointF(56, 30)});
    const bool cleaned = f.tools->mixerReservoir().alpha() == 0;
    // Alt-click loads the brush from the image.
    f.drag({QPointF(5, 5)}, Qt::AltModifier);
    const bool loaded = f.tools->mixerReservoir() == QColor(Qt::black);

    f.tools->setMixerOptions(saved);
    f.tools->setMixerReservoir(savedPaint);
    f.tools->setBrushSize(savedSize);

    ST_BEGIN("mixer_brush_tool");
    ST_PASS("mixer active=%d bar=%d commit=%d smeared=%d carried=%d cleaned=%d loaded=%d",
            active ? 1 : 0, bar ? 1 : 0, committed ? 1 : 0, smeared ? 1 : 0, carried ? 1 : 0,
            cleaned ? 1 : 0, loaded ? 1 : 0);
    if (!active || !bar || !committed || !smeared || !carried || !cleaned || !loaded) {
        return pictura::selfTest().fail(541, "mixer brush tool");
    }
    return 0;
}
