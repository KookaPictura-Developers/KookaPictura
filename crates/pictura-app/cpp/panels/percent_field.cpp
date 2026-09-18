#include "percent_field.h"

#include <QtCore/QEvent>
#include <QtGui/QIntValidator>
#include <QtGui/QMouseEvent>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QSlider>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

PercentField::PercentField(const QString& label, QWidget* parent)
    : QWidget(parent)
{
    setObjectName(QStringLiteral("percentField"));

    auto* layout = new QHBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(3);

    label_ = new QLabel(label, this);
    label_->setObjectName(QStringLiteral("percentLabel"));
    // The label is a scrub handle, so it takes the field's cursor affordance.
    label_->setCursor(Qt::SizeHorCursor);
    label_->installEventFilter(this);
    layout->addWidget(label_);

    edit_ = new QLineEdit(this);
    edit_->setObjectName(QStringLiteral("percentEdit"));
    edit_->setValidator(new QIntValidator(0, 100, edit_));
    edit_->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
    edit_->setFixedWidth(34);
    edit_->setText(QString::number(value_));
    edit_->installEventFilter(this);
    layout->addWidget(edit_);

    suffix_ = new QLabel(QStringLiteral("%"), edit_);
    suffix_->setObjectName(QStringLiteral("percentSuffix"));
    suffix_->setCursor(Qt::SizeHorCursor);
    suffix_->installEventFilter(this);

    arrow_ = new QToolButton(this);
    arrow_->setObjectName(QStringLiteral("percentArrow"));
    arrow_->setAutoRaise(true);
    arrow_->setArrowType(Qt::DownArrow);
    arrow_->setFixedWidth(16);
    arrow_->setToolTip(tr("Slider"));
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
    connect(slider_, &QSlider::sliderReleased, this, [this] { commitPending(); });
    popup_->installEventFilter(this);
    connect(arrow_, &QToolButton::clicked, this, [this] { showPopup(); });
    layoutSuffix();
}

QString PercentField::labelText() const
{
    return label_ ? label_->text() : QString();
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
    commitPending();
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
    pending_ = true;
}

void PercentField::commitPending()
{
    if (pending_) {
        pending_ = false;
        emit valueCommitted(value_);
    }
}

void PercentField::layoutSuffix()
{
    if (!suffix_ || !edit_) {
        return;
    }
    suffix_->adjustSize();
    edit_->setTextMargins(0, 0, suffix_->sizeHint().width() + 6, 0);
    suffix_->move(edit_->width() - suffix_->width() - 4,
                  (edit_->height() - suffix_->height()) / 2);
}

void PercentField::showPopup()
{
    popup_->adjustSize();
    const int x = (width() - popup_->width()) / 2;
    popup_->move(mapToGlobal(QPoint(x, height())));
    popup_->show();
}

bool PercentField::eventFilter(QObject* watched, QEvent* event)
{
    if (watched == edit_ && event->type() == QEvent::Resize) {
        layoutSuffix();
        return QWidget::eventFilter(watched, event);
    }
    if (watched == popup_ && event->type() == QEvent::Hide) {
        commitPending();
    }
    const bool isHandle = watched == label_ || watched == edit_ || watched == suffix_;
    if (!isHandle) {
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
        commitPending();
        return true;
    }
    return QWidget::eventFilter(watched, event);
}

} // namespace pictura
