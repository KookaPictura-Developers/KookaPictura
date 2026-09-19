#pragma once

#include <QtCore/QByteArray>
#include <QtCore/QList>
#include <QtCore/QPoint>
#include <QtCore/QSet>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtGui/QColor>
#include <QtWidgets/QMainWindow>

#include "panels/panel_column.h"
#include "tools.h"

class QDockWidget;
class QLabel;
class QListWidget;
class QSplitter;
class QTabWidget;
class QTimer;
class QWidget;

namespace pictura {

class CommandRegistry;
class ColorPanel;
class ColorState;
class HistogramPanel;
class HistoryPanel;
class ImageView;
class InfoPanel;
class LayersPanel;
class NavigatorPanel;
class OptionsBar;
class PanelColumn;
class PictureView;
class PlaceholderPanel;
class PreferencesDialog;
class SwatchesPanel;
class Toolbox;
struct SessionState;

// The CS6-shaped application frame: menu bar, tabbed document area, status bar,
// and dock areas. Owns the UI and the open documents; each document's state
// lives in its own cxx-qt PictureView.
class PicturaMainWindow : public QMainWindow {
    Q_OBJECT

public:
    enum class ScreenMode { Standard, FullWithMenuBar, Full };

    explicit PicturaMainWindow(QWidget* parent = nullptr);
    ~PicturaMainWindow() override;

    // Test hooks.
    CommandRegistry* registry() const { return registry_; }
    ImageView* imageView() const;       // active canvas, or nullptr with no document
    PictureView* activeView() const;    // active document, or nullptr
    // Creates two documents and drags the last tab before its neighbour through
    // the real tab bar; returns whether docs_, viewAt and the active view stayed
    // aligned with the new visual order.
    bool reorderDocumentsForTest();
    QStringList topLevelMenuTitles() const;
    bool registerPanel(QWidget* panel, Qt::DockWidgetArea area);
    const QSet<QString>& panelObjectNames() const { return panelNames_; }
    PanelColumn* panelColumn() const { return panelColumn_; }
    PreferencesDialog* preferencesDialog() const { return preferencesDialog_; }

    // M43 multi-column host. Columns live in the central splitter around the
    // document tabs; a column's side is its splitter order relative to the
    // tabs, never its geometry. M44 W5: `anchor` inserts the new column
    // immediately before (side Left) or after (side Right) another column.
    PanelColumn* createPanelColumn(PanelSide side, PanelColumn* anchor = nullptr);
    void removeColumnIfEmpty(PanelColumn* column);
    PanelSide sideOf(const PanelColumn* column) const;
    int columnCount() const;
    QList<PanelColumn*> panelColumns() const;
    PanelColumn* columnForPanel(const QString& objectName) const;
    PanelColumn* columnAtGlobal(const QPoint& globalPos) const;
    // -1 = not a new-column candidate, 0 = left, 1 = right.
    int newColumnSideAt(const QPoint& globalPos) const;
    // M44 W5: a point in a band beside an existing column (other than `exclude`)
    // resolves to a new column anchored on that side; returns the anchor.
    PanelColumn* columnEdgeAnchorAt(const QPoint& globalPos, const PanelColumn* exclude,
                                    int* side) const;

    // M45 T3: a floating-Tools drop resolved through the same column grammar.
    // `resolveToolboxDrop` shows the single `#2a7fff` indicator at the resolved
    // boundary; `commitToolboxDrop` hosts the Tools panel as a fixed-width
    // central-splitter pane there (a QDockWidget cannot sit between columns).
    bool resolveToolboxDrop(const QPoint& globalPos, PanelColumn** anchor, int* side);
    bool commitToolboxDrop(const QPoint& globalPos);

    // M43 Phase B test hooks. All drive the same resolve/commit drag path.
    int panelColumnCountForTest() const { return columnCount(); }
    QString panelColumnSideForTest(int index) const;
    bool newColumnDropForTest(const QString& panelName, const QString& side);
    // M44 W5: drop a panel beside an existing column; the new column lands
    // immediately adjacent to the anchor.
    bool newColumnBesideForTest(const QString& panelName, const QString& anchorPanel);
    // M45 T3: dock the floating Tools panel to the left/right of a widget column
    // (the primary column, or a throwaway dynamic anchor) through the real
    // resolve/commit path; returns true when the indicator showed and the pane
    // landed adjacent.
    bool toolboxBesideColumnForTest(const QString& side, bool dynamicAnchor);
    bool dropIntoGroupForTest(const QString& panelName, const QString& targetPanel, int index = 1);
    bool dropBoundaryForTest(const QString& panelName, const QString& targetPanel, bool above);
    // M43 Phase C test hook: re-runs the real startup restore path so a saved
    // layout can be applied and re-applied without a second frame.
    void applyPanelSessionForTest(const SessionState& state) { applyPanelSession(state); }

    // Bumped when the chrome layout changes shape (M42 removed the old dock
    // set); a persisted layout from another revision is discarded on restore so
    // stale chrome cannot reappear over the menu bar.
    static constexpr int kLayoutRevision = 2;
    int layoutRevisionForTest() const { return kLayoutRevision; }
    bool restoreStoredLayout(const QByteArray& layout, int revision);

    // The Tools panel must not join a tab group; re-dock it to its last side if
    // it somehow does (the tabify fallback, exposed for the self-test).
    void ensureToolsNotTabified();

    // Tool test hooks.
    ToolId activeTool() const;
    void setActiveTool(ToolId id);
    QColor foregroundColor() const { return foreground_; }
    bool hasPendingCrop() const;
    bool commitCrop();

    int brightnessLevel() const { return brightnessLevel_; }
    void setBrightnessLevel(int level);

    ScreenMode screenMode() const { return screenMode_; }
    void setScreenMode(ScreenMode mode);
    void cycleScreenMode(bool forward);

    int canvasColorIndex() const { return canvasColorIndex_; }
    void cycleCanvasColor(bool forward);

    bool panelsHidden() const { return panelsHidden_; }
    void setPanelsHidden(bool hidden);

    void refresh();
    void saveSession();

    // Mirror the active document's selection outline onto its canvas (or clear
    // it). Called from refresh() and on toolbar selection commits; view-only.
    void refreshSelectionOverlay();

    // Document operations.
    int documentCount() const { return docs_.size(); }
    int activeDocumentIndex() const;
    void setActiveDocumentIndex(int index);
    PictureView* viewAt(int index) const;
    ImageView* canvasAt(int index) const;
    QString documentPath(int index) const;
    QString documentName(int index) const;
    bool isDocumentDirty(int index) const;
    QString activeFilePath() const;
    QString activeDocumentName() const;
    bool isActiveDirty() const;

    // Takes ownership of `view` and adds it as a document tab.
    int addDocument(PictureView* view, const QString& path);
    bool newDocument(const QString& name, int width, int height, const QString& mode,
                     int depth, const QString& background);
    bool openPath(const QString& path);
    bool openAsSmartObjectPath(const QString& path);
    bool saveActive();
    bool saveActiveAs(const QString& path);
    bool revertActive();
    bool closeDocument(int index, bool interactive);
    bool closeActiveDocument(bool interactive);
    void showNewDocumentDialog();
    void showOpenDialog();

protected:
    void closeEvent(QCloseEvent* event) override;
    void keyPressEvent(QKeyEvent* event) override;
    void keyReleaseEvent(QKeyEvent* event) override;

private:
    struct DocEntry {
        PictureView* view = nullptr;
        ImageView* canvas = nullptr;
        QString path;
        int untitledNumber = 0;
    };

    void buildMenus();
    void buildPanels();
    void buildTools(int toolsColumns, bool useShiftKeyForToolSwitch);
    void buildStatusBar();
    void registerHandlers();
    void registerSelectHandlers();
    void applyPanelSession(const SessionState& state);
    void wirePanelColumn(PanelColumn* column);
    void clearDynamicColumns();
    void reapplyColumnStretch();
    void showPreferences(const QString& page);
    void retargetDock();
    void refreshPanels();
    void updateStatus();
    void updateToolHint();
    void updateTabTitle(int index);
    void updateWindowTitle();
    void removeDocument(int index);
    void rememberRecent(const QString& path);
    void rebuildRecentMenu();
    void applyBrightness(int level);

    QList<DocEntry> docs_;
    QTabWidget* tabs_ = nullptr;
    QSplitter* centerSplitter_ = nullptr;
    QTimer* panelRefreshTimer_ = nullptr;
    CommandRegistry* registry_ = nullptr;
    LayersPanel* layersPanel_ = nullptr;
    HistoryPanel* historyPanel_ = nullptr;
    NavigatorPanel* navigatorPanel_ = nullptr;
    ColorState* colorState_ = nullptr;
    ColorPanel* colorPanel_ = nullptr;
    SwatchesPanel* swatchesPanel_ = nullptr;
    InfoPanel* infoPanel_ = nullptr;
    HistogramPanel* histogramPanel_ = nullptr;
    PlaceholderPanel* gradientsPanel_ = nullptr;
    PlaceholderPanel* patternsPanel_ = nullptr;
    PlaceholderPanel* propertiesPanel_ = nullptr;
    PlaceholderPanel* adjustmentsPanel_ = nullptr;
    PlaceholderPanel* librariesPanel_ = nullptr;
    PlaceholderPanel* channelsPanel_ = nullptr;
    PlaceholderPanel* pathsPanel_ = nullptr;
    PlaceholderPanel* actionsPanel_ = nullptr;
    PlaceholderPanel* stylesPanel_ = nullptr;
    PanelColumn* panelColumn_ = nullptr;
    PreferencesDialog* preferencesDialog_ = nullptr;
    ToolController* tools_ = nullptr;
    OptionsBar* optionsBar_ = nullptr;
    Toolbox* toolbox_ = nullptr;
    QDockWidget* toolsDock_ = nullptr;
    // M45 T3: the column currently showing the floating-Tools drop indicator.
    PanelColumn* toolboxDropAnchor_ = nullptr;
    QLabel* zoomLabel_ = nullptr;
    QLabel* sizeLabel_ = nullptr;
    QLabel* hintLabel_ = nullptr;
    QLabel* backendLabel_ = nullptr;
    QString statusReadout_ = QStringLiteral("sizes");
    QColor foreground_;
    QSet<QString> panelNames_;
    QStringList recent_;
    int untitledCounter_ = 0;
    int brightnessLevel_ = 1;
    bool gpuCompute_ = true;
    bool gpuAvailable_ = true;
    bool useShiftKeyForToolSwitch_ = true;
    Qt::DockWidgetArea toolsArea_ = Qt::LeftDockWidgetArea;
    ScreenMode screenMode_ = ScreenMode::Standard;
    int canvasColorIndex_ = 0;
    bool panelsHidden_ = false;
    bool restoringPanelSession_ = false;
};

} // namespace pictura
