#pragma once

#include <QtCore/QByteArray>
#include <QtCore/QList>
#include <QtCore/QSet>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtGui/QColor>
#include <QtWidgets/QMainWindow>

#include "tools.h"

class QDockWidget;
class QLabel;
class QListWidget;
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
    QStringList topLevelMenuTitles() const;
    bool registerPanel(QWidget* panel, Qt::DockWidgetArea area);
    const QSet<QString>& panelObjectNames() const { return panelNames_; }
    PanelColumn* panelColumn() const { return panelColumn_; }
    PreferencesDialog* preferencesDialog() const { return preferencesDialog_; }

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
    void applyPanelSession(const SessionState& state);
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
    QLabel* zoomLabel_ = nullptr;
    QLabel* sizeLabel_ = nullptr;
    QLabel* hintLabel_ = nullptr;
    QLabel* backendLabel_ = nullptr;
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
};

} // namespace pictura
