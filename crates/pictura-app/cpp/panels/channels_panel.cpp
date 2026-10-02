#include "channels_panel.h"

#include "icons.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtGui/QImage>

#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

namespace {

constexpr int kThumbnailSize = 24;

const QStringList& channelNames()
{
    static const QStringList names = {QStringLiteral("RGB"), QStringLiteral("Red"),
                                      QStringLiteral("Green"), QStringLiteral("Blue")};
    return names;
}

QPixmap channelThumbnail(const QImage& source, int channel)
{
    if (source.isNull() || source.width() <= 0 || source.height() <= 0) {
        return QPixmap();
    }
    const QImage scaled =
        source.scaled(kThumbnailSize, kThumbnailSize, Qt::KeepAspectRatio, Qt::FastTransformation)
            .convertToFormat(QImage::Format_ARGB32);
    QImage gray(scaled.size(), QImage::Format_ARGB32);
    for (int y = 0; y < scaled.height(); ++y) {
        for (int x = 0; x < scaled.width(); ++x) {
            const QRgb px = scaled.pixel(x, y);
            int value = 0;
            switch (channel) {
            case 0:
                value = qGray(px);
                break;
            case 1:
                value = qRed(px);
                break;
            case 2:
                value = qGreen(px);
                break;
            default:
                value = qBlue(px);
                break;
            }
            gray.setPixel(x, y, qRgb(value, value, value));
        }
    }
    return QPixmap::fromImage(gray);
}

void applyEyeIcon(QToolButton* eye)
{
    eye->setIcon(icon(eye->isChecked() ? QStringLiteral("layers.eyeOn")
                                       : QStringLiteral("layers.eyeOff")));
}

} // namespace

ChannelsPanel::ChannelsPanel(QWidget* parent)
    : QWidget(parent)
{
    layout_ = new QVBoxLayout(this);
    layout_->addStretch(1);
    rebuildRows(channelNames());
    mode_ = QStringLiteral("rgb");
}

void ChannelsPanel::rebuildRows(const QStringList& names)
{
    for (const Row& row : rows_) {
        if (row.eye) {
            delete row.eye->parentWidget();
        }
    }
    rows_.clear();
    for (const QString& name : names) {
        auto* rowWidget = new QWidget(this);
        auto* rowLayout = new QHBoxLayout(rowWidget);
        rowLayout->setContentsMargins(0, 0, 0, 0);

        auto* eye = new QToolButton(rowWidget);
        eye->setCheckable(true);
        eye->setChecked(true);
        eye->setAutoRaise(true);
        applyEyeIcon(eye);
        connect(eye, &QToolButton::toggled, this, [eye]() { applyEyeIcon(eye); });

        auto* thumbnail = new QLabel(rowWidget);
        auto* label = new QLabel(name, rowWidget);

        rowLayout->addWidget(eye);
        rowLayout->addWidget(thumbnail);
        rowLayout->addWidget(label, 1);
        layout_->insertWidget(layout_->count() - 1, rowWidget);

        rows_.append(Row{name, eye, thumbnail});
    }
}

void ChannelsPanel::setView(PictureView* view)
{
    view_ = view;
    refresh();
}

void ChannelsPanel::refresh()
{
    // ponytail: presentational only. Toggling an eye does not change the canvas
    // (the model has no per-channel visibility and the compositor has no
    // channel-mask path). Add per-channel state and a mask path later.
    if (view_ && view_->has_document()) {
        const QString mode = view_->document_mode();
        if (mode != mode_) {
            rebuildRows(mode == QLatin1String("grayscale")
                            ? QStringList{QStringLiteral("Gray")}
                            : channelNames());
            mode_ = mode;
        }
    }

    QImage source;
    if (view_ && view_->has_document()) {
        // Thumbnails need only ~24 px, so take the coarsest pyramid level that
        // still resolves them rather than the full composite. The crop rect is
        // in level coordinates.
        const int levels = view_->display_level_count();
        int chosen = -1;
        int chosenWidth = 0;
        int chosenHeight = 0;
        for (int level = levels - 1; level >= 0; --level) {
            const QStringList parts =
                view_->display_level_size(level).split(QLatin1Char(' '), Qt::SkipEmptyParts);
            if (parts.size() != 2) {
                continue;
            }
            bool okWidth = false;
            bool okHeight = false;
            const int w = parts.at(0).toInt(&okWidth);
            const int h = parts.at(1).toInt(&okHeight);
            if (!okWidth || !okHeight || w <= 0 || h <= 0) {
                continue;
            }
            if (qMax(w, h) >= 64) {
                chosen = level;
                chosenWidth = w;
                chosenHeight = h;
                break;
            }
        }
        // ponytail: full-resolution fallback when the pyramid has no level
        // >=64 px (single-level or tiny document); add a smaller thumbnail
        // floor if the 24 px thumbs look soft on tiny docs.
        if (chosen >= 0) {
            source = view_->display_image(chosen, 0, 0, chosenWidth, chosenHeight);
        } else {
            source = view_->image();
        }
    }
    for (int i = 0; i < rows_.size(); ++i) {
        rows_[i].thumbnail->setPixmap(channelThumbnail(source, i));
    }
}

int ChannelsPanel::channelCountForTest() const { return rows_.size(); }

QString ChannelsPanel::channelNameForTest(int index) const
{
    return index >= 0 && index < rows_.size() ? rows_.at(index).name : QString();
}

QPixmap ChannelsPanel::channelThumbnailForTest(int index) const
{
    return index >= 0 && index < rows_.size() ? rows_.at(index).thumbnail->pixmap() : QPixmap();
}

bool ChannelsPanel::channelVisibleForTest(int index) const
{
    return index >= 0 && index < rows_.size() && rows_.at(index).eye->isChecked();
}

void ChannelsPanel::toggleChannelForTest(int index)
{
    if (index >= 0 && index < rows_.size()) {
        rows_[index].eye->toggle();
    }
}

} // namespace pictura
