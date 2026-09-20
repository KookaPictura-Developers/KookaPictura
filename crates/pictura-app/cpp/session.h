#pragma once

#include <QtCore/QByteArray>
#include <QtCore/QJsonArray>
#include <QtCore/QString>
#include <QtCore/QStringList>

namespace pictura {

// Opaque UI session state persisted across restarts. Not document data.
struct SessionState {
    QByteArray layout;               // QMainWindow::saveState()
    int layoutRevision = 0;          // chrome revision that wrote `layout`; a
                                     // mismatch discards the layout on restore
    int brightnessLevel = 1;         // Theme level
    bool gpuCompute = true;          // GPU compositing preference
    int layersThumbSize = 2;         // 0 None / 1 Small / 2 Medium / 3 Large
    int layersThumbContents = 0;     // 0 Entire Document / 1 Layer Bounds
    bool layersExpandNewEffects = true;
    int toolsColumns = 1;            // 1 or 2; out-of-range loads the default
    bool useShiftKeyForToolSwitch = true;
    // v5-v8 panel-column state. `panelGroups` is the legacy flat JSON array of
    // {name, order, visible, minimized, collapsed} for the primary column;
    // `panelColumns` is the ordered array of
    // {side, order, width, railMode, groups:[...]}. Both are kept opaque here so
    // an older or newer store round-trips unchanged.
    QString panelRailMode = QStringLiteral("normal");
    int railWidth = 0;               // legacy primary width seed
    bool autoCollapseIconic = false;
    bool autoShowHidden = false;
    QJsonArray panelGroups;
    QJsonArray panelColumns;
    int schemaVersion = 8;
    QStringList recent;
};

// Path of the session store, $XDG_STATE_HOME/kooka-pictura/state.json.
QString sessionFilePath();

// Load the session state, or defaults when the store is missing or corrupt.
SessionState loadSession();

// Write the session state atomically via QSaveFile. Returns false on failure.
bool saveSession(const SessionState& state);

} // namespace pictura
