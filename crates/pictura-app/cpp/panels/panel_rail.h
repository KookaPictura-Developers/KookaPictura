#pragma once

#include <QtCore/QString>
#include <QtGui/QIcon>
#include <QtWidgets/QToolBar>

namespace pictura {

// Narrow vertical icon rail whose buttons toggle panel docks. Each entry
// carries a command id so the rail and the Window menu share one toggle path.
class PanelRail : public QToolBar {
    Q_OBJECT

public:
    explicit PanelRail(QWidget* parent = nullptr);

    // Adds a checkable, icon-only button. `commandId` is emitted when toggled.
    void addPanel(const QString& commandId, const QIcon& icon, const QString& tooltip);
    void setPanelChecked(const QString& commandId, bool checked);

signals:
    void commandTriggered(const QString& commandId);
};

} // namespace pictura
