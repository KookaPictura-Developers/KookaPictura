#pragma once

#include <QtWidgets/QToolBar>

#include "tools.h"

class QStackedWidget;
class QHBoxLayout;
class QToolButton;

namespace pictura {

class ToolController;

// Context-sensitive options bar: one stacked page per tool, switched by the
// frame when the active tool changes.
class OptionsBar : public QToolBar {
    Q_OBJECT

public:
    explicit OptionsBar(ToolController* controller, QWidget* parent = nullptr);

    void showTool(ToolId id);

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
    void addModeButtons(QHBoxLayout* layout, QWidget* page, bool withIntersect);
    void addMagneticFields(QHBoxLayout* layout, QWidget* page);
    QToolButton* toolButton(ToolId id, QWidget* parent);

    ToolController* controller_ = nullptr;
    QStackedWidget* stack_ = nullptr;
};

} // namespace pictura
