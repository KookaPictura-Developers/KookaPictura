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
class QTemporaryDir;
class QTimer;
class QWidget;

namespace pictura {

class CommandRegistry;
class ColorPanel;
class ChannelsPanel;
class ColorState;
class CanvasScrollBars;
class FileDropRouter;
class HistogramPanel;
class HistoryPanel;
class BrushPanel;
class CloneSourcePanel;
class ImageView;
class InfoPanel;
class LayersPanel;
class NavigatorPanel;
class NotesPanel;
class PathsPanel;
class OptionsBar;
class PanelColumn;
class PictureView;
class PlaceholderPanel;
class PreferencesDialog;
class PropertiesPanel;
class SwatchesPanel;
class Toolbox;
class ToolHintBar;
struct FilterCommandSpec;
struct SessionState;

// True when a launch should seed the scratch white document. Only the self-test
// path needs it (headless implies self-test), so a normal launch starts on the
// empty workspace with the document commands disabled.
bool launchCreatesScratchDocument(bool selfTest, bool codecLoaded);

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
    // Every column the frame owns, including a whole column torn off into a
    // floating overlay (which is no longer a splitter pane). Used where a
    // floating column must still be reachable: drop targeting, the Window >
    // Panels owner lookup, and session serialization.
    QList<PanelColumn*> allPanelColumns() const;
    PanelColumn* columnForPanel(const QString& objectName) const;
    // Show a hidden panel or hide a shown one (the options bar's panel toggles).
    void togglePanel(const QString& objectName);
    PanelColumn* columnAtGlobal(const QPoint& globalPos) const;
    // -1 = not a new-column candidate, 0 = left, 1 = right.
    int newColumnSideAt(const QPoint& globalPos) const;
    // M44 W5: a point in a band beside an existing column (other than `exclude`)
    // resolves to a new column anchored on that side; returns the anchor.
    PanelColumn* columnEdgeAnchorAt(const QPoint& globalPos, const PanelColumn* exclude,
                                    int* side) const;

    // M47: move a whole column beside an anchor (side 0 = left, 1 = right) using
    // the same splitter history as `createPanelColumn`. `resolveColumnMoveTarget`
    // returns the anchor for a header drag (null for a bare workspace edge).
    PanelColumn* resolveColumnMoveTarget(const QPoint& globalPos, const PanelColumn* exclude,
                                         int* side) const;
    bool movePanelColumn(PanelColumn* column, int side, PanelColumn* anchor);

    // The tabless, atomic Tools column hosted in the central splitter (default
    // left, index 0). Its content is the `toolbox_` widget.
    PanelColumn* toolsColumn() const { return toolsColumn_; }

    // M43 Phase B test hooks. All drive the same resolve/commit drag path.
    int panelColumnCountForTest() const { return columnCount(); }
    QString panelColumnSideForTest(int index) const;
    bool newColumnDropForTest(const QString& panelName, const QString& side);
    // M44 W5: drop a panel beside an existing column; the new column lands
    // immediately adjacent to the anchor.
    bool newColumnBesideForTest(const QString& panelName, const QString& anchorPanel);
    bool dropIntoGroupForTest(const QString& panelName, const QString& targetPanel, int index = 1);
    bool dropBoundaryForTest(const QString& panelName, const QString& targetPanel, bool above);
    // M43 Phase C test hook: re-runs the real startup restore path so a saved
    // layout can be applied and re-applied without a second frame.
    void applyPanelSessionForTest(const SessionState& state) { applyPanelSession(state); }

    // Bumped when the chrome layout changes shape (M42 removed the old dock
    // set); a persisted layout from another revision is discarded on restore so
    // stale chrome cannot reappear over the menu bar. v9 moved Tools from a dock
    // into a central-splitter column, so the old layout is discarded.
    static constexpr int kLayoutRevision = 3;
    int layoutRevisionForTest() const { return kLayoutRevision; }
    bool restoreStoredLayout(const QByteArray& layout, int revision);

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
    QString documentTabTextForTest(int index) const;
    bool isDocumentDirty(int index) const;
    QString activeFilePath() const;
    QString activeDocumentName() const;
    bool isActiveDirty() const;
    // True when no document is open, so a double-click on the empty workspace
    // opens the Open dialog.
    bool workspaceOpenArmed() const { return docs_.isEmpty(); }

    // Takes ownership of `view` and adds it as a document tab.
    int addDocument(PictureView* view, const QString& path);
    bool newDocument(const QString& name, int width, int height, const QString& mode,
                     int depth, const QString& background);
    bool openPath(const QString& path);
    bool openImagePath(const QString& path);
    // Route `path` to the PSD/PSB reader or the Qt raster import by suffix.
    bool openDocumentAtPath(const QString& path);
    // True when `path`'s suffix routes to the native PSD/PSB reader rather than
    // the Qt image decode edge.
    static bool isNativeDocumentPath(const QString& path);
    bool openAsSmartObjectPath(const QString& path);
    bool editSmartObjectContents(const QString& layerPath);
    // Begin a Free Transform session on `path` in the active view and show its
    // overlay. Returns false without a transformable target.
    bool beginFreeTransform(const QString& path);
    // Begin a Skew / Distort / Perspective session on `path` and show its
    // overlay. Returns false without a transformable target or an unknown mode.
    bool beginTransformMode(const QString& path, const QString& mode);
    // Make `path` the active Layers-panel row (refresh first so a just-created
    // layer is present in the model). No-op without the panel or an empty path.
    void selectLayerPath(const QString& path);
    bool saveActive();
    bool saveActiveAs(const QString& path);
    // Prompt for a Save As path with the format-aware filters (preselecting the
    // active document's source format) and write through `saveActiveAs`. Warns
    // before flattening a layered document into a raster format.
    bool saveAsWithDialog();
    bool revertActive();
    bool closeDocument(int index, bool interactive);
    // The recent-files list, most recent first; setting it persists it.
    const QStringList& recentFiles() const { return recent_; }
    void setRecentFiles(const QStringList& files);
    // File > Open Recent, or null before the menu bar is built.
    QMenu* recentMenu() const;
    bool closeActiveDocument(bool interactive);
    void showNewDocumentDialog();
    void showOpenDialog();
    void showFileInfo();
    // Assign (`convert == false`) or convert to a chosen built-in profile; no-op
    // unless the active view holds an RGB document.
    void showProfileCommand(bool convert);
    // Edit > Color Settings…: show the sRGB working space and incoming-profile
    // policy; on OK persist the choice and apply it to every open view.
    void showColorSettings();

protected:
    void closeEvent(QCloseEvent* event) override;
    void keyPressEvent(QKeyEvent* event) override;
    void keyReleaseEvent(QKeyEvent* event) override;
    bool eventFilter(QObject* watched, QEvent* event) override;

private:
    struct DocEntry {
        PictureView* view = nullptr;
        ImageView* canvas = nullptr;
        CanvasScrollBars* canvasHost = nullptr;
        QString path;
        // Display name for a path-less import (Open Image / Open As Smart
        // Object); preferred over the generated Untitled-N name. The tab title
        // uses it while `path` stays empty so Save cannot overwrite the source.
        QString displayName;
        int untitledNumber = 0;
    };

    // One open Edit Contents editor: the untitled tab, the document that owns
    // the edited layer, and the per-session temporary file holding the source.
    struct SmartObjectEditSession {
        PictureView* editor = nullptr;
        PictureView* origin = nullptr;
        QString layerPath;
        QString filename;
        QTemporaryDir* temp = nullptr;
    };

    void buildMenus();
    void buildPanels();
    void buildTools(int toolsColumns, bool useShiftKeyForToolSwitch);
    void buildStatusBar();
    // Filter the tab pane so a double-click on the empty workspace (no document
    // open) opens the Open dialog.
    void installWorkspaceOpenGesture();
    void registerHandlers();
    void registerSelectHandlers();
    void registerEditHandlers();
    // Wire the CS6 Filter menu: one handler per implemented filter, plus Last
    // Filter / Last Filter Settings.
    void wireFilterMenu();
    void applyFilterCommand(const FilterCommandSpec& spec);
    // Surface why a filter was refused (status bar + stderr) instead of a
    // silent no-op.
    void reportFilterRefusal(PictureView* view);
    // frame_menus_align.cpp: Layer > Align / Align Layers To Selection /
    // Distribute, and the Move tool's buttons, over the selected layers.
    void wireAlignMenu();
    void alignSelectedLayers(int edge, bool toSelection);
    void distributeSelectedLayers(int edge);
    void updateAlignControls();
    void exportClipboard();
    void importSystemClipboard();
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
    // Repopulate File > Open Recent from `recent_` each time it opens.
    void refreshRecentMenu(QMenu* menu);
    void openRecent(const QString& path);
    void applyBrightness(int level);

    QList<DocEntry> docs_;
    QList<SmartObjectEditSession> editSessions_;
    QTabWidget* tabs_ = nullptr;
    FileDropRouter* fileDropRouter_ = nullptr;
    QSplitter* centerSplitter_ = nullptr;
    QTimer* panelRefreshTimer_ = nullptr;
    QTimer* sessionSaveTimer_ = nullptr;
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
    NotesPanel* notesPanel_ = nullptr;
    BrushPanel* brushPanel_ = nullptr;
    CloneSourcePanel* cloneSourcePanel_ = nullptr;
    PropertiesPanel* propertiesPanel_ = nullptr;
    PlaceholderPanel* adjustmentsPanel_ = nullptr;
    ChannelsPanel* channelsPanel_ = nullptr;
    PathsPanel* pathsPanel_ = nullptr;
    PlaceholderPanel* actionsPanel_ = nullptr;
    PlaceholderPanel* stylesPanel_ = nullptr;
    PanelColumn* panelColumn_ = nullptr;
    PreferencesDialog* preferencesDialog_ = nullptr;
    ToolController* tools_ = nullptr;
    OptionsBar* optionsBar_ = nullptr;
    Toolbox* toolbox_ = nullptr;
    // The tabless, atomic Tools column that hosts `toolbox_` in the splitter.
    PanelColumn* toolsColumn_ = nullptr;
    QLabel* zoomLabel_ = nullptr;
    QLabel* sizeLabel_ = nullptr;
    ToolHintBar* hintBar_ = nullptr;
    QLabel* backendLabel_ = nullptr;
    QString statusReadout_ = QStringLiteral("sizes");
    QColor foreground_;
    QSet<QString> panelNames_;
    QStringList recent_;
    // Actions refreshRecentMenu added (file rows, separator, Clear); the
    // registry's "No Recent Files" placeholder is not among them.
    QList<QAction*> recentActions_;
    int untitledCounter_ = 0;
    int brightnessLevel_ = 1;
    bool gpuCompute_ = true;
    int colorPolicy_ = 0;
    bool gpuAvailable_ = true;
    bool useShiftKeyForToolSwitch_ = true;
    ScreenMode screenMode_ = ScreenMode::Standard;
    int canvasColorIndex_ = 0;
    bool panelsHidden_ = false;
    bool restoringPanelSession_ = false;
    // System-clipboard mirror of the bridge clipboard (frame_menus_edit.cpp):
    // `clipboardMirrored_` while both hold the same pixels, `clipboardExported_`
    // while the system clipboard holds our own export, `clipboardSetting_` only
    // during our own write so its dataChanged is not taken for another app's.
    bool clipboardMirrored_ = false;
    bool clipboardExported_ = false;
    bool clipboardSetting_ = false;
};

} // namespace pictura
