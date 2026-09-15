#pragma once

#include <QtCore/QRect>
#include <QtWidgets/QDockWidget>
#include <QtWidgets/QWidget>

class QMouseEvent;
class QPaintEvent;

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

// The Tools dock: a two-column grid of icon-only tool buttons, the
// foreground/background swatches, and the screen-mode toggle.
class Toolbox : public QDockWidget {
    Q_OBJECT

public:
    explicit Toolbox(ToolController* controller, ColorState* colors, QWidget* parent = nullptr);

signals:
    void screenModeRequested();

private:
    ColorState* colors_ = nullptr;
};

} // namespace pictura
