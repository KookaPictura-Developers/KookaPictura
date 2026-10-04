// ImageView's channel visibility (the Channels panel's eyes): the mask, and
// the greyscale image a single visible channel draws as.

#include "image_view.h"

namespace pictura {

void ImageView::setChannelMask(int mask)
{
    mask &= 0x7;
    if (mask == channelMask_) {
        return;
    }
    channelMask_ = mask;
    update();
}

int ImageView::singleChannel() const
{
    switch (channelMask_) {
    case 0x1:
        return 0;
    case 0x2:
        return 1;
    case 0x4:
        return 2;
    default:
        return -1;
    }
}

const QImage& ImageView::channelImage()
{
    const int channel = singleChannel();
    if (channelImageKey_ == image_.cacheKey() && channelImageMask_ == channelMask_) {
        return channelImage_;
    }
    QImage source = image_.convertToFormat(QImage::Format_ARGB32);
    for (int y = 0; y < source.height(); ++y) {
        auto* row = reinterpret_cast<QRgb*>(source.scanLine(y));
        for (int x = 0; x < source.width(); ++x) {
            const QRgb px = row[x];
            const int v = channel == 0 ? qRed(px) : channel == 1 ? qGreen(px) : qBlue(px);
            row[x] = qRgba(v, v, v, qAlpha(px));
        }
    }
    channelImage_ = source;
    channelImageKey_ = image_.cacheKey();
    channelImageMask_ = channelMask_;
    return channelImage_;
}

} // namespace pictura
