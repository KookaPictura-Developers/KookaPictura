#include "info_panel.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/annotations.cxxqt.h"

#include <QtCore/QtGlobal>
#include <QtCore/QStringList>
#include <QtGui/QColor>
#include <QtWidgets/QFormLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QWidget>

namespace pictura {

InfoPanel::InfoPanel(QWidget* parent)
    : QWidget(parent)
{
    QWidget* body = this;
    auto* layout = new QFormLayout(body);
    positionLabel_ = new QLabel(body);
    colorLabel_ = new QLabel(body);
    selectionLabel_ = new QLabel(body);
    sizeLabel_ = new QLabel(body);
    layout->addRow(tr("Position"), positionLabel_);
    layout->addRow(tr("Color"), colorLabel_);
    layout->addRow(tr("Selection"), selectionLabel_);
    layout->addRow(tr("Dimensions"), sizeLabel_);
    samplersLabel_ = new QLabel(body);
    samplersLabel_->setObjectName(QStringLiteral("infoSamplers"));
    layout->addRow(samplersLabel_);
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

void InfoPanel::refresh()
{
    if (!view_ || !view_->has_document()) {
        positionLabel_->setText(QStringLiteral("—"));
        colorLabel_->setText(QStringLiteral("—"));
        selectionLabel_->setText(tr("none"));
        sizeLabel_->setText(QStringLiteral("—"));
        samplersLabel_->clear();
        return;
    }

    const int width = view_->document_width();
    const int height = view_->document_height();
    sizeLabel_->setText(QStringLiteral("%1 × %2").arg(width).arg(height));

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
    } else {
        colorLabel_->setText(QStringLiteral("—"));
    }

    const int selected = view_->selection_count();
    selectionLabel_->setText(selected > 0 ? tr("%1 px").arg(selected) : tr("none"));

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

} // namespace pictura
