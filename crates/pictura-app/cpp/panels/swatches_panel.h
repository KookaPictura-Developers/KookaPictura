#pragma once

#include <QtWidgets/QDockWidget>

namespace pictura {

class ColorState;

class SwatchesPanel : public QDockWidget {
    Q_OBJECT

public:
    explicit SwatchesPanel(ColorState* state, QWidget* parent = nullptr);
};

} // namespace pictura
