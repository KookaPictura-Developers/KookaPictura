#pragma once

#include <QtCore/QHash>
#include <QtCore/QJsonArray>
#include <QtCore/QList>
#include <QtCore/QPoint>
#include <QtCore/QRect>
#include <QtCore/QSet>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtGui/QIcon>
#include <QtWidgets/QWidget>

#include <functional>

class QBoxLayout;
class QHideEvent;
class QLabel;
class QMenu;
class QScrollArea;
class QShowEvent;
class QSplitter;
class QTabBar;
class QToolButton;

namespace pictura {

class PanelColumn;
class PanelGroup;

// A torn-off group in a frameless top-level `Qt::Tool` window. It hosts a real
// `PanelGroup` (same tabs, menu, minimize, iconic row) whose own tab-bar drag
// path routes back through the owning column, so it can be dragged back and
// re-docked. The column deletes the float once its group is empty or re-docked.
class PanelFloat : public QWidget {
    Q_OBJECT

public:
    explicit PanelFloat(QWidget* parent = nullptr);
    PanelGroup* group() const { return group_; }
    void setGroup(PanelGroup* group);

private:
    PanelGroup* group_ = nullptr;
};

// The frameless `Qt::Popup` that hosts a panel detached from its group. It
// reports its own hide so the column can put the panel back exactly once.
class PanelFlyout : public QWidget {
public:
    explicit PanelFlyout(QWidget* parent = nullptr);
    std::function<void()> onHidden;

protected:
    void hideEvent(QHideEvent* event) override;
};

// The right-hand panel host. A vertical QSplitter of PanelGroups inside a
// QScrollArea: group heights drag with the splitter handles, and a short window
// scrolls the column instead of forcing the main window to grow. There are no
// hard minimum widths or heights here. A thin header carries the
// `panelColumnToggle` that switches the whole column to the iconic icon strip.
class PanelColumn : public QWidget {
    Q_OBJECT

public:
    explicit PanelColumn(QWidget* parent = nullptr);

    void addGroup(PanelGroup* group);
    PanelGroup* groupForPanel(const QString& objectName) const;
    QList<PanelGroup*> groups() const { return groups_; }

    bool showPanel(const QString& objectName, bool visible);
    bool isPanelVisible(const QString& objectName) const;
    void closeGroup(PanelGroup* group);

    // `normal` (splitter of groups) <-> `iconic` (narrow icon strip).
    void setRailMode(bool iconic);
    bool railMode() const { return railMode_; }
    QToolButton* panelColumnToggle() const { return columnToggle_; }

    // v5 persisted state. `setAutoShowHidden(on)` also widens the iconic strip
    // to include the hidden panels' icons (the small M41 effect for the flag).
    void setAutoCollapseIconic(bool on);
    void setAutoShowHidden(bool on);
    bool autoCollapseIconic() const { return autoCollapseIconic_; }
    bool autoShowHidden() const { return autoShowHidden_; }
    void setPreferredWidth(int width);

    // Per-group order/visibility/minimized/collapsed as a compact JSON array;
    // round-trips exactly and skips groups whose stored name is gone.
    QJsonArray savePanelState() const;
    void restorePanelState(const QJsonArray& state);

    // Test hooks.
    QStringList groupTitlesForTest() const;
    QString groupOfForTest(const QString& objectName) const;
    bool scrollableForTest() const;
    int minimumWidthForTest() const;
    bool railModeForTest() const { return railMode_; }
    QToolButton* panelColumnToggleForTest() const { return columnToggle_; }
    bool iconStripVisibleForTest() const;
    int dividerCountForTest() const;
    bool iconLabelsShownForTest() const { return iconLabelsShown_; }
    bool openIconFlyoutForTest(const QString& objectName);
    bool iconFlyoutVisibleForTest() const;
    QStringList tabMenuActionsForTest() const;
    bool triggerTabMenuForTest(const QString& text);
    bool autoCollapseIconicForTest() const { return autoCollapseIconic_; }
    bool autoShowHiddenForTest() const { return autoShowHidden_; }

    // Phase C drag/drop. These hooks drive the same begin/update/commit path
    // the tab-bar mouse handler uses; they never synthesize OS drags.
    bool beginTabDragForTest(const QString& objectName);
    void dragToForTest(const QPoint& globalPos);
    bool dropForTest(const QPoint& globalPos);
    void cancelDragForTest();
    bool dropIndicatorVisibleForTest() const;
    QRect dropIndicatorGeometryForTest() const;
    int dropIndexForTest() const;
    int floatCountForTest() const;
    QStringList floatPanelNamesForTest(int index) const;
    bool tearOffForTest(const QString& groupName);
    bool redockForTest(int floatIndex, int boundaryIndex);
    QPoint boundaryPointForTest(int boundary) const;
    bool ensureGroupVisibleForTest(const QString& panelName);
signals:
    void interfaceOptionsRequested();
    void stateChanged();

protected:
    bool eventFilter(QObject* watched, QEvent* event) override;
    void showEvent(QShowEvent* event) override;

private:
    struct DropTarget {
        bool valid = false;
        bool outside = false;
        bool onTabBar = false;
        PanelGroup* group = nullptr;
        int tabIndex = -1;
        int boundary = -1;
    };

    void updateColumnToggle();
    void buildIconStrip();
    void clearIconStrip();
    void updateIconStripLabels();
    void ensureFlyout();
    void openIconFlyout(const QString& objectName, const QPoint& globalPos);
    void closeIconFlyout();
    void restoreFlyoutPanel();
    void showTabMenu(PanelGroup* group, const QPoint& globalPos);
    QMenu* buildTabMenu(PanelGroup* group);
    QToolButton* makeIconButton(QWidget* parent, const QString& objectName,
                                const QString& title, const QIcon& icon);

    void wireGroup(PanelGroup* group);
    void insertGroupAt(PanelGroup* group, int index);
    bool removeGroup(PanelGroup* group);
    void cleanupEmptyGroup(PanelGroup* group);
    DropTarget resolveDrop(const QPoint& globalPos) const;
    int boundaryIndexForGlobalY(const QPoint& globalPos) const;
    void showIndicatorFor(const DropTarget& target);
    void clearIndicator();
    QList<PanelGroup*> visibleGroups() const;
    bool applyPanelDrop(PanelGroup* source, const QString& name, const DropTarget& target);
    bool applyGroupDrop(PanelGroup* group, const DropTarget& target);
    void beginPanelDrag(PanelGroup* group, const QString& objectName, const QPoint& globalPos);
    void beginGroupDrag(PanelGroup* group, const QPoint& globalPos);
    void updateDrag(const QPoint& globalPos);
    bool commitDrop();
    void cancelDrag();
    PanelFloat* createFloat(PanelGroup* group, const QPoint& globalPos);
    void destroyFloat(PanelFloat* floatWindow);
    PanelFloat* floatForGroup(PanelGroup* group) const;
    PanelGroup* findGroupByName(const QString& objectName) const;

    QWidget* header_ = nullptr;
    QToolButton* columnToggle_ = nullptr;
    QScrollArea* scroll_ = nullptr;
    QSplitter* splitter_ = nullptr;
    QWidget* iconStrip_ = nullptr;
    QBoxLayout* iconStripLayout_ = nullptr;
    QList<QLabel*> stripLabels_;
    QList<PanelGroup*> groups_;
    QHash<QString, bool> panelVisible_;

    bool railMode_ = false;
    bool iconLabelsShown_ = false;
    int pendingWidth_ = 0;

    PanelFlyout* flyout_ = nullptr;
    QBoxLayout* flyoutLayout_ = nullptr;
    QWidget* flyoutPanel_ = nullptr;
    PanelGroup* flyoutGroup_ = nullptr;
    QString flyoutName_;
    bool restoringFlyout_ = false;

    bool autoCollapseIconic_ = false;
    bool autoShowHidden_ = false;
    PanelGroup* menuGroup_ = nullptr;

    QWidget* indicator_ = nullptr;
    QSet<PanelGroup*> wired_;
    QList<PanelFloat*> floats_;

    bool dragActive_ = false;
    bool dragIsPanel_ = false;
    PanelGroup* dragGroup_ = nullptr;
    QString dragPanel_;
    QPoint dragGrabOffset_;
    int dragOriginalIndex_ = -1;
    PanelFloat* dragFloat_ = nullptr;
    DropTarget dropTarget_;
};

} // namespace pictura
