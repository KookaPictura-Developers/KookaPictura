#include "histogram_panel.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QRectF>
#include <QtGui/QImage>
#include <QtGui/QPainter>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QWidget>

#include <algorithm>

namespace pictura {

HistogramView::HistogramView(QWidget* parent)
    : QWidget(parent)
{
    setMinimumHeight(120);
}

void HistogramView::setBins(const QVector<quint32>& bins, const QColor& color)
{
    bins_ = bins;
    color_ = color;
    update();
}

void HistogramView::clear()
{
    bins_.clear();
    update();
}

void HistogramView::paintEvent(QPaintEvent*)
{
    QPainter painter(this);
    painter.fillRect(rect(), QColor(32, 32, 32));
    painter.setPen(QColor(80, 80, 80));
    painter.drawRect(rect().adjusted(0, 0, -1, -1));
    if (bins_.isEmpty()) {
        return;
    }
    quint32 peak = 0;
    for (quint32 count : bins_) {
        peak = std::max(peak, count);
    }
    if (peak == 0) {
        return;
    }
    const QRect area = rect().adjusted(1, 1, -1, -1);
    const double barWidth = double(area.width()) / bins_.size();
    painter.setPen(Qt::NoPen);
    painter.setBrush(color_);
    for (int i = 0; i < bins_.size(); ++i) {
        const double barHeight = double(bins_[i]) / peak * area.height();
        painter.drawRect(QRectF(area.x() + i * barWidth, area.bottom() + 1 - barHeight,
                                std::max(1.0, barWidth), barHeight));
    }
}

HistogramPanel::HistogramPanel(QWidget* parent)
    : QDockWidget(tr("Histogram"), parent)
{
    auto* body = new QWidget(this);
    auto* layout = new QVBoxLayout(body);
    channel_ = new QComboBox(body);
    channel_->addItem(tr("Red"));
    channel_->addItem(tr("Green"));
    channel_->addItem(tr("Blue"));
    channel_->addItem(tr("Luminance"));
    layout->addWidget(channel_);
    histogram_ = new HistogramView(body);
    layout->addWidget(histogram_, 1);
    setWidget(body);

    connect(channel_, &QComboBox::currentIndexChanged, this, [this](int) { applyChannel(); });
}

void HistogramPanel::setView(PictureView* view)
{
    view_ = view;
    refresh();
}

void HistogramPanel::refresh()
{
    recompute();
    applyChannel();
}

void HistogramPanel::recompute()
{
    bins_ = {};
    if (!view_) {
        return;
    }
    const QImage image = view_->image().convertToFormat(QImage::Format_RGB32);
    if (image.isNull()) {
        return;
    }
    for (int y = 0; y < image.height(); ++y) {
        const QRgb* line = reinterpret_cast<const QRgb*>(image.constScanLine(y));
        for (int x = 0; x < image.width(); ++x) {
            const QRgb pixel = line[x];
            ++bins_[0][qRed(pixel)];
            ++bins_[1][qGreen(pixel)];
            ++bins_[2][qBlue(pixel)];
            ++bins_[3][qGray(pixel)];
        }
    }
}

void HistogramPanel::applyChannel()
{
    if (!view_) {
        histogram_->clear();
        return;
    }
    const int index = std::clamp(channel_->currentIndex(), 0, 3);
    static const QColor colors[4] = {
        QColor(230, 90, 90),
        QColor(90, 220, 90),
        QColor(90, 130, 240),
        QColor(220, 220, 220),
    };
    QVector<quint32> bins(256);
    for (int i = 0; i < 256; ++i) {
        bins[i] = bins_[index][i];
    }
    histogram_->setBins(bins, colors[index]);
}

} // namespace pictura
