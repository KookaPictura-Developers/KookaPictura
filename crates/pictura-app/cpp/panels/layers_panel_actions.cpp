#include "layers_panel.h"

#include "layers_panel_internal.h"

#include <QtCore/QHash>
#include <QtCore/QString>
#include <QtCore/QStringList>

namespace pictura {

void LayersPanel::addLayerAt(const QString& path)
{
    if (!view_) {
        return;
    }
    clearSolo();
    const QString created = view_->add_layer_in(path);
    if (!created.isEmpty()) {
        refresh();
        selectPath(created);
    }
}

void LayersPanel::addGroupAt(const QString& path)
{
    if (!view_) {
        return;
    }
    clearSolo();
    const QString created = view_->add_group_in(path);
    if (!created.isEmpty()) {
        expandedPaths_.insert(created);
        refresh();
        selectPath(created);
    }
}

void LayersPanel::duplicateSelection()
{
    if (!view_) {
        return;
    }
    const QStringList paths = selectedPaths();
    if (paths.isEmpty()) {
        return;
    }
    clearSolo();
    const QStringList created = view_->duplicate_layers(paths);
    if (!created.isEmpty()) {
        refresh();
        selectPath(created.last());
    }
}

void LayersPanel::deleteSelection()
{
    if (!view_) {
        return;
    }
    const QStringList paths = selectedPaths();
    if (paths.isEmpty()) {
        return;
    }
    clearSolo();
    view_->delete_layers(paths);
}

void LayersPanel::groupSelection()
{
    if (!view_) {
        return;
    }
    const QStringList paths = selectedPaths();
    if (paths.isEmpty()) {
        return;
    }
    clearSolo();
    const QString created = view_->group_layers(paths);
    if (!created.isEmpty()) {
        expandedPaths_.insert(created);
        refresh();
        selectPath(created);
    }
}

void LayersPanel::ungroupSelection()
{
    if (!view_) {
        return;
    }
    const QStringList paths = selectedPaths();
    if (paths.isEmpty()) {
        return;
    }
    clearSolo();
    view_->ungroup_layers(paths);
}

void LayersPanel::moveCurrent(int delta)
{
    if (!view_) {
        return;
    }
    const QString path = currentPath();
    if (path.isEmpty()) {
        return;
    }
    clearSolo();
    view_->move_layer_path(path, delta);
}

void LayersPanel::toggleSolo(const QString& path)
{
    if (path.isEmpty() || !view_) {
        return;
    }
    if (soloActive_) {
        restoreSolo();
        return;
    }
    const QHash<QString, bool> snapshot = model_->visibilityByPath();
    if (snapshot.isEmpty()) {
        return;
    }
    soloSnapshot_ = snapshot;
    soloActive_ = true;
    soloPath_ = path;
    view_->apply_visibility(soloPaths(path, snapshot), QStringLiteral("Solo Visibility"));
}

void LayersPanel::restoreSolo()
{
    if (!view_) {
        clearSolo();
        return;
    }
    QStringList visible;
    for (auto it = soloSnapshot_.cbegin(); it != soloSnapshot_.cend(); ++it) {
        if (it.value()) {
            visible.push_back(it.key());
        }
    }
    clearSolo();
    view_->apply_visibility(visible, QStringLiteral("Restore Visibility"));
}

void LayersPanel::clearSolo()
{
    soloActive_ = false;
    soloPath_.clear();
    soloSnapshot_.clear();
}

QStringList LayersPanel::soloPaths(const QString& path, const QHash<QString, bool>& snapshot) const
{
    QStringList result{path};
    int slash = path.lastIndexOf(QLatin1Char('/'));
    while (slash > 0) {
        const QString ancestor = path.left(slash);
        result.push_back(ancestor);
        slash = ancestor.lastIndexOf(QLatin1Char('/'));
    }
    if (model_->indexForPath(path).data(KindRole).toString() == QLatin1String("group")) {
        const QString prefix = path + QLatin1Char('/');
        for (auto it = snapshot.cbegin(); it != snapshot.cend(); ++it) {
            if (it.value() && it.key().startsWith(prefix)) {
                result.push_back(it.key());
            }
        }
    }
    result.removeDuplicates();
    return result;
}

} // namespace pictura
