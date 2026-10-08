#include "filter_param_controls.h"
#include "color_picker_dialog.h"
#include "panels/jump_slider.h"

#include <QtCore/QSignalBlocker>
#include <QtCore/QVariant>
#include <QtGui/QColor>
#include <QtGui/QFontMetrics>
#include <QtGui/QMouseEvent>
#include <QtGui/QPainter>
#include <QtGui/QPen>
#include <QtGui/QResizeEvent>
#include <QtMath>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDoubleSpinBox>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSlider>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QWidget>

#include <functional>

namespace pictura {

namespace {

int slotCount(FilterControl control)
{
    switch (control) {
    case FilterControl::Color:
        return 3;
    case FilterControl::Placement:
        return 2;
    case FilterControl::BlurCenter:
        return 0;
    default:
        return 1;
    }
}

// A value box only needs room for five digits and a decimal (with a sign); a
// wider box would just waste the row.
int compactSpinWidth(const QWidget* widget)
{
    return widget->fontMetrics().horizontalAdvance(QStringLiteral("-00000.0")) + 34;
}

// Display-only draggable centre pad for Radial Blur: concentric rings for a
// spin, rays for a zoom. It carries no value.
class BlurCenterWidget : public QWidget {
public:
    explicit BlurCenterWidget(QWidget* parent = nullptr) : QWidget(parent)
    {
        setMinimumSize(120, 120);
        setCursor(Qt::CrossCursor);
    }

    void setZoom(bool zoom)
    {
        zoom_ = zoom;
        update();
    }

protected:
    void resizeEvent(QResizeEvent*) override
    {
        if (!moved_) {
            center_ = QPointF(width() / 2.0, height() / 2.0);
        }
    }

    void paintEvent(QPaintEvent*) override
    {
        QPainter painter(this);
        painter.fillRect(rect(), QColor(48, 48, 48));
        const double radius = qMin(width(), height()) / 2.0 - 4.0;
        painter.setPen(QPen(QColor(160, 160, 160), 1));
        for (int i = 1; i <= 4; ++i) {
            painter.drawEllipse(center_, radius * i / 4.0, radius * i / 4.0);
        }
        if (zoom_) {
            for (int degrees = 0; degrees < 360; degrees += 30) {
                const double a = qDegreesToRadians(static_cast<double>(degrees));
                painter.drawLine(QPointF(center_.x() + qCos(a) * radius * 0.15,
                                         center_.y() + qSin(a) * radius * 0.15),
                                 QPointF(center_.x() + qCos(a) * radius,
                                         center_.y() + qSin(a) * radius));
            }
        }
        painter.setPen(QPen(Qt::white, 2));
        painter.drawLine(QPointF(center_.x() - 6, center_.y()),
                         QPointF(center_.x() + 6, center_.y()));
        painter.drawLine(QPointF(center_.x(), center_.y() - 6),
                         QPointF(center_.x(), center_.y() + 6));
    }

    void mousePressEvent(QMouseEvent* event) override { moveTo(event->position()); }

    void mouseMoveEvent(QMouseEvent* event) override
    {
        if (event->buttons() & Qt::LeftButton) {
            moveTo(event->position());
        }
    }

private:
    void moveTo(const QPointF& pos)
    {
        moved_ = true;
        center_ = QPointF(qBound(0.0, pos.x(), static_cast<double>(width())),
                          qBound(0.0, pos.y(), static_cast<double>(height())));
        update();
    }

    QPointF center_{60.0, 60.0};
    bool zoom_ = false;
    bool moved_ = false;
};

// Draggable normalized-placement pad for Lens Flare: shows the image (or a
// plain frame) with a crosshair. `moved` reports normalized (0..1) x/y.
class PlacementPad : public QWidget {
public:
    explicit PlacementPad(QWidget* parent = nullptr) : QWidget(parent)
    {
        setMinimumSize(120, 120);
        setCursor(Qt::CrossCursor);
    }

    std::function<void(double, double)> moved;

    void setImage(const QImage& image)
    {
        image_ = image;
        update();
    }

    // Silent: used by the spin boxes so the two never feed back.
    void setPosition(double x, double y)
    {
        x_ = qBound(0.0, x, 1.0);
        y_ = qBound(0.0, y, 1.0);
        update();
    }

protected:
    void paintEvent(QPaintEvent*) override
    {
        QPainter painter(this);
        if (!image_.isNull()) {
            painter.drawImage(rect(), image_);
        } else {
            painter.fillRect(rect(), QColor(48, 48, 48));
            painter.setPen(QPen(QColor(160, 160, 160), 1));
            painter.drawRect(rect().adjusted(0, 0, -1, -1));
        }
        const QPointF p(x_ * width(), y_ * height());
        painter.setPen(QPen(Qt::white, 2));
        painter.drawLine(QPointF(p.x() - 7, p.y()), QPointF(p.x() + 7, p.y()));
        painter.drawLine(QPointF(p.x(), p.y() - 7), QPointF(p.x(), p.y() + 7));
    }

    void mousePressEvent(QMouseEvent* event) override { moveTo(event->position()); }

    void mouseMoveEvent(QMouseEvent* event) override
    {
        if (event->buttons() & Qt::LeftButton) {
            moveTo(event->position());
        }
    }

private:
    void moveTo(const QPointF& pos)
    {
        x_ = qBound(0.0, pos.x() / width(), 1.0);
        y_ = qBound(0.0, pos.y() / height(), 1.0);
        update();
        if (moved) {
            moved(x_, y_);
        }
    }

    QImage image_;
    double x_ = 0.5;
    double y_ = 0.5;
};

QColor buttonColor(const QPushButton* button)
{
    return button->property("filterColor").value<QColor>();
}

// A colour parameter shows as a swatch, as CS6's does; the hex is only the tooltip.
void setButtonColor(QPushButton* button, const QColor& color)
{
    button->setProperty("filterColor", color);
    button->setToolTip(color.name());
    button->setStyleSheet(
        QStringLiteral("QPushButton { background-color: %1; border: 1px solid #000; }")
            .arg(color.name()));
}

} // namespace

FilterParamControls::FilterParamControls(const QList<FilterParamSpec>& params, QWidget* parent,
                                         const QImage& padImage)
    : QObject(parent), parent_(parent), padImage_(padImage)
{
    for (const FilterParamSpec& spec : params) {
        addControl(spec);
    }

    // Radial Blur: the Blur Method choice flips the display-only centre pad.
    BlurCenterWidget* blurCenter = nullptr;
    QComboBox* blurMethod = nullptr;
    for (const Control& control : controls_) {
        if (control.spec.control == FilterControl::BlurCenter) {
            blurCenter = static_cast<BlurCenterWidget*>(control.center);
        } else if (control.spec.control == FilterControl::Choice
                   && control.spec.label == QStringLiteral("Blur Method:")) {
            blurMethod = control.combo;
        }
    }
    if (blurCenter && blurMethod) {
        connect(blurMethod, &QComboBox::currentIndexChanged, blurCenter,
                [blurCenter](int index) { blurCenter->setZoom(index == 1); });
        blurCenter->setZoom(blurMethod->currentIndex() == 1);
    }
}

QList<QWidget*> FilterParamControls::rows() const
{
    QList<QWidget*> result;
    for (const Control& control : controls_) {
        result.append(control.row);
    }
    return result;
}

void FilterParamControls::addControl(const FilterParamSpec& spec)
{
    const QList<double>& initial = spec.initial;
    Control control;
    control.spec = spec;

    const auto fallback = [&initial](int i, double d) {
        return i < initial.size() ? initial.at(i) : d;
    };

    switch (spec.control) {
    case FilterControl::CheckBox: {
        control.box = new QCheckBox(spec.label, parent_);
        control.box->setChecked(fallback(0, spec.value) >= 0.5);
        connect(control.box, &QCheckBox::toggled, this, [this] { emit changed(); });
        break;
    }
    case FilterControl::Choice: {
        control.combo = new QComboBox(parent_);
        control.combo->addItems(spec.choices);
        control.combo->setCurrentIndex(static_cast<int>(fallback(0, spec.value)));
        connect(control.combo, &QComboBox::currentIndexChanged, this,
                [this] { emit changed(); });
        break;
    }
    case FilterControl::Color: {
        control.colorButton = new QPushButton(parent_);
        control.colorButton->setObjectName(QStringLiteral("filterColorSwatch"));
        control.colorButton->setFixedSize(24, 22);
        const QColor color(static_cast<int>(fallback(0, spec.initial.value(0, 0))),
                           static_cast<int>(fallback(1, spec.initial.value(1, 0))),
                           static_cast<int>(fallback(2, spec.initial.value(2, 0))));
        setButtonColor(control.colorButton, color);
        connect(control.colorButton, &QPushButton::clicked, this, [this, button = control.colorButton,
                                                                    spec] {
            QString title = spec.label;
            title.remove(QLatin1Char(':'));
            const QColor picked = ColorPickerDialog::getColor(buttonColor(button), parent_, title);
            if (picked.isValid()) {
                setButtonColor(button, picked);
                emit changed();
            }
        });
        break;
    }
    case FilterControl::Placement: {
        control.x = new QDoubleSpinBox(parent_);
        control.y = new QDoubleSpinBox(parent_);
        for (QDoubleSpinBox* spin : {control.x, control.y}) {
            spin->setRange(spec.minimum, spec.maximum);
            spin->setDecimals(3);
            spin->setSingleStep(0.01);
            spin->setAlignment(Qt::AlignLeft | Qt::AlignVCenter);
            spin->setFixedWidth(compactSpinWidth(spin));
        }
        control.x->setValue(fallback(0, spec.initial.value(0, 0.5)));
        control.y->setValue(fallback(1, spec.initial.value(1, 0.5)));
        auto* pad = new PlacementPad(parent_);
        pad->setImage(padImage_);
        pad->setPosition(control.x->value(), control.y->value());
        control.pad = pad;
        connect(control.x, &QDoubleSpinBox::valueChanged, this,
                [this, pad, y = control.y](double value) {
                    pad->setPosition(value, y->value());
                    emit changed();
                });
        connect(control.y, &QDoubleSpinBox::valueChanged, this,
                [this, pad, x = control.x](double value) {
                    pad->setPosition(x->value(), value);
                    emit changed();
                });
        pad->moved = [this, x = control.x, y = control.y](double px, double py) {
            QSignalBlocker blockX(x);
            QSignalBlocker blockY(y);
            x->setValue(px);
            y->setValue(py);
            emit changed();
        };
        break;
    }
    case FilterControl::BlurCenter: {
        control.center = new BlurCenterWidget(parent_);
        break;
    }
    default: {
        control.spin = new QDoubleSpinBox(parent_);
        control.spin->setRange(spec.minimum, spec.maximum);
        control.spin->setDecimals(spec.decimals);
        control.spin->setAlignment(Qt::AlignLeft | Qt::AlignVCenter);
        control.spin->setFixedWidth(compactSpinWidth(control.spin));
        control.spin->setValue(fallback(0, spec.value));
        control.slider = new JumpSlider(Qt::Horizontal, parent_);
        control.slider->setRange(0, 1000);
        const double span = spec.maximum - spec.minimum;
        const double frac = span > 0.0 ? (control.spin->value() - spec.minimum) / span : 0.0;
        control.slider->setValue(static_cast<int>(frac * 1000.0));
        connect(control.slider, &QSlider::sliderPressed, this,
                [this] { sliderDragging_ = true; });
        connect(control.slider, &QSlider::sliderReleased, this, [this] {
            sliderDragging_ = false;
            emit changed();
        });
        // Dragging moves the number but holds the canvas preview until release.
        connect(control.slider, &QSlider::valueChanged, this,
                [this, spin = control.spin, spec](int value) {
                    const double f = static_cast<double>(value) / 1000.0;
                    QSignalBlocker block(spin);
                    spin->setValue(spec.minimum + f * (spec.maximum - spec.minimum));
                    if (!sliderDragging_) {
                        emit changed();
                    }
                });
        connect(control.spin, &QDoubleSpinBox::valueChanged, this, [this, slider = control.slider,
                                                                    spec](double value) {
            const double span = spec.maximum - spec.minimum;
            const double f = span > 0.0 ? (value - spec.minimum) / span : 0.0;
            QSignalBlocker block(slider);
            slider->setValue(static_cast<int>(f * 1000.0));
            emit changed();
        });
        break;
    }
    }

    // Each parameter is a vertical row: its label and value on one line, and
    // (for a slider) the track beneath, starting flush with the label.
    control.row = new QWidget(parent_);
    auto* rowLayout = new QVBoxLayout(control.row);
    rowLayout->setContentsMargins(0, 0, 0, 0);
    rowLayout->setSpacing(2);
    switch (spec.control) {
    case FilterControl::CheckBox:
        rowLayout->addWidget(control.box);
        break;
    case FilterControl::Choice: {
        auto* line = new QHBoxLayout;
        line->addWidget(new QLabel(spec.label, control.row));
        line->addStretch(1);
        line->addWidget(control.combo);
        rowLayout->addLayout(line);
        break;
    }
    case FilterControl::Color: {
        auto* line = new QHBoxLayout;
        line->addWidget(new QLabel(spec.label, control.row));
        line->addStretch(1);
        line->addWidget(control.colorButton);
        rowLayout->addLayout(line);
        break;
    }
    case FilterControl::Placement: {
        rowLayout->addWidget(new QLabel(spec.label, control.row));
        auto* line = new QHBoxLayout;
        line->addWidget(control.pad, 1);
        line->addWidget(new QLabel(QStringLiteral("X"), control.row));
        line->addWidget(control.x);
        line->addWidget(new QLabel(QStringLiteral("Y"), control.row));
        line->addWidget(control.y);
        rowLayout->addLayout(line);
        break;
    }
    case FilterControl::BlurCenter:
        rowLayout->addWidget(new QLabel(spec.label, control.row));
        rowLayout->addWidget(control.center);
        break;
    default: {
        auto* line = new QHBoxLayout;
        line->addWidget(new QLabel(spec.label, control.row));
        line->addStretch(1);
        line->addWidget(control.spin);
        const QString unit = spec.suffix.trimmed();
        if (!unit.isEmpty()) {
            auto* unitLabel = new QLabel(unit, control.row);
            unitLabel->setObjectName(QStringLiteral("filterUnit"));
            line->addWidget(unitLabel);
        }
        rowLayout->addLayout(line);
        rowLayout->addWidget(control.slider);
        break;
    }
    }
    controls_.append(control);
}

void FilterParamControls::setValues(const QList<double>& initial)
{
    const QSignalBlocker block(this);
    int slot = 0;
    for (Control& control : controls_) {
        switch (control.spec.control) {
        case FilterControl::CheckBox:
            control.box->setChecked(slot < initial.size() ? initial.at(slot) >= 0.5
                                                          : control.box->isChecked());
            break;
        case FilterControl::Choice:
            control.combo->setCurrentIndex(slot < initial.size()
                                               ? static_cast<int>(initial.at(slot))
                                               : control.combo->currentIndex());
            break;
        case FilterControl::Color: {
            const double r = slot < initial.size() ? initial.at(slot) : buttonColor(control.colorButton).red();
            const double g = slot + 1 < initial.size() ? initial.at(slot + 1) : buttonColor(control.colorButton).green();
            const double b = slot + 2 < initial.size() ? initial.at(slot + 2) : buttonColor(control.colorButton).blue();
            setButtonColor(control.colorButton,
                           QColor(static_cast<int>(r), static_cast<int>(g), static_cast<int>(b)));
            break;
        }
        case FilterControl::Placement:
            if (slot < initial.size()) {
                control.x->setValue(initial.at(slot));
            }
            if (slot + 1 < initial.size()) {
                control.y->setValue(initial.at(slot + 1));
            }
            if (control.pad) {
                static_cast<PlacementPad*>(control.pad)->setPosition(control.x->value(),
                                                                     control.y->value());
            }
            break;
        case FilterControl::BlurCenter:
            break;
        default:
            if (slot < initial.size()) {
                control.spin->setValue(initial.at(slot));
            }
            break;
        }
        slot += slotCount(control.spec.control);
    }
}

double FilterParamControls::controlValue(const Control& control, int slot) const
{
    switch (control.spec.control) {
    case FilterControl::CheckBox:
        return control.box->isChecked() ? 1.0 : 0.0;
    case FilterControl::Choice:
        return static_cast<double>(control.combo->currentIndex());
    case FilterControl::Color: {
        const QColor color = buttonColor(control.colorButton);
        return slot == 0 ? color.red() : slot == 1 ? color.green() : color.blue();
    }
    case FilterControl::Placement:
        return slot == 0 ? control.x->value() : control.y->value();
    case FilterControl::BlurCenter:
        return 0.0;
    default:
        return control.spin->value();
    }
}

QList<double> FilterParamControls::values() const
{
    QList<double> result;
    for (const Control& control : controls_) {
        const int n = slotCount(control.spec.control);
        for (int s = 0; s < n; ++s) {
            result.append(controlValue(control, s));
        }
    }
    return result;
}


} // namespace pictura
