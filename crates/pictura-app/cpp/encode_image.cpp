#include "encode_image.h"

#include <QtCore/QByteArray>
#include <QtCore/QSaveFile>
#include <QtCore/QString>
#include <QtCore/QtGlobal>
#include <QtGui/QImage>
#include <QtGui/QImageWriter>
#include <QtGui/QPainter>

#include <algorithm>

namespace {

QString toQString(::rust::Str text)
{
    return QString::fromUtf8(text.data(), static_cast<int>(text.size()));
}

} // namespace

bool encode_image_rgba(::rust::Slice<const uint8_t> rgba, int32_t width, int32_t height,
                       ::rust::Str path, ::rust::Str format, int32_t quality, int32_t scale)
{
    if (width <= 0 || height <= 0) {
        return false;
    }
    if (rgba.size() < static_cast<size_t>(width) * static_cast<size_t>(height) * 4u) {
        return false;
    }

    const QImage source(rgba.data(), width, height, width * 4, QImage::Format_RGBA8888);

    QImage image = source;
    if (scale > 0 && scale != 100) {
        const int scaledWidth = std::max(1, qRound(width * scale / 100.0));
        const int scaledHeight = std::max(1, qRound(height * scale / 100.0));
        image = source.scaled(scaledWidth, scaledHeight, Qt::KeepAspectRatio, Qt::SmoothTransformation);
    }

    const QString fmt = toQString(format).toUpper();
    if (fmt == QLatin1String("JPG") || fmt == QLatin1String("JPEG")
        || fmt == QLatin1String("BMP")) {
        // JPEG has no alpha, and Qt's BMP writer drops it too: composite the
        // transparent pixels onto opaque white so the result is deterministic.
        QImage flat(image.width(), image.height(), QImage::Format_RGB32);
        flat.fill(Qt::white);
        QPainter painter(&flat);
        painter.drawImage(0, 0, image);
        painter.end();
        image = flat;
    }

    const QByteArray formatBytes = fmt.toUtf8();
    const int qualityOrDefault = (quality > 0 && quality <= 100) ? quality : -1;
    // QSaveFile writes to a temp file and renames on commit, so a mid-write
    // failure never truncates an existing file -- the same atomicity the PSD
    // codec path gets from tmp+rename.
    QSaveFile file(toQString(path));
    if (!file.open(QIODevice::WriteOnly)) {
        return false;
    }
    QImageWriter writer(&file, formatBytes);
    writer.setQuality(qualityOrDefault);
    if (!writer.write(image)) {
        file.cancelWriting();
        return false;
    }
    return file.commit();
}
