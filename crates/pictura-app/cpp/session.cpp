#include "session.h"

#include <QtCore/QByteArray>
#include <QtCore/QDir>
#include <QtCore/QFile>
#include <QtCore/QFileInfo>
#include <QtCore/QJsonArray>
#include <QtCore/QJsonDocument>
#include <QtCore/QJsonObject>
#include <QtCore/QJsonParseError>
#include <QtCore/QSaveFile>

namespace pictura {

// ponytail: stopgap for the deferred typed pictura-prefs / prefs.toml store
// (XC-002); keeps the C++ session file so M16 needs no new dependency.
QString sessionFilePath()
{
    const QString base = qEnvironmentVariable("XDG_STATE_HOME");
    const QString root =
        base.isEmpty() ? QDir::homePath() + QStringLiteral("/.local/state") : base;
    return root + QStringLiteral("/kooka-pictura/state.json");
}

SessionState loadSession()
{
    SessionState state;

    QFile file(sessionFilePath());
    if (!file.open(QIODevice::ReadOnly)) {
        return state;
    }

    QJsonParseError error{};
    const QJsonDocument doc = QJsonDocument::fromJson(file.readAll(), &error);
    if (error.error != QJsonParseError::NoError || !doc.isObject()) {
        return SessionState{};
    }

    const QJsonObject obj = doc.object();
    state.schemaVersion = obj.value(QStringLiteral("schemaVersion")).toInt(1);
    state.brightnessLevel = obj.value(QStringLiteral("brightnessLevel")).toInt(1);
    state.layout =
        QByteArray::fromBase64(obj.value(QStringLiteral("layout")).toString().toLatin1());
    const QJsonArray recent = obj.value(QStringLiteral("recent")).toArray();
    for (const QJsonValue& entry : recent) {
        if (entry.isString()) {
            state.recent.append(entry.toString());
        }
    }
    return state;
}

bool saveSession(const SessionState& state)
{
    const QString path = sessionFilePath();
    if (!QDir().mkpath(QFileInfo(path).absolutePath())) {
        return false;
    }

    QJsonObject obj;
    obj.insert(QStringLiteral("schemaVersion"), state.schemaVersion);
    obj.insert(QStringLiteral("brightnessLevel"), state.brightnessLevel);
    obj.insert(QStringLiteral("layout"), QString::fromLatin1(state.layout.toBase64()));
    QJsonArray recent;
    for (const QString& path : state.recent) {
        recent.append(path);
    }
    obj.insert(QStringLiteral("recent"), recent);

    QSaveFile file(path);
    if (!file.open(QIODevice::WriteOnly)) {
        return false;
    }
    file.write(QJsonDocument(obj).toJson(QJsonDocument::Compact));
    return file.commit();
}

} // namespace pictura
