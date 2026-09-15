#pragma once

#include <QtCore/QSet>
#include <QtCore/QStringList>
#include <QtWidgets/QMainWindow>

class QCloseEvent;
class QDockWidget;
class QKeyEvent;
class QLabel;
class QListWidget;

namespace pictura {

class CommandRegistry;
class ImageView;
class PictureView;

// The CS6-shaped application frame: menu bar, central canvas, status bar, and
// dock areas. Owns the UI; document state stays in the cxx-qt PictureView.
class PicturaMainWindow : public QMainWindow {
    Q_OBJECT

public:
    enum class ScreenMode { Standard, FullWithMenuBar, Full };

    explicit PicturaMainWindow(PictureView* view, QWidget* parent = nullptr);
    ~PicturaMainWindow() override;

    // Test hooks.
    CommandRegistry* registry() const { return registry_; }
    ImageView* imageView() const { return imageView_; }
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

    // Re-read the document into the canvas and update the layer list, selection
    // readout, and status bar. Connected to PictureView::changed.
    void refresh();

    // Persist layout + brightness through the session store.
    void saveSession();

protected:
    void closeEvent(QCloseEvent* event) override;
    void keyPressEvent(QKeyEvent* event) override;

private:
    void buildMenus();
    void buildPanels();
    void buildStatusBar();
    void registerHandlers();
    void updateStatus();
    void applyBrightness(int level);

    PictureView* view_ = nullptr;
    ImageView* imageView_ = nullptr;
    CommandRegistry* registry_ = nullptr;
    QDockWidget* layersDock_ = nullptr;
    QListWidget* layerList_ = nullptr;
    QLabel* zoomLabel_ = nullptr;
    QLabel* sizeLabel_ = nullptr;
    QLabel* hintLabel_ = nullptr;
    QSet<QString> panelNames_;
    int brightnessLevel_ = 1;
    ScreenMode screenMode_ = ScreenMode::Standard;
    int canvasColorIndex_ = 0;
    bool panelsHidden_ = false;
};

} // namespace pictura
