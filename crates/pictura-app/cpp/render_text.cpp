#include "render_text.h"

#include <QtCore/QRect>
#include <QtCore/QtGlobal>
#include <QtGui/QColor>
#include <QtGui/QFont>
#include <QtGui/QImage>
#include <QtGui/QPainter>

#include <algorithm>

::rust::Vec<uint8_t> render_text_rgba(::rust::Str family, double pixel_size, ::rust::Str text,
                                      int32_t justify, uint8_t r, uint8_t g, uint8_t b, uint8_t a,
                                      int32_t width, int32_t height)
{
    if (width <= 0 || height <= 0) {
        return {};
    }
    QImage img(width, height, QImage::Format_RGBA8888_Premultiplied);
    img.fill(Qt::transparent);

    QFont font(QString::fromUtf8(family.data(), static_cast<int>(family.size())));
    font.setPixelSize(std::max(1, qRound(pixel_size)));

    QPainter p(&img);
    p.setRenderHint(QPainter::Antialiasing);
    p.setRenderHint(QPainter::TextAntialiasing);
    p.setPen(QColor(r, g, b, a));
    p.setFont(font);

    int align = Qt::AlignLeft;
    if (justify == 1) {
        align = Qt::AlignRight;
    } else if (justify == 2) {
        align = Qt::AlignHCenter;
    }
    align |= Qt::AlignTop | Qt::TextWordWrap;

    QString t = QString::fromUtf8(text.data(), static_cast<int>(text.size()));
    t.replace(QChar('\r'), QChar('\n'));
    p.drawText(QRect(0, 0, width, height), align, t);
    p.end();

    const QImage rgba = img.convertToFormat(QImage::Format_RGBA8888);
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
