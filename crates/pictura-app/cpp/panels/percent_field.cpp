#include "percent_field.h"

#include <QtCore/QEvent>
#include <QtGui/QIntValidator>
#include <QtGui/QMouseEvent>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QSlider>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

PercentField::PercentField(QWidget* parent)
    : QWidget(parent)
{
    setObjectName(QStringLiteral("percentField"));

    auto* layout = new QHBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(0);

    edit_ = new QLineEdit(this);
    edit_->setObjectName(QStringLiteral("percentEdit"));
    edit_->setValidator(new QIntValidator(0, 100, edit_));
    edit_->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
    edit_->setFixedWidth(34);
    edit_->setText(QString::number(value_));
    edit_->installEventFilter(this);

    arrow_ = new QToolButton(this);
    arrow_->setObjectName(QStringLiteral("percentArrow"));
    arrow_->setAutoRaise(true);
    arrow_->setArrowType(Qt::DownArrow);
    arrow_->setFixedWidth(16);
    arrow_->setToolTip(tr("Slider"));

    layout->addWidget(edit_);
    layout->addWidget(arrow_);

    popup_ = new QWidget(this, Qt::Popup);
    auto* popupLayout = new QVBoxLayout(popup_);
    popupLayout->setContentsMargins(6, 6, 6, 6);
    slider_ = new QSlider(Qt::Horizontal, popup_);
    slider_->setRange(0, 100);
    slider_->setValue(value_);
    slider_->setMinimumWidth(120);
    popupLayout->addWidget(slider_);

    connect(edit_, &QLineEdit::editingFinished, this, [this] { commitEdit(); });
    connect(slider_, &QSlider::valueChanged, this, [this](int pct) {
        if (!syncing_) {
            applyUserValue(pct);
        }
    });
    connect(arrow_, &QToolButton::clicked, this, [this] { showPopup(); });
}

void PercentField::setValue(int pct)
{
    value_ = qBound(0, pct, 100);
    edit_->setText(QString::number(value_));
    syncing_ = true;
    slider_->setValue(value_);
    syncing_ = false;
}

void PercentField::commitEdit()
{
    bool ok = false;
    const int pct = edit_->text().toInt(&ok);
    if (!ok || pct < 0 || pct > 100) {
        edit_->setText(QString::number(value_));
        return;
    }
    applyUserValue(pct);
}

void PercentField::applyUserValue(int pct)
{
    pct = qBound(0, pct, 100);
    if (pct == value_) {
        edit_->setText(QString::number(value_));
        return;
    }
    value_ = pct;
    edit_->setText(QString::number(value_));
    syncing_ = true;
    slider_->setValue(value_);
    syncing_ = false;
    emit valueChanged(value_);
}

void PercentField::showPopup()
{
    popup_->adjustSize();
    popup_->move(arrow_->mapToGlobal(QPoint(0, arrow_->height())));
    popup_->show();
}

bool PercentField::eventFilter(QObject* watched, QEvent* event)
{
    if (watched != edit_) {
        return QWidget::eventFilter(watched, event);
    }
    if (event->type() == QEvent::MouseButtonPress) {
        auto* mouse = static_cast<QMouseEvent*>(event);
        if (mouse->button() == Qt::LeftButton && isEnabled()) {
            scrubbing_ = false;
            scrubOrigin_ = mouse->globalPosition().toPoint();
            scrubStart_ = value_;
        }
    } else if (event->type() == QEvent::MouseMove) {
        auto* mouse = static_cast<QMouseEvent*>(event);
        if ((mouse->buttons() & Qt::LeftButton) && isEnabled()) {
            const int delta = mouse->globalPosition().toPoint().x() - scrubOrigin_.x();
            if (scrubbing_ || qAbs(delta) >= 3) {
                scrubbing_ = true;
                // ponytail: 1 %/px scrub; scale by a modifier if it feels coarse.
                applyUserValue(scrubStart_ + delta);
                return true;
            }
        }
    } else if (event->type() == QEvent::MouseButtonRelease && scrubbing_) {
        scrubbing_ = false;
        return true;
    }
    return QWidget::eventFilter(watched, event);
}

} // namespace pictura
