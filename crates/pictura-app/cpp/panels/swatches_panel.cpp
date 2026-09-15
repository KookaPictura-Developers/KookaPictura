#include "swatches_panel.h"

#include "color_panel.h"

#include <QtGui/QColor>
#include <QtWidgets/QGridLayout>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QWidget>

namespace pictura {

namespace {
constexpr int kRows = 4;
constexpr int kCols = 12;

QColor swatchColor(int row, int col)
{
    if (row == kRows - 1) {
        const int gray = col * 255 / (kCols - 1);
        return QColor(gray, gray, gray);
    }
    const int hue = col * 30;
    const int saturation = row == 1 ? 130 : 255;
    const int value = row == 2 ? 160 : 255;
    return QColor::fromHsv(hue, saturation, value);
}
} // namespace

SwatchesPanel::SwatchesPanel(ColorState* state, QWidget* parent)
    : QDockWidget(tr("Swatches"), parent)
{
    auto* body = new QWidget(this);
    auto* layout = new QVBoxLayout(body);
    auto* grid = new QGridLayout();
    grid->setSpacing(2);

    for (int row = 0; row < kRows; ++row) {
        for (int col = 0; col < kCols; ++col) {
            const QColor color = swatchColor(row, col);
            auto* button = new QPushButton(body);
            button->setFixedSize(20, 20);
            button->setFlat(true);
            button->setToolTip(color.name().toUpper());
            button->setStyleSheet(
                QStringLiteral("QPushButton { background-color: %1; border: 1px solid #303030; }")
                    .arg(color.name()));
            connect(button, &QPushButton::clicked, this, [state, color] {
                if (state) {
                    state->setForeground(color);
                }
            });
            grid->addWidget(button, row, col);
        }
    }
    layout->addLayout(grid);
    layout->addStretch(1);
    setWidget(body);
}

} // namespace pictura
