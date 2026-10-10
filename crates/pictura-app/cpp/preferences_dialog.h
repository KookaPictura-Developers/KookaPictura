#pragma once

#include <QtCore/QHash>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtWidgets/QDialog>

class QCheckBox;
class QComboBox;
class QListWidget;
class QStackedWidget;

namespace pictura {

class NumericField;

// The M41 CS6 Preferences dialog: a left pane list plus a QStackedWidget.
// General, Interface, File Handling, and Performance are real pages; the other
// CS6 panes are listed but disabled. Modeless so it never blocks the headless
// self-test.
class PreferencesDialog : public QDialog {
    Q_OBJECT

public:
    static const QString kGeneral;
    static const QString kInterface;
    static const QString kFileHandling;
    static const QString kPerformance;

    explicit PreferencesDialog(QWidget* parent = nullptr);

    // Select `page` (a no-op when unknown) and show non-modally.
    void openOn(const QString& page);

    void setShiftKeyForToolSwitch(bool on);
    void setAutoCollapseIconic(bool on);
    void setAutoShowHidden(bool on);
    void setBrightnessLevel(int level);
    void setGpuCompute(bool on);
    void setGpuComputeEnabled(bool on);
    // File Handling: Automatically Save Recovery Information Every `minutes`.
    void setAutoSaveRecovery(bool on, int minutes);

    // Test hooks.
    QStringList pagesForTest() const;
    QString currentPageForTest() const;
    bool setCheckboxForTest(const QString& key, bool on);
    bool checkboxForTest(const QString& key) const;

signals:
    void useShiftKeyForToolSwitchChanged(bool on);
    void autoCollapseIconicChanged(bool on);
    void autoShowHiddenChanged(bool on);
    void brightnessLevelChanged(int level);
    void gpuComputeChanged(bool on);
    void autoSaveRecoveryChanged(bool on, int minutes);

private:
    void showPage(const QString& page);
    QCheckBox* makeCheckbox(const QString& key, const QString& label, QWidget* page);
    QCheckBox* checkbox(const QString& key) const;

    QListWidget* pageList_ = nullptr;
    QStackedWidget* stack_ = nullptr;
    QStringList realPages_;
    QHash<QString, QCheckBox*> checkboxes_;
    NumericField* brightness_ = nullptr;
    QComboBox* autoSaveMinutes_ = nullptr;
};

} // namespace pictura
