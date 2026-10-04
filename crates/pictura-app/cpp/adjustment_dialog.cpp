#include "adjustment_dialog.h"

#include "panels/adjustment_controls.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/filter_tools.cxxqt.h"
#include "pictura_app/src/cxxqt_object/image_adjust.cxxqt.h"

#include <QtGui/QColor>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QDialogButtonBox>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

namespace {

::rust::Slice<const std::uint8_t> slice(const QByteArray& bytes)
{
    return {reinterpret_cast<const std::uint8_t*>(bytes.constData()), std::size_t(bytes.size())};
}

QByteArray bytes(const ::rust::Vec<std::uint8_t>& v)
{
    return QByteArray(reinterpret_cast<const char*>(v.data()), qsizetype(v.size()));
}

} // namespace

AdjustmentDialog::AdjustmentDialog(PictureView* view, const QByteArray& block,
                                   const QRect& visible, QWidget* parent)
    : QDialog(parent)
    , view_(view)
    , block_(block)
    , visible_(visible)
{
    setObjectName(QStringLiteral("adjustmentDialog"));
    auto* layout = new QVBoxLayout(this);
    controls_ = new AdjustmentControls(this);
    controls_->setCurveSource(
        [this](int channel) { return QString(image_adjustment_curve(slice(block_), channel)); });
    controls_->setPage(image_adjustment_page(slice(block_)));
    title_ = controls_->title();
    setWindowTitle(title_);
    layout->addWidget(controls_, 1);
    preview_ = new QCheckBox(QStringLiteral("Preview"), this);
    preview_->setObjectName(QStringLiteral("adjustmentPreview"));
    preview_->setChecked(true);
    layout->addWidget(preview_);
    auto* buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, this);
    connect(buttons, &QDialogButtonBox::accepted, this, &QDialog::accept);
    connect(buttons, &QDialogButtonBox::rejected, this, &QDialog::reject);
    layout->addWidget(buttons);

    connect(controls_, &AdjustmentControls::valueChanged, this,
            [this](const QString& key, double value) {
                edited(bytes(image_adjustment_set(slice(block_), key, value)));
            });
    connect(controls_, &AdjustmentControls::curveChanged, this,
            [this](int channel, const QString& points) {
                edited(bytes(image_adjustment_set_curve(slice(block_), channel, points)));
            });
    connect(preview_, &QCheckBox::toggled, this, [this](bool on) {
        if (on) {
            preview();
        } else {
            cancelPreview();
        }
    });
    preview();
}

AdjustmentDialog::~AdjustmentDialog()
{
    cancelPreview();
}

void AdjustmentDialog::edited(const QByteArray& next)
{
    // A refused edit (Levels' input black past its white) keeps the last
    // good block; reload the controls to match it.
    if (next.isEmpty()) {
        controls_->setPage(image_adjustment_page(slice(block_)));
        return;
    }
    block_ = next;
    if (preview_->isChecked()) {
        preview();
    }
}

void AdjustmentDialog::preview()
{
    if (view_) {
        previewing_ = image_adjust_preview(*view_, slice(block_), visible_.x(), visible_.y(),
                                           visible_.width(), visible_.height())
            || previewing_;
    }
}

void AdjustmentDialog::cancelPreview()
{
    if (view_ && previewing_) {
        filter_preview_cancel(*view_);
    }
    previewing_ = false;
}

void AdjustmentDialog::accept()
{
    // The commit re-applies to the whole layer from the pre-preview pixels.
    if (view_ && image_adjust_apply(*view_, slice(block_), title_)) {
        previewing_ = false;
    } else {
        cancelPreview();
    }
    QDialog::accept();
}

void AdjustmentDialog::reject()
{
    cancelPreview();
    QDialog::reject();
}

QWidget* AdjustmentDialog::controlForTest(const QString& key) const
{
    return controls_->controlForTest(key);
}

bool runAdjustmentDialog(QWidget* parent, PictureView* view, const QString& kind,
                         const QColor& foreground, const QColor& background, const QRect& visible)
{
    if (!view) {
        return false;
    }
    const QByteArray block = bytes(image_adjustment_default(
        kind, foreground.rgb() & 0xffffffu, background.rgb() & 0xffffffu));
    if (block.isEmpty()) {
        return false;
    }
    AdjustmentDialog dialog(view, block, visible, parent);
    return dialog.exec() == QDialog::Accepted;
}

} // namespace pictura
