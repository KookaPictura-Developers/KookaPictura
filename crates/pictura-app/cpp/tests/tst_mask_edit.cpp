// Painting and filling the active layer mask (#108): with the mask edit target
// active, the Brush and Edit > Fill write the mask's coverage (black hides,
// white reveals) and record their normal history state, leaving the layer's
// pixels untouched.

#include <QtTest/QtTest>

#include "frame.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/impl_layers/layer_masks.cxxqt.h"
#include "pictura_app/src/cxxqt_object/paint_tools/fills.cxxqt.h"

#include "qt_test_support.h"

class MaskEditTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void maskEditTarget();
    void paintAndFillWriteCoverage();

private:
    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void MaskEditTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void MaskEditTest::maskEditTarget()
{
    pictura::PicturaMainWindow& frame = *window_;
    const bool created = frame.newDocument(QStringLiteral("MaskTarget"), 16, 16,
                                           QStringLiteral("rgb"), 8, QStringLiteral("white"));
    pictura::PictureView* view = frame.activeView();
    QVERIFY2(created && view, "mask target fixture");
    const int doc = frame.activeDocumentIndex();
    const QString path = view->add_solid_fill(0xff2244aau);
    QVERIFY(!path.isEmpty());
    QVERIFY(view->rasterize_fill_content(path));
    frame.selectLayerPath(path);

    // Without a mask the target refuses; with one it round-trips and clears.
    QVERIFY2(!pictura::mask_edit_target_set(*view, path), "no mask yet");
    QVERIFY2(pictura::layer_mask_add(*view, QStringLiteral("reveal-all")), "add mask");
    QVERIFY2(pictura::mask_edit_target_set(*view, path), "target the mask");
    QCOMPARE(pictura::mask_edit_target_get(*view), path);
    QVERIFY2(pictura::mask_edit_target_set(*view, QString()), "clear the target");
    QVERIFY(pictura::mask_edit_target_get(*view).isEmpty());
    QVERIFY2(!pictura::mask_edit_target_set(*view, QStringLiteral("9")), "unknown path");

    // Deleting the mask empties a live target.
    QVERIFY(pictura::mask_edit_target_set(*view, path));
    QVERIFY(pictura::layer_mask_delete(*view));
    QVERIFY(pictura::mask_edit_target_get(*view).isEmpty());
    frame.closeDocument(doc, false);
}

void MaskEditTest::paintAndFillWriteCoverage()
{
    pictura::PicturaMainWindow& frame = *window_;
    const bool created = frame.newDocument(QStringLiteral("MaskPaint"), 32, 32,
                                           QStringLiteral("rgb"), 8, QStringLiteral("white"));
    pictura::PictureView* view = frame.activeView();
    QVERIFY2(created && view, "mask paint fixture");
    const int doc = frame.activeDocumentIndex();
    const QString path = view->add_solid_fill(0xff2244aau);
    QVERIFY(!path.isEmpty());
    QVERIFY(view->rasterize_fill_content(path));
    frame.selectLayerPath(path);
    QVERIFY2(pictura::layer_mask_add(*view, QStringLiteral("reveal-all")), "add mask");
    QVERIFY2(pictura::mask_edit_target_set(*view, path), "target the mask");

    const quint32 fill = view->sample_argb(16, 16);
    const int base = view->history_count();

    // Black on the mask hides the layer; the white background shows through.
    QVERIFY(view->begin_paint(0xff000000u, 0xffffffffu, 20, 100, 100, 0, 100, 100, 25,
                              QStringLiteral("normal"), false, false));
    QVERIFY(view->paint_dab(16.0, 16.0, 1.0));
    QVERIFY(view->end_paint());
    QCOMPARE(view->history_count(), base + 1);
    QCOMPARE(view->history_label(base), QStringLiteral("Brush"));
    QVERIFY2(view->sample_argb(16, 16) != fill, "the mask hid the layer");

    // Undo restores the coverage bit-for-bit; the layer shows again.
    QVERIFY(view->undo());
    QCOMPARE(view->sample_argb(16, 16), fill);

    // White on the mask reveals a layer hidden by a black mask.
    QVERIFY(pictura::layer_mask_delete(*view));
    QVERIFY2(pictura::layer_mask_add(*view, QStringLiteral("hide-all")), "hide all");
    QVERIFY(pictura::mask_edit_target_set(*view, path));
    QVERIFY(view->begin_paint(0xffffffffu, 0xff000000u, 20, 100, 100, 0, 100, 100, 25,
                              QStringLiteral("normal"), false, false));
    QVERIFY(view->paint_dab(16.0, 16.0, 1.0));
    QVERIFY(view->end_paint());
    QCOMPARE(view->sample_argb(16, 16), fill);

    // Cancelling a mask stroke puts the coverage back.
    QVERIFY(view->begin_paint(0xff000000u, 0xffffffffu, 20, 100, 100, 0, 100, 100, 25,
                              QStringLiteral("normal"), false, false));
    QVERIFY(view->paint_dab(16.0, 16.0, 1.0));
    view->cancel_paint();
    QCOMPARE(view->sample_argb(16, 16), fill);

    // Edit > Fill on the mask writes coverage through the selection.
    QVERIFY(pictura::layer_mask_delete(*view));
    QVERIFY2(pictura::layer_mask_add(*view, QStringLiteral("reveal-all")), "reveal all");
    QVERIFY(pictura::mask_edit_target_set(*view, path));
    view->select_rect(0, 0, 16, 32, QStringLiteral("new"), 0.0);
    const int fillBase = view->history_count();
    QVERIFY(pictura::edit_fill(*view, 0xff000000u, -1, QStringLiteral("normal"), 100, false));
    QCOMPARE(view->history_count(), fillBase + 1);
    QCOMPARE(view->history_label(fillBase), QStringLiteral("Fill"));
    QVERIFY2(view->sample_argb(8, 16) != fill, "the filled mask hid the layer");
    QCOMPARE(view->sample_argb(24, 16), fill);

    frame.closeDocument(doc, false);
}

QTEST_MAIN(MaskEditTest)
#include "tst_mask_edit.moc"
