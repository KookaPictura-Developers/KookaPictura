#include "color_panel.h"

#include <QtCore/QSignalBlocker>
#include <QtGui/QLinearGradient>
#include <QtGui/QMouseEvent>
#include <QtGui/QPainter>
#include <QtWidgets/QGridLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSlider>
#include <QtWidgets/QVBoxLayout>

#include <algorithm>
#include <cmath>

namespace pictura {

ColorState::ColorState(QObject* parent)
    : QObject(parent)
{
}

void ColorState::setForeground(const QColor& color)
{
    if (!color.isValid() || color == foreground_) {
        return;
    }
    foreground_ = color;
    emit foregroundChanged(foreground_);
}

void ColorState::setBackground(const QColor& color)
{
    if (!color.isValid() || color == background_) {
        return;
    }
    background_ = color;
    emit backgroundChanged(background_);
}

void ColorState::setForegroundActive(bool foreground)
{
    if (foregroundActive_ == foreground) {
        return;
    }
    foregroundActive_ = foreground;
    emit activeChanged(foregroundActive_);
}

HueSpectrum::HueSpectrum(QWidget* parent)
    : QWidget(parent)
{
    setMinimumHeight(24);
}

void HueSpectrum::setHuePicked(std::function<void(int)> callback)
{
    picked_ = std::move(callback);
}

void HueSpectrum::paintEvent(QPaintEvent*)
{
    QPainter painter(this);
    QLinearGradient gradient(0, 0, width(), 0);
    for (int i = 0; i <= 6; ++i) {
        gradient.setColorAt(double(i) / 6.0, QColor::fromHsv((i * 60) % 360, 255, 255));
    }
    painter.fillRect(rect(), gradient);
    painter.setPen(QColor(20, 20, 20));
    painter.drawRect(rect().adjusted(0, 0, -1, -1));
}

void HueSpectrum::mousePressEvent(QMouseEvent* event)
{
    if (event->button() == Qt::LeftButton) {
        pickAt(event->position());
        event->accept();
        return;
    }
    QWidget::mousePressEvent(event);
}

void HueSpectrum::mouseMoveEvent(QMouseEvent* event)
{
    if (event->buttons() & Qt::LeftButton) {
        pickAt(event->position());
        event->accept();
        return;
    }
    QWidget::mouseMoveEvent(event);
}

void HueSpectrum::pickAt(const QPointF& pos)
{
    if (width() <= 0 || !picked_) {
        return;
    }
    const int hue = std::clamp(int(std::lround(pos.x() / width() * 359.0)), 0, 359);
    picked_(hue);
}

ColorPanel::ColorPanel(ColorState* state, QWidget* parent)
    : QDockWidget(tr("Color"), parent)
    , state_(state)
{
    auto* body = new QWidget(this);
    auto* layout = new QVBoxLayout(body);

    auto* swatchRow = new QHBoxLayout();
    fgSwatch_ = new QPushButton(body);
    bgSwatch_ = new QPushButton(body);
    fgSwatch_->setFixedSize(48, 28);
    bgSwatch_->setFixedSize(48, 28);
    fgSwatch_->setFlat(true);
    bgSwatch_->setFlat(true);
    swatchRow->addWidget(fgSwatch_);
    swatchRow->addWidget(bgSwatch_);
    swatchRow->addStretch(1);
    layout->addLayout(swatchRow);

    auto* grid = new QGridLayout();
    const char* rgbLabels[3] = {"R", "G", "B"};
    for (int i = 0; i < 3; ++i) {
        rgb_[i] = new QSlider(Qt::Horizontal, body);
        rgb_[i]->setRange(0, 255);
        rgbValue_[i] = new QLabel(body);
        grid->addWidget(new QLabel(QString::fromLatin1(rgbLabels[i]), body), i, 0);
        grid->addWidget(rgb_[i], i, 1);
        grid->addWidget(rgbValue_[i], i, 2);
    }
    const char* hsbLabels[3] = {"H", "S", "B"};
    for (int i = 0; i < 3; ++i) {
        hsb_[i] = new QSlider(Qt::Horizontal, body);
        hsb_[i]->setRange(0, i == 0 ? 359 : 255);
        hsbValue_[i] = new QLabel(body);
        grid->addWidget(new QLabel(QString::fromLatin1(hsbLabels[i]), body), 3 + i, 0);
        grid->addWidget(hsb_[i], 3 + i, 1);
        grid->addWidget(hsbValue_[i], 3 + i, 2);
    }
    layout->addLayout(grid);

    hex_ = new QLineEdit(body);
    layout->addWidget(hex_);

    spectrum_ = new HueSpectrum(body);
    layout->addWidget(spectrum_);
    layout->addStretch(1);
    setWidget(body);

    for (int i = 0; i < 3; ++i) {
        connect(rgb_[i], &QSlider::valueChanged, this, [this](int) { applyRgb(); });
        connect(hsb_[i], &QSlider::valueChanged, this, [this](int) { applyHsb(); });
    }
    connect(hex_, &QLineEdit::editingFinished, this, [this] {
        const QColor color(hex_->text().trimmed());
        if (color.isValid()) {
            selectColor(color);
        }
    });
    spectrum_->setHuePicked([this](int hue) {
        int h = 0;
        int s = 0;
        int v = 0;
        activeColor().getHsv(&h, &s, &v);
        if (s == 0) {
            s = 255;
        }
        if (v == 0) {
            v = 255;
        }
        selectColor(QColor::fromHsv(hue, s, v));
    });
    connect(fgSwatch_, &QPushButton::clicked, this, [this] {
        if (state_) {
            state_->setForegroundActive(true);
        }
        syncControls();
    });
    connect(bgSwatch_, &QPushButton::clicked, this, [this] {
        if (state_) {
            state_->setForegroundActive(false);
        }
        syncControls();
    });
    if (state_) {
        connect(state_, &ColorState::foregroundChanged, this, [this](const QColor&) {
            syncControls();
        });
        connect(state_, &ColorState::backgroundChanged, this, [this](const QColor&) {
            syncControls();
        });
        connect(state_, &ColorState::activeChanged, this, [this](bool) { syncControls(); });
    }

    syncControls();
}

void ColorPanel::applyRgb()
{
    selectColor(QColor(rgb_[0]->value(), rgb_[1]->value(), rgb_[2]->value()));
}

void ColorPanel::applyHsb()
{
    selectColor(QColor::fromHsv(hsb_[0]->value(), hsb_[1]->value(), hsb_[2]->value()));
}

void ColorPanel::selectColor(const QColor& color)
{
    if (!state_ || !color.isValid()) {
        return;
    }
    if (state_->foregroundActive()) {
        state_->setForeground(color);
    } else {
        state_->setBackground(color);
    }
}

QColor ColorPanel::activeColor() const
{
    if (!state_) {
        return QColor();
    }
    return state_->foregroundActive() ? state_->foreground() : state_->background();
}

void ColorPanel::syncControls()
{
    if (!state_) {
        return;
    }
    const QColor color = activeColor();
    {
        const QSignalBlocker blockR(rgb_[0]);
        const QSignalBlocker blockG(rgb_[1]);
        const QSignalBlocker blockB(rgb_[2]);
        rgb_[0]->setValue(color.red());
        rgb_[1]->setValue(color.green());
        rgb_[2]->setValue(color.blue());
        rgbValue_[0]->setText(QString::number(color.red()));
        rgbValue_[1]->setText(QString::number(color.green()));
        rgbValue_[2]->setText(QString::number(color.blue()));
    }
    int h = 0;
    int s = 0;
    int v = 0;
    color.getHsv(&h, &s, &v);
    {
        const QSignalBlocker blockH(hsb_[0]);
        const QSignalBlocker blockS(hsb_[1]);
        const QSignalBlocker blockV(hsb_[2]);
        hsb_[0]->setValue(h);
        hsb_[1]->setValue(s);
        hsb_[2]->setValue(v);
        hsbValue_[0]->setText(QString::number(h));
        hsbValue_[1]->setText(QString::number(s));
        hsbValue_[2]->setText(QString::number(v));
    }
    {
        const QSignalBlocker block(hex_);
        hex_->setText(color.name().toUpper());
    }
    paintSwatch(fgSwatch_, state_->foreground(), state_->foregroundActive());
    paintSwatch(bgSwatch_, state_->background(), !state_->foregroundActive());
}

void ColorPanel::paintSwatch(QPushButton* button, const QColor& color, bool active)
{
    const int border = active ? 2 : 1;
    const QColor borderColor = active ? QColor(Qt::black) : QColor(120, 120, 120);
    button->setStyleSheet(QStringLiteral(
                              "QPushButton { background-color: %1; border: %2px solid %3; }")
                              .arg(color.name())
                              .arg(border)
                              .arg(borderColor.name()));
}

} // namespace pictura
