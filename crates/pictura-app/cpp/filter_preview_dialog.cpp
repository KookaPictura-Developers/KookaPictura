#include "filter_preview_dialog.h"
#include "filter_param_controls.h"
#include "dialogs.h"
#include "panels/jump_slider.h"

#include <QtCore/QSignalBlocker>
#include <QtCore/QTimer>
#include <QtCore/QVariant>
#include <QtGui/QColor>
#include <QtGui/QFontMetrics>
#include <QtGui/QIcon>
#include <QtGui/QImage>
#include <QtGui/QMouseEvent>
#include <QtGui/QPainter>
#include <QtGui/QPen>
#include <QtGui/QPixmap>
#include <QtGui/QResizeEvent>
#include <QtGui/QShowEvent>
#include <QtMath>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDoubleSpinBox>
#include <QtWidgets/QFrame>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSizePolicy>
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

// More parameter rows than this spill into a second input column, matching CS6's
// 3-column dialogs for the input-heavy filters (Smart Sharpen, Wave, ...).
const int kManyParams = 8;

// A stacked dialog's rows span this width, wider than the preview alone.
const int kStackedRowWidth = 360;

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

    // CS6's columns: preview and the first inputs on the left, any overflow
    // inputs in the middle, OK / Cancel / Preview on the right.
    auto* body = new QHBoxLayout;
    body->setSpacing(12);

    auto* leftColumn = new QVBoxLayout;
    leftColumn->setSpacing(8);
    if (spec.previewPane) {
        thumbnail_ = new QLabel(this);
        thumbnail_->setObjectName(QStringLiteral("filterThumbnail"));
        thumbnail_->setMinimumSize(200, 200);
        thumbnail_->setAlignment(Qt::AlignCenter);
        thumbnail_->setFrameShape(QFrame::StyledPanel);
        // Ignored: the pixmap must not drive the layout back through sizeHint;
        // the label just fills its column and the image is re-rendered to fit.
        thumbnail_->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Ignored);
        // Stretch 1: extra dialog height grows the preview, not a blank gap.
        leftColumn->addWidget(thumbnail_, 1);

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
        leftColumn->addLayout(zoomRow);
    }
    body->addLayout(leftColumn, 1);

    // One self-contained row per parameter, built by addControl; a filter with
    // many inputs spills the overflow into the middle column.
    controls_ = new FilterParamControls(spec_.params, this, view_ ? view_->image() : QImage());
    connect(controls_, &FilterParamControls::changed, this, &FilterPreviewDialog::valuesChanged);
    QList<QWidget*> rows = controls_->rows();
    if (spec.stacked) {
        // Checkboxes trail the column so the sliders run on unbroken.
        QList<QWidget*> checks;
        for (int i = rows.size() - 1; i >= 0; --i) {
            if (spec.params.at(i).control == FilterControl::CheckBox) {
                checks.prepend(rows.takeAt(i));
            }
        }
        rows.append(checks);
        for (QWidget* row : rows) {
            row->setMinimumWidth(kStackedRowWidth);
        }
    }
    QVBoxLayout* middleColumn = nullptr;
    const int split = rows.size() > kManyParams && !spec.stacked ? (rows.size() + 1) / 2
                                                                 : rows.size();
    for (int i = 0; i < rows.size(); ++i) {
        QVBoxLayout* column = leftColumn;
        if (i >= split) {
            if (!middleColumn) {
                middleColumn = new QVBoxLayout;
                middleColumn->setSpacing(8);
                body->addLayout(middleColumn, 1);
            }
            column = middleColumn;
        }
        column->addWidget(rows.at(i));
    }
    if (!spec.previewPane) {
        leftColumn->addStretch(1);
    }
    if (middleColumn) {
        middleColumn->addStretch(1);
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
    body->addLayout(buttonColumn);
    outer->addLayout(body);

    connect(this, &QDialog::rejected, this, &FilterPreviewDialog::discardPreview);
}

void FilterPreviewDialog::applyInitial(const QList<double>& initial)
{
    controls_->setValues(initial);
    updateThumbnail();
}

QList<double> FilterPreviewDialog::values() const
{
    return controls_->values();
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
    const QSize target = thumbnail_->size().expandedTo(thumbnail_->minimumSize());
    thumbnail_->setPixmap(
        QPixmap::fromImage(section).scaled(target, Qt::KeepAspectRatio, Qt::SmoothTransformation));
}

void FilterPreviewDialog::resizeEvent(QResizeEvent* event)
{
    QDialog::resizeEvent(event);
    // The preview pane grows with the dialog; re-render it at the new size.
    updateThumbnail();
}

void FilterPreviewDialog::showEvent(QShowEvent* event)
{
    QDialog::showEvent(event);
    // Preview on open, as CS6 does. Deferred a turn so the dialog paints before
    // a slow filter runs.
    if (!shownOnce_) {
        shownOnce_ = true;
        QTimer::singleShot(0, this, [this] {
            if (isVisible()) {
                valuesChanged();
            }
        });
    }
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
