#include "toolbox.h"

#include "tools.h"

#include <QtGui/QAction>
#include <QtGui/QActionGroup>
#include <QtGui/QKeySequence>
#include <QtWidgets/QToolBar>

namespace pictura {

Toolbox::Toolbox(ToolController* controller, QWidget* parent)
    : QDockWidget(QStringLiteral("Tools"), parent)
{
    setObjectName(QStringLiteral("toolsPanel"));

    auto* bar = new QToolBar(this);
    bar->setOrientation(Qt::Vertical);
    bar->setMovable(false);

    auto* group = new QActionGroup(this);
    group->setExclusive(true);

    for (ToolId id : allToolIds()) {
        const ToolInfo& info = toolInfo(id);
        auto* action = bar->addAction(QString::fromLatin1(info.label));
        action->setCheckable(true);
        action->setChecked(controller && controller->activeTool() == id);
        action->setShortcut(QKeySequence(QString(info.shortcut)));
        action->setToolTip(QStringLiteral("%1 (%2) — %3")
                               .arg(QString::fromLatin1(info.label), QString(info.shortcut),
                                    QString::fromLatin1(info.hint)));
        group->addAction(action);

        if (controller) {
            connect(action, &QAction::triggered, controller,
                    [controller, id]() { controller->setActiveTool(id); });
            connect(controller, &ToolController::activeToolChanged, action,
                    [action, id](ToolId active) { action->setChecked(active == id); });
        }
    }

    setWidget(bar);
}

} // namespace pictura
