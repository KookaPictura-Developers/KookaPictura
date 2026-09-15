#include "panel_rail.h"

#include <QtCore/QSize>
#include <QtCore/QVariant>
#include <QtGui/QAction>

namespace pictura {

PanelRail::PanelRail(QWidget* parent)
    : QToolBar(parent)
{
    setObjectName(QStringLiteral("panelRail"));
    setOrientation(Qt::Vertical);
    setMovable(false);
    setIconSize(QSize(20, 20));
    setToolButtonStyle(Qt::ToolButtonIconOnly);
}

void PanelRail::addPanel(const QString& commandId, const QIcon& icon, const QString& tooltip)
{
    auto* action = new QAction(icon, tooltip, this);
    action->setCheckable(true);
    action->setToolTip(tooltip);
    action->setData(commandId);
    addAction(action);
    connect(action, &QAction::triggered, this,
            [this, commandId]() { emit commandTriggered(commandId); });
}

void PanelRail::setPanelChecked(const QString& commandId, bool checked)
{
    for (QAction* action : actions()) {
        if (action->data().toString() == commandId) {
            action->setChecked(checked);
            return;
        }
    }
}

} // namespace pictura
