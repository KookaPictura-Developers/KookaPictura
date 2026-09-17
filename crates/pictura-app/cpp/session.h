#pragma once

#include <QtCore/QByteArray>
#include <QtCore/QString>
#include <QtCore/QStringList>

namespace pictura {

// Opaque UI session state persisted across restarts. Not document data.
struct SessionState {
    QByteArray layout;               // QMainWindow::saveState()
    int brightnessLevel = 1;         // Theme level
    bool gpuCompute = true;          // GPU compositing preference
    int layersThumbSize = 2;         // 0 None / 1 Small / 2 Medium / 3 Large
    int layersThumbContents = 0;     // 0 Entire Document / 1 Layer Bounds
    bool layersExpandNewEffects = true;
    int toolsColumns = 1;            // 1 or 2; out-of-range loads the default
    bool useShiftKeyForToolSwitch = true;
    int schemaVersion = 4;
    QStringList recent;
};

// Path of the session store, $XDG_STATE_HOME/kooka-pictura/state.json.
QString sessionFilePath();

// Load the session state, or defaults when the store is missing or corrupt.
SessionState loadSession();

// Write the session state atomically via QSaveFile. Returns false on failure.
bool saveSession(const SessionState& state);

} // namespace pictura
