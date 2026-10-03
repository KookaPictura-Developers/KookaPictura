// The Move tool's options bar: CS6's six Align and six Distribute buttons.
// Part of OptionsBar; ported from photorust's MainWindow::addMoveOptions.

#include "options_bar.h"

#include "icons.h"

#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QStackedWidget>
#include <QtWidgets/QToolButton>

namespace pictura {

namespace {

// Menu order, which is also the engine's edge index (0 Top ... 5 Right).
constexpr const char* kEdges[] = {"Top",  "VerticalCenter",   "Bottom",
                                  "Left", "HorizontalCenter", "Right"};
constexpr const char* kEdgeTips[] = {"top edges",  "vertical centers",   "bottom edges",
                                     "left edges", "horizontal centers", "right edges"};

} // namespace

// ponytail: no Auto-Select, Show Transform Controls, or Auto-Align Layers.
QWidget* OptionsBar::buildMovePage(ToolId id)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(toolButton(id, page));
    layout->addSpacing(8);

    // The bar aligns against the selection when there is one (CS6's Align
    // Layers To Selection), else the selected layers against each other.
    const auto addRow = [&](bool distribute) {
        for (int edge = 0; edge < 6; ++edge) {
            const QString name = QLatin1String(distribute ? "distribute" : "align")
                + QLatin1String(kEdges[edge]);
            auto* button = new QToolButton(page);
            button->setObjectName(QStringLiteral("options") + name.at(0).toUpper() + name.mid(1));
            button->setIcon(icon(QStringLiteral("layer.") + name));
            button->setIconSize(QSize(18, 18));
            button->setAutoRaise(true);
            button->setEnabled(false);
            button->setToolTip(QLatin1String(distribute ? "Distribute " : "Align ")
                               + QLatin1String(kEdgeTips[edge]));
            connect(button, &QToolButton::clicked, this, [this, distribute, edge]() {
                if (distribute) {
                    emit distributeRequested(edge);
                } else {
                    emit alignRequested(edge);
                }
            });
            (distribute ? distributeButtons_ : alignButtons_).push_back(button);
            layout->addWidget(button);
            // A gap between the vertical three and the horizontal three.
            if (edge == 2) {
                layout->addSpacing(6);
            }
        }
    };
    addRow(false);
    layout->addSpacing(12);
    addRow(true);
    layout->addStretch(1);
    return page;
}

void OptionsBar::setAlignEnabled(bool align, bool distribute)
{
    for (QToolButton* button : alignButtons_) {
        button->setEnabled(align);
    }
    for (QToolButton* button : distributeButtons_) {
        button->setEnabled(distribute);
    }
}

} // namespace pictura
