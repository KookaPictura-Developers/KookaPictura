#pragma once

#include <QtWidgets/QWidget>

namespace pictura {

class ColorState;

class SwatchesPanel : public QWidget {
    Q_OBJECT

public:
    explicit SwatchesPanel(ColorState* state, QWidget* parent = nullptr);
};

} // namespace pictura
