#include "layers_panel.h"

#include "layers_panel_internal.h"

#include <QtCore/QItemSelectionModel>
#include <QtCore/QModelIndex>
#include <QtCore/QPoint>
#include <QtCore/QStringList>
#include <QtGui/QAction>
#include <QtGui/QIcon>
#include <QtGui/QPixmap>
#include <QtWidgets/QMenu>
#include <QtWidgets/QTreeView>

namespace pictura {

namespace {

QString labelName(int label)
{
    switch (label) {
    case 1:
        return QStringLiteral("Red");
    case 2:
        return QStringLiteral("Orange");
    case 3:
        return QStringLiteral("Yellow");
    case 4:
        return QStringLiteral("Green");
    case 5:
        return QStringLiteral("Blue");
    case 6:
        return QStringLiteral("Violet");
    case 7:
        return QStringLiteral("Gray");
    default:
        return QStringLiteral("None");
    }
}

} // namespace

void LayersPanel::showContextMenu(const QPoint& pos)
{
    if (!view_) {
        return;
    }
    const QModelIndex index = tree_->indexAt(pos);
    if (!index.isValid()) {
        return;
    }
    if (delegate_->eyeRect(tree_->visualRect(index)).contains(pos)) {
        showEyeMenu(pos, index);
        return;
    }
    const QString path = pathForProxyIndex(index);
    if (path.isEmpty()) {
        return;
    }
    if (!tree_->selectionModel()->isSelected(index)) {
        tree_->setCurrentIndex(index);
    }

    QMenu menu(tree_);
    populateRowMenu(menu, path, index.data(ColorRole).toInt());
    menu.exec(tree_->viewport()->mapToGlobal(pos));
}

void LayersPanel::populateRowMenu(QMenu& menu, const QString& path, int color)
{
    QAction* rename = menu.addAction(tr("Rename"));
    connect(rename, &QAction::triggered, this, [this, path] {
        const QModelIndex target = proxyIndexForPath(path);
        if (target.isValid()) {
            tree_->setCurrentIndex(target);
            tree_->edit(target);
        }
    });
    menu.addSeparator();
    const auto addAction = [this, &menu](const QString& text, auto fn) {
        QAction* action = menu.addAction(text);
        connect(action, &QAction::triggered, this, fn);
    };
    addAction(tr("New Layer"), [this, path] { addLayerAt(path); });
    addAction(tr("New Group"), [this, path] { addGroupAt(path); });
    addAction(tr("Duplicate Layer(s)"), [this] { duplicateSelection(); });
    addAction(tr("Delete Layer(s)"), [this] { deleteSelection(); });
    addAction(tr("Group Layers"), [this] { groupSelection(); });
    addAction(tr("Ungroup Layers"), [this] { ungroupSelection(); });
    addAction(tr("Move Layer Up"), [this] { moveCurrent(1); });
    addAction(tr("Move Layer Down"), [this] { moveCurrent(-1); });
    menu.addSeparator();
    addColorLabelActions(menu.addMenu(tr("Color Label")), color);
}

void LayersPanel::showEyeMenu(const QPoint& pos, const QModelIndex& index)
{
    const QString path = pathForProxyIndex(index);
    QMenu menu(tree_);
    QAction* only = menu.addAction(tr("Show/Hide This Layer Only"));
    connect(only, &QAction::triggered, this, [this, path] { toggleSolo(path); });
    QAction* all = menu.addAction(tr("Show/Hide All Layers"));
    connect(all, &QAction::triggered, this, [this] {
        if (!view_) {
            return;
        }
        clearSolo();
        view_->apply_visibility(model_->paths(), QStringLiteral("Show All Layers"));
    });
    menu.exec(tree_->viewport()->mapToGlobal(pos));
}

void LayersPanel::addColorLabelActions(QMenu* menu, int currentLabel)
{
    for (int label = 0; label <= 7; ++label) {
        QAction* action = menu->addAction(labelName(label));
        action->setCheckable(true);
        action->setChecked(currentLabel == label);
        const QPixmap swatch = labelSwatch(label);
        if (!swatch.isNull()) {
            action->setIcon(QIcon(swatch));
        }
        connect(action, &QAction::triggered, this, [this, label] {
            const QStringList paths = selectedPaths();
            if (view_ && !paths.isEmpty()) {
                view_->set_layers_color(paths, label);
            }
        });
    }
}

bool LayersPanel::performPanelMenuAction(const QString& actionId)
{
    if (actionId == QLatin1String("newLayer")) {
        openNewLayerDialog();
    } else if (actionId == QLatin1String("duplicate")) {
        duplicateSelection();
    } else if (actionId == QLatin1String("delete")) {
        deleteSelection();
    } else if (actionId == QLatin1String("newGroup")) {
        openNewGroupDialog();
    } else if (actionId == QLatin1String("group")) {
        groupSelection();
    } else if (actionId == QLatin1String("ungroup")) {
        ungroupSelection();
    } else if (actionId == QLatin1String("hide")) {
        if (view_) {
            const QStringList paths = selectedPaths();
            if (!paths.isEmpty()) {
                view_->set_layers_visible(paths, false);
            }
        }
    } else if (actionId == QLatin1String("deleteHidden")) {
        if (view_) {
            view_->delete_hidden_layers();
        }
    } else if (actionId == QLatin1String("link")) {
        if (view_) {
            const QStringList paths = selectedPaths();
            if (!paths.isEmpty()) {
                view_->link_layers(paths, true);
            }
        }
    } else if (actionId == QLatin1String("selectLinked")) {
        if (view_) {
            const QString path = currentPath();
            if (!path.isEmpty()) {
                selectPaths(view_->select_linked(path), path);
            }
        }
    } else if (actionId == QLatin1String("moveUp")) {
        moveCurrent(1);
    } else if (actionId == QLatin1String("moveDown")) {
        moveCurrent(-1);
    } else if (actionId == QLatin1String("mergeDown") || actionId == QLatin1String("mergeVisible")
               || actionId == QLatin1String("mergeClippingMask")) {
        if (view_) {
            const QStringList paths = selectedPaths();
            const QString path = currentPath();
            if (actionId == QLatin1String("mergeDown")) {
                if (!paths.isEmpty()) {
                    view_->merge_layers(paths);
                }
            } else if (actionId == QLatin1String("mergeVisible")) {
                if (!path.isEmpty()) {
                    view_->merge_visible(path);
                }
            } else if (!path.isEmpty()) {
                view_->merge_clipping_mask(path);
            }
        }
    } else if (actionId == QLatin1String("flatten")) {
        if (view_) {
            view_->flatten_image();
        }
    } else if (actionId == QLatin1String("panelOptions")) {
        openPanelOptions();
    } else if (actionId.startsWith(QLatin1String("adjustment:"))) {
        if (view_) {
            view_->add_adjustment(actionId.mid(QLatin1String("adjustment:").size()));
        }
    } else {
        return false;
    }
    return true;
}

} // namespace pictura
