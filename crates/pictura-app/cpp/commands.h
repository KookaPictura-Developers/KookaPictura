#pragma once

#include <QtCore/QHash>
#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtGui/QKeySequence>

#include <functional>
#include <vector>

class QAction;
class QMenuBar;

namespace pictura {

// Stable command identifiers. Command ids are the dispatch key and the key for
// future localization and custom menu sets; never derive them from labels.
namespace command_ids {
inline constexpr char FileOpen[] = "file.open";
inline constexpr char FileNew[] = "file.new";
inline constexpr char FileSave[] = "file.save";
inline constexpr char FileSaveAs[] = "file.saveAs";
inline constexpr char FileRevert[] = "file.revert";
inline constexpr char FileClose[] = "file.close";
inline constexpr char FileCloseAll[] = "file.closeAll";
inline constexpr char FileExit[] = "file.exit";
inline constexpr char EditUndo[] = "edit.undo";
inline constexpr char EditRedo[] = "edit.redo";
inline constexpr char EditStepBackward[] = "edit.stepBackward";
inline constexpr char EditStepForward[] = "edit.stepForward";
inline constexpr char ImageRotate90Cw[] = "image.rotate90cw";
inline constexpr char ImageRotate90Ccw[] = "image.rotate90ccw";
inline constexpr char ImageRotate180[] = "image.rotate180";
inline constexpr char ImageFlipHorizontal[] = "image.flipHorizontal";
inline constexpr char ImageFlipVertical[] = "image.flipVertical";
inline constexpr char ImageCrop[] = "image.crop";
inline constexpr char SelectAll[] = "select.all";
inline constexpr char SelectDeselect[] = "select.deselect";
inline constexpr char ViewZoomIn[] = "view.zoomIn";
inline constexpr char ViewZoomOut[] = "view.zoomOut";
inline constexpr char ViewFitOnScreen[] = "view.fitOnScreen";
inline constexpr char ViewActualPixels[] = "view.actualPixels";
inline constexpr char ViewOptions[] = "view.options";
inline constexpr char ViewScreenModeStandard[] = "view.screenMode.standard";
inline constexpr char ViewScreenModeFullWithMenuBar[] = "view.screenMode.fullWithMenuBar";
inline constexpr char ViewScreenModeFull[] = "view.screenMode.full";
inline constexpr char WindowPanelsLayers[] = "window.panels.layers";
inline constexpr char WindowPanelsTools[] = "window.panels.tools";
inline constexpr char WindowPanelsNavigator[] = "window.panels.navigator";
inline constexpr char WindowPanelsHistory[] = "window.panels.history";
inline constexpr char WindowPanelsColor[] = "window.panels.color";
inline constexpr char WindowPanelsSwatches[] = "window.panels.swatches";
inline constexpr char WindowPanelsInfo[] = "window.panels.info";
inline constexpr char WindowPanelsHistogram[] = "window.panels.histogram";
inline constexpr char WindowPanelsGradients[] = "window.panels.gradients";
inline constexpr char WindowPanelsPatterns[] = "window.panels.patterns";
inline constexpr char WindowPanelsProperties[] = "window.panels.properties";
inline constexpr char WindowPanelsAdjustments[] = "window.panels.adjustments";
inline constexpr char WindowPanelsLibraries[] = "window.panels.libraries";
inline constexpr char WindowPanelsChannels[] = "window.panels.channels";
inline constexpr char WindowPanelsPaths[] = "window.panels.paths";
inline constexpr char WindowPanelsActions[] = "window.panels.actions";
inline constexpr char HelpAbout[] = "help.about";
} // namespace command_ids

// One entry of the command table. An entry with an empty `id` is a separator.
struct CommandSpec {
    QString id;
    QStringList path;                 // top-level menu first, leaf last
    QString label;
    QKeySequence shortcut;
    bool implemented = false;
    bool checkable = false;
};

// Declarative command table plus menu construction and dispatch. The registry
// owns no application behavior: the frame supplies handlers, providers, and
// panel-facing actions by id.
class CommandRegistry : public QObject {
    Q_OBJECT

public:
    explicit CommandRegistry(QObject* parent = nullptr);

    void add(const CommandSpec& spec);
    void add(const QString& id, const QStringList& path, const QString& label,
             const QKeySequence& shortcut = QKeySequence(), bool implemented = false);
    void addSeparator(const QStringList& path);

    void setHandler(const QString& id, std::function<void()> handler);
    bool hasHandler(const QString& id) const;

    // Extra enablement predicate for `id`; combined with `implemented` and the
    // presence of a handler.
    void setEnabledProvider(const QString& id, std::function<bool()> enabled);
    // Checked-state provider for a checkable command.
    void setCheckedProvider(const QString& id, std::function<bool()> checked);
    // Dynamic label for `id`; overrides the static label when set.
    void setLabelProvider(const QString& id, std::function<QString()> label);

    QAction* action(const QString& id) const;

    // Build (or rebuild) `menuBar` from the command table.
    void buildMenuBar(QMenuBar* menuBar);

    // Re-evaluate enabled/checked/label for every action.
    void refresh();

    // Run the handler for `id`; returns false and does nothing when none is set.
    bool dispatch(const QString& id);

    // Titles of the top-level menus in creation order.
    QStringList topLevelTitles() const;

private:
    struct Entry {
        CommandSpec spec;
        QAction* action = nullptr;
    };
    std::vector<Entry> entries_;
    QHash<QString, std::function<void()>> handlers_;
    QHash<QString, std::function<bool()>> enabledProviders_;
    QHash<QString, std::function<bool()>> checkedProviders_;
    QHash<QString, std::function<QString()>> labelProviders_;
    QMenuBar* menuBar_ = nullptr;
};

// Populate `registry` with the full documented CS6 command tree
// (docs/02-ui-ux/menus.md). Commands without a handler are added with
// `implemented = false`; the ids in `command_ids` are added as implemented.
void addDefaultCommands(CommandRegistry& registry);

} // namespace pictura
