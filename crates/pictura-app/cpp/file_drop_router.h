#pragma once

#include <QtCore/QObject>
#include <QtCore/QStringList>

class QMimeData;

namespace pictura {

class PicturaMainWindow;

// Routes an OS file drag over the frame's drop targets: a document ImageView
// places each file into the active document, every other watched target opens
// each file as a new tab. A drag with no local regular file is never consumed,
// so the Layers-panel internal drag and tab reordering keep their own handlers.
class FileDropRouter : public QObject {
    Q_OBJECT

public:
    explicit FileDropRouter(PicturaMainWindow* frame);

    bool eventFilter(QObject* watched, QEvent* event) override;

    // The local regular files carried by `mime`; directories and non-file URLs
    // are discarded and a URL-less drag yields an empty list.
    static QStringList localPaths(const QMimeData* mime);

private:
    PicturaMainWindow* frame_ = nullptr;
};

} // namespace pictura
