#include "filter_preview_dialog.h"
#include "dialogs.h"

#include <QtCore/QSignalBlocker>
#include <QtCore/QVariant>
#include <QtGui/QColor>
#include <QtGui/QIcon>
#include <QtGui/QImage>
#include <QtGui/QMouseEvent>
#include <QtGui/QPainter>
#include <QtGui/QPen>
#include <QtGui/QPixmap>
#include <QtMath>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QColorDialog>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDoubleSpinBox>
#include <QtWidgets/QFormLayout>
#include <QtWidgets/QFrame>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QScrollArea>
#include <QtWidgets/QSlider>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QWidget>

#include <functional>
#include <limits>

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/filter_tools.cxxqt.h"

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

// CS6 preview zoom steps, as a percentage of the base thumbnail size.
const int kZoomLevels[] = {25, 50, 100, 200, 400};
const int kZoomCount = 5;

// A magnifier zoom button; falls back to a plain +/- glyph on a theme that has
// no zoom icons.
QToolButton* makeZoomButton(bool zoomIn, QWidget* parent)
{
    auto* button = new QToolButton(parent);
    const QIcon icon =
        QIcon::fromTheme(zoomIn ? QStringLiteral("zoom-in") : QStringLiteral("zoom-out"));
    if (!icon.isNull()) {
        button->setIcon(icon);
    } else {
        button->setText(zoomIn ? QStringLiteral("+") : QStringLiteral("\u2212"));
        button->setToolButtonStyle(Qt::ToolButtonTextOnly);
    }
    button->setAutoRaise(true);
    return button;
}

} // namespace

FilterPreviewDialog::FilterPreviewDialog(PictureView* view, const FilterCommandSpec& spec,
                                         const FilterPreviewView& previewView, QWidget* parent)
    : QDialog(parent), view_(view), spec_(spec)
{
    setWindowTitle(spec.label);
    previewVisible_ = previewView.visible;
    canvasZoom_ = previewView.canvasZoom;
    // Seed the preview zoom from the canvas zoom, snapped to a preview level.
    const double percent = canvasZoom_ * 100.0;
    int best = 2;
    double bestDelta = std::numeric_limits<double>::max();
    for (int i = 0; i < kZoomCount; ++i) {
        const double delta = qAbs(kZoomLevels[i] - percent);
        if (delta < bestDelta) {
            bestDelta = delta;
            best = i;
        }
    }
    zoom_ = best;

    auto* outer = new QVBoxLayout(this);

    // Top: the preview pane (when the filter has one) on the left, the
    // OK / Cancel / Preview column on the right, matching CS6.
    auto* top = new QHBoxLayout;
    if (spec.previewPane) {
        auto* previewColumn = new QVBoxLayout;
        thumbnail_ = new QLabel(this);
        thumbnail_->setObjectName(QStringLiteral("filterThumbnail"));
        thumbnail_->setFixedSize(200, 200);
        thumbnail_->setAlignment(Qt::AlignCenter);
        thumbnail_->setFrameShape(QFrame::StyledPanel);
        previewColumn->addWidget(thumbnail_, 0, Qt::AlignTop);

        auto* zoomRow = new QHBoxLayout;
        auto* zoomOut = makeZoomButton(false, this);
        zoomOut->setObjectName(QStringLiteral("filterZoomOut"));
        auto* zoomIn = makeZoomButton(true, this);
        zoomIn->setObjectName(QStringLiteral("filterZoomIn"));
        zoomLabel_ = new QLabel(this);
        zoomLabel_->setObjectName(QStringLiteral("filterZoomLabel"));
        zoomLabel_->setAlignment(Qt::AlignCenter);
        connect(zoomIn, &QToolButton::clicked, this, [this] {
            zoom_ = qMin(zoom_ + 1, kZoomCount - 1);
            updateThumbnail();
        });
        connect(zoomOut, &QToolButton::clicked, this, [this] {
            zoom_ = qMax(zoom_ - 1, 0);
            updateThumbnail();
        });
        zoomRow->addStretch(1);
        zoomRow->addWidget(zoomOut);
        zoomRow->addWidget(zoomLabel_);
        zoomRow->addWidget(zoomIn);
        zoomRow->addStretch(1);
        previewColumn->addLayout(zoomRow);
        previewColumn->addStretch(1);
        top->addLayout(previewColumn, 1);
    } else {
        top->addStretch(1);
    }

    auto* buttonColumn = new QVBoxLayout;
    auto* ok = new QPushButton(QStringLiteral("OK"), this);
    ok->setDefault(true);
    auto* cancel = new QPushButton(QStringLiteral("Cancel"), this);
    connect(ok, &QPushButton::clicked, this, &QDialog::accept);
    connect(cancel, &QPushButton::clicked, this, &QDialog::reject);
    preview_ = new QCheckBox(QStringLiteral("Preview"), this);
    preview_->setObjectName(QStringLiteral("filterPreview"));
    preview_->setChecked(true);
    connect(preview_, &QCheckBox::toggled, this, [this](bool on) {
        if (on) {
            valuesChanged();
        } else {
            discardPreview();
        }
    });
    buttonColumn->addWidget(ok);
    buttonColumn->addWidget(cancel);
    buttonColumn->addSpacing(6);
    buttonColumn->addWidget(preview_);
    buttonColumn->addStretch(1);
    top->addLayout(buttonColumn);
    outer->addLayout(top);

    auto* formWidget = new QWidget(this);
    auto* form = new QFormLayout(formWidget);
    for (int i = 0; i < spec_.params.size(); ++i) {
        addControl(spec_.params.at(i).initial, i);
    }
    // addControl appended into controls_; render each row into the form.
    for (const Control& control : controls_) {
        switch (control.spec.control) {
        case FilterControl::CheckBox:
            form->addRow(QString(), control.box);
            break;
        case FilterControl::Choice:
            form->addRow(control.spec.label, control.combo);
            break;
        case FilterControl::Color:
            form->addRow(control.spec.label, control.colorButton);
            break;
        case FilterControl::Placement: {
            auto* row = new QWidget(this);
            auto* layout = new QHBoxLayout(row);
            layout->setContentsMargins(0, 0, 0, 0);
            layout->addWidget(control.pad, 1);
            layout->addWidget(new QLabel(QStringLiteral("X"), row));
            layout->addWidget(control.x);
            layout->addWidget(new QLabel(QStringLiteral("Y"), row));
            layout->addWidget(control.y);
            form->addRow(control.spec.label, row);
            break;
        }
        case FilterControl::BlurCenter:
            form->addRow(control.spec.label, control.center);
            break;
        default: {
            // CS6 puts the value box on the label line and the slider beneath.
            form->addRow(control.spec.label, control.spin);
            form->addRow(QString(), control.slider);
            break;
        }
        }
    }
    auto* scroll = new QScrollArea(this);
    scroll->setWidgetResizable(true);
    scroll->setFrameShape(QFrame::NoFrame);
    scroll->setWidget(formWidget);
    scroll->setMaximumHeight(600);
    outer->addWidget(scroll, 1);

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

    connect(this, &QDialog::rejected, this, &FilterPreviewDialog::discardPreview);
}

void FilterPreviewDialog::addControl(const QList<double>& initial, int index)
{
    const FilterParamSpec& spec = spec_.params.at(index);
    Control control;
    control.spec = spec;

    const auto fallback = [&initial](int i, double d) {
        return i < initial.size() ? initial.at(i) : d;
    };

    switch (spec.control) {
    case FilterControl::CheckBox: {
        control.box = new QCheckBox(spec.label, this);
        control.box->setChecked(fallback(0, spec.value) >= 0.5);
        connect(control.box, &QCheckBox::toggled, this, [this] { valuesChanged(); });
        break;
    }
    case FilterControl::Choice: {
        control.combo = new QComboBox(this);
        control.combo->addItems(spec.choices);
        control.combo->setCurrentIndex(static_cast<int>(fallback(0, spec.value)));
        connect(control.combo, &QComboBox::currentIndexChanged, this,
                [this] { valuesChanged(); });
        break;
    }
    case FilterControl::Color: {
        control.colorButton = new QPushButton(this);
        const QColor color(static_cast<int>(fallback(0, spec.initial.value(0, 0))),
                           static_cast<int>(fallback(1, spec.initial.value(1, 0))),
                           static_cast<int>(fallback(2, spec.initial.value(2, 0))));
        control.colorButton->setProperty("filterColor", color);
        control.colorButton->setText(color.name());
        connect(control.colorButton, &QPushButton::clicked, this, [this, button = control.colorButton,
                                                                    spec] {
            const QColor picked =
                QColorDialog::getColor(buttonColor(button), this, spec.label);
            if (picked.isValid()) {
                button->setProperty("filterColor", picked);
                button->setText(picked.name());
                valuesChanged();
            }
        });
        break;
    }
    case FilterControl::Placement: {
        control.x = new QDoubleSpinBox(this);
        control.y = new QDoubleSpinBox(this);
        for (QDoubleSpinBox* spin : {control.x, control.y}) {
            spin->setRange(spec.minimum, spec.maximum);
            spin->setDecimals(3);
            spin->setSingleStep(0.01);
        }
        control.x->setValue(fallback(0, spec.initial.value(0, 0.5)));
        control.y->setValue(fallback(1, spec.initial.value(1, 0.5)));
        auto* pad = new PlacementPad(this);
        if (view_) {
            pad->setImage(view_->image());
        }
        pad->setPosition(control.x->value(), control.y->value());
        control.pad = pad;
        connect(control.x, &QDoubleSpinBox::valueChanged, this,
                [this, pad, y = control.y](double value) {
                    pad->setPosition(value, y->value());
                    valuesChanged();
                });
        connect(control.y, &QDoubleSpinBox::valueChanged, this,
                [this, pad, x = control.x](double value) {
                    pad->setPosition(x->value(), value);
                    valuesChanged();
                });
        pad->moved = [this, x = control.x, y = control.y](double px, double py) {
            QSignalBlocker blockX(x);
            QSignalBlocker blockY(y);
            x->setValue(px);
            y->setValue(py);
            valuesChanged();
        };
        break;
    }
    case FilterControl::BlurCenter: {
        control.center = new BlurCenterWidget(this);
        break;
    }
    default: {
        control.spin = new QDoubleSpinBox(this);
        control.spin->setRange(spec.minimum, spec.maximum);
        control.spin->setDecimals(spec.decimals);
        control.spin->setSuffix(spec.suffix);
        control.spin->setValue(fallback(0, spec.value));
        control.slider = new QSlider(Qt::Horizontal, this);
        control.slider->setRange(0, 1000);
        const double span = spec.maximum - spec.minimum;
        const double frac = span > 0.0 ? (control.spin->value() - spec.minimum) / span : 0.0;
        control.slider->setValue(static_cast<int>(frac * 1000.0));
        connect(control.slider, &QSlider::valueChanged, this, [this, spin = control.spin, spec](
                                                                 int value) {
            const double f = static_cast<double>(value) / 1000.0;
            spin->setValue(spec.minimum + f * (spec.maximum - spec.minimum));
        });
        connect(control.spin, &QDoubleSpinBox::valueChanged, this, [this, slider = control.slider,
                                                                    spec](double value) {
            const double span = spec.maximum - spec.minimum;
            const double f = span > 0.0 ? (value - spec.minimum) / span : 0.0;
            QSignalBlocker block(slider);
            slider->setValue(static_cast<int>(f * 1000.0));
            valuesChanged();
        });
        break;
    }
    }
    controls_.append(control);
}

void FilterPreviewDialog::applyInitial(const QList<double>& initial)
{
    initial_ = initial;
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
            const QColor color(static_cast<int>(r), static_cast<int>(g), static_cast<int>(b));
            control.colorButton->setProperty("filterColor", color);
            control.colorButton->setText(color.name());
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
    updateThumbnail();
}

double FilterPreviewDialog::controlValue(const Control& control, int slot) const
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

QList<double> FilterPreviewDialog::values() const
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

void FilterPreviewDialog::valuesChanged()
{
    if (preview_ && !preview_->isChecked()) {
        return;
    }
    if (!view_ || !view_->has_document()) {
        return;
    }
    const QList<double> current = values();
    // Restrict the preview to the visible section when the caller supplied one;
    // the commit still filters the whole layer.
    const bool shown =
        previewVisible_.isEmpty()
            ? filter_preview(*view_, spec_.kind, current)
            : filter_preview_section(*view_, spec_.kind, current,
                                     static_cast<int>(previewVisible_.left()),
                                     static_cast<int>(previewVisible_.top()),
                                     static_cast<int>(previewVisible_.width()),
                                     static_cast<int>(previewVisible_.height()));
    if (shown) {
        previewShown_ = true;
        updateThumbnail();
    }
}

void FilterPreviewDialog::discardPreview()
{
    if (previewShown_ && view_) {
        filter_preview_cancel(*view_);
        previewShown_ = false;
    }
}

void FilterPreviewDialog::updateThumbnail()
{
    const int level = kZoomLevels[qBound(0, zoom_, kZoomCount - 1)];
    if (zoomLabel_) {
        zoomLabel_->setText(QStringLiteral("%1%").arg(level));
    }
    if (!thumbnail_ || !view_ || !view_->has_document()) {
        return;
    }
    const QImage image = view_->image();
    if (image.isNull()) {
        return;
    }
    // Preview the current canvas section, not the whole image: at 100% the pane
    // shows 200 document pixels 1:1; a higher zoom shows fewer pixels larger.
    const double shown = 200.0 * 100.0 / static_cast<double>(level);
    const QPointF center = previewVisible_.isNull()
                               ? QPointF(image.width() / 2.0, image.height() / 2.0)
                               : previewVisible_.center();
    QRectF crop(center.x() - shown / 2.0, center.y() - shown / 2.0, shown, shown);
    crop = crop.intersected(QRectF(0, 0, image.width(), image.height()));
    if (crop.isEmpty()) {
        crop = QRectF(0, 0, image.width(), image.height());
    }
    const QImage section = image.copy(crop.toRect());
    thumbnail_->setPixmap(QPixmap::fromImage(section)
                              .scaled(200, 200, Qt::KeepAspectRatio, Qt::SmoothTransformation));
}

bool FilterPreviewDialog::get(PictureView* view, const FilterCommandSpec& spec,
                              const FilterPreviewView& previewView, const QList<double>& initial,
                              QList<double>* out, QWidget* parent)
{
    FilterPreviewDialog dialog(view, spec, previewView, parent);
    dialog.applyInitial(initial);
    if (runDialog(dialog, parent) != QDialog::Accepted) {
        return false;
    }
    *out = dialog.values();
    return true;
}

} // namespace pictura
