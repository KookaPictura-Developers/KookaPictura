#pragma once

#include <QtCore/QList>
#include <QtCore/QSet>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtWidgets/QMainWindow>

class QDockWidget;
class QLabel;
class QListWidget;
class QTabWidget;

namespace pictura {

class CommandRegistry;
class ImageView;
class PictureView;

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
    bool registerPanel(QDockWidget* dock, Qt::DockWidgetArea area);
    const QSet<QString>& panelObjectNames() const { return panelNames_; }

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
    void buildStatusBar();
    void registerHandlers();
    void retargetDock();
    void updateStatus();
    void updateTabTitle(int index);
    void updateWindowTitle();
    void removeDocument(int index);
    void rememberRecent(const QString& path);
    void rebuildRecentMenu();
    void applyBrightness(int level);

    QList<DocEntry> docs_;
    QTabWidget* tabs_ = nullptr;
    CommandRegistry* registry_ = nullptr;
    QDockWidget* layersDock_ = nullptr;
    QListWidget* layerList_ = nullptr;
    QLabel* zoomLabel_ = nullptr;
    QLabel* sizeLabel_ = nullptr;
    QLabel* hintLabel_ = nullptr;
    QSet<QString> panelNames_;
    QStringList recent_;
    int untitledCounter_ = 0;
    int brightnessLevel_ = 1;
    ScreenMode screenMode_ = ScreenMode::Standard;
    int canvasColorIndex_ = 0;
    bool panelsHidden_ = false;
};

} // namespace pictura
