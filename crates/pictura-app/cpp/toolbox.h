#pragma once

#include <QtCore/QList>
#include <QtCore/QMap>
#include <QtCore/QPoint>
#include <QtCore/QRect>
#include <QtCore/QStringList>
#include <QtGui/QColor>
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
class QVBoxLayout;

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
    // Exchange foreground and background (CS6 double-headed-arrow swap).
    void swapForegroundBackground();

    // The column width scales the swatches; the widget never dictates a wider
    // dock. `side` is clamped to a square that still reads as two swatches.
    void setSide(int side);

    // Test hooks.
    QRect swapRectForTest() const { return swapRect(); }
    int swapCountForTest() const { return swapCount_; }
    QColor foregroundForTest() const;
    QColor backgroundForTest() const;

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
    QRect swapRect() const;
    int swatchSize() const;
    int resetSize() const;

    ColorState* state_ = nullptr;
    int swapCount_ = 0;
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

    // M47 T4.3: while hosted as a vertical pane in the central splitter the dock
    // keeps a fixed width but fills the splitter height.
    void setSplitterPane(bool on);
    bool isSplitterPane() const { return splitterPane_; }

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
    // The custom title bar, so the self-test can drive the real gesture path.
    QWidget* titleBarForTest() const { return titleBar_; }
    // The grab offset recorded at the start of a title-bar drag: the frame uses
    // it to place a floating fallback under the cursor when no splitter target
    // resolves.
    QPoint titleDragOffset() const { return titleDragOffset_; }
    bool hasFlyoutTriangleForTest(int group) const;
    QString titleTextForTest() const;
    int minimumWidthForTest() const { return minimumWidth(); }
    int contentWidthForTest() const;
    int contentHeightForTest() const;
    int foregroundBackgroundWidthForTest() const;
    // Exchanges the foreground/background swatches (the frame's `X` key).
    void swapForegroundBackground();
    // Resets the swatches to the default black/white pair (the frame's `D` key).
    void resetForegroundBackground();
    ForegroundBackgroundWidget* foregroundBackgroundForTest() const { return fgbg_; }
    // Floated/docked body geometry: the trailing stretch is 0 while floating so
    // a floated dock can hug its content height.
    int bodyStretchForTest() const;
    int bodyHeightForTest() const;
    int bodySizeHintHeightForTest() const;
    // M44 T1: while floating the dock height is pinned to the content height and
    // cannot be drag-resizable.
    int floatHeightForTest() const { return floatHeight_; }
    bool floatHeightLockedForTest() const;

signals:
    void screenModeRequested();
    void columnsChanged(int columns);
    // M45 T3: the floating Tools panel's title-bar drag. The frame resolves the
    // drop through the column grammar and hosts the pane at that boundary.
    void toolbarDragMoved(const QPoint& globalPos);
    void toolbarDragFinished(const QPoint& globalPos);

protected:
    bool eventFilter(QObject* watched, QEvent* event) override;

private:
    ToolId groupCurrentTool(int group) const;
    void refreshSlot(int group);
    void selectMember(int group, ToolId id);
    void cycleGroup(int group);
    void showSlotMenu(int group);
    void reflow();
    void updateContentMetrics();
    int contentWidth(int columns) const;
    int contentHeight(int columns) const;
    void updateTitleIcon();

    ToolController* controller_ = nullptr;
    ColorState* colors_ = nullptr;
    QMap<int, ToolId> currentByGroup_;
    QList<QToolButton*> slotButtons_;
    QList<QMenu*> slotMenus_;
    QGridLayout* grid_ = nullptr;
    QWidget* gridWidget_ = nullptr;
    QWidget* titleBar_ = nullptr;
    QToolButton* titleToggle_ = nullptr;
    QVBoxLayout* bodyLayout_ = nullptr;
    ForegroundBackgroundWidget* fgbg_ = nullptr;
    QToolButton* screenMode_ = nullptr;
    int columns_ = 1;
    bool shiftKeyForToolSwitch_ = true;
    // M47 T4.3: true while the dock is a pane in the central splitter.
    bool splitterPane_ = false;
    // M45 T1: one guard for the single content-metrics recompute; the M43
    // width lock and M44 height lock are now one pass over both axes.
    bool metricsClamping_ = false;
    // M46: a floating title-bar press arms the drag; Qt's dock drag then grabs
    // the mouse, so move/release arrive on the dock before it completes.
    bool titleDragPending_ = false;
    bool titleDragMoved_ = false;
    QPoint titlePressGlobal_;
    // M47: cursor offset within the dock/title at press, for the floating
    // follow and the frame's float-at-cursor fallback.
    QPoint titleDragOffset_;
    // M44 T1: while floating the height is pinned to this content height.
    int floatHeight_ = 0;
};

} // namespace pictura
