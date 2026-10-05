#include "preferences_dialog.h"

#include "panels/numeric_field.h"
#include "theme.h"

#include <QtWidgets/QCheckBox>
#include <QtWidgets/QFormLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QListWidget>
#include <QtWidgets/QStackedWidget>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

const QString PreferencesDialog::kGeneral = QStringLiteral("General");
const QString PreferencesDialog::kInterface = QStringLiteral("Interface");
const QString PreferencesDialog::kPerformance = QStringLiteral("Performance");

namespace {

// The CS6 pane list; the real pages are the ones present in `realPages_`.
const char* const kPaneNames[] = {
    "General",
    "Interface",
    "File Handling",
    "Performance",
    "Cursors",
    "Transparency & Gamut",
    "Units & Rulers",
    "Guides, Grid, & Slices",
    "Plug-ins",
    "Type",
    "3D",
};

} // namespace

PreferencesDialog::PreferencesDialog(QWidget* parent)
    : QDialog(parent)
{
    setObjectName(QStringLiteral("preferencesDialog"));
    setWindowTitle(tr("Preferences"));
    setWindowModality(Qt::NonModal);

    auto* layout = new QHBoxLayout(this);
    pageList_ = new QListWidget(this);
    pageList_->setObjectName(QStringLiteral("preferencesPages"));
    pageList_->setFixedWidth(160);
    layout->addWidget(pageList_);

    stack_ = new QStackedWidget(this);
    stack_->setObjectName(QStringLiteral("preferencesStack"));
    layout->addWidget(stack_, 1);

    realPages_ = {kGeneral, kInterface, kPerformance};

    // General: the app's real theme brightness is the only General setting.
    auto* generalPage = new QWidget(this);
    generalPage->setObjectName(QStringLiteral("preferencesGeneralPage"));
    auto* generalForm = new QFormLayout(generalPage);
    NumericFieldConfig brightnessConfig;
    brightnessConfig.minimum = 0;
    brightnessConfig.maximum = Theme::kLevelCount - 1;
    brightnessConfig.decimals = 0;
    brightnessConfig.objectName = QStringLiteral("preferencesBrightness");
    brightness_ = new NumericField(QString(), brightnessConfig, generalPage);
    brightness_->setValue(Theme::kDefaultLevel);
    generalForm->addRow(tr("Brightness"), brightness_);
    stack_->addWidget(generalPage);

    auto* interfacePage = new QWidget(this);
    interfacePage->setObjectName(QStringLiteral("preferencesInterfacePage"));
    auto* interfaceLayout = new QVBoxLayout(interfacePage);
    makeCheckbox(QStringLiteral("useShiftKeyForToolSwitch"),
                 tr("Use Shift Key For Tool Switch"), interfacePage);
    makeCheckbox(QStringLiteral("autoCollapseIconic"), tr("Auto-Collapse Iconic Panels"),
                 interfacePage);
    makeCheckbox(QStringLiteral("autoShowHidden"), tr("Auto-Show Hidden Panels"),
                 interfacePage);
    interfaceLayout->addStretch(1);
    stack_->addWidget(interfacePage);

    auto* performancePage = new QWidget(this);
    performancePage->setObjectName(QStringLiteral("preferencesPerformancePage"));
    auto* performanceLayout = new QVBoxLayout(performancePage);
    makeCheckbox(QStringLiteral("useGpuCompute"), tr("Use GPU Compute"), performancePage);
    performanceLayout->addStretch(1);
    stack_->addWidget(performancePage);

    // List rows carry their pane name in Qt::UserRole: the CS6 pane order does
    // not match the stack's real-page order, so the row maps by name.
    for (const char* name : kPaneNames) {
        const QString pageName = QString::fromUtf8(name);
        auto* item = new QListWidgetItem(pageName, pageList_);
        item->setData(Qt::UserRole, pageName);
        if (!realPages_.contains(pageName)) {
            item->setFlags(item->flags() & ~Qt::ItemIsEnabled);
        }
    }
    connect(pageList_, &QListWidget::currentRowChanged, this, [this](int row) {
        const QListWidgetItem* item = pageList_->item(row);
        if (!item) {
            return;
        }
        const int index = realPages_.indexOf(item->data(Qt::UserRole).toString());
        if (index >= 0) {
            stack_->setCurrentIndex(index);
        }
    });
    connect(brightness_, &NumericField::valueChanged, this,
            [this](double level) { emit brightnessLevelChanged(qRound(level)); });
    pageList_->setCurrentRow(0);
}

QCheckBox* PreferencesDialog::makeCheckbox(const QString& key, const QString& label,
                                           QWidget* page)
{
    auto* box = new QCheckBox(label, page);
    box->setObjectName(QStringLiteral("preferences_") + key);
    page->layout()->addWidget(box);
    checkboxes_.insert(key, box);
    connect(box, &QCheckBox::toggled, this, [this, key](bool on) {
        if (key == QStringLiteral("useShiftKeyForToolSwitch")) {
            emit useShiftKeyForToolSwitchChanged(on);
        } else if (key == QStringLiteral("autoCollapseIconic")) {
            emit autoCollapseIconicChanged(on);
        } else if (key == QStringLiteral("autoShowHidden")) {
            emit autoShowHiddenChanged(on);
        } else if (key == QStringLiteral("useGpuCompute")) {
            emit gpuComputeChanged(on);
        }
    });
    return box;
}

QCheckBox* PreferencesDialog::checkbox(const QString& key) const
{
    return checkboxes_.value(key, nullptr);
}

void PreferencesDialog::setShiftKeyForToolSwitch(bool on)
{
    if (QCheckBox* box = checkbox(QStringLiteral("useShiftKeyForToolSwitch"))) {
        box->setChecked(on);
    }
}

void PreferencesDialog::setAutoCollapseIconic(bool on)
{
    if (QCheckBox* box = checkbox(QStringLiteral("autoCollapseIconic"))) {
        box->setChecked(on);
    }
}

void PreferencesDialog::setAutoShowHidden(bool on)
{
    if (QCheckBox* box = checkbox(QStringLiteral("autoShowHidden"))) {
        box->setChecked(on);
    }
}

void PreferencesDialog::setBrightnessLevel(int level)
{
    brightness_->setValue(level);
}

void PreferencesDialog::setGpuCompute(bool on)
{
    if (QCheckBox* box = checkbox(QStringLiteral("useGpuCompute"))) {
        box->setChecked(on);
    }
}

void PreferencesDialog::setGpuComputeEnabled(bool on)
{
    if (QCheckBox* box = checkbox(QStringLiteral("useGpuCompute"))) {
        box->setEnabled(on);
    }
}

void PreferencesDialog::showPage(const QString& page)
{
    const int index = realPages_.indexOf(page);
    if (index < 0) {
        return;
    }
    for (int row = 0; row < pageList_->count(); ++row) {
        if (pageList_->item(row)->data(Qt::UserRole).toString() == page) {
            pageList_->setCurrentRow(row);
            break;
        }
    }
    stack_->setCurrentIndex(index);
}

void PreferencesDialog::openOn(const QString& page)
{
    showPage(page);
    show();
    raise();
    activateWindow();
}

QStringList PreferencesDialog::pagesForTest() const
{
    return realPages_;
}

QString PreferencesDialog::currentPageForTest() const
{
    return realPages_.value(stack_->currentIndex());
}

bool PreferencesDialog::setCheckboxForTest(const QString& key, bool on)
{
    QCheckBox* box = checkbox(key);
    if (!box) {
        return false;
    }
    box->setChecked(on);
    return true;
}

bool PreferencesDialog::checkboxForTest(const QString& key) const
{
    QCheckBox* box = checkbox(key);
    return box && box->isChecked();
}

} // namespace pictura
