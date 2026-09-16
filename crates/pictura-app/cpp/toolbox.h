#pragma once

#include <QtCore/QList>
#include <QtCore/QMap>
#include <QtCore/QRect>
#include <QtWidgets/QDockWidget>
#include <QtWidgets/QWidget>

#include "tools.h"

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

// The Tools dock: a CS6 single-column list of flyout slots (one button per
// group), the foreground/background swatches, and the screen-mode toggle.
class Toolbox : public QDockWidget {
    Q_OBJECT

public:
    explicit Toolbox(ToolController* controller, ColorState* colors, QWidget* parent = nullptr);

    // The 23 group slot buttons in catalogue order (self-test accessor).
    QList<QToolButton*> slotButtons() const { return slotButtons_; }

signals:
    void screenModeRequested();

private:
    ToolId groupCurrentTool(int group) const;
    void refreshSlot(int group);
    void selectMember(int group, ToolId id);
    void cycleGroup(int group);

    ToolController* controller_ = nullptr;
    ColorState* colors_ = nullptr;
    QMap<int, ToolId> currentByGroup_;
    QList<QToolButton*> slotButtons_;
};

} // namespace pictura
