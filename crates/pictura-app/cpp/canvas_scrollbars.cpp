#include "canvas_scrollbars.h"

#include "canvas_range.h"
#include "image_view.h"

#include <QtCore/QSignalBlocker>
#include <QtWidgets/QGridLayout>
#include <QtWidgets/QScrollBar>

#include <algorithm>
#include <cmath>

namespace pictura {

CanvasScrollBars::CanvasScrollBars(QWidget* parent)
    : QWidget(parent)
{
    grid_ = new QGridLayout(this);
    grid_->setContentsMargins(0, 0, 0, 0);
    grid_->setSpacing(0);
    hbar_ = new QScrollBar(Qt::Horizontal, this);
    vbar_ = new QScrollBar(Qt::Vertical, this);
    grid_->addWidget(vbar_, 0, 1);
    grid_->addWidget(hbar_, 1, 0);
    grid_->setColumnStretch(0, 1);
    grid_->setRowStretch(0, 1);
    hbar_->hide();
    vbar_->hide();

    connect(hbar_, &QScrollBar::valueChanged, this, [this](int value) {
        if (syncing_ || !view_) {
            return;
        }
        const QImage& image = view_->image();
        const OffsetRange range = offsetRangeFor(QSizeF(image.width(), image.height()),
                                                 view_->zoom(), QSizeF(view_->size()));
        view_->setOffset(QPointF(range.minX + value, view_->offset().y()));
    });
    connect(vbar_, &QScrollBar::valueChanged, this, [this](int value) {
        if (syncing_ || !view_) {
            return;
        }
        const QImage& image = view_->image();
        const OffsetRange range = offsetRangeFor(QSizeF(image.width(), image.height()),
                                                 view_->zoom(), QSizeF(view_->size()));
        view_->setOffset(QPointF(view_->offset().x(), range.minY + value));
    });
}

void CanvasScrollBars::setView(ImageView* view)
{
    if (view_ == view) {
        syncFromView();
        return;
    }
    if (view_) {
        disconnect(view_, nullptr, this, nullptr);
        view_->hide();
        grid_->removeWidget(view_);
    }
    view_ = view;
    if (view_) {
        view_->setParent(this);
        grid_->addWidget(view_, 0, 0);
        view_->show();
        connect(view_, &ImageView::viewChanged, this, [this]() { syncFromView(); });
    }
    syncFromView();
}

void CanvasScrollBars::resizeEvent(QResizeEvent* event)
{
    syncFromView();
    QWidget::resizeEvent(event);
}

void CanvasScrollBars::syncFromView()
{
    if (!view_ || syncing_) {
        return;
    }
    syncing_ = true;

    const QImage& image = view_->image();
    const bool haveImage = !image.isNull() && image.width() > 0 && image.height() > 0;
    const double zoom = view_->zoom();
    const QSizeF imageSize(image.width(), image.height());
    const double contentW = imageSize.width() * zoom;
    const double contentH = imageSize.height() * zoom;
    // Apply with the current geometry, settle, then re-apply: a bar's own
    // thickness can change the viewport just enough to re-decide, and this
    // two-pass keeps it from oscillating.
    for (int pass = 0; pass < 2; ++pass) {
        const bool showH = haveImage && contentW > view_->width();
        const bool showV = haveImage && contentH > view_->height();
        if (hbar_->isHidden() == showH) {
            hbar_->setVisible(showH);
        }
        if (vbar_->isHidden() == showV) {
            vbar_->setVisible(showV);
        }
        grid_->activate();
    }

    const OffsetRange range = offsetRangeFor(imageSize, zoom, QSizeF(view_->size()));
    {
        const QSignalBlocker blockH(hbar_);
        const QSignalBlocker blockV(vbar_);
        const int rangeX = int(std::lround(std::max(0.0, range.maxX - range.minX)));
        const int rangeY = int(std::lround(std::max(0.0, range.maxY - range.minY)));
        hbar_->setRange(0, rangeX);
        hbar_->setPageStep(std::max(1, view_->width()));
        hbar_->setSingleStep(std::max(1, view_->width() / 10));
        hbar_->setValue(int(std::lround(view_->offset().x() - range.minX)));
        vbar_->setRange(0, rangeY);
        vbar_->setPageStep(std::max(1, view_->height()));
        vbar_->setSingleStep(std::max(1, view_->height() / 10));
        vbar_->setValue(int(std::lround(view_->offset().y() - range.minY)));
    }

    syncing_ = false;
}

} // namespace pictura
