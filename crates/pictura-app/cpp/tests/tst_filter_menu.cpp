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
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSlider>
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
    void previewShowsWhenDialogOpens();
    void zoomChangesOnlyThumbnailAndLabel();
    void zoomFollowsKeyboardShortcuts();
    void sliderDragDefersPreviewUntilRelease();
    void numericFieldIsCompactAndSliderAlignsLeft();
    void dialogEntriesEndWithEllipsis();
    void colorParameterIsASwatch();
    void artisticDialogDefaults();
    void underpaintingStaysInOneColumn();
    void smartSharpenStacksInOneWideColumn();

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
        if (info.id == filterId(QStringLiteral("Noise"), QStringLiteral("Reduce Noise"))) {
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
        {QStringLiteral("Distort"), QStringLiteral("Shear"), 18},
        {QStringLiteral("Render"), QStringLiteral("Clouds"), 8},
        {QStringLiteral("Render"), QStringLiteral("Lens Flare"), 4},
        {QStringLiteral("Render"), QStringLiteral("Lighting Effects"), 22},
        {QStringLiteral("Noise"), QStringLiteral("Add Noise"), 4},
        {QStringLiteral("Stylize"), QStringLiteral("Diffuse"), 1},
        {QStringLiteral("Stylize"), QStringLiteral("Extrude"), 6},
        {QStringLiteral("Stylize"), QStringLiteral("Glowing Edges"), 3},
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

void FilterMenuTest::previewShowsWhenDialogOpens()
{
    QVERIFY(window_->newDocument(QStringLiteral("OpenPreview"), 64, 64, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    const QImage before = view->image();
    bool shownUntouched = false;
    QTimer::singleShot(0, [&] {
        auto* dialog = activeFilterDialog();
        if (!dialog) {
            return;
        }
        // No control is touched: opening alone must preview.
        QTRY_VERIFY_WITH_TIMEOUT(view->image() != before, 2000);
        shownUntouched = true;
        dialog->reject();
    });
    QVERIFY(window_->registry()->dispatch(
        filterId(QStringLiteral("Noise"), QStringLiteral("Add Noise"))));
    QVERIFY(shownUntouched);
    QCOMPARE(view->image(), before);
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

// Ctrl++ / Ctrl+- zoom the preview as the buttons do (#243), even with a
// parameter field focused and the frame's input blocked by the open dialog;
// `+` arrives as Ctrl+= or Ctrl+Shift+= on most layouts.
void FilterMenuTest::zoomFollowsKeyboardShortcuts()
{
    QStringList seen;
    QTimer::singleShot(0, [&] {
        auto* dialog = activeFilterDialog();
        if (!dialog) {
            return;
        }
        dialog->activateWindow();
        auto* label = dialog->findChild<QLabel*>(QStringLiteral("filterZoomLabel"));
        auto* field = dialog->findChild<QDoubleSpinBox*>();
        if (!QTest::qWaitForWindowActive(dialog) || !label || !field) {
            dialog->reject();
            return;
        }
        field->setFocus();
        seen.append(label->text());
        QTest::keyClick(field, Qt::Key_Equal, Qt::ControlModifier);
        seen.append(label->text());
        QTest::keyClick(field, Qt::Key_Minus, Qt::ControlModifier);
        seen.append(label->text());
        QTest::keyClick(field, Qt::Key_Plus, Qt::ControlModifier | Qt::ShiftModifier);
        seen.append(label->text());
        QTest::keyClick(field, Qt::Key_Plus, Qt::ControlModifier | Qt::KeypadModifier);
        seen.append(label->text());
        dialog->reject();
    });
    QVERIFY(window_->registry()->dispatch(
        filterId(QStringLiteral("Blur"), QStringLiteral("Gaussian Blur"))));
    QCOMPARE(seen, QStringList({QStringLiteral("100%"), QStringLiteral("200%"),
                                QStringLiteral("100%"), QStringLiteral("200%"),
                                QStringLiteral("400%")}));
}

void FilterMenuTest::sliderDragDefersPreviewUntilRelease()
{
    QVERIFY(window_->newDocument(QStringLiteral("Drag"), 64, 64, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    const QImage before = view->image();
    bool held = false;
    bool released = false;
    QTimer::singleShot(0, [&] {
        auto* dialog = activeFilterDialog();
        if (!dialog) {
            return;
        }
        auto* slider = dialog->findChild<QSlider*>();
        if (!slider) {
            dialog->reject();
            return;
        }
        slider->setSliderDown(true);
        slider->setValue(qMin(slider->maximum(), slider->value() + 300));
        held = view->image() == before;
        slider->setSliderDown(false);
        released = view->image() != before;
        dialog->reject();
    });
    QVERIFY(window_->registry()->dispatch(
        filterId(QStringLiteral("Noise"), QStringLiteral("Add Noise"))));
    QVERIFY(held);
    QVERIFY(released);
}

void FilterMenuTest::numericFieldIsCompactAndSliderAlignsLeft()
{
    pictura::PictureView* view = window_->activeView();
    QVERIFY(view != nullptr);
    const pictura::FilterCommandSpec* spec = pictura::filterCommandForPath(
        {QStringLiteral("Filter"), QStringLiteral("Blur"), QStringLiteral("Gaussian Blur")});
    QVERIFY(spec != nullptr);
    pictura::FilterPreviewDialog dialog(view, *spec);
    auto* spin = dialog.findChild<QDoubleSpinBox*>();
    auto* slider = dialog.findChild<QSlider*>();
    auto* unit = dialog.findChild<QLabel*>(QStringLiteral("filterUnit"));
    QVERIFY(spin != nullptr);
    QVERIFY(slider != nullptr);
    QVERIFY(unit != nullptr);
    // The unit is a label beside the box, not text baked into it.
    QVERIFY(spin->suffix().isEmpty());
    QCOMPARE(unit->text(), QStringLiteral("pixels"));
    QVERIFY(spin->minimumWidth() == spin->maximumWidth());
    QVERIFY(spin->maximumWidth() < 160);
    dialog.show();
    QApplication::processEvents();
    // The slider starts at the row's left edge (label column), left of the
    // right-aligned value box.
    const int sliderX = slider->mapTo(&dialog, QPoint(0, 0)).x();
    const int spinX = spin->mapTo(&dialog, QPoint(0, 0)).x();
    QVERIFY2(sliderX < spinX,
             qPrintable(QStringLiteral("slider x=%1 spin x=%2").arg(sliderX).arg(spinX)));
    dialog.reject();
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

void FilterMenuTest::colorParameterIsASwatch()
{
    const pictura::FilterCommandSpec* spec = pictura::filterCommandForPath(
        {QStringLiteral("Filter"), QStringLiteral("Artistic"), QStringLiteral("Neon Glow")});
    QVERIFY(spec != nullptr);
    pictura::FilterPreviewDialog dialog(window_->activeView(), *spec);
    auto* swatch = dialog.findChild<QPushButton*>(QStringLiteral("filterColorSwatch"));
    QVERIFY(swatch != nullptr);
    // A box filled with the colour, as in CS6 — not the hex code as text.
    QVERIFY(swatch->text().isEmpty());
    QVERIFY(swatch->styleSheet().contains(QStringLiteral("#0000ff")));
    const QList<double> values = dialog.values();
    QCOMPARE(values, (QList<double>{5.0, 15.0, 0.0, 0.0, 255.0}));
}

void FilterMenuTest::artisticDialogDefaults()
{
    const auto open = [this](const QString& leaf) {
        const pictura::FilterCommandSpec* spec =
            pictura::filterCommandForPath({QStringLiteral("Filter"), QStringLiteral("Artistic"), leaf});
        return pictura::FilterPreviewDialog(window_->activeView(), *spec).values();
    };
    QCOMPARE(open(QStringLiteral("Plastic Wrap")), (QList<double>{15.0, 9.0, 7.0}));
    // Palette Knife has no randomness, so no Seed.
    QCOMPARE(open(QStringLiteral("Palette Knife")), (QList<double>{25.0, 3.0, 0.0}));
    // No foreground/background swatches; trailing slot is the Seed.
    QCOMPARE(open(QStringLiteral("Colored Pencil")), (QList<double>{4.0, 8.0, 25.0, 1.0}));
    QCOMPARE(open(QStringLiteral("Watercolor")), (QList<double>{9.0, 1.0, 1.0, 1.0}));
    // Texture Canvas (2), Light Direction Bottom (0) / Top (4).
    QCOMPARE(open(QStringLiteral("Rough Pastels")),
             (QList<double>{6.0, 4.0, 2.0, 100.0, 20.0, 0.0, 0.0}));
    QCOMPARE(open(QStringLiteral("Underpainting")),
             (QList<double>{6.0, 16.0, 2.0, 100.0, 4.0, 4.0, 0.0, 1.0}));
}

void FilterMenuTest::underpaintingStaysInOneColumn()
{
    const pictura::FilterCommandSpec* spec = pictura::filterCommandForPath(
        {QStringLiteral("Filter"), QStringLiteral("Artistic"), QStringLiteral("Underpainting")});
    QVERIFY(spec != nullptr);
    pictura::FilterPreviewDialog dialog(window_->activeView(), *spec);
    dialog.show();
    QApplication::processEvents();
    // Relief, Light Direction, Invert and Seed run on below Scaling rather
    // than spilling into a second column.
    const QList<QSlider*> sliders = dialog.findChildren<QSlider*>();
    QVERIFY(sliders.size() >= 2);
    const int x = sliders.first()->mapTo(&dialog, QPoint(0, 0)).x();
    for (QSlider* slider : sliders) {
        QCOMPARE(slider->mapTo(&dialog, QPoint(0, 0)).x(), x);
    }
    dialog.reject();
}


void FilterMenuTest::smartSharpenStacksInOneWideColumn()
{
    const pictura::FilterCommandSpec* spec = pictura::filterCommandForPath(
        {QStringLiteral("Filter"), QStringLiteral("Sharpen"), QStringLiteral("Smart Sharpen")});
    QVERIFY(spec != nullptr);
    QVERIFY(spec->stacked);
    pictura::FilterPreviewDialog dialog(window_->activeView(), *spec);
    dialog.show();
    QApplication::processEvents();
    // Every slider, Shadow and Highlight included, shares one wide column.
    const QList<QSlider*> sliders = dialog.findChildren<QSlider*>();
    QCOMPARE(sliders.size(), 10);
    const int x = sliders.first()->mapTo(&dialog, QPoint(0, 0)).x();
    int lastY = -1;
    for (QSlider* slider : sliders) {
        const QPoint at = slider->mapTo(&dialog, QPoint(0, 0));
        QCOMPARE(at.x(), x);
        QVERIFY(at.y() > lastY);
        lastY = at.y();
        QVERIFY2(slider->parentWidget()->width() >= 360, "rows are widened");
    }
    // More Accurate trails the column, below Highlight Radius.
    QCheckBox* accurate = nullptr;
    for (QCheckBox* box : dialog.findChildren<QCheckBox*>()) {
        if (box->text() == QStringLiteral("More Accurate")) {
            accurate = box;
        }
    }
    QVERIFY(accurate != nullptr);
    QVERIFY(accurate->mapTo(&dialog, QPoint(0, 0)).y() > lastY);
    dialog.reject();
}

QTEST_MAIN(FilterMenuTest)
#include "tst_filter_menu.moc"
