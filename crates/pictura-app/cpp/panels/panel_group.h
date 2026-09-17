#pragma once

#include <QtCore/QList>
#include <QtCore/QPoint>
#include <QtCore/QRect>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtGui/QIcon>
#include <QtWidgets/QWidget>

class QBoxLayout;
class QMenu;
class QTabBar;
class QTabWidget;
class QToolButton;

namespace pictura {

// One CS6 panel group: a top-tab QTabWidget whose tab text is the panel title.
// There is deliberately no group-title label; the tabs are the group chrome.
// A group can roll up to its tab bar (`Minimize`) or shrink to a row of panel
// icons (`Collapse to Icons`); the iconic row reuses the column's flyout.
class PanelGroup : public QWidget {
    Q_OBJECT

public:
    explicit PanelGroup(QWidget* parent = nullptr);

    void addPanel(QWidget* panel, const QString& title, const QIcon& icon);
    QStringList titles() const;
    QStringList visibleTitles() const;
    // Reorder the tabs to match `order` (names not present are ignored).
    void setPanelOrder(const QStringList& order);
    bool isPanelVisible(const QString& objectName) const;
    QList<QWidget*> panels() const;
    QList<QWidget*> visiblePanels() const;
    bool containsPanel(const QString& objectName) const;
    bool removePanel(const QString& objectName);
    bool setPanelVisible(const QString& objectName, bool visible);
    QString currentPanelName() const;
    QString titleForPanel(const QString& objectName) const;
    QIcon iconForPanel(const QString& objectName) const;

    // Phase C drag/drop primitives. `takePanel` removes a tab and hands back
    // the widget plus its tab text/icon/index so a drop can re-parent it
    // elsewhere; `insertPanel` puts it back at an index. A panel is never in
    // two tab stacks: take first, then insert.
    QWidget* takePanel(const QString& objectName, QString* title = nullptr,
                       QIcon* icon = nullptr, int* index = nullptr);
    void insertPanel(QWidget* panel, const QString& title, const QIcon& icon, int index);
    QTabBar* tabBar() const;
    int indexOfPanel(const QString& objectName) const;
    // The width the per-widget corner `▾` button reserves in the header, so the
    // column minimum can include it (M43 corner-button fix).
    int headerCornerWidthForTest() const;

    // Tab-bar geometry used by the column's drop resolver. All coordinates are
    // global except `tabInsertionX`, which is in tab-bar space.
    QRect tabBarGlobalRect() const;
    int tabInsertionIndexAt(const QPoint& globalPos) const;
    int tabInsertionX(int index) const;
    QPoint tabInsertionGlobalPointForTest(int index) const;

    // Minimize keeps the tab bar and hides the content; collapse-to-icons
    // replaces the tabs with the panel icon row. These are distinct states.
    bool isMinimized() const { return minimized_; }
    void setMinimized(bool minimized);
    bool isCollapsedToIcons() const { return collapsedToIcons_; }
    void setCollapsedToIcons(bool collapsed);

    // Detach/attach the current panel for a `Qt::Popup` flyout. The panel is
    // removed from the tab stack while detached and reinserted at its index.
    QWidget* detachPanel(const QString& objectName);
    bool attachPanel(const QString& objectName);

    // M44: make the first visible panel current (a stored layout used to leave
    // the last visible panel active after `setPanelVisible` walked the tabs).
    void setCurrentToFirstVisible();

    // Phase B consumes this to seed iconic mode; Phase A only records it.
    bool defaultIconic() const { return defaultIconic_; }
    void setDefaultIconic(bool iconic) { defaultIconic_ = iconic; }

    // Test hooks.
    int tabPositionForTest() const;
    int titleCountForTest() const;
    QStringList titleTextsForTest() const;
    QIcon titleIconForTest(const QString& title) const;
    bool groupLabelForTest() const;
    bool isMinimizedForTest() const { return minimized_; }
    void setMinimizedForTest(bool minimized) { setMinimized(minimized); }
    bool isCollapsedToIconsForTest() const { return collapsedToIcons_; }
    void setCollapsedToIconsForTest(bool collapsed) { setCollapsedToIcons(collapsed); }
    bool contentHiddenForTest() const;
    bool tabBarVisibleForTest() const;
    bool iconRowVisibleForTest() const;
    // M44 default-active check: the current tab index and the first visible one.
    int currentTabIndexForTest() const;
    int firstVisibleTabIndexForTest() const;

    // Phase D: the per-widget header action button and its per-panel menu.
    QToolButton* headerMenuButtonForTest() const { return headerButton_; }
    bool headerMenuAtRightForTest() const;
    QStringList panelMenuTextsForTest() const;
    bool panelMenuEnabledForTest(const QString& text) const;
    QString panelMenuToolTipForTest(const QString& text) const;
    bool triggerPanelMenuForTest(const QString& text);
    static bool panelHasMenu(const QString& panelName);
    static QStringList menuTextsForPanel(const QString& panelName);

signals:
    void panelActivated(const QString& objectName, const QPoint& globalPos);
    void tabContextMenuRequested(const QPoint& globalPos);
    // Phase C: a press on a tab that moves past the drag threshold. A press on
    // the empty part of the tab bar is a whole-group drag (empty objectName).
    void tabDragStarted(const QString& objectName, const QPoint& globalPos);
    void groupDragStarted(const QPoint& globalPos);
    void dragMoved(const QPoint& globalPos);
    void dragFinished(const QPoint& globalPos);
    void dragCanceled();

protected:
    bool eventFilter(QObject* watched, QEvent* event) override;

private:
    void rebuildIconRow();
    void applyMinimize();
    void updateHeaderMenu();
    void runPanelMenuAction(const QString& actionId);
    QToolButton* makeIconButton(const QIcon& icon, const QString& title,
                                const QString& objectName);

    QTabWidget* tabs_ = nullptr;
    QWidget* iconRow_ = nullptr;
    QBoxLayout* iconRowLayout_ = nullptr;
    QToolButton* headerButton_ = nullptr;
    QMenu* headerMenu_ = nullptr;
    bool defaultIconic_ = false;
    bool collapsedToIcons_ = false;
    bool minimized_ = false;
    int savedMaxHeight_ = QWIDGETSIZE_MAX;

    bool pressPending_ = false;
    bool dragging_ = false;
    QPoint pressGlobal_;
    QString pressedPanel_;

    struct Detached {
        QWidget* panel = nullptr;
        QString title;
        QIcon icon;
        int index = 0;
    } detached_;
};

} // namespace pictura
