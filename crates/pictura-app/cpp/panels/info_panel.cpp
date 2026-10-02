#include "info_panel.h"

#include "icons.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/annotations.cxxqt.h"

#include <QtCore/QPair>
#include <QtCore/QStringList>
#include <QtCore/QtGlobal>
#include <QtGui/QAction>
#include <QtGui/QColor>
#include <QtWidgets/QFrame>
#include <QtWidgets/QGridLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QMenu>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QWidget>

#include <cmath>
#include <cstdint>

namespace pictura {

namespace {

// CS6 shows CMYK as 0-100%; QColor's CMYK components are 0-255.
int cmykPercent(int component)
{
    return int(std::lround(component * 100.0 / 255.0));
}

// Photoshop's "Doc:" memory line: kilobytes below a megabyte, else megabytes.
QString formatBytes(double bytes)
{
    if (bytes >= 1024.0 * 1024.0) {
        return QStringLiteral("%1M").arg(bytes / (1024.0 * 1024.0), 0, 'f', 1);
    }
    return QStringLiteral("%1K").arg(bytes / 1024.0, 0, 'f', 1);
}

struct Lab {
    double l;
    double a;
    double b;
};

// sRGB (D65) to CIE L*a*b*; L 0-100, a/b roughly -128..127.
Lab srgbToLab(const QColor& color)
{
    const auto linear = [](double v) {
        v /= 255.0;
        return v <= 0.04045 ? v / 12.92 : std::pow((v + 0.055) / 1.055, 2.4);
    };
    const double r = linear(color.red());
    const double g = linear(color.green());
    const double b = linear(color.blue());
    const double x = (0.4124564 * r + 0.3575761 * g + 0.1804375 * b) / 0.95047;
    const double y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
    const double z = (0.0193339 * r + 0.1191920 * g + 0.9503041 * b) / 1.08883;
    const auto f = [](double t) {
        const double e = 216.0 / 24389.0;
        const double k = 24389.0 / 27.0;
        return t > e ? std::cbrt(t) : (k * t + 16.0) / 116.0;
    };
    const double fx = f(x);
    const double fy = f(y);
    const double fz = f(z);
    return Lab{116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)};
}

} // namespace

InfoPanel::InfoPanel(QWidget* parent)
    : QWidget(parent)
{
    auto* outer = new QVBoxLayout(this);
    outer->setContentsMargins(4, 4, 4, 4);
    outer->setSpacing(4);

    grid_ = new QGridLayout;
    grid_->setContentsMargins(0, 0, 0, 0);
    grid_->setHorizontalSpacing(10);
    grid_->setVerticalSpacing(6);
    grid_->setColumnStretch(0, 1);
    grid_->setColumnStretch(1, 1);
    outer->addLayout(grid_);

    topLeft_ = addReadout(grid_, 0, 0,
                          {QStringLiteral("R"), QStringLiteral("G"), QStringLiteral("B")},
                          QStringLiteral("tool.eyedropper"), QStringLiteral("8-bit"),
                          MenuKind::Color);
    topLeft_->colorMode = ColorReadout::Rgb;
    topRight_ = addReadout(grid_, 0, 1,
                           {QStringLiteral("C"), QStringLiteral("M"), QStringLiteral("Y"),
                            QStringLiteral("K")},
                           QStringLiteral("tool.eyedropper"), QStringLiteral("8-bit"),
                           MenuKind::Color);
    topRight_->colorMode = ColorReadout::Cmyk;
    position_ = addReadout(grid_, 1, 0, {QStringLiteral("X"), QStringLiteral("Y")},
                           QStringLiteral("info.crosshair"), QString(), MenuKind::Unit);
    size_ = addReadout(grid_, 1, 1, {QStringLiteral("W"), QStringLiteral("H")},
                       QStringLiteral("info.bounds"), QString(), MenuKind::Unit);

    auto* rule = new QFrame(this);
    rule->setFrameShape(QFrame::HLine);
    rule->setObjectName(QStringLiteral("infoRule"));
    outer->addWidget(rule);

    docLabel_ = new QLabel(this);
    docLabel_->setObjectName(QStringLiteral("infoDocSize"));
    outer->addWidget(docLabel_);

    samplersLabel_ = new QLabel(this);
    samplersLabel_->setObjectName(QStringLiteral("infoSamplers"));
    samplersLabel_->setWordWrap(true);
    outer->addWidget(samplersLabel_);

    refresh();
}

InfoPanel::Readout* InfoPanel::addReadout(QGridLayout* grid, int row, int column,
                                          const QStringList& keys, const QString& iconId,
                                          const QString& footer, MenuKind menu)
{
    auto* block = new QWidget(this);
    auto* layout = new QGridLayout(block);
    layout->setContentsMargins(4, 3, 4, 3);
    layout->setHorizontalSpacing(4);
    layout->setVerticalSpacing(1);

    auto* readout = new Readout;
    readout->widget = block;

    readout->button = new QToolButton(block);
    readout->button->setIcon(pictura::icon(iconId));
    readout->button->setIconSize(QSize(14, 14));
    readout->button->setAutoRaise(true);
    readout->button->setCursor(Qt::PointingHandCursor);
    layout->addWidget(readout->button, 0, 0, 1, 1, Qt::AlignTop);

    auto* keysHost = new QWidget(block);
    auto* valuesHost = new QWidget(block);
    readout->keysHost = new QVBoxLayout(keysHost);
    readout->valuesHost = new QVBoxLayout(valuesHost);
    readout->keysHost->setContentsMargins(0, 0, 0, 0);
    readout->keysHost->setSpacing(1);
    readout->valuesHost->setContentsMargins(0, 0, 0, 0);
    readout->valuesHost->setSpacing(1);
    layout->addWidget(keysHost, 0, 1);
    layout->addWidget(valuesHost, 0, 2);
    layout->setColumnStretch(2, 1);

    if (!footer.isEmpty()) {
        // ponytail: static footer; a bit-depth menu can replace it when the
        // document exposes a depth switch.
        readout->footer = new QLabel(footer, block);
        readout->footer->setObjectName(QStringLiteral("infoFooter"));
        readout->footer->setAlignment(Qt::AlignHCenter);
        layout->addWidget(readout->footer, 1, 0, 1, 3);
    }

    rebuildRows(readout, keys);

    if (menu != MenuKind::None) {
        auto* m = new QMenu(readout->button);
        if (menu == MenuKind::Color) {
            const QList<QPair<QString, ColorReadout>> options = {
                {QStringLiteral("Grayscale"), ColorReadout::Grayscale},
                {QStringLiteral("RGB"), ColorReadout::Rgb},
                {QStringLiteral("HSB"), ColorReadout::Hsb},
                {QStringLiteral("CMYK"), ColorReadout::Cmyk},
                {QStringLiteral("Lab"), ColorReadout::Lab},
            };
            for (const auto& option : options) {
                QAction* action = m->addAction(option.first);
                action->setCheckable(true);
                action->setData(int(option.second));
                connect(action, &QAction::triggered, this,
                        [this, readout, mode = option.second] { applyColorMode(readout, mode); });
            }
            connect(m, &QMenu::aboutToShow, this, [readout, m] {
                for (QAction* action : m->actions()) {
                    action->setChecked(action->data().toInt() == int(readout->colorMode));
                }
            });
        } else {
            const QList<QPair<QString, MeasureUnit>> options = {
                {QStringLiteral("Pixels"), MeasureUnit::Pixels},
                {QStringLiteral("Inches"), MeasureUnit::Inches},
                {QStringLiteral("Centimeters"), MeasureUnit::Centimeters},
                {QStringLiteral("Millimeters"), MeasureUnit::Millimeters},
                {QStringLiteral("Points"), MeasureUnit::Points},
                {QStringLiteral("Picas"), MeasureUnit::Picas},
                {QStringLiteral("Percent"), MeasureUnit::Percent},
            };
            for (const auto& option : options) {
                QAction* action = m->addAction(option.first);
                action->setCheckable(true);
                action->setData(int(option.second));
                connect(action, &QAction::triggered, this,
                        [this, readout, unit = option.second] { applyUnit(readout, unit); });
            }
            connect(m, &QMenu::aboutToShow, this, [readout, m] {
                for (QAction* action : m->actions()) {
                    action->setChecked(action->data().toInt() == int(readout->unit));
                }
            });
        }
        readout->button->setMenu(m);
        readout->button->setPopupMode(QToolButton::InstantPopup);
    }

    grid->addWidget(block, row, column);
    return readout;
}

void InfoPanel::rebuildRows(Readout* readout, const QStringList& keys)
{
    while (QLayoutItem* item = readout->keysHost->takeAt(0)) {
        delete item->widget();
        delete item;
    }
    while (QLayoutItem* item = readout->valuesHost->takeAt(0)) {
        delete item->widget();
        delete item;
    }
    readout->values.clear();
    readout->keyNames = keys;
    for (const QString& key : keys) {
        auto* keyLabel = new QLabel(key + QStringLiteral(" :"), readout->widget);
        keyLabel->setObjectName(QStringLiteral("infoKey"));
        keyLabel->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
        auto* valueLabel = new QLabel(readout->widget);
        valueLabel->setObjectName(QStringLiteral("infoValue"));
        valueLabel->setAlignment(Qt::AlignLeft | Qt::AlignVCenter);
        valueLabel->setMinimumWidth(26);
        readout->keysHost->addWidget(keyLabel);
        readout->valuesHost->addWidget(valueLabel);
        readout->values.append(valueLabel);
    }
}

void InfoPanel::setValues(Readout* readout, const QStringList& values)
{
    if (!readout) {
        return;
    }
    for (int i = 0; i < readout->values.size(); ++i) {
        readout->values.at(i)->setText(i < values.size() ? values.at(i) : QString());
    }
}

void InfoPanel::applyColorMode(Readout* readout, ColorReadout mode)
{
    readout->colorMode = mode;
    QStringList keys;
    switch (mode) {
    case ColorReadout::Grayscale:
        keys << QStringLiteral("K");
        break;
    case ColorReadout::Rgb:
        keys << QStringLiteral("R") << QStringLiteral("G") << QStringLiteral("B");
        break;
    case ColorReadout::Hsb:
        keys << QStringLiteral("H") << QStringLiteral("S") << QStringLiteral("B");
        break;
    case ColorReadout::Cmyk:
        keys << QStringLiteral("C") << QStringLiteral("M") << QStringLiteral("Y")
             << QStringLiteral("K");
        break;
    case ColorReadout::Lab:
        keys << QStringLiteral("L") << QStringLiteral("a") << QStringLiteral("b");
        break;
    }
    rebuildRows(readout, keys);
    refresh();
}

void InfoPanel::applyUnit(Readout* readout, MeasureUnit unit)
{
    readout->unit = unit;
    refresh();
}

void InfoPanel::refreshColorBlock(Readout* readout, const QColor& color)
{
    switch (readout->colorMode) {
    case ColorReadout::Grayscale:
        setValues(readout, {QString::number(qGray(color.rgb()))});
        break;
    case ColorReadout::Rgb:
        setValues(readout, {QString::number(color.red()), QString::number(color.green()),
                            QString::number(color.blue())});
        break;
    case ColorReadout::Hsb: {
        const int hue = color.hue() < 0 ? 0 : color.hue();
        setValues(readout, {QString::number(hue),
                            QString::number(qRound(color.saturationF() * 100.0)),
                            QString::number(qRound(color.valueF() * 100.0))});
        break;
    }
    case ColorReadout::Cmyk: {
        const QColor cmyk = color.toCmyk();
        setValues(readout, {QString::number(cmykPercent(cmyk.cyan())),
                            QString::number(cmykPercent(cmyk.magenta())),
                            QString::number(cmykPercent(cmyk.yellow())),
                            QString::number(cmykPercent(cmyk.black()))});
        break;
    }
    case ColorReadout::Lab: {
        const Lab lab = srgbToLab(color);
        setValues(readout, {QString::number(qRound(lab.l)), QString::number(qRound(lab.a)),
                            QString::number(qRound(lab.b))});
        break;
    }
    }
}

QString InfoPanel::formatMeasure(double value, MeasureUnit unit, double percentBase) const
{
    switch (unit) {
    case MeasureUnit::Pixels:
        return QString::number(qRound(value));
    case MeasureUnit::Inches:
        return QString::number(value / 72.0, 'f', 2);
    case MeasureUnit::Centimeters:
        return QString::number(value / 72.0 * 2.54, 'f', 2);
    case MeasureUnit::Millimeters:
        return QString::number(value / 72.0 * 25.4, 'f', 2);
    case MeasureUnit::Points:
        return QString::number(value / 72.0 * 72.0, 'f', 2);
    case MeasureUnit::Picas:
        return QString::number(value / 72.0 * 6.0, 'f', 2);
    case MeasureUnit::Percent:
        return percentBase > 0.0 ? QString::number(value / percentBase * 100.0, 'f', 2)
                                 : QString();
    }
    return QString();
}

QString InfoPanel::blockText(const Readout* readout) const
{
    QStringList parts;
    for (int i = 0; i < readout->values.size(); ++i) {
        const QString key = i < readout->keyNames.size() ? readout->keyNames.at(i) : QString();
        parts << QStringLiteral("%1 : %2").arg(key, readout->values.at(i)->text());
    }
    return parts.join(QStringLiteral("  "));
}

void InfoPanel::rebuildTopRight()
{
    grid_->removeWidget(topRight_->widget);
    delete topRight_->widget;
    delete topRight_;
    if (rulerMode_) {
        topRight_ = addReadout(grid_, 0, 1, {QStringLiteral("A"), QStringLiteral("L")},
                               QStringLiteral("info.protractor"), QString(), MenuKind::None);
    } else {
        topRight_ = addReadout(grid_, 0, 1,
                               {QStringLiteral("C"), QStringLiteral("M"), QStringLiteral("Y"),
                                QStringLiteral("K")},
                               QStringLiteral("tool.eyedropper"), QStringLiteral("8-bit"),
                               MenuKind::Color);
        topRight_->colorMode = ColorReadout::Cmyk;
    }
}

void InfoPanel::setView(PictureView* view)
{
    view_ = view;
    refresh();
}

void InfoPanel::setCursorPosition(const QPointF& imagePos)
{
    cursor_ = imagePos;
    refresh();
}

void InfoPanel::setRulerMode(bool on)
{
    if (rulerMode_ == on) {
        return;
    }
    rulerMode_ = on;
    rebuildTopRight();
    refresh();
}

void InfoPanel::refresh()
{
    if (!view_ || !view_->has_document()) {
        setValues(topLeft_, {});
        setValues(topRight_, {});
        setValues(position_, {});
        setValues(size_, {});
        docLabel_->clear();
        samplersLabel_->clear();
        return;
    }

    const int width = view_->document_width();
    const int height = view_->document_height();

    const int x = int(std::floor(cursor_.x()));
    const int y = int(std::floor(cursor_.y()));

    setValues(position_, {formatMeasure(x, position_->unit, width),
                          formatMeasure(y, position_->unit, height)});

    const bool inside = x >= 0 && y >= 0 && x < width && y < height;
    if (inside) {
        const QColor color = QColor::fromRgba(QRgb(view_->sample_argb(x, y)));
        refreshColorBlock(topLeft_, color);
        if (!rulerMode_) {
            refreshColorBlock(topRight_, color);
        }
    } else {
        setValues(topLeft_, {});
        if (!rulerMode_) {
            setValues(topRight_, {});
        }
    }

    if (rulerMode_) {
        // [X, Y, W, H, A, D1]; empty when no measuring line exists.
        const ::rust::Vec<double> m = ruler_measurement(*view_);
        if (m.size() == 6) {
            setValues(topRight_, {QStringLiteral("%1°").arg(m[4], 0, 'f', 1),
                                  QString::number(m[5], 'f', 1)});
            setValues(size_, {formatMeasure(m[2], size_->unit, width),
                              formatMeasure(m[3], size_->unit, height)});
        } else {
            setValues(topRight_, {});
            setValues(size_, {});
        }
    } else {
        const QString bounds = view_->selection_bounds();
        const QStringList parts = bounds.split(QLatin1Char(' '), Qt::SkipEmptyParts);
        if (parts.size() == 4) {
            setValues(size_, {formatMeasure(parts[2].toDouble(), size_->unit, width),
                              formatMeasure(parts[3].toDouble(), size_->unit, height)});
        } else {
            setValues(size_, {});
        }
    }

    const ::rust::Vec<double> bytes = document_size_bytes(*view_);
    if (bytes.size() == 2) {
        docLabel_->setText(tr("Doc: %1/%2").arg(formatBytes(bytes[0]), formatBytes(bytes[1])));
    } else {
        docLabel_->clear();
    }

    // CS6 lists the colour samplers below the readouts, numbered as on the
    // canvas; each reads the composite at its pixel.
    QStringList samplers;
    const int count = marker_count(*view_, 0);
    for (int i = 0; i < count; ++i) {
        const ::rust::Vec<std::int32_t> p = marker_at(*view_, 0, i);
        if (p.size() != 2) {
            continue;
        }
        const bool pixelInside = p[0] >= 0 && p[1] >= 0 && p[0] < width && p[1] < height;
        const QColor c = QColor::fromRgba(QRgb(pixelInside ? view_->sample_argb(p[0], p[1]) : 0));
        samplers << (pixelInside ? QStringLiteral("#%1  %2, %3  R %4  G %5  B %6")
                                       .arg(i + 1)
                                       .arg(p[0])
                                       .arg(p[1])
                                       .arg(c.red())
                                       .arg(c.green())
                                       .arg(c.blue())
                                 : QStringLiteral("#%1  %2, %3  —").arg(i + 1).arg(p[0]).arg(p[1]));
    }
    samplersLabel_->setText(samplers.join(QLatin1Char('\n')));
}

QString InfoPanel::samplerTextForTest() const { return samplersLabel_->text(); }

QString InfoPanel::colorBlockTextForTest(int index) const
{
    return blockText(index == 1 ? topRight_ : topLeft_);
}

QString InfoPanel::positionTextForTest() const
{
    QStringList values;
    for (const QLabel* label : position_->values) {
        values << label->text();
    }
    return values.join(QString()).isEmpty() ? QString() : values.join(QStringLiteral(", "));
}

QString InfoPanel::sizeTextForTest() const
{
    QStringList values;
    for (const QLabel* label : size_->values) {
        values << label->text();
    }
    return values.join(QString()).isEmpty() ? QString() : values.join(QStringLiteral(" × "));
}

QString InfoPanel::rulerTextForTest() const
{
    QStringList values;
    for (const QLabel* label : topRight_->values) {
        values << label->text();
    }
    return values.join(QString()).isEmpty() ? QString() : values.join(QStringLiteral("  "));
}

QString InfoPanel::docTextForTest() const { return docLabel_->text(); }

void InfoPanel::setColorModeForTest(int index, const QString& mode)
{
    Readout* readout = index == 1 ? topRight_ : topLeft_;
    if (mode.compare(QStringLiteral("Grayscale"), Qt::CaseInsensitive) == 0) {
        applyColorMode(readout, ColorReadout::Grayscale);
    } else if (mode.compare(QStringLiteral("RGB"), Qt::CaseInsensitive) == 0) {
        applyColorMode(readout, ColorReadout::Rgb);
    } else if (mode.compare(QStringLiteral("HSB"), Qt::CaseInsensitive) == 0) {
        applyColorMode(readout, ColorReadout::Hsb);
    } else if (mode.compare(QStringLiteral("CMYK"), Qt::CaseInsensitive) == 0) {
        applyColorMode(readout, ColorReadout::Cmyk);
    } else if (mode.compare(QStringLiteral("Lab"), Qt::CaseInsensitive) == 0) {
        applyColorMode(readout, ColorReadout::Lab);
    }
}

void InfoPanel::setMeasurementUnitForTest(int index, const QString& unit)
{
    Readout* readout = index == 1 ? size_ : position_;
    if (unit.compare(QStringLiteral("Pixels"), Qt::CaseInsensitive) == 0) {
        applyUnit(readout, MeasureUnit::Pixels);
    } else if (unit.compare(QStringLiteral("Inches"), Qt::CaseInsensitive) == 0) {
        applyUnit(readout, MeasureUnit::Inches);
    } else if (unit.compare(QStringLiteral("Centimeters"), Qt::CaseInsensitive) == 0) {
        applyUnit(readout, MeasureUnit::Centimeters);
    } else if (unit.compare(QStringLiteral("Millimeters"), Qt::CaseInsensitive) == 0) {
        applyUnit(readout, MeasureUnit::Millimeters);
    } else if (unit.compare(QStringLiteral("Points"), Qt::CaseInsensitive) == 0) {
        applyUnit(readout, MeasureUnit::Points);
    } else if (unit.compare(QStringLiteral("Picas"), Qt::CaseInsensitive) == 0) {
        applyUnit(readout, MeasureUnit::Picas);
    } else if (unit.compare(QStringLiteral("Percent"), Qt::CaseInsensitive) == 0) {
        applyUnit(readout, MeasureUnit::Percent);
    }
}

} // namespace pictura
