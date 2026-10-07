#include "black_white_dialog.h"

#include "color_picker_dialog.h"

#include <QtWidgets/QCheckBox>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSpinBox>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

namespace {

struct Channel {
    const char* label;
    const char* key;
    QColor color;
    int defaultValue;
};

const Channel kChannels[] = {
    {"Reds", "reds", QColor(230, 40, 40), 40},
    {"Yellows", "yellows", QColor(225, 210, 30), 60},
    {"Greens", "greens", QColor(40, 200, 60), 40},
    {"Cyans", "cyans", QColor(30, 200, 225), 60},
    {"Blues", "blues", QColor(50, 70, 225), 20},
    {"Magentas", "magentas", QColor(210, 40, 200), 80},
};

} // namespace

BlackWhiteDialog::BlackWhiteDialog(PictureView* view, const QByteArray& block,
                                   const QRect& visible, QWidget* parent)
    : AdjustmentDialog(view, block, visible, QStringLiteral("Black & White"), parent)
{
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(14, 12, 14, 12);
    layout->setSpacing(6);
    for (const Channel& channel : kChannels) {
        addRampRow(layout, QLatin1String(channel.label), QLatin1String(channel.key), -200, 300,
                   {Qt::black, channel.color}, QStringLiteral(" %"));
        layout->addSpacing(4);
    }

    auto* tintRow = new QHBoxLayout;
    auto* tint = new QCheckBox(QStringLiteral("Tint"), this);
    tint->setObjectName(QStringLiteral("tint"));
    tint->setChecked(param(QStringLiteral("tint")) != 0.0);
    QToolButton* color = swatch(QStringLiteral("tintColor"));
    paintSwatch(color, QColor::fromRgb(QRgb(uint(param(QStringLiteral("tintColor"))))));
    color->setEnabled(tint->isChecked());
    auto* reset = new QPushButton(QStringLiteral("Default"), this);
    reset->setObjectName(QStringLiteral("blackWhiteDefault"));
    reset->setAutoDefault(false);
    tintRow->addWidget(tint);
    tintRow->addWidget(color);
    tintRow->addSpacing(8);
    tintRow->addWidget(reset);
    tintRow->addStretch(1);
    layout->addLayout(tintRow);
    layout->addSpacing(4);
    layout->addWidget(previewCheck());
    layout->addStretch(1);
    layout->addLayout(bottomButtons());

    connect(tint, &QCheckBox::toggled, this, [this, color](bool on) {
        color->setEnabled(on);
        setParam(QStringLiteral("tint"), on ? 1.0 : 0.0);
    });
    connect(color, &QToolButton::clicked, this, [this, color]() {
        const QColor picked = ColorPickerDialog::getColor(color->property("color").value<QColor>(),
                                                          this, QStringLiteral("Tint Color"));
        if (picked.isValid()) {
            paintSwatch(color, picked);
            setParam(QStringLiteral("tintColor"), double(picked.rgb() & 0xffffffu));
        }
    });
    connect(reset, &QPushButton::clicked, this, [this]() {
        QList<QPair<QString, double>> edits;
        for (const Channel& channel : kChannels) {
            edits << qMakePair(QLatin1String(channel.key), double(channel.defaultValue));
        }
        setParams(edits);
        for (const Channel& channel : kChannels) {
            auto* spin = findChild<QSpinBox*>(QLatin1String(channel.key));
            const QSignalBlocker block(spin);
            spin->setValue(channel.defaultValue);
            findChild<QWidget*>(QLatin1String(channel.key) + QStringLiteral("Slider"))
                ->setProperty("value", channel.defaultValue);
        }
    });
    setMinimumWidth(460);
}

QWidget* BlackWhiteDialog::controlForTest(const QString& key) const
{
    return findChild<QWidget*>(key);
}

} // namespace pictura
