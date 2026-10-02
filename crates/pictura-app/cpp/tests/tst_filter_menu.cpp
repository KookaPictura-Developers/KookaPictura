#include <QtTest/QtTest>

#include <QtCore/QStringList>
#include <QtCore/QTimer>
#include <QtGui/QAction>
#include <QtGui/QImage>
#include <QtWidgets/QApplication>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QDialog>
#include <QtWidgets/QDoubleSpinBox>
#include <QtWidgets/QLabel>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QWidget>

#include "commands.h"
#include "filter_commands.h"
#include "filter_preview_dialog.h"
#include "frame.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/filter_tools.cxxqt.h"

#include "qt_test_support.h"

namespace {

QString filterId(const QString& family, const QString& leaf)
{
    return pictura::commandIdForPath({QStringLiteral("Filter"), family, leaf});
}

// Dialogs are shown non-modally (no compositor parent-dim), so they are not
// active modal widgets; find the open filter dialog among the top-level windows.
pictura::FilterPreviewDialog* activeFilterDialog()
{
    for (QWidget* widget : QApplication::topLevelWidgets()) {
        if (auto* dialog = qobject_cast<pictura::FilterPreviewDialog*>(widget)) {
            return dialog;
        }
    }
    return nullptr;
}

} // namespace

class FilterMenuTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void lastFilterDisabledBeforeFirstCommit();
    void implementedEntriesAreEnabled();
    void stubEntriesStayDisabled();
    void dialogCollectsParameterSlots();
    void radialBlurHasNoThumbnail();
    void parameterlessAppliesDirectly();
    void previewCancelRestores();
    void lastFilterReappliesAndNamesItself();
    void parameterizedCommandOpensDialogAndCommits();
    void lastFilterSettingsReopensPrefilled();
    void everyRowsKindIsSupportedAndArityMatches();
    void dialogRejectRestoresPixels();
    void nonPixelActiveLayerDisablesFilters();
    void addedRowsAreImplemented();
    void grayscaleDocumentAppliesThroughDialog();
    void previewToggleShowsAndRevertsPixels();
    void zoomChangesOnlyThumbnailAndLabel();
    void dialogEntriesEndWithEllipsis();

private:
    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void FilterMenuTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
    QVERIFY(window_->newDocument(QStringLiteral("FilterMenu"), 64, 64, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    QVERIFY(window_->activeView() != nullptr);
}

void FilterMenuTest::lastFilterDisabledBeforeFirstCommit()
{
    pictura::CommandRegistry* registry = window_->registry();
    QVERIFY(registry != nullptr);
    registry->refresh();
    QAction* last = registry->action(QString::fromLatin1(pictura::command_ids::FilterLastFilter));
    QVERIFY(last != nullptr);
    QVERIFY(!last->isEnabled());
}

void FilterMenuTest::implementedEntriesAreEnabled()
{
    pictura::CommandRegistry* registry = window_->registry();
    QVERIFY(registry != nullptr);
    for (const pictura::CommandInfo& info : registry->describe()) {
        if (info.id == filterId(QStringLiteral("Blur"), QStringLiteral("Gaussian Blur"))
            || info.id == filterId(QStringLiteral("Sharpen"), QStringLiteral("Unsharp Mask"))
            || info.id == filterId(QStringLiteral("Stylize"), QStringLiteral("Extrude"))) {
            QVERIFY2(info.implemented, qPrintable(info.id));
            QVERIFY2(info.enabled, qPrintable(info.id));
        }
    }
    // Every table row must resolve back to its menu path and be implemented.
    for (const pictura::FilterCommandSpec& spec : pictura::filterCommands()) {
        QCOMPARE(pictura::filterCommandForPath(spec.path), &spec);
        bool found = false;
        for (const pictura::CommandInfo& info : registry->describe()) {
            if (info.id == pictura::commandIdForPath(spec.path)) {
                found = true;
                QVERIFY2(info.implemented, qPrintable(spec.kind));
            }
        }
        QVERIFY2(found, qPrintable(spec.kind));
    }
}

void FilterMenuTest::stubEntriesStayDisabled()
{
    pictura::CommandRegistry* registry = window_->registry();
    QVERIFY(registry != nullptr);
    for (const pictura::CommandInfo& info : registry->describe()) {
        if (info.id == filterId(QStringLiteral("Noise"), QStringLiteral("Reduce Noise"))
            || info.id == filterId(QStringLiteral("Stylize"), QStringLiteral("Glowing Edges"))) {
            QVERIFY2(!info.implemented, qPrintable(info.id));
            QVERIFY2(!info.enabled, qPrintable(info.id));
        }
    }
}

void FilterMenuTest::dialogCollectsParameterSlots()
{
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    struct Case {
        QString family;
        QString leaf;
        int count;
    };
    const Case cases[] = {
        {QStringLiteral("Blur"), QStringLiteral("Gaussian Blur"), 1},
        {QStringLiteral("Blur"), QStringLiteral("Radial Blur"), 3},
        {QStringLiteral("Distort"), QStringLiteral("Shear"), 7},
        {QStringLiteral("Render"), QStringLiteral("Clouds"), 8},
        {QStringLiteral("Render"), QStringLiteral("Lens Flare"), 4},
        {QStringLiteral("Noise"), QStringLiteral("Add Noise"), 4},
        {QStringLiteral("Stylize"), QStringLiteral("Extrude"), 6},
        {QStringLiteral("Sharpen"), QStringLiteral("Smart Sharpen"), 12},
    };
    for (const Case& c : cases) {
        const pictura::FilterCommandSpec* spec =
            pictura::filterCommandForPath({QStringLiteral("Filter"), c.family, c.leaf});
        QVERIFY2(spec != nullptr, qPrintable(c.leaf));
        pictura::FilterPreviewDialog dialog(view, *spec);
        QCOMPARE(dialog.values().size(), c.count);
    }
}

void FilterMenuTest::radialBlurHasNoThumbnail()
{
    const pictura::FilterCommandSpec* spec = pictura::filterCommandForPath(
        {QStringLiteral("Filter"), QStringLiteral("Blur"), QStringLiteral("Radial Blur")});
    QVERIFY(spec != nullptr);
    QVERIFY(!spec->previewPane);
    pictura::FilterPreviewDialog dialog(window_->activeView(), *spec);
    QCOMPARE(dialog.windowTitle(), QStringLiteral("Radial Blur"));
}

void FilterMenuTest::parameterlessAppliesDirectly()
{
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    const int before = view->history_count();
    QVERIFY(window_->registry()->dispatch(
        filterId(QStringLiteral("Sharpen"), QStringLiteral("Sharpen"))));
    QCOMPARE(view->history_count(), before + 1);
    QCOMPARE(view->history_label(view->history_index()), QStringLiteral("Filter"));
}

void FilterMenuTest::previewCancelRestores()
{
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    const QImage before = view->image();
    QVERIFY(filter_preview(*view, QStringLiteral("gaussian-blur"), {4.0}));
    QVERIFY(filter_preview_cancel(*view));
    QCOMPARE(view->image(), before);
}

void FilterMenuTest::lastFilterReappliesAndNamesItself()
{
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    QVERIFY(apply_filter_params(*view, QStringLiteral("gaussian-blur"), {5.0}));
    QVERIFY(filter_has_last(*view));
    QCOMPARE(filter_last_kind(*view), QStringLiteral("gaussian-blur"));
    QCOMPARE(filter_last_params(*view), QList<double>({5.0}));

    pictura::CommandRegistry* registry = window_->registry();
    QVERIFY(registry != nullptr);
    registry->refresh();
    QAction* last = registry->action(QString::fromLatin1(pictura::command_ids::FilterLastFilter));
    QVERIFY(last != nullptr);
    QVERIFY(last->isEnabled());
    QCOMPARE(last->text(), QStringLiteral("Gaussian Blur"));

    const int before = view->history_count();
    QVERIFY(registry->dispatch(QString::fromLatin1(pictura::command_ids::FilterLastFilter)));
    QCOMPARE(view->history_count(), before + 1);
}

void FilterMenuTest::parameterizedCommandOpensDialogAndCommits()
{
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    QList<double> seen;
    QTimer::singleShot(0, [&seen] {
        if (auto* dialog = activeFilterDialog()) {
            seen = dialog->values();
            dialog->accept();
        }
    });
    const int before = view->history_count();
    QVERIFY(window_->registry()->dispatch(
        filterId(QStringLiteral("Blur"), QStringLiteral("Gaussian Blur"))));
    QCOMPARE(seen.size(), 1);
    QCOMPARE(view->history_count(), before + 1);
    QCOMPARE(filter_last_kind(*view), QStringLiteral("gaussian-blur"));
}

void FilterMenuTest::lastFilterSettingsReopensPrefilled()
{
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    QVERIFY(apply_filter_params(*view, QStringLiteral("gaussian-blur"), {7.0}));
    QList<double> seen;
    QTimer::singleShot(0, [&seen] {
        if (auto* dialog = activeFilterDialog()) {
            seen = dialog->values();
            dialog->accept();
        }
    });
    QVERIFY(window_->registry()->dispatch(
        QString::fromLatin1(pictura::command_ids::FilterLastFilterSettings)));
    QCOMPARE(seen, QList<double>({7.0}));
}

void FilterMenuTest::everyRowsKindIsSupportedAndArityMatches()
{
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    for (const pictura::FilterCommandSpec& spec : pictura::filterCommands()) {
        QVERIFY2(pictura::filter_kind_supported(spec.kind), qPrintable(spec.kind));
        pictura::FilterPreviewDialog dialog(view, spec);
        QCOMPARE(static_cast<int>(dialog.values().size()),
                 static_cast<int>(pictura::filter_param_arity(spec.kind)));
        QVERIFY2(filter_preview(*view, spec.kind, dialog.values()), qPrintable(spec.kind));
        QVERIFY(filter_preview_cancel(*view));
    }
}

void FilterMenuTest::dialogRejectRestoresPixels()
{
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    const pictura::FilterCommandSpec* spec = pictura::filterCommandForPath(
        {QStringLiteral("Filter"), QStringLiteral("Blur"), QStringLiteral("Gaussian Blur")});
    QVERIFY(spec != nullptr);
    const QImage before = view->image();
    pictura::FilterPreviewDialog dialog(view, *spec);
    auto* spin = dialog.findChild<QDoubleSpinBox*>();
    QVERIFY(spin != nullptr);
    spin->setValue(spin->value() + 3.0);
    dialog.reject();
    QCOMPARE(view->image(), before);
}

void FilterMenuTest::nonPixelActiveLayerDisablesFilters()
{
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    const QString previous = view->active_layer_path();
    view->set_active_layer(QString());
    QVERIFY(!filter_target_ready(*view));
    window_->registry()->refresh();
    QAction* blur = window_->registry()->action(
        filterId(QStringLiteral("Blur"), QStringLiteral("Gaussian Blur")));
    QVERIFY(blur != nullptr);
    QVERIFY(!blur->isEnabled());
    view->set_active_layer(previous);
    window_->registry()->refresh();
}

void FilterMenuTest::addedRowsAreImplemented()
{
    pictura::CommandRegistry* registry = window_->registry();
    QVERIFY(registry != nullptr);
    const QString ids[] = {
        filterId(QStringLiteral("Blur"), QStringLiteral("Blur")),
        filterId(QStringLiteral("Blur"), QStringLiteral("Blur More")),
        pictura::commandIdForPath({QStringLiteral("Filter"), QStringLiteral("Oil Paint…")}),
    };
    for (const QString& id : ids) {
        bool found = false;
        for (const pictura::CommandInfo& info : registry->describe()) {
            if (info.id == id) {
                found = true;
                QVERIFY2(info.implemented, qPrintable(id));
            }
        }
        QVERIFY2(found, qPrintable(id));
    }
}

void FilterMenuTest::grayscaleDocumentAppliesThroughDialog()
{
    QVERIFY(window_->newDocument(QStringLiteral("Gray"), 32, 32, QStringLiteral("grayscale"), 8,
                                 QStringLiteral("white")));
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    const QImage before = view->image();
    const int historyBefore = view->history_count();
    QTimer::singleShot(0, [] {
        if (auto* dialog = activeFilterDialog()) {
            dialog->accept();
        }
    });
    QVERIFY(window_->registry()->dispatch(
        filterId(QStringLiteral("Noise"), QStringLiteral("Add Noise"))));
    QCOMPARE(view->history_count(), historyBefore + 1);
    QCOMPARE(filter_last_kind(*view), QStringLiteral("add-noise"));
    QVERIFY(view->image() != before);
}

void FilterMenuTest::previewToggleShowsAndRevertsPixels()
{
    QVERIFY(window_->newDocument(QStringLiteral("Preview"), 64, 64, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    const QImage before = view->image();
    bool changedOnEdit = false;
    bool restoredOnUncheck = false;
    bool changedOnRecheck = false;
    QTimer::singleShot(0, [&] {
        auto* dialog = activeFilterDialog();
        if (!dialog) {
            return;
        }
        auto* amount = dialog->findChild<QDoubleSpinBox*>();
        auto* preview = dialog->findChild<QCheckBox*>(QStringLiteral("filterPreview"));
        if (!amount || !preview) {
            dialog->reject();
            return;
        }
        amount->setValue(amount->value() + 50.0);
        changedOnEdit = view->image() != before;
        preview->setChecked(false);
        restoredOnUncheck = view->image() == before;
        preview->setChecked(true);
        changedOnRecheck = view->image() != before;
        dialog->reject();
    });
    QVERIFY(window_->registry()->dispatch(
        filterId(QStringLiteral("Noise"), QStringLiteral("Add Noise"))));
    QCOMPARE(view->image(), before);
    QVERIFY(changedOnEdit);
    QVERIFY(restoredOnUncheck);
    QVERIFY(changedOnRecheck);
}

void FilterMenuTest::zoomChangesOnlyThumbnailAndLabel()
{
    QVERIFY(window_->newDocument(QStringLiteral("Zoom"), 64, 64, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    const QImage before = view->image();
    QString first;
    QString afterIn;
    QString afterOut;
    QTimer::singleShot(0, [&] {
        auto* dialog = activeFilterDialog();
        if (!dialog) {
            return;
        }
        auto* label = dialog->findChild<QLabel*>(QStringLiteral("filterZoomLabel"));
        auto* zoomIn = dialog->findChild<QToolButton*>(QStringLiteral("filterZoomIn"));
        auto* zoomOut = dialog->findChild<QToolButton*>(QStringLiteral("filterZoomOut"));
        if (!label || !zoomIn || !zoomOut) {
            dialog->reject();
            return;
        }
        first = label->text();
        zoomIn->click();
        afterIn = label->text();
        zoomOut->click();
        zoomOut->click();
        afterOut = label->text();
        dialog->reject();
    });
    QVERIFY(window_->registry()->dispatch(
        filterId(QStringLiteral("Blur"), QStringLiteral("Gaussian Blur"))));
    QCOMPARE(first, QStringLiteral("100%"));
    QCOMPARE(afterIn, QStringLiteral("200%"));
    QCOMPARE(afterOut, QStringLiteral("50%"));
    QCOMPARE(view->image(), before);
}

void FilterMenuTest::dialogEntriesEndWithEllipsis()
{
    pictura::CommandRegistry* registry = window_->registry();
    QVERIFY(registry != nullptr);
    registry->refresh();
    for (const pictura::FilterCommandSpec& spec : pictura::filterCommands()) {
        QAction* action = registry->action(pictura::commandIdForPath(spec.path));
        QVERIFY2(action != nullptr, qPrintable(spec.kind));
        if (spec.params.isEmpty()) {
            QVERIFY2(!action->text().endsWith(QStringLiteral("…")), qPrintable(spec.kind));
        } else {
            QVERIFY2(action->text().endsWith(QStringLiteral("…")), qPrintable(spec.kind));
        }
    }
}

QTEST_MAIN(FilterMenuTest)
#include "tst_filter_menu.moc"
