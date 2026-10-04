#include "paragraph_panel.h"

#include "options_bar.h"
#include "tools.h"

#include <QtWidgets/QButtonGroup>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QDoubleSpinBox>
#include <QtWidgets/QGridLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

namespace {

// The options bar's glyph tint, so the two sets of alignment buttons match.
const QColor kGlyph(0xd4, 0xd4, 0xd4);

// Laid out left, centre, right as CS6 does; the button ids are the
// justification (0 left, 1 right, 2 centre).
constexpr int kOrder[] = {0, 2, 1};
constexpr const char* kNames[] = {"paragraphAlignLeft", "paragraphAlignRight",
                                  "paragraphAlignCenter"};

} // namespace

ParagraphPanel::ParagraphPanel(QWidget* parent)
    : QWidget(parent)
{
    auto* root = new QVBoxLayout(this);
    root->setContentsMargins(6, 6, 6, 6);
    root->setSpacing(6);

    auto* row = new QHBoxLayout();
    row->setSpacing(2);
    align_ = new QButtonGroup(this);
    for (const int justification : kOrder) {
        auto* button = new QToolButton(this);
        button->setObjectName(QLatin1String(kNames[justification]));
        button->setCheckable(true);
        button->setAutoRaise(true);
        button->setIconSize(QSize(20, 20));
        align_->addButton(button, justification);
        row->addWidget(button);
    }
    auto* justify = new QToolButton(this);
    justify->setText(QStringLiteral("≡"));
    justify->setAutoRaise(true);
    justify->setEnabled(false);
    justify->setToolTip(QStringLiteral("Justify all lines — not implemented yet"));
    row->addWidget(justify);
    row->addStretch(1);
    root->addLayout(row);

    auto* grid = new QGridLayout();
    grid->setSpacing(4);
    // One field per row keeps the panel inside the column's minimum width.
    const auto addField = [&](int row, const QString& label, const QString& tip) {
        auto* field = new QDoubleSpinBox(this);
        field->setRange(-1296, 1296);
        field->setSuffix(QStringLiteral(" pt"));
        field->setEnabled(false);
        field->setToolTip(tip + QStringLiteral(" — not implemented yet"));
        grid->addWidget(new QLabel(label, this), row, 0);
        grid->addWidget(field, row, 1);
    };
    addField(0, QStringLiteral("Indent Left:"), QStringLiteral("Indent left margin"));
    addField(1, QStringLiteral("Indent Right:"), QStringLiteral("Indent right margin"));
    addField(2, QStringLiteral("First Line:"), QStringLiteral("Indent first line"));
    addField(3, QStringLiteral("Space Before:"), QStringLiteral("Add space before paragraph"));
    addField(4, QStringLiteral("Space After:"), QStringLiteral("Add space after paragraph"));
    grid->setColumnStretch(1, 1);
    root->addLayout(grid);

    auto* hyphenate = new QCheckBox(QStringLiteral("Hyphenate"), this);
    hyphenate->setEnabled(false);
    hyphenate->setToolTip(QStringLiteral("Hyphenate — needs paragraph text, not implemented yet"));
    root->addWidget(hyphenate);
    root->addStretch(1);

    refresh();
}

void ParagraphPanel::setController(ToolController* controller)
{
    controller_ = controller;
    if (!controller_) {
        return;
    }
    connect(controller_, &ToolController::typeOptionsChanged, this, &ParagraphPanel::refresh);
    connect(controller_, &ToolController::activeToolChanged, this, &ParagraphPanel::refresh);
    connect(align_, &QButtonGroup::idClicked, this, [this](int justification) {
        TypeOptions o = controller_->typeOptions();
        o.justification = justification;
        controller_->setTypeOptions(o);
    });
    refresh();
}

void ParagraphPanel::refresh()
{
    const ToolId tool = controller_ ? controller_->activeTool() : ToolId::HorizontalType;
    const bool vertical = tool == ToolId::VerticalType || tool == ToolId::VerticalTypeMask;
    const bool first = align_->button(0)->icon().isNull();
    if (first || vertical != vertical_) {
        vertical_ = vertical;
        const QStringList tips = vertical_
            ? QStringList{QStringLiteral("Top align text"), QStringLiteral("Bottom align text"),
                          QStringLiteral("Center text")}
            : QStringList{QStringLiteral("Left align text"), QStringLiteral("Right align text"),
                          QStringLiteral("Center text")};
        for (const int justification : kOrder) {
            QAbstractButton* button = align_->button(justification);
            button->setIcon(typeAlignIcon(justification, vertical_, kGlyph));
            button->setToolTip(tips.at(justification));
        }
    }
    const int justification = controller_ ? controller_->typeOptions().justification : 0;
    if (QAbstractButton* button = align_->button(justification)) {
        button->setChecked(true);
    }
}

} // namespace pictura
