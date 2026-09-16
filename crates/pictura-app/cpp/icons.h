#pragma once

#include <QtCore/QString>
#include <QtGui/QCursor>
#include <QtGui/QIcon>

namespace pictura {

// Asset id -> resource lookup. Ids are file names without the extension:
// command ids verbatim ("file.saveAs") and tools/cursors as "tool.<tool>".
QIcon icon(const QString& id);
// Renders the cursor SVG at the given hotspot in the 24x24 cursor space.
QCursor cursor(const QString& id, int hotX, int hotY);
// Compatibility overload: centres the hotspot at (12, 12).
QCursor cursor(const QString& id);

} // namespace pictura
