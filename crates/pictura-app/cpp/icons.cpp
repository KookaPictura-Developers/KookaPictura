#include "icons.h"

#include <QtCore/QFile>
#include <QtCore/QString>
#include <QtGui/QCursor>
#include <QtGui/QIcon>
#include <QtGui/QPainter>
#include <QtGui/QPixmap>
#include <QtSvg/QSvgRenderer>
#include <QtWidgets/QApplication>

namespace pictura {

QIcon icon(const QString& id)
{
    const QString path = QStringLiteral(":/icons/") + id + QStringLiteral(".svg");
    if (!QFile::exists(path)) {
        return QIcon();
    }
    return QIcon(path);
}

QCursor cursor(const QString& id)
{
    const QString path = QStringLiteral(":/cursors/") + id + QStringLiteral(".svg");
    if (!QFile::exists(path)) {
        return QCursor();
    }

    QSvgRenderer renderer(path);
    if (!renderer.isValid()) {
        return QCursor();
    }

    // ponytail: single-DPR render; add a multi-resolution pixmap set if HiDPI
    // cursors look soft.
    const qreal dpr = qApp ? qApp->devicePixelRatio() : 1.0;
    const int size = qRound(24 * dpr);
    QPixmap pixmap(size, size);
    pixmap.fill(Qt::transparent);
    QPainter painter(&pixmap);
    renderer.render(&painter);
    painter.end();
    pixmap.setDevicePixelRatio(dpr);

    const bool eyedropper = id == QStringLiteral("tool.eyedropper");
    const qreal hotX = eyedropper ? 2 : 12;
    const qreal hotY = eyedropper ? 22 : 12;
    return QCursor(pixmap, qRound(hotX * dpr), qRound(hotY * dpr));
}

} // namespace pictura
