#pragma once

#include <QtWidgets/QDockWidget>

namespace pictura {

class ToolController;

// The Tools dock: one checkable action per tool in a vertical toolbar, wired to
// the shared controller.
class Toolbox : public QDockWidget {
    Q_OBJECT

public:
    explicit Toolbox(ToolController* controller, QWidget* parent = nullptr);
};

} // namespace pictura
