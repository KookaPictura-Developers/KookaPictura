// The retouch tools: Blur (#26), Sharpen (#27), Smudge (#28), and Dodge (#29).

#include <QtTest/QtTest>

#include "panels/numeric_field.h"
#include "selftest_paint_fixture.h"

#include <QtGui/QPainter>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>

#include "qt_test_support.h"

namespace {
using paint_fixture::BrushState;
using paint_fixture::Fixture;
} // namespace

class RetouchToolsTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void blurTool();
    void sharpenTool();
    void smudgeTool();
    void dodgeTool();

private:
    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void RetouchToolsTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void RetouchToolsTest::blurTool()
{
    pictura::PicturaMainWindow& frame = *window_;
    // White left of x = 20, black right of it.
    QImage seed(40, 40, QImage::Format_RGB32);
    seed.fill(Qt::white);
    QPainter(&seed).fillRect(20, 0, 20, 40, Qt::black);
    Fixture f(frame, seed, QStringLiteral("pictura_blur_seed"));
    if (!f.ok()) {
        QFAIL("blur fixture");
    }
    const BrushState brush(f.tools);
    f.tools->setBrushSize(12);
    const pictura::RetouchOptions options = f.tools->retouchOptions(pictura::ToolId::Blur);

    frame.setActiveTool(pictura::ToolId::Blur);
    auto* mode = frame.findChild<QComboBox*>(QStringLiteral("optionsBlurMode"));
    auto* strength = frame.findChild<pictura::NumericField*>(QStringLiteral("optionsBlurStrength"));
    auto* sampleAll = frame.findChild<QCheckBox*>(QStringLiteral("optionsBlurSampleAll"));
    if (!mode || !strength || !sampleAll) {
        QFAIL("blur bar");
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
    const bool sampled = f.tools->retouchOptions(pictura::ToolId::Blur).sampleAllLayers && f.committedOnce(empty, "Blur");
    sampleAll->setChecked(false);
    strength->setValue(options.strength);

    QVERIFY2(bar, "bar");
    QVERIFY2(softened, "softened");
    QVERIFY2(nothing, "nothing");
    QVERIFY2(sampled, "sampled");
}

void RetouchToolsTest::sharpenTool()
{
    pictura::PicturaMainWindow& frame = *window_;
    // A grey step: 100 left of x = 20, 160 right of it.
    QImage seed(40, 40, QImage::Format_RGB32);
    seed.fill(QColor(100, 100, 100));
    QPainter(&seed).fillRect(20, 0, 20, 40, QColor(160, 160, 160));
    Fixture f(frame, seed, QStringLiteral("pictura_sharpen_seed"));
    if (!f.ok()) {
        QFAIL("sharpen fixture");
    }
    const BrushState brush(f.tools);
    f.tools->setBrushSize(12);
    const pictura::RetouchOptions options = f.tools->retouchOptions(pictura::ToolId::Sharpen);

    frame.setActiveTool(pictura::ToolId::Sharpen);
    auto* mode = frame.findChild<QComboBox*>(QStringLiteral("optionsSharpenMode"));
    auto* strength =
        frame.findChild<pictura::NumericField*>(QStringLiteral("optionsSharpenStrength"));
    auto* protect = frame.findChild<QCheckBox*>(QStringLiteral("optionsSharpenProtectDetail"));
    if (!mode || !strength || !protect) {
        QFAIL("sharpen bar");
    }
    const bool bar = mode->count() == 7 && strength->value() == 50 && protect->isChecked()
        && pictura::isBrushTool(pictura::ToolId::Sharpen);

    strength->setValue(100);
    protect->setChecked(false);
    const int base = f.view->history_index();
    f.drag({QPointF(20, 5), QPointF(20, 20), QPointF(20, 35)});
    const bool steeper = f.committedOnce(base, "Sharpen")
        && qRed(f.view->sample_argb(19, 20)) < 100 && qRed(f.view->sample_argb(20, 20)) > 160
        && qRed(f.view->sample_argb(2, 20)) == 100;
    protect->setChecked(options.protectDetail);
    strength->setValue(options.strength);

    QVERIFY2(bar, "bar");
    QVERIFY2(steeper, "steeper");
}

void RetouchToolsTest::smudgeTool()
{
    pictura::PicturaMainWindow& frame = *window_;
    // Black left of x = 20, white right of it.
    QImage seed(60, 40, QImage::Format_RGB32);
    seed.fill(Qt::white);
    QPainter(&seed).fillRect(0, 0, 20, 40, Qt::black);
    Fixture f(frame, seed, QStringLiteral("pictura_smudge_seed"));
    if (!f.ok()) {
        QFAIL("smudge fixture");
    }
    const BrushState brush(f.tools);
    f.tools->setBrushSize(12);
    const pictura::RetouchOptions options = f.tools->retouchOptions(pictura::ToolId::Smudge);
    const QColor foreground = f.tools->foreground();

    frame.setActiveTool(pictura::ToolId::Smudge);
    auto* strength =
        frame.findChild<pictura::NumericField*>(QStringLiteral("optionsSmudgeStrength"));
    auto* finger = frame.findChild<QCheckBox*>(QStringLiteral("optionsSmudgeFingerPainting"));
    if (!strength || !finger) {
        QFAIL("smudge bar");
    }
    const bool bar = strength->value() == 50 && !finger->isChecked()
        && frame.findChild<QCheckBox*>(QStringLiteral("optionsSmudgeSampleAll"))
        && pictura::isBrushTool(pictura::ToolId::Smudge);

    strength->setValue(90);
    QList<QPointF> path;
    for (int x = 14; x <= 40; ++x) {
        path << QPointF(x, 12);
    }
    const int base = f.view->history_index();
    f.drag(path);
    const QRgb white = QColor(Qt::white).rgba();
    const bool dragged = f.committedOnce(base, "Smudge")
        && qRed(f.view->sample_argb(30, 12)) < 240 && f.view->sample_argb(50, 35) == white;

    // Finger Painting drags the foreground into the white.
    finger->setChecked(true);
    f.tools->setForeground(QColor(220, 20, 20));
    path.clear();
    for (int x = 30; x <= 50; ++x) {
        path << QPointF(x, 30);
    }
    f.drag(path);
    const QRgb painted = f.view->sample_argb(34, 30);
    const bool fingerOk = f.view->history_index() == base + 2
        && qRed(painted) > qGreen(painted) + 40;
    finger->setChecked(options.fingerPainting);
    strength->setValue(options.strength);
    f.tools->setForeground(foreground);

    QVERIFY2(bar, "bar");
    QVERIFY2(dragged, "dragged");
    QVERIFY2(fingerOk, "fingerOk");
}

void RetouchToolsTest::dodgeTool()
{
    pictura::PicturaMainWindow& frame = *window_;
    QImage seed(40, 40, QImage::Format_RGB32);
    seed.fill(QColor(90, 90, 90));
    Fixture f(frame, seed, QStringLiteral("pictura_dodge_seed"));
    if (!f.ok()) {
        QFAIL("dodge fixture");
    }
    const BrushState brush(f.tools);
    f.tools->setBrushSize(12);

    frame.setActiveTool(pictura::ToolId::Dodge);
    auto* range = frame.findChild<QComboBox*>(QStringLiteral("optionsDodgeRange"));
    auto* exposure =
        frame.findChild<pictura::NumericField*>(QStringLiteral("optionsDodgeExposure"));
    auto* protect = frame.findChild<QCheckBox*>(QStringLiteral("optionsDodgeProtectTones"));
    if (!range || !exposure || !protect) {
        QFAIL("dodge bar");
    }
    const bool bar = range->count() == 3 && range->currentIndex() == 1 && exposure->value() == 50
        && protect->isChecked() && pictura::isBrushTool(pictura::ToolId::Dodge);

    const int base = f.view->history_index();
    f.drag({QPointF(5, 20), QPointF(15, 20), QPointF(25, 20), QPointF(35, 20)});
    const int once = qRed(f.view->sample_argb(20, 20));
    const bool lifted = f.committedOnce(base, "Dodge") && once > 95 && once < 150
        && qRed(f.view->sample_argb(20, 2)) == 90;
    f.drag({QPointF(5, 20), QPointF(15, 20), QPointF(25, 20), QPointF(35, 20)});
    const bool deeper = qRed(f.view->sample_argb(20, 20)) > once;

    QVERIFY2(bar, "bar");
    QVERIFY2(lifted, "lifted");
    QVERIFY2(deeper, "deeper");
}

QTEST_MAIN(RetouchToolsTest)
#include "tst_retouch_tools.moc"
