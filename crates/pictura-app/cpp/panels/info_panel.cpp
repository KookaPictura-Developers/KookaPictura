#include "info_panel.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/annotations.cxxqt.h"

#include <QtCore/QtGlobal>
#include <QtCore/QStringList>
#include <QtGui/QColor>
#include <QtWidgets/QFormLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QWidget>

#include <cmath>

namespace pictura {

namespace {
// CS6 shows CMYK as 0-100%; QColor's CMYK components are 0-255.
int cmykPercent(int component)
{
    return int(std::lround(component * 100.0 / 255.0));
}
} // namespace

InfoPanel::InfoPanel(QWidget* parent)
    : QWidget(parent)
{
    QWidget* body = this;
    auto* layout = new QFormLayout(body);
    form_ = layout;
    positionLabel_ = new QLabel(body);
    colorLabel_ = new QLabel(body);
    cmykLabel_ = new QLabel(body);
    cmykLabel_->setObjectName(QStringLiteral("infoCmyk"));
    selectionLabel_ = new QLabel(body);
    sizeLabel_ = new QLabel(body);
    rulerLabel_ = new QLabel(body);
    rulerLabel_->setObjectName(QStringLiteral("infoRuler"));

    // CS6 shows RGB and CMYK side by side on one colour row.
    auto* colorRow = new QWidget(body);
    auto* colorLayout = new QHBoxLayout(colorRow);
    colorLayout->setContentsMargins(0, 0, 0, 0);
    colorLayout->setSpacing(10);
    colorLayout->addWidget(colorLabel_);
    colorLayout->addWidget(cmykLabel_);
    colorLayout->addStretch(1);

    layout->addRow(tr("Position"), positionLabel_);
    layout->addRow(tr("Color"), colorRow);
    layout->addRow(tr("Selection"), selectionLabel_);
    layout->addRow(tr("Dimensions"), sizeLabel_);
    layout->addRow(tr("Ruler"), rulerLabel_);
    form_->setRowVisible(rulerLabel_, false);
    samplersLabel_ = new QLabel(body);
    samplersLabel_->setObjectName(QStringLiteral("infoSamplers"));
    layout->addRow(samplersLabel_);
    // ponytail: no "Doc: n/n" memory-footprint line; there is no document-size bridge.
    refresh();
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
    refresh();
}

void InfoPanel::refresh()
{
    if (!view_ || !view_->has_document()) {
        positionLabel_->setText(QStringLiteral("—"));
        colorLabel_->setText(QStringLiteral("—"));
        cmykLabel_->clear();
        selectionLabel_->setText(tr("none"));
        sizeLabel_->setText(QStringLiteral("—"));
        rulerLabel_->clear();
        form_->setRowVisible(rulerLabel_, rulerMode_);
        samplersLabel_->clear();
        return;
    }

    const int width = view_->document_width();
    const int height = view_->document_height();

    const int x = qRound(cursor_.x());
    const int y = qRound(cursor_.y());
    positionLabel_->setText(QStringLiteral("%1, %2").arg(x).arg(y));

    if (x >= 0 && y >= 0 && x < width && y < height) {
        const QColor color = QColor::fromRgba(QRgb(view_->sample_argb(x, y)));
        colorLabel_->setText(QStringLiteral("%1  rgb(%2, %3, %4)")
                                 .arg(color.name().toUpper())
                                 .arg(color.red())
                                 .arg(color.green())
                                 .arg(color.blue()));
        const QColor cmyk = color.toCmyk();
        cmykLabel_->setText(QStringLiteral("C %1  M %2  Y %3  K %4")
                                .arg(cmykPercent(cmyk.cyan()))
                                .arg(cmykPercent(cmyk.magenta()))
                                .arg(cmykPercent(cmyk.yellow()))
                                .arg(cmykPercent(cmyk.black())));
    } else {
        colorLabel_->setText(QStringLiteral("—"));
        cmykLabel_->clear();
    }

    const int selected = view_->selection_count();
    selectionLabel_->setText(selected > 0 ? tr("%1 px").arg(selected) : tr("none"));

    // The Ruler's measure() is [X, Y, W, H, A, D1]; empty when no line exists.
    form_->setRowVisible(rulerLabel_, rulerMode_);
    if (rulerMode_) {
        const ::rust::Vec<double> m = ruler_measurement(*view_);
        const bool hasRuler = m.size() == 6;
        rulerLabel_->setText(hasRuler ? QStringLiteral("A: %1°   L: %2")
                                            .arg(m[4], 0, 'f', 1)
                                            .arg(m[5], 0, 'f', 1)
                                      : QString());
        // In ruler mode the W/H block belongs to the ruler, not the document,
        // and stays blank until a measuring line exists.
        sizeLabel_->setText(hasRuler ? QStringLiteral("%1 × %2")
                                           .arg(m[2], 0, 'f', 1)
                                           .arg(m[3], 0, 'f', 1)
                                     : QString());
    } else {
        rulerLabel_->clear();
        sizeLabel_->setText(QStringLiteral("%1 × %2").arg(width).arg(height));
    }

    // CS6 lists the color samplers below the main readouts, numbered as on the
    // canvas; each reads the composite at its pixel.
    QStringList samplers;
    const int count = marker_count(*view_, 0);
    for (int i = 0; i < count; ++i) {
        const ::rust::Vec<std::int32_t> p = marker_at(*view_, 0, i);
        if (p.size() != 2) {
            continue;
        }
        const bool inside = p[0] >= 0 && p[1] >= 0 && p[0] < width && p[1] < height;
        const QColor c = QColor::fromRgba(QRgb(inside ? view_->sample_argb(p[0], p[1]) : 0));
        samplers << (inside ? QStringLiteral("#%1  %2, %3  R %4  G %5  B %6")
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
QString InfoPanel::colorTextForTest() const { return colorLabel_->text(); }
QString InfoPanel::cmykTextForTest() const { return cmykLabel_->text(); }
QString InfoPanel::sizeTextForTest() const { return sizeLabel_->text(); }
QString InfoPanel::rulerTextForTest() const { return rulerLabel_->text(); }

} // namespace pictura
