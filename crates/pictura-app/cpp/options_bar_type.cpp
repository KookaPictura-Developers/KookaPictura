// The options bar of the four Type tools.
// Part of OptionsBar; split from options_bar.cpp along the page seam.

#include "options_bar.h"

#include "panels/numeric_field.h"

#include <QtCore/QSignalBlocker>
#include <QtWidgets/QButtonGroup>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QFontComboBox>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QStackedWidget>
#include <QtWidgets/QToolButton>

namespace pictura {

namespace {

// The same tool in the other orientation (the bar's Toggle Text Orientation).
ToolId otherOrientation(ToolId id)
{
    switch (id) {
    case ToolId::HorizontalType:
        return ToolId::VerticalType;
    case ToolId::VerticalType:
        return ToolId::HorizontalType;
    case ToolId::HorizontalTypeMask:
        return ToolId::VerticalTypeMask;
    default:
        return ToolId::HorizontalTypeMask;
    }
}

} // namespace

// Toggle Text Orientation, font family, size (px), anti-aliasing (None /
// Sharp), and alignment (left / centre / right; top / centre / bottom for
// vertical type), then Cancel and Commit while text is being typed. The four
// tools share one TypeOptions, so each page re-reads it when shown.
// ponytail: no font style, Crisp / Strong / Smooth, colour swatch (the text is
// the foreground colour), Warp Text, or Character / Paragraph panels button.
QWidget* OptionsBar::buildTypePage(ToolId id)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(toolButton(id, page));
    const bool vertical = id == ToolId::VerticalType || id == ToolId::VerticalTypeMask;

    auto* orientation = new QToolButton(page);
    orientation->setObjectName(QStringLiteral("optionsTypeOrientation"));
    orientation->setText(vertical ? QStringLiteral("↕T") : QStringLiteral("↔T"));
    orientation->setToolTip(QStringLiteral("Toggle text orientation"));
    layout->addWidget(orientation);

    auto* family = new QFontComboBox(page);
    family->setObjectName(QStringLiteral("optionsTypeFamily"));
    family->setToolTip(QStringLiteral("Font family"));
    layout->addWidget(family);

    auto* size = new NumericField(
        QStringLiteral("Size:"),
        numericConfig(1, 1296, 1, 1, QStringLiteral(" px"), false,
                      QStringLiteral("optionsTypeSize")),
        page);
    layout->addWidget(size);

    auto* antialias = new QComboBox(page);
    antialias->setObjectName(QStringLiteral("optionsTypeAntialias"));
    antialias->setToolTip(QStringLiteral("Anti-aliasing method"));
    antialias->addItems({QStringLiteral("None"), QStringLiteral("Sharp")});
    layout->addWidget(antialias);

    auto* align = new QButtonGroup(page);
    const char* const names[] = {"optionsTypeAlignLeft", "optionsTypeAlignRight",
                                 "optionsTypeAlignCenter"};
    const QStringList tips = vertical
        ? QStringList{QStringLiteral("Top align text"), QStringLiteral("Bottom align text"),
                      QStringLiteral("Center text")}
        : QStringList{QStringLiteral("Left align text"), QStringLiteral("Right align text"),
                      QStringLiteral("Center text")};
    const QStringList glyphs = vertical
        ? QStringList{QStringLiteral("⤒"), QStringLiteral("⤓"), QStringLiteral("↕")}
        : QStringList{QStringLiteral("⇤"), QStringLiteral("⇥"), QStringLiteral("↔")};
    // Laid out left, centre, right as CS6 does; the ids are the justification.
    for (int justification : {0, 2, 1}) {
        auto* button = new QToolButton(page);
        button->setObjectName(QLatin1String(names[justification]));
        button->setCheckable(true);
        button->setText(glyphs.at(justification));
        button->setToolTip(tips.at(justification));
        align->addButton(button, justification);
        layout->addWidget(button);
    }

    auto* cancel = new QToolButton(page);
    cancel->setObjectName(QStringLiteral("optionsTypeCancel"));
    cancel->setText(QStringLiteral("✘"));
    cancel->setToolTip(QStringLiteral("Cancel all current edits (Esc)"));
    layout->addWidget(cancel);
    auto* commit = new QToolButton(page);
    commit->setObjectName(QStringLiteral("optionsTypeCommit"));
    commit->setText(QStringLiteral("✓"));
    commit->setToolTip(QStringLiteral("Commit all current edits (Ctrl+Enter)"));
    layout->addWidget(commit);
    layout->addStretch(1);
    if (!controller_) {
        return page;
    }

    const auto sync = [=, this]() {
        const TypeOptions o = controller_->typeOptions();
        const QSignalBlocker blockFamily(family);
        const QSignalBlocker blockAntialias(antialias);
        family->setCurrentFont(QFont(o.family));
        size->setValue(o.size);
        antialias->setCurrentIndex(o.antialias ? 1 : 0);
        if (QAbstractButton* button = align->button(o.justification)) {
            button->setChecked(true);
        }
        const bool editing = controller_->textActive();
        cancel->setEnabled(editing);
        commit->setEnabled(editing);
    };
    sync();
    const auto update = [this](auto edit) {
        TypeOptions o = controller_->typeOptions();
        edit(o);
        controller_->setTypeOptions(o);
    };
    connect(controller_, &ToolController::activeToolChanged, page, sync);
    connect(controller_, &ToolController::typeOptionsChanged, page, sync);
    connect(controller_, &ToolController::textEditingChanged, page, [cancel, commit](bool on) {
        cancel->setEnabled(on);
        commit->setEnabled(on);
    });
    connect(orientation, &QToolButton::clicked, this,
            [this, id]() { controller_->setActiveTool(otherOrientation(id)); });
    connect(family, &QFontComboBox::currentFontChanged, this, [update](const QFont& font) {
        update([&font](TypeOptions& o) { o.family = font.family(); });
    });
    connect(size, &NumericField::valueChanged, this,
            [update](double v) { update([v](TypeOptions& o) { o.size = v; }); });
    connect(antialias, &QComboBox::currentIndexChanged, this,
            [update](int i) { update([i](TypeOptions& o) { o.antialias = i == 1; }); });
    connect(align, &QButtonGroup::idClicked, this,
            [update](int j) { update([j](TypeOptions& o) { o.justification = j; }); });
    connect(cancel, &QToolButton::clicked, this, [this]() { controller_->cancelText(); });
    connect(commit, &QToolButton::clicked, this, [this]() { controller_->commitText(); });
    return page;
}

} // namespace pictura
