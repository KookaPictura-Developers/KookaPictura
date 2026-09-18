#include "panel_column.h"

#include "panel_column_internal.h"

#include "frame.h"
#include "icons.h"
#include "panel_group.h"
#include "theme.h"

#include <QtCore/QEvent>
#include <QtCore/QJsonObject>
#include <QtCore/QMetaObject>
#include <QtCore/QRect>
#include <QtCore/QSize>
#include <QtGui/QAction>
#include <QtGui/QCursor>
#include <QtGui/QFontMetrics>
#include <QtGui/QGuiApplication>
#include <QtGui/QHideEvent>
#include <QtGui/QMouseEvent>
#include <QtGui/QPalette>
#include <QtGui/QScreen>
#include <QtGui/QShowEvent>
#include <QtWidgets/QApplication>
#include <QtWidgets/QBoxLayout>
#include <QtWidgets/QFrame>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QMainWindow>
#include <QtWidgets/QMenu>
#include <QtWidgets/QScrollArea>
#include <QtWidgets/QScrollBar>
#include <QtWidgets/QSplitter>
#include <QtWidgets/QTabBar>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

PanelFlyout::PanelFlyout(QWidget* parent)
    : QWidget(parent)
{
    setWindowFlags(Qt::Popup);
}

void PanelFlyout::hideEvent(QHideEvent* event)
{
    QWidget::hideEvent(event);
    if (onHidden) {
        onHidden();
    }
}

PanelFloat::PanelFloat(QWidget* parent)
    : QWidget(parent)
{
    setObjectName(QStringLiteral("panelFloat"));
    // ponytail: in-window overlay — a plain raised child, never a top-level
    // window, so it is clipped to the main window and stays out of the task
    // list. OS-window float chrome and multi-monitor float are non-goals.
    setWindowFlags(Qt::Widget);
    setAttribute(Qt::WA_StyledBackground, true);
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(1, 1, 1, 1);
    layout->setSpacing(0);
}

void PanelFloat::setGroup(PanelGroup* group)
{
    group_ = group;
    if (group_) {
        if (auto* layout = qobject_cast<QBoxLayout*>(this->layout())) {
            layout->addWidget(group_);
        }
        group_->setFloating(true);
        if (QToolButton* close = group_->floatCloseButton()) {
            QObject::connect(close, &QToolButton::clicked, this, [this]() {
                if (onClose) {
                    onClose();
                }
            });
        }
    }
}

} // namespace pictura
