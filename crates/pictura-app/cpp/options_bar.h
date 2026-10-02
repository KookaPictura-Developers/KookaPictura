#pragma once

#include <QtWidgets/QToolBar>

#include <functional>

#include "tools.h"

class QStackedWidget;
class QHBoxLayout;
class QIcon;
class QToolButton;

namespace pictura {

class ToolController;
class BrushPresetPicker;
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
    // The Brush Preset picker behind every brush tip button, made on first use.
    BrushPresetPicker* brushPicker();

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
    // options_bar_paint.cpp: Red Eye, Color Replacement, and Mixer Brush, and
    // the brush tip button every brush bar shares.
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
    // options_bar_erase.cpp: Background Eraser and Magic Eraser.
    QWidget* buildBackgroundEraserPage(ToolId id);
    QWidget* buildMagicEraserPage(ToolId id);
    // options_bar_fill.cpp: Gradient and Paint Bucket.
    QWidget* buildGradientPage(ToolId id);
    QWidget* buildPaintBucketPage(ToolId id);
    // options_bar_retouch.cpp: Blur, Sharpen, Smudge, Dodge, Burn, and Sponge.
    QWidget* buildRetouchPage(ToolId id);
    QWidget* buildTonePage(ToolId id);
    // options_bar_pen.cpp: Pen and Freeform Pen.
    QWidget* buildPenPage(ToolId id);
    QWidget* buildShapePage(ToolId id);
    QToolButton* buildArrowheadsButton(
        QWidget* page, const ShapeOptions& initial,
        const std::function<void(const std::function<void(ShapeOptions&)>&)>& update);
    // options_bar_type.cpp: the four Type tools.
    QWidget* buildTypePage(ToolId id);
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
    BrushPresetPicker* brushPicker_ = nullptr;
};

// Built-in pattern `index` as an icon, for the pattern pickers.
QIcon patternIcon(int index);

} // namespace pictura
