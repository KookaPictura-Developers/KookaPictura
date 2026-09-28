#include "file_drop_router.h"

#include "frame.h"
#include "image_view.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QEvent>
#include <QtCore/QFileInfo>
#include <QtCore/QMimeData>
#include <QtCore/QUrl>
#include <QtGui/QDragEnterEvent>
#include <QtGui/QDragMoveEvent>
#include <QtGui/QDropEvent>

namespace pictura {

FileDropRouter::FileDropRouter(PicturaMainWindow* frame)
    : QObject(frame), frame_(frame)
{
}

QStringList FileDropRouter::localPaths(const QMimeData* mime)
{
    QStringList paths;
    if (!mime) {
        return paths;
    }
    for (const QUrl& url : mime->urls()) {
        if (!url.isLocalFile()) {
            continue;
        }
        const QString path = url.toLocalFile();
        if (QFileInfo(path).isFile()) {
            paths.append(path);
        }
    }
    return paths;
}

bool FileDropRouter::eventFilter(QObject* watched, QEvent* event)
{
    if (event->type() == QEvent::DragEnter || event->type() == QEvent::DragMove) {
        auto* drag = static_cast<QDragMoveEvent*>(event);
        if (localPaths(drag->mimeData()).isEmpty()) {
            return false;
        }
        drag->acceptProposedAction();
        return true;
    }
    if (event->type() != QEvent::Drop) {
        return QObject::eventFilter(watched, event);
    }

    auto* drop = static_cast<QDropEvent*>(event);
    const QStringList paths = localPaths(drop->mimeData());
    if (paths.isEmpty()) {
        return false;
    }

    PictureView* view = frame_->activeView();
    const bool place = qobject_cast<ImageView*>(watched) != nullptr && view != nullptr;
    bool changed = false;
    QString lastPlaced;
    if (place) {
        for (const QString& path : paths) {
            const QString layer = PicturaMainWindow::isNativeDocumentPath(path)
                ? view->place_smart_object(path)
                : view->place_image(path);
            if (!layer.isEmpty()) {
                lastPlaced = layer;
            }
            changed = changed || !layer.isEmpty();
        }
    } else {
        for (const QString& path : paths) {
            const bool opened = frame_->openDocumentAtPath(path);
            changed = changed || opened;
        }
    }
    if (changed) {
        if (place && !lastPlaced.isEmpty()) {
            frame_->beginFreeTransform(lastPlaced);
            frame_->selectLayerPath(lastPlaced);
        }
        frame_->refresh();
    }
    drop->acceptProposedAction();
    return true;
}

} // namespace pictura
