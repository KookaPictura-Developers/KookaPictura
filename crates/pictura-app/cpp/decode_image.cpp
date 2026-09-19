#include "decode_image.h"

#include <QtGui/QImage>

::rust::Vec<uint8_t> decode_image_rgba(::rust::Slice<const uint8_t> data, int32_t& width,
                                       int32_t& height)
{
    width = 0;
    height = 0;
    const QImage decoded = QImage::fromData(data.data(), static_cast<int>(data.size()));
    if (decoded.isNull()) {
        return {};
    }
    const QImage rgba = decoded.convertToFormat(QImage::Format_RGBA8888);
    if (rgba.isNull()) {
        return {};
    }
    width = rgba.width();
    height = rgba.height();
    const int stride = width * 4;
    ::rust::Vec<uint8_t> out;
    out.reserve(static_cast<size_t>(stride) * static_cast<size_t>(height));
    for (int y = 0; y < height; ++y) {
        const uchar* line = rgba.constScanLine(y);
        for (int i = 0; i < stride; ++i) {
            out.push_back(line[i]);
        }
    }
    return out;
}
