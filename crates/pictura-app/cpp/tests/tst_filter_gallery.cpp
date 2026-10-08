#include <QtTest/QtTest>

#include <QtCore/QTimer>
#include <QtGui/QImage>
#include <QtWidgets/QApplication>
#include <QtWidgets/QListWidget>
#include <QtWidgets/QToolButton>

#include "commands.h"
#include "filter_gallery_dialog.h"
#include "frame.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include "qt_test_support.h"

namespace {

QString galleryId()
{
    return pictura::commandIdForPath({QStringLiteral("Filter"), QStringLiteral("Filter Gallery…")});
}

QStringList kinds(const QList<pictura::FilterGalleryDialog::Effect>& effects)
{
    QStringList out;
    for (const auto& effect : effects) {
        out.append(effect.kind);
    }
    return out;
}

} // namespace

class FilterGalleryTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void init();
    void categoriesFollowCs6();
    void menuEntryIsEnabled();
    void stackPreviewsAndCancelRestores();
    void okCommitsOneHistoryState();
    void effectLayersAddDeleteAndHide();
    void thumbnailsRender();
    void clickingTheEyeHidesTheEffect();

private:
    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void FilterGalleryTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void FilterGalleryTest::init()
{
    QVERIFY(window_->newDocument(QStringLiteral("Gallery"), 64, 64, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
}

void FilterGalleryTest::categoriesFollowCs6()
{
    const auto categories = pictura::FilterGalleryDialog::categories();
    QStringList names;
    for (const auto& category : categories) {
        names.append(category.first);
    }
    // Distort's Diffuse Glow and Glass have no kernel yet; Ocean Ripple keeps
    // the category.
    QCOMPARE(names, (QStringList{QStringLiteral("Artistic"), QStringLiteral("Brush Strokes"),
                                 QStringLiteral("Distort"), QStringLiteral("Sketch"),
                                 QStringLiteral("Stylize"), QStringLiteral("Texture")}));
    QCOMPARE(categories.at(0).second.size(), 15);
    QCOMPARE(categories.at(1).second.size(), 8);
    QCOMPARE(categories.at(3).second.size(), 14);
    QCOMPARE(categories.at(4).second.size(), 1);
    QCOMPARE(categories.at(4).second.first()->kind, QStringLiteral("glowing-edges"));
    QCOMPARE(categories.at(5).second.size(), 6);
}

void FilterGalleryTest::menuEntryIsEnabled()
{
    bool found = false;
    for (const pictura::CommandInfo& info : window_->registry()->describe()) {
        if (info.id == galleryId()) {
            found = true;
            QVERIFY(info.implemented);
            QVERIFY(info.enabled);
        }
    }
    QVERIFY(found);
}

void FilterGalleryTest::stackPreviewsAndCancelRestores()
{
    pictura::PictureView* view = window_->activeView();
    const QImage before = view->image();
    pictura::FilterGalleryDialog dialog(view);
    dialog.selectFilter(QStringLiteral("grain"));
    dialog.show();
    QTRY_VERIFY_WITH_TIMEOUT(view->image() != before, 3000);
    dialog.reject();
    QCOMPARE(view->image(), before);
}

void FilterGalleryTest::okCommitsOneHistoryState()
{
    pictura::PictureView* view = window_->activeView();
    const QImage before = view->image();
    const int states = view->history_count();
    pictura::FilterGalleryDialog dialog(view);
    dialog.selectFilter(QStringLiteral("grain"));
    dialog.addEffect();
    dialog.selectFilter(QStringLiteral("texturizer"));
    QCOMPARE(kinds(dialog.effects()), (QStringList{QStringLiteral("grain"),
                                                   QStringLiteral("texturizer")}));
    dialog.show();
    QTRY_VERIFY_WITH_TIMEOUT(view->image() != before, 3000);
    QVERIFY(dialog.commit());
    QVERIFY(view->image() != before);
    QCOMPARE(view->history_count(), states + 1);
    // CS6 reopens the gallery on the stack it last applied.
    pictura::FilterGalleryDialog again(view);
    QCOMPARE(kinds(again.effects()), kinds(dialog.effects()));
}

void FilterGalleryTest::effectLayersAddDeleteAndHide()
{
    pictura::PictureView* view = window_->activeView();
    const QImage before = view->image();
    pictura::FilterGalleryDialog dialog(view);
    // The gallery reopens on the session's last stack; start from one effect.
    while (dialog.effects().size() > 1) {
        dialog.deleteEffect();
    }
    dialog.selectFilter(QStringLiteral("cutout"));
    dialog.addEffect();
    QCOMPARE(dialog.effects().size(), 2);
    QCOMPARE(dialog.selectedEffect(), 1);
    // The new layer copies the selected one, settings included.
    QCOMPARE(dialog.effects().at(1).values, dialog.effects().at(0).values);
    dialog.deleteEffect();
    QCOMPARE(dialog.effects().size(), 1);
    // The last effect cannot be deleted.
    dialog.deleteEffect();
    QCOMPARE(dialog.effects().size(), 1);
    // With every effect hidden, OK changes nothing.
    dialog.setEffectVisible(0, false);
    QVERIFY(!dialog.effects().at(0).visible);
    QVERIFY(!dialog.commit());
    QCOMPARE(view->image(), before);
}

void FilterGalleryTest::thumbnailsRender()
{
    pictura::FilterGalleryDialog dialog(window_->activeView());
    auto* thumb = dialog.findChild<QToolButton*>(QStringLiteral("galleryThumb:cutout"));
    QVERIFY(thumb != nullptr);
    QTRY_VERIFY_WITH_TIMEOUT(!thumb->icon().isNull(), 3000);
}

void FilterGalleryTest::clickingTheEyeHidesTheEffect()
{
    pictura::PictureView* view = window_->activeView();
    const QImage before = view->image();
    pictura::FilterGalleryDialog dialog(view);
    while (dialog.effects().size() > 1) {
        dialog.deleteEffect();
    }
    dialog.selectFilter(QStringLiteral("grain"));
    dialog.setEffectVisible(0, true);
    dialog.show();
    QTRY_VERIFY_WITH_TIMEOUT(view->image() != before, 3000);
    auto* list = dialog.findChild<QListWidget*>(QStringLiteral("galleryEffects"));
    QVERIFY(list != nullptr);
    const QRect row = list->visualItemRect(list->item(0));
    // The eye sits at the row's left edge.
    QTest::mouseClick(list->viewport(), Qt::LeftButton, {}, QPoint(row.left() + 10, row.center().y()));
    QVERIFY(!dialog.effects().at(0).visible);
    // Hidden, the effect leaves the preview.
    QTRY_COMPARE_WITH_TIMEOUT(view->image(), before, 3000);
    QTest::mouseClick(list->viewport(), Qt::LeftButton, {}, QPoint(row.left() + 10, row.center().y()));
    QVERIFY(dialog.effects().at(0).visible);
    QTRY_VERIFY_WITH_TIMEOUT(view->image() != before, 3000);
    dialog.reject();
    QCOMPARE(view->image(), before);
}

QTEST_MAIN(FilterGalleryTest)
#include "tst_filter_gallery.moc"
