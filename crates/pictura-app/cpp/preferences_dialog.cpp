#include "preferences_dialog.h"

#include "theme.h"

#include <QtWidgets/QCheckBox>
#include <QtWidgets/QFormLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QListWidget>
#include <QtWidgets/QSpinBox>
#include <QtWidgets/QStackedWidget>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

const QString PreferencesDialog::kGeneral = QStringLiteral("General");
const QString PreferencesDialog::kInterface = QStringLiteral("Interface");

namespace {

// The CS6 pane list; only the first two have real pages in M41.
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

    realPages_ = {kGeneral, kInterface};

    // General: the app's real theme brightness is the only General setting.
    auto* generalPage = new QWidget(this);
    generalPage->setObjectName(QStringLiteral("preferencesGeneralPage"));
    auto* generalForm = new QFormLayout(generalPage);
    brightness_ = new QSpinBox(generalPage);
    brightness_->setObjectName(QStringLiteral("preferencesBrightness"));
    brightness_->setRange(0, Theme::kLevelCount - 1);
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

    for (const char* name : kPaneNames) {
        auto* item = new QListWidgetItem(QString::fromUtf8(name), pageList_);
        if (!realPages_.contains(QString::fromUtf8(name))) {
            item->setFlags(item->flags() & ~Qt::ItemIsEnabled);
        }
    }
    connect(pageList_, &QListWidget::currentRowChanged, this, [this](int row) {
        if (row >= 0 && row < realPages_.size()) {
            stack_->setCurrentIndex(row);
        }
    });
    connect(brightness_, QOverload<int>::of(&QSpinBox::valueChanged), this,
            [this](int level) { emit brightnessLevelChanged(level); });
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

void PreferencesDialog::showPage(const QString& page)
{
    const int index = realPages_.indexOf(page);
    if (index < 0) {
        return;
    }
    pageList_->setCurrentRow(index);
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
