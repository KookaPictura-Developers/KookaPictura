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
class PicturaMainWindow;

// A panel column's side is a property of its order in the central splitter
// relative to the document tabs, never of its on-screen geometry (M43).
enum class PanelSide { Left, Right };

// A torn-off group in an in-window frameless overlay. It hosts a real
// `PanelGroup` (same tabs, menu, minimize, iconic row) whose own tab-bar drag
// path routes back through the owning column, so it can be dragged back and
// re-docked. It is a child of the main window (never a top-level `Qt::Tool`),
// so it is clipped to the window and never appears in the task list. The column
// deletes the overlay once its group is empty or re-docked.
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

    // M43 multi-column host. `side` is derived from the splitter order relative
    // to the document tabs; `dynamic` marks a column created by a drop, which is
    // removed when it holds no groups.
    PanelSide side() const;
    bool isDynamic() const { return dynamic_; }
    void setDynamic(bool dynamic) { dynamic_ = dynamic; }

    bool showPanel(const QString& objectName, bool visible);
    bool isPanelVisible(const QString& objectName) const;
    void closeGroup(PanelGroup* group);

    // M43 session v6: detach a group so it can be adopted by another column
    // (used to rebuild a stored multi-column layout at startup). Returns the
    // live group, unwired from this column, or nullptr when unknown.
    PanelGroup* takeGroup(const QString& groupObjectName);

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
    // M44 chrome checks.
    int groupDividerWidthForTest() const;
    QString stripLabelTextForTest(const QString& objectName) const;
    bool stripLabelVisibleForTest(const QString& objectName) const;
    void setIconStripWidthForTest(int width);
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
    bool tearOffPanelForTest(const QString& objectName);
    bool redockForTest(int floatIndex, int boundaryIndex);
    bool floatIsWindowForTest(int index) const;
    QRect floatGeometryForTest(int index) const;
    QRect floatHostRectForTest() const;
    bool floatClampedForTest(int index, const QPoint& globalTopLeft);
    QPoint boundaryPointForTest(int boundary) const;
    bool ensureGroupVisibleForTest(const QString& panelName);
    // M43 Phase B: begin a whole-group drag and a compact-strip entry point.
    bool beginGroupDragForTest(const QString& panelName);
    QPoint stripEntryPointForTest(const QString& panelName, int where) const;

    // Phase B iconic-strip drag. These drive the same begin/update/commit path
    // the strip button's mouse handler uses.
    QStringList stripOrderForTest() const;
    bool beginStripDragForTest(const QString& objectName);
    QPoint stripInsertionPointForTest(int index) const;
    int stripDropIndexForTest() const;
    bool dropStripOnGroupForTest(const QString& objectName, const QString& targetPanel);

    // Phase C flyout refinement.
    QString flyoutSideForTest() const;
    QString activeIconNameForTest() const { return activeIconName_; }
    QRect iconFlyoutGeometryForTest() const;
    QString flyoutHeaderTitleForTest() const;
    bool flyoutHeaderCloseForTest() const;
    bool triggerFlyoutCloseForTest();
    // M44 C1/S2: the flyout shares the docked widget's scoped style/container.
    bool popupStyleParityForTest(const QString& objectName) const;

    // M44 Phase C: whole-widget-column docking and compact drop.
    bool dragActiveForTest() const { return dragActive_; }
    bool dragSourceGroupAliveForTest() const;
    QWidget* dragHandleForTest(const QString& groupName) const;
    QStringList compactStripGroupOrderForTest() const;
    int compactStripGroupIndexForTest(const QString& panelName) const;
    QPoint compactStripBoundaryPointForTest(int boundary) const;
    bool beginGroupGripDragForTest(const QString& groupName);

    // Phase D per-widget header action menu.
    QToolButton* widgetMenuButtonForTest(const QString& groupObjectName) const;
    QStringList widgetMenuTextsForTest(const QString& panelName) const;
    bool widgetMenuHasCloseForTest(const QString& panelName) const;
    bool triggerWidgetMenuForTest(const QString& panelName, const QString& text);
signals:
    void interfaceOptionsRequested();
    void stateChanged();

protected:
    bool eventFilter(QObject* watched, QEvent* event) override;
    void showEvent(QShowEvent* event) override;

private:
    // The frozen M43 drop-target grammar. `onTabBar`/`onStrip`/`outside` are
    // kept alongside `kind` so the M41/M42 hooks keep their meaning.
    enum class DropKind {
        None,
        Reorder,
        IntoGroup,
        AboveGroup,
        BelowGroup,
        OnStrip,
        NewColumnLeft,
        NewColumnRight,
        Outside,
    };

    struct DropTarget {
        bool valid = false;
        bool outside = false;
        bool onTabBar = false;
        bool onStrip = false;
        PanelGroup* group = nullptr;
        int tabIndex = -1;
        int boundary = -1;
        int stripIndex = -1;
        DropKind kind = DropKind::None;
        // M44 W5: a whole-column drop beside another column creates the new
        // column adjacent to this anchor instead of at the workspace end.
        PanelColumn* anchorColumn = nullptr;
    };

    // One icon in the iconic strip, in strip order. `button` is the drag source
    // and the anchor for the strip insertion maths.
    struct StripEntry {
        PanelGroup* group = nullptr;
        QString name;
        QToolButton* button = nullptr;
    };

    // M44 C3: the compact strip renders one container per group (a drag-handle
    // grip above its icons) with a dark divider between containers. The divider
    // and container geometry drive the group-relative proximity rule.
    struct StripDivider {
        PanelGroup* after = nullptr;
        QWidget* widget = nullptr;
    };

    void updateColumnToggle();
    void updateMinimumWidth();
    void buildIconStrip();
    void clearIconStrip();
    void updateIconStripLabels();
    void ensureFlyout();
    void openIconFlyout(const QString& objectName, const QPoint& globalPos);
    void closeIconFlyout();
    void restoreFlyoutPanel();
    void placeFlyout(const QRect& buttonGlobalRect, const QSize& size);
    QString flyoutSide() const;
    QToolButton* flyoutButtonFor(const QString& objectName) const;
    void setActiveIcon(const QString& objectName);
    QToolButton* stripButtonFor(const QString& objectName) const;
    void showTabMenu(PanelGroup* group, const QPoint& globalPos);
    QMenu* buildTabMenu(PanelGroup* group);
    QToolButton* makeIconButton(QWidget* parent, const QString& objectName,
                                const QString& title, const QIcon& icon);

    void wireGroup(PanelGroup* group);
    void insertGroupAt(PanelGroup* group, int index);
    bool removeGroup(PanelGroup* group);
    void cleanupEmptyGroup(PanelGroup* group);
    DropTarget resolveDrop(const QPoint& globalPos) const;
    DropTarget resolveLocalDrop(const QPoint& globalPos) const;
    bool resolveIconicDrop(const QPoint& globalPos, DropTarget& target) const;
    int boundaryIndexForGlobalY(const QPoint& globalPos) const;
    int stripInsertionIndexAt(const QPoint& globalPos) const;
    void showIndicatorFor(const DropTarget& target);
    void clearIndicator();
    QList<PanelGroup*> visibleGroups() const;
    bool applyPanelDrop(PanelGroup* source, const QString& name, const DropTarget& target);
    bool applyStripDrop(PanelGroup* source, const QString& name, int stripIndex);
    bool applyGroupDrop(PanelGroup* group, const DropTarget& target);
    bool applyNewColumnDrop(PanelSide side, PanelColumn* anchor);
    void beginPanelDrag(PanelGroup* group, const QString& objectName, const QPoint& globalPos);
    void beginGroupDrag(PanelGroup* group, const QPoint& globalPos);
    void updateDrag(const QPoint& globalPos);
    bool commitDrop();
    void cancelDrag();
    PanelFloat* createFloat(PanelGroup* group, const QPoint& globalPos);
    void destroyFloat(PanelFloat* floatWindow);
    void moveFloat(PanelFloat* floatWindow, const QPoint& globalTopLeft);
    QRect floatBounds(QWidget* host) const;
    PanelFloat* floatForGroup(PanelGroup* group) const;
    PanelGroup* findGroupByName(const QString& objectName) const;
    QWidget* makeGroupGrip(PanelGroup* group);
    QWidget* stripGroupBoxFor(PanelGroup* group) const;

    QWidget* header_ = nullptr;
    QToolButton* columnToggle_ = nullptr;
    QScrollArea* scroll_ = nullptr;
    QSplitter* splitter_ = nullptr;
    QWidget* iconStrip_ = nullptr;
    QBoxLayout* iconStripLayout_ = nullptr;
    QList<QLabel*> stripLabels_;
    QList<StripEntry> stripEntries_;
    // M44 C3: group containers and inter-group dividers in strip order.
    QList<PanelGroup*> stripGroupOrder_;
    QHash<PanelGroup*, QWidget*> stripGroupBoxes_;
    QList<StripDivider> stripDividers_;
    QString activeIconName_;
    QList<PanelGroup*> groups_;
    QHash<QString, bool> panelVisible_;

    bool railMode_ = false;
    bool dynamic_ = false;
    bool iconLabelsShown_ = false;
    int pendingWidth_ = 0;
    int normalWidthBeforeIconic_ = 0;

    PanelFlyout* flyout_ = nullptr;
    QBoxLayout* flyoutLayout_ = nullptr;
    QWidget* flyoutHeader_ = nullptr;
    QLabel* flyoutTitle_ = nullptr;
    QToolButton* flyoutClose_ = nullptr;
    QWidget* flyoutPanel_ = nullptr;
    PanelGroup* flyoutGroup_ = nullptr;
    QString flyoutName_;
    bool restoringFlyout_ = false;

    bool autoCollapseIconic_ = false;
    bool autoShowHidden_ = false;
    PanelGroup* menuGroup_ = nullptr;

    QWidget* indicator_ = nullptr;
    QWidget* stripIndicator_ = nullptr;
    QSet<PanelGroup*> wired_;
    QList<PanelFloat*> floats_;

    bool dragActive_ = false;
    bool dragIsPanel_ = false;
    PanelGroup* dragGroup_ = nullptr;
    // M44 W4: the panel's original group is kept alive for the whole drag so the
    // tab bar that owns the implicit mouse grab survives until release; it is
    // emptied by `createFloat` and cleaned up on commit/cancel.
    PanelGroup* dragSourceGroup_ = nullptr;
    QString dragPanel_;
    QPoint dragGrabOffset_;
    int dragOriginalIndex_ = -1;
    PanelFloat* dragFloat_ = nullptr;
    DropTarget dropTarget_;

    bool stripPressPending_ = false;
    bool stripDragging_ = false;
    QPoint stripPressGlobal_;
    QToolButton* stripDragButton_ = nullptr;
    // M44 C3: the group whose grip press/held state is active.
    PanelGroup* stripGripGroup_ = nullptr;
};

} // namespace pictura
