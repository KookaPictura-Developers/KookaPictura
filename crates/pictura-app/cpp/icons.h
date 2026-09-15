#pragma once

#include <QtCore/QString>
#include <QtGui/QCursor>
#include <QtGui/QIcon>

namespace pictura {

// Asset id -> resource lookup. Ids are file names without the extension:
// command ids verbatim ("file.saveAs") and tools/cursors as "tool.<tool>".
QIcon icon(const QString& id);
QCursor cursor(const QString& id);

} // namespace pictura
