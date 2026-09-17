#pragma once

#include <QtCore/QList>
#include <QtCore/QMap>
#include <QtCore/QRect>
#include <QtCore/QStringList>
#include <QtWidgets/QDockWidget>
#include <QtWidgets/QWidget>

#include "tools.h"

class QAction;
class QEvent;
class QGridLayout;
class QMenu;
class QMouseEvent;
class QPaintEvent;
class QToolButton;

namespace pictura {

class ColorState;
class ToolController;

// CS6-style foreground/background swatches: two overlapping squares with the
// active one outlined, plus a corner reset to the default black/white pair.
class ForegroundBackgroundWidget : public QWidget {
    Q_OBJECT

public:
    explicit ForegroundBackgroundWidget(ColorState* state, QWidget* parent = nullptr);

    void resetColors();

signals:
    void clicked();
    void reset();

protected:
    void paintEvent(QPaintEvent* event) override;
    void mousePressEvent(QMouseEvent* event) override;

private:
    QRect foregroundRect() const;
    QRect backgroundRect() const;
    QRect resetRect() const;

    ColorState* state_ = nullptr;
};

// The Tools dock: a CS6 list of flyout slots (one button per group) that reflows
// between one and two columns, the foreground/background swatches, and the
// screen-mode toggle.
class Toolbox : public QDockWidget {
    Q_OBJECT

public:
    explicit Toolbox(ToolController* controller, ColorState* colors, QWidget* parent = nullptr);

    // The 23 group slot buttons in catalogue order (self-test accessor).
    QList<QToolButton*> slotButtons() const { return slotButtons_; }

    // Current column count, 1 or 2.
    int columns() const { return columns_; }
    void setColumns(int columns);

    // Gated by `Use Shift Key For Tool Switch`: with it on a plain letter
    // activates the slot's current member and `Shift`+letter cycles; with it off
    // the letter alone cycles. Returns true when a group handled the key.
    void setShiftKeyForToolSwitch(bool on) { shiftKeyForToolSwitch_ = on; }
    bool shiftKeyForToolSwitch() const { return shiftKeyForToolSwitch_; }
    bool handleToolKey(const QChar& key, bool shift);

    // Self-test hooks.
    void openSlotFlyoutForTest(int group);
    void openFlyoutForTest(int group) { openSlotFlyoutForTest(group); }
    QMenu* slotMenuForTest(int group) const;
    QList<QAction*> slotMenuActionsForTest(int group) const;
    QStringList flyoutKeysForTest(int group) const;
    void cycleGroupForTest(int group) { cycleGroup(group); }
    QToolButton* titleBarToggleForTest() const { return titleToggle_; }
    bool hasFlyoutTriangleForTest(int group) const;

signals:
    void screenModeRequested();
    void columnsChanged(int columns);

protected:
    bool eventFilter(QObject* watched, QEvent* event) override;

private:
    ToolId groupCurrentTool(int group) const;
    void refreshSlot(int group);
    void selectMember(int group, ToolId id);
    void cycleGroup(int group);
    void showSlotMenu(int group);
    void reflow();
    void updateTitleIcon();

    ToolController* controller_ = nullptr;
    ColorState* colors_ = nullptr;
    QMap<int, ToolId> currentByGroup_;
    QList<QToolButton*> slotButtons_;
    QList<QMenu*> slotMenus_;
    QGridLayout* grid_ = nullptr;
    QWidget* gridWidget_ = nullptr;
    QToolButton* titleToggle_ = nullptr;
    int columns_ = 1;
    bool shiftKeyForToolSwitch_ = true;
};

} // namespace pictura
