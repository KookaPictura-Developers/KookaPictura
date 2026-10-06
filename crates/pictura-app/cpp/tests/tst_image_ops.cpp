// Image > Duplicate / Trim (#84): ported from photorust.

#include <QtTest/QtTest>

#include "frame.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/image_adjust/image_ops.cxxqt.h"
#include "pictura_app/src/cxxqt_object/paint_tools/fills.cxxqt.h"

#include "qt_test_support.h"

class ImageOpsTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void duplicateDocument();
    void trimDocument();

private:
    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void ImageOpsTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void ImageOpsTest::duplicateDocument()
{
    pictura::PicturaMainWindow& frame = *window_;
    QVERIFY(frame.newDocument(QStringLiteral("Dup"), 20, 16, QStringLiteral("rgb"), 8,
                              QStringLiteral("white")));
    pictura::PictureView* source = frame.activeView();
    QVERIFY(source);
    source->select_all();
    QVERIFY(pictura::edit_fill(*source, QColor(30, 60, 90).rgba(), -1, QStringLiteral("normal"),
                              100, false));
    source->deselect();
    const int doc = frame.activeDocumentIndex();

    const int before = frame.documentCount();
    auto* target = new pictura::PictureView(&frame);
    QVERIFY(pictura::duplicate_into(*source, *target, QStringLiteral("copy"), false));
    QCOMPARE(target->document_width(), 20);
    QCOMPARE(target->document_height(), 16);
    QCOMPARE(target->sample_argb(5, 5), QColor(30, 60, 90).rgba());
    QCOMPARE(target->history_count(), 1);
    QCOMPARE(target->history_label(0), QStringLiteral("Duplicate"));
    delete target;
    QCOMPARE(frame.documentCount(), before);

    QVERIFY(pictura::duplicate_name(*source).endsWith(QStringLiteral(" copy")));
    frame.closeDocument(doc, false);
}

void ImageOpsTest::trimDocument()
{
    pictura::PicturaMainWindow& frame = *window_;
    // A transparent 24×24 canvas; fill a 5×4 block at (3, 2), then trim.
    QVERIFY(frame.newDocument(QStringLiteral("Trim"), 24, 24, QStringLiteral("rgb"), 8,
                              QStringLiteral("transparent")));
    pictura::PictureView* view = frame.activeView();
    QVERIFY(view);
    const int doc = frame.activeDocumentIndex();

    QVERIFY(view->select_rect(3, 2, 5, 4, QStringLiteral("new"), 0.0));
    QVERIFY(pictura::edit_fill(*view, QColor(200, 0, 0).rgba(), -1, QStringLiteral("normal"), 100,
                               false));
    view->deselect();

    QVERIFY(pictura::trim_image(*view));
    QCOMPARE(view->history_count(), 1);
    QCOMPARE(view->history_label(0), QStringLiteral("Trim"));
    QCOMPARE(view->document_width(), 5);
    QCOMPARE(view->document_height(), 4);
    QCOMPARE(view->sample_argb(0, 0), QColor(200, 0, 0).rgba());

    // A second trim has nothing to remove.
    QVERIFY(!pictura::trim_image(*view));
    frame.closeDocument(doc, false);
}

QTEST_MAIN(ImageOpsTest)
#include "tst_image_ops.moc"
