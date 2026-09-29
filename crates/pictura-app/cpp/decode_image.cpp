#include "decode_image.h"

#include <QtGui/QImage>
#include <QtGui/QImageReader>

::rust::Vec<uint8_t> decode_image_rgba(::rust::Slice<const uint8_t> data, int32_t& width,
                                       int32_t& height)
{
    width = 0;
    height = 0;
    // Qt's QImageReader refuses any allocation over its 256 MB default, which
    // would reject a 16000²-class raster (a 16507×16196 RGBA decode is ~1020 MB)
    // before the app's own budget sees it. The import budget
    // (pictura_codec::ImageBudget) is the authoritative cap, so raise Qt's limit
    // to cover the largest allowed decode (a 30 000 px RGB raster is ~2.7 GB).
    // Static and idempotent; read when the reader is constructed below.
    QImageReader::setAllocationLimit(4096);
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
