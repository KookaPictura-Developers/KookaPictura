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
#include <QtGui/QResizeEvent>
#include <QtGui/QScreen>
#include <QtGui/QShowEvent>
#include <QtWidgets/QApplication>
#include <QtWidgets/QBoxLayout>
#include <QtWidgets/QFrame>
#include <QtWidgets/QGraphicsOpacityEffect>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QMainWindow>
#include <QtWidgets/QMenu>
#include <QtWidgets/QScrollArea>
#include <QtWidgets/QScrollBar>
#include <QtWidgets/QSizeGrip>
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
    setMinimumWidth(kFloatMinWidth);
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(1, 1, 1, 1);
    layout->setSpacing(0);
    // M47: one shared effect dims the whole overlay while it hovers a valid drop
    // target; `setWindowOpacity` is a no-op on a child widget.
    opacityEffect_ = new QGraphicsOpacityEffect(this);
    opacityEffect_->setOpacity(1.0);
    setGraphicsEffect(opacityEffect_);
    // M47: the overlay is resizable like a docked column; the grip rides the
    // bottom-right corner and never makes this a top-level window.
    sizeGrip_ = new QSizeGrip(this);
    sizeGrip_->setObjectName(QStringLiteral("panelFloatSizeGrip"));
    sizeGrip_->setToolTip(tr("Resize"));
}

void PanelFloat::setContent(QWidget* content)
{
    if (!content) {
        return;
    }
    content_ = content;
    if (auto* layout = qobject_cast<QBoxLayout*>(this->layout())) {
        layout->addWidget(content);
    }
}

void PanelFloat::setGroup(PanelGroup* group)
{
    group_ = group;
    if (group_) {
        setContent(group_);
        group_->setFloating(true);
        if (QToolButton* close = group_->floatCloseButton()) {
            QObject::connect(close, &QToolButton::clicked, this, [this]() {
                if (onClose) {
                    onClose();
                }
            });
        }
        QObject::connect(group_, &PanelGroup::collapsedToIconsChanged, this,
                         [this](bool) { syncToContent(); });
    }
}

void PanelFloat::syncToContent()
{
    if (!group_) {
        return;
    }
    if (group_->isCollapsedToIcons()) {
        setMinimumHeight(kFloatIconMinHeight);
        // M47: snap to the icon row rather than only growing, so collapsing a
        // tall expanded float actually shrinks the overlay.
        const int target = qMax(group_->sizeHint().height(), kFloatIconMinHeight);
        resize(width(), target);
        return;
    }
    // Expanded: the overlay keeps a top-bar + tab-bar floor and grows to fit the
    // group.
    setMinimumHeight(kFloatMinHeight);
    const int target = group_->sizeHint().height();
    if (target > height()) {
        resize(width(), target);
    }
}

void PanelFloat::setDragDimmed(bool dimmed)
{
    if (opacityEffect_) {
        opacityEffect_->setOpacity(dimmed ? 0.6 : 1.0);
    }
}

qreal PanelFloat::dragOpacityForTest() const
{
    return opacityEffect_ ? opacityEffect_->opacity() : 1.0;
}

void PanelFloat::showTabIndicator(PanelGroup* group, int index)
{
    QTabBar* bar = group ? group->tabBar() : nullptr;
    if (!bar) {
        return;
    }
    if (!indicator_) {
        // Same `#2a7fff` mark as the docked column's indicator, owned by the
        // float because the docked indicator cannot reach outside its viewport.
        indicator_ = new QWidget(this);
        indicator_->setObjectName(QStringLiteral("panelFloatDropIndicator"));
        indicator_->setAttribute(Qt::WA_TransparentForMouseEvents);
        indicator_->setStyleSheet(QStringLiteral("background-color:#2a7fff;"));
    }
    const int x = group->tabInsertionX(index);
    const QPoint origin = bar->mapTo(this, QPoint(x, 0));
    indicator_->setGeometry(QRect(origin.x() - 1, origin.y(), 3, bar->height()));
    indicator_->show();
    indicator_->raise();
}

void PanelFloat::hideTabIndicator()
{
    if (indicator_) {
        indicator_->hide();
    }
}

void PanelFloat::resizeEvent(QResizeEvent* event)
{
    QWidget::resizeEvent(event);
    if (!sizeGrip_) {
        return;
    }
    sizeGrip_->resize(sizeGrip_->sizeHint());
    sizeGrip_->move(width() - sizeGrip_->width() - 2, height() - sizeGrip_->height() - 2);
    sizeGrip_->raise();
}

} // namespace pictura
