// The options bars of the stamp tools: Clone Stamp, Pattern Stamp, and History
// Brush. Part of OptionsBar; split from options_bar.cpp along the page seam.

#include "options_bar.h"

#include "panels/numeric_field.h"

#include "pictura_app/src/cxxqt_object/paint_tools.cxxqt.h"

#include <QtGui/QIcon>
#include <QtGui/QImage>
#include <QtGui/QPixmap>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QStackedWidget>
#include <QtWidgets/QToolButton>

#include <algorithm>
#include <cstdint>

namespace pictura {

namespace {

QIcon patternIcon(int index)
{
    const int side = stamp_pattern_side();
    const ::rust::Vec<std::uint8_t> rgba = stamp_pattern_tile(index);
    if (rgba.size() != static_cast<size_t>(side) * side * 4) {
        return QIcon();
    }
    const QImage tile(rgba.data(), side, side, side * 4, QImage::Format_RGBA8888);
    return QIcon(QPixmap::fromImage(tile.copy()));
}

} // namespace

// The Brush's tip, Mode, Opacity, and Flow, shared through the same controller
// fields. The stamps' Mode lists the Brush modes that make sense for copied
// pixels (no Clear).
void OptionsBar::addStampPaintFields(QHBoxLayout* layout, QWidget* page)
{
    addBrushTipFields(layout, page);

    layout->addWidget(new QLabel(QStringLiteral("Mode:"), page));
    auto* mode = new QComboBox(page);
    mode->setObjectName(QStringLiteral("optionsStampMode"));
    mode->addItem(QStringLiteral("Normal"), QStringLiteral("normal"));
    mode->addItem(QStringLiteral("Dissolve"), QStringLiteral("dissolve"));
    mode->addItem(QStringLiteral("Behind"), QStringLiteral("behind"));
    layout->addWidget(mode);

    auto addField = [&](const QString& label, const QString& name, int value,
                        void (ToolController::*setter)(int)) {
        auto* field = new NumericField(
            label, numericConfig(0, 100, 1, 0, QStringLiteral("%"), true, name), page);
        field->setValue(value);
        layout->addWidget(field);
        if (controller_) {
            connect(field, &NumericField::valueChanged, this,
                    [this, setter](double v) { (controller_->*setter)(qRound(v)); });
        }
    };
    addField(QStringLiteral("Opacity:"), QStringLiteral("optionsStampOpacity"),
             controller_ ? controller_->brushOpacity() : 100, &ToolController::setBrushOpacity);
    addField(QStringLiteral("Flow:"), QStringLiteral("optionsStampFlow"),
             controller_ ? controller_->brushFlow() : 100, &ToolController::setBrushFlow);

    if (controller_) {
        mode->setCurrentIndex(std::max(0, mode->findData(controller_->brushMode())));
        connect(mode, &QComboBox::currentIndexChanged, this, [this, mode](int) {
            controller_->setBrushMode(mode->currentData().toString());
        });
    }
}

// CS6's Clone Stamp and Pattern Stamp bars: the paint fields plus Aligned and
// Sample (with Ignore Adjustment Layers beside All Layers), or the pattern
// picker, Aligned, and Impressionist.
QWidget* OptionsBar::buildStampPage(ToolId id)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(toolButton(id, page));
    addStampPaintFields(layout, page);

    const StampOptions initial = controller_ ? controller_->stampOptions() : StampOptions{};
    const auto update = [this](auto edit) {
        if (controller_) {
            StampOptions o = controller_->stampOptions();
            edit(o);
            controller_->setStampOptions(o);
        }
    };
    const bool clone = id == ToolId::CloneStamp;

    if (!clone) {
        auto* pattern = new QComboBox(page);
        pattern->setObjectName(QStringLiteral("optionsPatternStampPattern"));
        pattern->setToolTip(QStringLiteral("Pattern"));
        pattern->setIconSize(QSize(20, 20));
        for (int i = 0; i < stamp_pattern_count(); ++i) {
            pattern->addItem(patternIcon(i), stamp_pattern_name(i));
        }
        pattern->setCurrentIndex(initial.pattern);
        layout->addWidget(pattern);
        connect(pattern, &QComboBox::currentIndexChanged, this,
                [update](int i) { update([i](StampOptions& o) { o.pattern = i; }); });
    }

    auto* aligned = new QCheckBox(QStringLiteral("Aligned"), page);
    aligned->setObjectName(clone ? QStringLiteral("optionsCloneAligned")
                                 : QStringLiteral("optionsPatternAligned"));
    aligned->setChecked(clone ? initial.cloneAligned : initial.patternAligned);
    layout->addWidget(aligned);
    connect(aligned, &QCheckBox::toggled, this, [update, clone](bool on) {
        update([on, clone](StampOptions& o) { (clone ? o.cloneAligned : o.patternAligned) = on; });
    });

    if (clone) {
        layout->addWidget(new QLabel(QStringLiteral("Sample:"), page));
        auto* sample = new QComboBox(page);
        sample->setObjectName(QStringLiteral("optionsCloneSample"));
        sample->addItems({QStringLiteral("Current Layer"), QStringLiteral("Current & Below"),
                          QStringLiteral("All Layers")});
        sample->setCurrentIndex(initial.cloneSample);
        layout->addWidget(sample);
        auto* ignore = new QCheckBox(QStringLiteral("Ignore Adjustment Layers"), page);
        ignore->setObjectName(QStringLiteral("optionsCloneIgnoreAdjustments"));
        ignore->setChecked(initial.ignoreAdjustments);
        ignore->setVisible(initial.cloneSample == 2);
        layout->addWidget(ignore);
        connect(sample, &QComboBox::currentIndexChanged, this, [update, ignore](int i) {
            ignore->setVisible(i == 2);
            update([i](StampOptions& o) { o.cloneSample = i; });
        });
        connect(ignore, &QCheckBox::toggled, this,
                [update](bool on) { update([on](StampOptions& o) { o.ignoreAdjustments = on; }); });
    } else {
        auto* impressionist = new QCheckBox(QStringLiteral("Impressionist"), page);
        impressionist->setObjectName(QStringLiteral("optionsPatternImpressionist"));
        impressionist->setToolTip(QStringLiteral("Impressionist: not implemented yet"));
        impressionist->setEnabled(false);
        layout->addWidget(impressionist);
    }

    layout->addStretch(1);
    return page;
}

// CS6's History Brush bar: the paint fields. The source is chosen in the
// History panel.
QWidget* OptionsBar::buildHistoryBrushPage(ToolId id)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(toolButton(id, page));
    addStampPaintFields(layout, page);
    layout->addStretch(1);
    return page;
}

} // namespace pictura
