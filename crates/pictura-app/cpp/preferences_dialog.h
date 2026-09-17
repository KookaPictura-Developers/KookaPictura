#pragma once

#include <QtCore/QHash>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtWidgets/QDialog>

class QCheckBox;
class QListWidget;
class QSpinBox;
class QStackedWidget;

namespace pictura {

// The M41 CS6 Preferences dialog: a left pane list plus a QStackedWidget. Only
// `General` and `Interface` are real pages; the other CS6 panes are listed but
// disabled. Modeless so it never blocks the headless self-test.
class PreferencesDialog : public QDialog {
    Q_OBJECT

public:
    static const QString kGeneral;
    static const QString kInterface;

    explicit PreferencesDialog(QWidget* parent = nullptr);

    // Select `page` (a no-op when unknown) and show non-modally.
    void openOn(const QString& page);
    void showPage(const QString& page);

    void setShiftKeyForToolSwitch(bool on);
    void setAutoCollapseIconic(bool on);
    void setAutoShowHidden(bool on);
    void setBrightnessLevel(int level);

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

private:
    QCheckBox* makeCheckbox(const QString& key, const QString& label, QWidget* page);
    QCheckBox* checkbox(const QString& key) const;
    int pageIndex(const QString& name) const;

    QListWidget* pageList_ = nullptr;
    QStackedWidget* stack_ = nullptr;
    QStringList realPages_;
    QHash<QString, QCheckBox*> checkboxes_;
    QSpinBox* brightness_ = nullptr;
};

} // namespace pictura
