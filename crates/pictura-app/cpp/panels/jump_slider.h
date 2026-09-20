#pragma once

#include <QtCore/QPoint>
#include <QtCore/QRect>
#include <QtGui/QMouseEvent>
#include <QtWidgets/QSlider>
#include <QtWidgets/QStyle>
#include <QtWidgets/QStyleOptionSlider>

namespace pictura {

// A horizontal slider that jumps to the clicked point and then tracks the
// cursor while held. Under the Fusion style the stock QSlider only page-steps on
// a left groove click (`SH_Slider_PageSetButtons` includes `LeftButton`), so the
// tracking branch never runs.
class JumpSlider : public QSlider {
public:
    using QSlider::QSlider;

protected:
    void mousePressEvent(QMouseEvent* event) override
    {
        if (event->button() == Qt::LeftButton) {
            setSliderDown(true);
            setValue(valueAt(event->position().toPoint()));
            event->accept();
            return;
        }
        QSlider::mousePressEvent(event);
    }

    void mouseMoveEvent(QMouseEvent* event) override
    {
        if (isSliderDown()) {
            setValue(valueAt(event->position().toPoint()));
            event->accept();
            return;
        }
        QSlider::mouseMoveEvent(event);
    }

    void mouseReleaseEvent(QMouseEvent* event) override
    {
        if (event->button() == Qt::LeftButton && isSliderDown()) {
            setValue(valueAt(event->position().toPoint()));
            setSliderDown(false);
            event->accept();
            return;
        }
        QSlider::mouseReleaseEvent(event);
    }

private:
    int valueAt(const QPoint& pos) const
    {
        QStyleOptionSlider option;
        initStyleOption(&option);
        const QRect groove =
            style()->subControlRect(QStyle::CC_Slider, &option, QStyle::SC_SliderGroove, this);
        const QRect handle =
            style()->subControlRect(QStyle::CC_Slider, &option, QStyle::SC_SliderHandle, this);
        const int span = qMax(1, groove.width() - handle.width());
        const int offset = pos.x() - groove.x() - handle.width() / 2;
        return QStyle::sliderValueFromPosition(minimum(), maximum(), offset, span);
    }
};

} // namespace pictura
