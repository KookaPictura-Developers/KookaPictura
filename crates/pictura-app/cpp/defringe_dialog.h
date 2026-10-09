#pragma once

#include <QtWidgets/QDialog>

class QSpinBox;

namespace pictura {

// CS6's Layer > Matting > Defringe: a single Width (pixels) field. The engine
// replaces the colour of edge pixels within that width with the nearest
// interior colour. Ported from the edge-cleanup family.
class DefringeDialog : public QDialog {
    Q_OBJECT

public:
    explicit DefringeDialog(int defaultWidth = 1, QWidget* parent = nullptr);

    int width() const;

    // Run the dialog modally; on OK write the chosen width to `out` and return
    // true. False on Cancel (out is left untouched).
    static bool get(int* out, QWidget* parent = nullptr);

private:
    QSpinBox* width_ = nullptr;
};

} // namespace pictura
