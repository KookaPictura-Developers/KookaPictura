#pragma once

#include <QtCore/QJsonArray>
#include <QtCore/QList>
#include <QtCore/QString>

namespace pictura {

// One named panel arrangement. `layout` is the remembered `panelColumns` array;
// `factory` is the creation snapshot for a user workspace (empty for built-ins).
struct WorkspaceRecord {
    QString id;
    QString name;
    QString kind; // "builtin" | "user"
    QJsonArray layout;
    QJsonArray factory;
    bool hasLayout = false;
};

// Versioned store of named workspaces and the active workspace, independent of
// the session state store.
struct WorkspaceStore {
    int schemaVersion = 1;
    QString activeWorkspace;
    QList<WorkspaceRecord> workspaces;
};

// $XDG_STATE_HOME/kooka-pictura/workspaces, beside the session store.
QString workspaceStoreDir();

// Read the store. Returns false (leaving *out empty) when the index is missing
// or unparseable; a single bad workspace file is skipped, not fatal.
bool loadWorkspaceStore(WorkspaceStore* out);

// Write the store atomically, merging unknown keys already on disk.
bool saveWorkspaceStore(const WorkspaceStore& store);

} // namespace pictura
