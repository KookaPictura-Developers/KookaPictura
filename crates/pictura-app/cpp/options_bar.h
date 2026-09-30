#pragma once

#include <QtWidgets/QToolBar>

#include "tools.h"

class QStackedWidget;
class QHBoxLayout;
class QToolButton;

namespace pictura {

class ToolController;
class NumericField;
struct NumericFieldConfig;

// Context-sensitive options bar: one stacked page per tool, switched by the
// frame when the active tool changes.
class OptionsBar : public QToolBar {
    Q_OBJECT

public:
    explicit OptionsBar(ToolController* controller, QWidget* parent = nullptr);

    void showTool(ToolId id);

signals:
    // A panel toggle (Toggle the Brush panel, Toggle the Clone Source panel)
    // asks the frame to show or hide the panel with this object name.
    void panelToggleRequested(const QString& panel);

public:
    // Self-test hook: the controller the size field is wired to.
    ToolController* controllerForTest() const { return controller_; }

private:
    QWidget* buildPage(ToolId id);
    QWidget* buildCombinePage(ToolId id, bool withTolerance);
    QWidget* buildSelectionPage(ToolId id);
    QWidget* buildWandPage(ToolId id);
    QWidget* buildCropPage(ToolId id);
    QWidget* buildAnnotationPage(ToolId id);
    QWidget* buildPaintPage(ToolId id);
    QWidget* buildHealingPage(ToolId id);
    QWidget* buildPatchPage(ToolId id);
    QWidget* buildContentAwareMovePage(ToolId id);
    QWidget* buildCountPage(ToolId id);
    // options_bar_paint.cpp: Red Eye, Color Replacement, and Mixer Brush.
    QWidget* buildRedEyePage(ToolId id);
    QWidget* buildColorReplacementPage(ToolId id);
    QWidget* buildMixerBrushPage(ToolId id);
    void addBrushTipFields(QHBoxLayout* layout, QWidget* page);
    // options_bar_stamp.cpp: Clone Stamp, Pattern Stamp, History Brush, Art
    // History Brush, and Eraser.
    QWidget* buildStampPage(ToolId id);
    QWidget* buildHistoryBrushPage(ToolId id);
    QWidget* buildArtHistoryBrushPage(ToolId id);
    QWidget* buildEraserPage(ToolId id);
    void addStampPaintFields(QHBoxLayout* layout, QWidget* page);
    NumericField* addPercentField(QHBoxLayout* layout, QWidget* page, const QString& label,
                                  const QString& name, int value,
                                  void (ToolController::*setter)(int));
    static NumericFieldConfig numericConfig(double lo, double hi, double step, int decimals,
                                            const QString& suffix, bool popup,
                                            const QString& name);
    void addModeButtons(QHBoxLayout* layout, QWidget* page, bool withIntersect);
    void addMagneticFields(QHBoxLayout* layout, QWidget* page);
    QToolButton* toolButton(ToolId id, QWidget* parent);

    ToolController* controller_ = nullptr;
    QStackedWidget* stack_ = nullptr;
};

} // namespace pictura
