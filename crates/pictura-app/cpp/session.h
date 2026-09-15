#pragma once

#include <QtCore/QByteArray>
#include <QtCore/QString>
#include <QtCore/QStringList>

namespace pictura {

// Opaque UI session state persisted across restarts. Not document data.
struct SessionState {
    QByteArray layout;               // QMainWindow::saveState()
    int brightnessLevel = 1;         // Theme level
    int schemaVersion = 1;
    QStringList recent;
};

// Path of the session store, $XDG_STATE_HOME/kooka-pictura/state.json.
QString sessionFilePath();

// Load the session state, or defaults when the store is missing or corrupt.
SessionState loadSession();

// Write the session state atomically via QSaveFile. Returns false on failure.
bool saveSession(const SessionState& state);

} // namespace pictura
