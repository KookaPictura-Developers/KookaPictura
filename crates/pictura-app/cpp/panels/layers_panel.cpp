#include "layers_panel.h"

#include "layer_new_dialog.h"
#include "layers_filter_bar.h"
#include "layers_filter_proxy.h"
#include "layers_panel_internal.h"

#include "icons.h"
#include "percent_field.h"
#include "session.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QAbstractItemModel>
#include <QtCore/QEvent>
#include <QtCore/QHash>
#include <QtCore/QItemSelection>
#include <QtCore/QItemSelectionModel>
#include <QtCore/QMimeData>
#include <QtCore/QModelIndex>
#include <QtCore/QPoint>
#include <QtCore/QSet>
#include <QtCore/QSize>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtCore/QTimer>
#include <QtCore/QVariant>
#include <QtCore/QVector>
#include <QtGui/QAction>
#include <QtGui/QColor>
#include <QtGui/QDragEnterEvent>
#include <QtGui/QDropEvent>
#include <QtGui/QFont>
#include <QtGui/QIcon>
#include <QtGui/QImage>
#include <QtGui/QMouseEvent>
#include <QtGui/QPainter>
#include <QtGui/QPalette>
#include <QtGui/QPen>
#include <QtGui/QPixmap>
#include <QtWidgets/QAbstractItemView>
#include <QtWidgets/QApplication>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDialog>
#include <QtWidgets/QDialogButtonBox>
#include <QtWidgets/QFormLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QHeaderView>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QMenu>
#include <QtWidgets/QStyle>
#include <QtWidgets/QStyledItemDelegate>
#include <QtWidgets/QStyleOptionViewItem>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QTreeView>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QWidget>

#include <array>
#include <functional>
#include <memory>
#include <utility>

namespace pictura {

// CS6 sheet colors (PSD `lclr`), index 1..7; 0 is no label.
QColor layerLabelColor(int label)
{
    switch (label) {
    case 1:
        return QColor(255, 0, 0);
    case 2:
        return QColor(255, 153, 0);
    case 3:
        return QColor(255, 255, 0);
    case 4:
        return QColor(0, 204, 0);
    case 5:
        return QColor(0, 102, 255);
    case 6:
        return QColor(153, 0, 204);
    case 7:
        return QColor(153, 153, 153);
    default:
        return QColor();
    }
}

QString layerTooltip(const LayerRow& layer)
{
    return QStringLiteral("%1 (%2)").arg(layer.name, layer.kind);
}

QPixmap labelSwatch(int label)
{
    const QColor color = layerLabelColor(label);
    if (!color.isValid()) {
        return {};
    }
    QPixmap swatch(10, 10);
    swatch.fill(color);
    return swatch;
}

LayersPanel::LayersPanel(QWidget* parent)
    : QWidget(parent)
{
    QWidget* body = this;
    auto* layout = new QVBoxLayout(body);

    // Panel Options are session state (schema v3); clamp a corrupt store.
    const SessionState session = pictura::loadSession();
    thumbSizeIndex_ = qBound(0, session.layersThumbSize, 3);
    thumbContents_ = qBound(0, session.layersThumbContents, 1);
    expandNewEffects_ = session.layersExpandNewEffects;
    thumbEntireDocument_ = thumbContents_ == 0;

    filterBar_ = new LayerFilterBar(body);
    layout->addWidget(filterBar_);

    auto* controls = new QHBoxLayout();
    blend_ = new QComboBox(body);
    for (const BlendEntry& entry : kBlends) {
        blend_->addItem(QString::fromLatin1(entry.name), QString::fromLatin1(entry.key));
    }
    blend_->setEnabled(false);
    controls->addWidget(blend_, 1);
    opacity_ = new PercentField(tr("Opacity"), body);
    opacity_->setObjectName(QStringLiteral("layersOpacityField"));
    opacity_->setEnabled(false);
    opacity_->setToolTip(tr("Opacity"));
    controls->addWidget(opacity_);
    layout->addLayout(controls);

    // The lock strip sits directly above the layer list; Fill shares its row.
    // Each icon is one flag; "All" is the derived four-bit set. The panel
    // reflects state, so toggling an individual flag off unchecks "All".
    auto* locks = new QHBoxLayout();
    const auto makeLock = [this, body, locks](const QString& assetId, const QString& tooltip,
                                              const QString& flag) {
        auto* button = new QToolButton(body);
        button->setCheckable(true);
        button->setEnabled(false);
        button->setIcon(pictura::icon(assetId));
        button->setIconSize(QSize(20, 20));
        button->setAutoRaise(true);
        button->setToolTip(tooltip);
        locks->addWidget(button);
        connect(button, &QToolButton::toggled, this, [this, flag](bool on) {
            if (syncing_ || !view_) {
                return;
            }
            const QStringList paths = selectedPaths();
            if (!paths.isEmpty()) {
                view_->set_layers_lock(paths, flag, on);
            }
        });
        return button;
    };
    lockTransparency_ = makeLock(QStringLiteral("layers.lockAlpha"),
                                 tr("Lock Transparent Pixels"), QStringLiteral("transparency"));
    lockPixels_ = makeLock(QStringLiteral("layers.lockPaint"), tr("Lock Image Pixels"),
                           QStringLiteral("pixels"));
    lockPosition_ = makeLock(QStringLiteral("layers.lockPosition"), tr("Lock Position"),
                             QStringLiteral("position"));
    lockNesting_ = makeLock(QStringLiteral("layers.lockNesting"), tr("Lock Nesting"),
                            QStringLiteral("nesting"));
    lockNesting_->setVisible(false);
    lockAll_ = makeLock(QStringLiteral("layers.lockAll"), tr("Lock All"),
                        QStringLiteral("all"));
    locks->addStretch(1);
    fill_ = new PercentField(tr("Fill"), body);
    fill_->setObjectName(QStringLiteral("layersFillField"));
    fill_->setEnabled(false);
    fill_->setToolTip(tr("Fill"));
    locks->addWidget(fill_);
    layout->addLayout(locks);

    model_ = new LayersModel(this);
    proxy_ = new LayersFilterProxyModel(model_, this);

    tree_ = new LayersTreeView(body);
    tree_->setModel(proxy_);
    delegate_ = new LayerRowDelegate(tree_);
    delegate_->setThumbnailSize(kThumbSizePx.at(thumbSizeIndex_));
    tree_->setItemDelegate(delegate_);
    // The delegate draws the nesting indent and group chevron itself, so the
    // eye stays anchored at the panel's left edge for every depth.
    tree_->setRootIsDecorated(false);
    tree_->setIndentation(0);
    tree_->setItemsExpandable(true);
    tree_->setExpandsOnDoubleClick(false);
    // The viewport filter starts a rename only inside the name rect; Qt's own
    // DoubleClicked trigger would edit the whole row, so it is disabled.
    tree_->setEditTriggers(QAbstractItemView::NoEditTriggers);
    tree_->setAllColumnsShowFocus(true);
    tree_->setSelectionBehavior(QAbstractItemView::SelectRows);
    tree_->setSelectionMode(QAbstractItemView::ExtendedSelection);
    tree_->setUniformRowHeights(true);
    tree_->setHeaderHidden(true);
    tree_->header()->setSectionResizeMode(0, QHeaderView::Stretch);
    tree_->setContextMenuPolicy(Qt::CustomContextMenu);
    tree_->viewport()->installEventFilter(this);
    // Self-contained layer drag/drop: the tree resolves the drop to a target
    // path + mode and the panel turns it into one reparent/move undo step.
    tree_->setDragPathsProvider([this] {
        const QString path = currentPath();
        return path.isEmpty() ? QStringList{} : QStringList{path};
    });
    tree_->setPathResolver([this](const QModelIndex& index) { return pathForProxyIndex(index); });
    tree_->setDropHandler([this](const QString& dragged, const QString& target, int mode) {
        return view_ && view_->move_layer_to(dragged, target, mode);
    });
    tree_->setDropValidator([this](const QString& dragged, const QString& target, int mode) {
        return view_ && view_->can_move_layer_to(dragged, target, mode);
    });
    layout->addWidget(tree_, 1);

    auto* buttons = new QHBoxLayout();
    const auto stripIconButton = [body, buttons](const QString& objectName,
                                                 const QString& assetId) {
        auto* button = new QToolButton(body);
        button->setObjectName(objectName);
        button->setIcon(pictura::icon(assetId));
        button->setIconSize(QSize(20, 20));
        button->setAutoRaise(true);
        buttons->addWidget(button);
        return button;
    };

    // CS6 strip order. Link/fx/mask have icons but no behaviour yet, so they
    // stay disabled rather than pretending.
    auto* linkButton =
        stripIconButton(QStringLiteral("layersStripLink"), QStringLiteral("layers.link"));
    linkButton->setEnabled(false);
    linkButton->setToolTip(tr("Link Layers — not implemented yet"));
    auto* fxButton = stripIconButton(QStringLiteral("layersStripFx"), QStringLiteral("layers.fx"));
    fxButton->setEnabled(false);
    fxButton->setToolTip(tr("Layer Style — not implemented yet"));
    auto* maskButton =
        stripIconButton(QStringLiteral("layersStripMask"), QStringLiteral("layers.mask"));
    maskButton->setEnabled(false);
    maskButton->setToolTip(tr("Add Layer Mask — not implemented yet"));

    auto* addButton = stripIconButton(QStringLiteral("layersStripFillAdjustment"),
                                      QStringLiteral("layers.fillAdjustment"));
    addButton->setPopupMode(QToolButton::InstantPopup);
    addButton->setToolTip(tr("New Fill / Adjustment Layer"));
    auto* menu = new QMenu(addButton);
    // ponytail: opaque black; the toolbox foreground swatch is C++-only and not
    // yet plumbed to the bridge. Swap in the live color when it is.
    QAction* solidFill = menu->addAction(tr("Solid Color…"));
    connect(solidFill, &QAction::triggered, this, [this] {
        if (view_) {
            view_->add_solid_fill(0xff000000u);
        }
    });
    QAction* gradientFill = menu->addAction(tr("Gradient…"));
    connect(gradientFill, &QAction::triggered, this, [this] {
        if (view_) {
            view_->add_gradient_fill();
        }
    });
    menu->addSeparator();
    const QStringList kinds = {
        QStringLiteral("invert"),
        QStringLiteral("posterize"),
        QStringLiteral("threshold"),
        QStringLiteral("brightness-contrast"),
        QStringLiteral("hue-saturation"),
    };
    for (const QString& kind : kinds) {
        QAction* action = menu->addAction(kind);
        connect(action, &QAction::triggered, this, [this, kind] {
            if (view_) {
                view_->add_adjustment(kind);
            }
        });
    }
    addButton->setMenu(menu);

    auto* newGroupButton =
        stripIconButton(QStringLiteral("layersStripGroup"), QStringLiteral("layers.group"));
    newGroupButton->setToolTip(tr("New Group — drop a layer here to group it"));
    newGroupButton->setProperty("layerDropAction", QStringLiteral("group"));
    newGroupButton->setAcceptDrops(true);
    newGroupButton->installEventFilter(this);
    auto* newLayerButton =
        stripIconButton(QStringLiteral("layersStripNewLayer"), QStringLiteral("layers.newLayer"));
    newLayerButton->setToolTip(tr("New Layer — drop a layer here to duplicate it"));
    newLayerButton->setProperty("layerDropAction", QStringLiteral("duplicate"));
    newLayerButton->setAcceptDrops(true);
    newLayerButton->installEventFilter(this);
    auto* deleteButton =
        stripIconButton(QStringLiteral("layersStripDelete"), QStringLiteral("layers.delete"));
    deleteButton->setToolTip(tr("Delete — drop a layer here to delete it"));
    deleteButton->setProperty("layerDropAction", QStringLiteral("delete"));
    deleteButton->setAcceptDrops(true);
    deleteButton->installEventFilter(this);
    layout->addLayout(buttons);

    connect(tree_->selectionModel(), &QItemSelectionModel::currentChanged, this,
            [this](const QModelIndex&, const QModelIndex&) { syncControls(); });
    connect(tree_->selectionModel(), &QItemSelectionModel::selectionChanged, this,
            [this](const QItemSelection&, const QItemSelection&) { syncControls(); });
    connect(tree_, &QTreeView::expanded, this, [this](const QModelIndex& index) {
        const QString path = pathForProxyIndex(index);
        if (!path.isEmpty()) {
            expandedPaths_.insert(path);
        }
    });
    connect(tree_, &QTreeView::collapsed, this, [this](const QModelIndex& index) {
        expandedPaths_.remove(pathForProxyIndex(index));
    });
    connect(filterBar_, &LayerFilterBar::filterChanged, this, &LayersPanel::applyFilter);
    connect(blend_, &QComboBox::currentIndexChanged, this, [this](int index) {
        if (syncing_ || !view_) {
            return;
        }
        const QStringList paths = selectedPaths();
        if (!paths.isEmpty()) {
            view_->set_layers_blend(paths, blend_->itemData(index).toString());
        }
    });
    connect(opacity_, &PercentField::valueChanged, this, [this](double pct) {
        if (syncing_ || !view_) {
            return;
        }
        const QStringList paths = selectedPaths();
        if (!paths.isEmpty()) {
            view_->preview_layers_opacity(paths, qRound(pct * 255.0 / 100.0));
        }
    });
    connect(opacity_, &PercentField::valueCommitted, this, [this](double pct) {
        if (syncing_ || !view_) {
            return;
        }
        const QStringList paths = selectedPaths();
        if (!paths.isEmpty()) {
            view_->commit_layers_opacity(paths, qRound(pct * 255.0 / 100.0));
        }
    });
    connect(fill_, &PercentField::valueChanged, this, [this](double pct) {
        if (syncing_ || !view_) {
            return;
        }
        const QStringList paths = selectedPaths();
        if (!paths.isEmpty()) {
            view_->preview_layers_fill(paths, qRound(pct * 255.0 / 100.0));
        }
    });
    connect(fill_, &PercentField::valueCommitted, this, [this](double pct) {
        if (syncing_ || !view_) {
            return;
        }
        const QStringList paths = selectedPaths();
        if (!paths.isEmpty()) {
            view_->commit_layers_fill(paths, qRound(pct * 255.0 / 100.0));
        }
    });
    connect(tree_, &QTreeView::customContextMenuRequested, this,
            &LayersPanel::showContextMenu);
    connect(newGroupButton, &QToolButton::clicked, this, [this] {
        if (QApplication::keyboardModifiers().testFlag(Qt::AltModifier)) {
            openNewGroupDialog();
        } else {
            addGroupAt(currentPath());
        }
    });
    connect(newLayerButton, &QToolButton::clicked, this, [this] {
        if (QApplication::keyboardModifiers().testFlag(Qt::AltModifier)) {
            openNewLayerDialog();
        } else {
            addLayerAt(currentPath());
        }
    });
    connect(deleteButton, &QToolButton::clicked, this, [this] { deleteSelection(); });
}

void LayersPanel::setView(PictureView* view)
{
    if (viewConnection_) {
        QObject::disconnect(viewConnection_);
    }
    // A different view means a document switch: solo paths and the filter are
    // transient, so both reset.
    if (view_ != view) {
        clearSolo();
        const LayerFilter defaultFilter{true};
        if (filterBar_) {
            filterBar_->setFilter(defaultFilter);
        }
        if (proxy_) {
            proxy_->setFilter(defaultFilter);
        }
    }
    view_ = view;
    model_->setView(view);
    if (view_) {
        viewConnection_ = connect(view_, &PictureView::changed, this, [this] {
            QTimer::singleShot(0, this, [this] { refresh(); });
        });
    }
    refresh();
}

void LayersPanel::refresh()
{
    const QStringList selected = selectedPaths();
    const QString selectedPath = currentPath();
    // A Tab rename commits through `changed`; the queued refresh must keep the
    // next row's editor alive rather than dropping it on the model reset.
    const bool wasEditing = tree_->viewport()->findChild<QLineEdit*>() != nullptr;
    const int thumbSize = delegate_ ? delegate_->thumbnailSize() : 24;

    QVector<LayerRow> rows;
    if (view_) {
        const int count = view_->layer_row_count();
        rows.reserve(count);
        for (int i = 0; i < count; ++i) {
            LayerRow row;
            row.path = view_->layer_row_path(i);
            row.depth = view_->layer_row_depth(i);
            row.name = view_->layer_row_name(i);
            row.kind = view_->layer_row_kind(i);
            row.visible = view_->layer_row_visible(i);
            row.blend = view_->layer_row_blend(i);
            row.opacity = view_->layer_row_opacity(i);
            row.fill = view_->layer_row_fill(i);
            row.lockBits = view_->layer_row_lock(i);
            row.color = view_->layer_row_color(i);
            row.clipping = view_->layer_row_clipping(i);
            row.hasMask = view_->layer_row_has_mask(i);
            row.hasAdjustment = view_->layer_row_has_adjustment(i);
            row.expandable = view_->layer_row_expandable(i);
            row.childCount = view_->layer_row_child_count(i);
            row.linked = view_->layer_row_linked(i);
            row.placed = view_->layer_row_placed(i);
            row.thumbnail = view_->layer_row_thumbnail(i, thumbSize, thumbEntireDocument_);
            row.maskThumbnail = view_->layer_row_mask_thumbnail(i, thumbSize);
            rows.push_back(std::move(row));
        }
    }
    model_->setRows(std::move(rows));

    // Restore expansion by path; keep a path the filter hid (the document still
    // has it) so toggling the filter off restores its expansion. Iterate a copy:
    // `expand()` emits `expanded`, which re-inserts into the set.
    const QSet<QString> previousExpansion = expandedPaths_;
    QSet<QString> live;
    for (const QString& path : previousExpansion) {
        if (!model_->indexForPath(path).isValid()) {
            continue;
        }
        live.insert(path);
        const QModelIndex index = proxyIndexForPath(path);
        if (index.isValid()) {
            tree_->expand(index);
        }
    }
    expandedPaths_ = live;
    if (proxy_->filter().enabled && proxy_->hasActiveCriteria()) {
        expandMatchingGroups();
    }

    if (!selected.isEmpty()) {
        selectPaths(selected, selectedPath);
    }
    if (!tree_->currentIndex().isValid() && proxy_->rowCount() > 0) {
        tree_->setCurrentIndex(proxy_->index(0, 0));
    }
    if (wasEditing) {
        const QModelIndex editIndex = proxyIndexForPath(selectedPath);
        if (editIndex.isValid()) {
            tree_->setCurrentIndex(editIndex);
            tree_->edit(editIndex);
        }
    }
    syncControls();
}

int LayersPanel::currentLayer() const
{
    const QString path = currentPath();
    if (path.isEmpty() || path.contains(QLatin1Char('/'))) {
        return -1;
    }
    bool ok = false;
    const int index = path.toInt(&ok);
    return ok ? index : -1;
}

QString LayersPanel::currentPath() const
{
    return pathForProxyIndex(tree_->currentIndex());
}

void LayersPanel::selectLayer(int index)
{
    if (index < 0) {
        return;
    }
    selectPath(QString::number(index));
}

void LayersPanel::selectPath(const QString& path)
{
    if (path.isEmpty()) {
        return;
    }
    const QModelIndex index = proxyIndexForPath(path);
    if (index.isValid()) {
        tree_->setCurrentIndex(index);
    }
}

void LayersPanel::selectPaths(const QStringList& paths, const QString& current)
{
    if (!model_ || !tree_ || !tree_->selectionModel()) {
        return;
    }
    if (paths.isEmpty()) {
        tree_->selectionModel()->clearSelection();
        return;
    }
    QItemSelection selection;
    QModelIndex first;
    QModelIndex currentIndex;
    for (const QString& path : paths) {
        const QModelIndex index = proxyIndexForPath(path);
        if (!index.isValid()) {
            continue;
        }
        selection.select(index, index);
        if (!first.isValid()) {
            first = index;
        }
        if (path == current) {
            currentIndex = index;
        }
    }
    if (selection.isEmpty()) {
        return;
    }
    tree_->selectionModel()->select(
        selection, QItemSelectionModel::ClearAndSelect | QItemSelectionModel::Rows);
    if (!currentIndex.isValid()) {
        currentIndex = first;
    }
    tree_->selectionModel()->setCurrentIndex(currentIndex, QItemSelectionModel::NoUpdate);
    tree_->scrollTo(currentIndex);
}

QStringList LayersPanel::selectedPaths() const
{
    QStringList paths;
    if (!model_ || !tree_ || !tree_->selectionModel()) {
        return paths;
    }
    const QModelIndexList rows = tree_->selectionModel()->selectedRows(0);
    for (const QModelIndex& index : rows) {
        const QString path = pathForProxyIndex(index);
        if (!path.isEmpty()) {
            paths.push_back(path);
        }
    }
    return paths;
}

bool LayersPanel::eventFilter(QObject* watched, QEvent* event)
{
    if (auto* button = qobject_cast<QToolButton*>(watched)) {
        const QString dropAction = button->property("layerDropAction").toString();
        if (!dropAction.isEmpty()) {
            if (event->type() == QEvent::DragEnter) {
                auto* drag = static_cast<QDragEnterEvent*>(event);
                if (drag->mimeData()->hasFormat(kLayerMimeType)) {
                    drag->acceptProposedAction();
                    return true;
                }
            } else if (event->type() == QEvent::Drop) {
                auto* drop = static_cast<QDropEvent*>(event);
                if (drop->mimeData()->hasFormat(kLayerMimeType)) {
                    const QStringList paths =
                        QString::fromUtf8(drop->mimeData()->data(kLayerMimeType))
                            .split(QLatin1Char('\n'), Qt::SkipEmptyParts);
                    if (view_ && !paths.isEmpty()) {
                        if (dropAction == QLatin1String("delete")) {
                            view_->delete_layers(paths);
                        } else if (dropAction == QLatin1String("duplicate")) {
                            // A Background cannot be duplicated into a locked
                            // `Background copy`; convert it in place instead.
                            const QModelIndex row = model_->indexForPath(paths.first());
                            if (row.isValid()
                                && row.data(KindRole).toString()
                                    == QLatin1String("background")) {
                                openBackgroundConversion(paths.first());
                            } else {
                                view_->duplicate_layers(paths);
                            }
                        } else if (dropAction == QLatin1String("group")) {
                            view_->group_layers(paths);
                        }
                    }
                    drop->acceptProposedAction();
                    return true;
                }
            }
        }
    }
    if (watched == tree_->viewport() && event->type() == QEvent::MouseButtonPress) {
        auto* mouse = static_cast<QMouseEvent*>(event);
        if (mouse->button() == Qt::LeftButton) {
            const QPoint pos = mouse->position().toPoint();
            const QModelIndex index = tree_->indexAt(pos);
            if (index.isValid() && index.data(ExpandableRole).toBool()) {
                const QRect chevron =
                    delegate_->chevronRect(tree_->visualRect(index), index.data(DepthRole).toInt());
                if (chevron.contains(pos)) {
                    tree_->setExpanded(index, !tree_->isExpanded(index));
                    return true;
                }
            }
            if (index.isValid() && (mouse->modifiers() & Qt::ControlModifier)) {
                const QRect thumb = delegate_->thumbRect(tree_->visualRect(index), index);
                if (!thumb.isEmpty() && thumb.contains(pos)) {
                    // Ctrl+click the thumbnail selects the layer's pixels;
                    // consume the click so it starts no drag and opens no editor.
                    if (view_) {
                        view_->select_layer_alpha(pathForProxyIndex(index));
                    }
                    return true;
                }
            }
            if (index.isValid() && delegate_->eyeRect(tree_->visualRect(index)).contains(pos)) {
                const QString path = pathForProxyIndex(index);
                if (mouse->modifiers() & Qt::AltModifier) {
                    toggleSolo(path);
                    return true;
                }
                QStringList targets = selectedPaths();
                if (!targets.contains(path)) {
                    targets = QStringList{path};
                }
                if (view_ && !targets.isEmpty()) {
                    view_->set_layers_visible(targets, !index.data(VisibleRole).toBool());
                }
                return true;
            }
        }
    }
    if (watched == tree_->viewport() && event->type() == QEvent::MouseButtonDblClick) {
        auto* mouse = static_cast<QMouseEvent*>(event);
        if (mouse->button() == Qt::LeftButton) {
            const QPoint pos = mouse->position().toPoint();
            const QModelIndex index = tree_->indexAt(pos);
            if (index.isValid()) {
                const QString path = pathForProxyIndex(index);
                if (index.data(KindRole).toString() == QLatin1String("background")) {
                    // A control keeps its own action; any other content-band
                    // double-click converts the Background through the dialog.
                    const QRect vr = tree_->visualRect(index);
                    const bool control = delegate_->eyeRect(vr).contains(pos)
                        || delegate_->thumbRect(vr, index).contains(pos);
                    if (!control) {
                        openBackgroundConversion(path);
                    }
                } else if (delegate_->nameRect(tree_->visualRect(index), index).contains(pos)) {
                    tree_->edit(index);
                } else {
                    openLayerStyle(path);
                }
                return true;
            }
        }
    }
    return QWidget::eventFilter(watched, event);
}

void LayersPanel::openBackgroundConversion(const QString& path)
{
    if (!view_ || path.isEmpty()) {
        return;
    }
    LayerNewSpec spec;
    if (bgConvertArmed_) {
        if (!bgConvertAccept_) {
            return;
        }
        spec.name = bgConvertName_;
        spec.color = bgConvertColor_;
    } else {
        const QString defaultName = view_->next_layer_name(QStringLiteral("Layer"));
        if (!LayerNewDialog::getNameColor(this, defaultName, &spec)) {
            return;
        }
    }
    if (view_->convert_background(path, spec.name, spec.color)) {
        refresh();
        selectPath(path);
    }
}

void LayersPanel::openLayerStyle(const QString&)
{
    // ponytail: no Layer Style dialog exists yet (the fx button and Blending
    // Options are inert). A double-click outside the name is a deliberate no-op
    // affordance until one is built.
}

void LayersPanel::syncControls()
{
    const QModelIndex current = tree_->currentIndex();
    const QStringList paths = selectedPaths();
    if (view_) {
        view_->set_active_layer(paths.size() == 1 ? currentPath() : QString());
    }
    const bool active = view_ && !paths.isEmpty();
    syncing_ = true;
    blend_->setEnabled(false);
    opacity_->setEnabled(false);
    fill_->setEnabled(false);
    const std::array<QToolButton*, 5> lockButtons = {
        lockTransparency_, lockPixels_, lockPosition_, lockNesting_, lockAll_};
    for (QToolButton* button : lockButtons) {
        button->setEnabled(false);
        button->setChecked(false);
    }
    if (active) {
        // Show the current row's value; apply to the whole selection.
        const QModelIndex valueIndex =
            current.isValid() ? current : model_->indexForPath(paths.first());
        const int blend = blend_->findData(valueIndex.data(BlendRole).toString());
        blend_->setCurrentIndex(blend >= 0 ? blend : 0);
        opacity_->setValue(qRound(valueIndex.data(OpacityRole).toInt() * 100.0 / 255.0));
        fill_->setValue(qRound(valueIndex.data(FillRole).toInt() * 100.0 / 255.0));
        const int lockBits = valueIndex.data(LockRole).toInt();
        lockTransparency_->setChecked((lockBits & 0x01) != 0);
        lockPixels_->setChecked((lockBits & 0x02) != 0);
        lockPosition_->setChecked((lockBits & 0x04) != 0);
        lockNesting_->setChecked((lockBits & 0x08) != 0);
        lockAll_->setChecked((lockBits & 0x0F) == 0x0F);

        // Enable iff at least one selected node is eligible (frozen table).
        bool lockAny = false;
        bool blendAny = false;
        bool opacityAny = false;
        bool fillAny = false;
        for (const QString& path : paths) {
            const QModelIndex index = model_->indexForPath(path);
            const QString kind = index.data(KindRole).toString();
            const bool background = kind == QLatin1String("background");
            const bool group = kind == QLatin1String("group");
            const bool fullLock = (index.data(LockRole).toInt() & 0x0F) == 0x0F;
            if (!background) {
                lockAny = true;
            }
            if (!background && !fullLock) {
                blendAny = true;
                opacityAny = true;
            }
            if (!background && !fullLock && !group) {
                fillAny = true;
            }
        }
        for (QToolButton* button : lockButtons) {
            button->setEnabled(lockAny);
        }
        blend_->setEnabled(blendAny);
        opacity_->setEnabled(opacityAny);
        fill_->setEnabled(fillAny);
    }
    syncing_ = false;
}

void LayersPanel::openPanelOptions()
{
    QDialog dialog(this);
    dialog.setWindowTitle(tr("Layers Panel Options"));
    auto* form = new QFormLayout(&dialog);
    auto* sizeBox = new QComboBox(&dialog);
    sizeBox->addItem(tr("None"));
    sizeBox->addItem(tr("Small"));
    sizeBox->addItem(tr("Medium"));
    sizeBox->addItem(tr("Large"));
    sizeBox->setCurrentIndex(qBound(0, thumbSizeIndex_, 3));
    auto* contentsBox = new QComboBox(&dialog);
    contentsBox->addItem(tr("Entire Document"));
    contentsBox->addItem(tr("Layer Bounds"));
    contentsBox->setCurrentIndex(qBound(0, thumbContents_, 1));
    auto* expandBox = new QCheckBox(tr("Expand New Effects"), &dialog);
    expandBox->setChecked(expandNewEffects_);
    form->addRow(tr("Thumbnail Size"), sizeBox);
    form->addRow(tr("Thumbnail Contents"), contentsBox);
    form->addRow(expandBox);
    auto* buttons =
        new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, &dialog);
    form->addRow(buttons);
    connect(buttons, &QDialogButtonBox::accepted, &dialog, &QDialog::accept);
    connect(buttons, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
    if (dialog.exec() != QDialog::Accepted) {
        return;
    }
    thumbSizeIndex_ = sizeBox->currentIndex();
    thumbContents_ = contentsBox->currentIndex();
    expandNewEffects_ = expandBox->isChecked();
    thumbEntireDocument_ = thumbContents_ == 0;
    if (delegate_) {
        delegate_->setThumbnailSize(kThumbSizePx.at(thumbSizeIndex_));
    }
    persistOptions();
    refresh();
}

void LayersPanel::persistOptions()
{
    SessionState state = pictura::loadSession();
    state.layersThumbSize = thumbSizeIndex_;
    state.layersThumbContents = thumbContents_;
    state.layersExpandNewEffects = expandNewEffects_;
    state.schemaVersion = 3;
    pictura::saveSession(state);
}

QModelIndex LayersPanel::proxyIndexForPath(const QString& path) const
{
    if (!proxy_ || !model_) {
        return {};
    }
    return proxy_->mapFromSource(model_->indexForPath(path));
}

QString LayersPanel::pathForProxyIndex(const QModelIndex& index) const
{
    if (!model_ || !index.isValid()) {
        return {};
    }
    return model_->pathForIndex(proxy_->mapToSource(index));
}

void LayersPanel::expandMatchingGroups()
{
    std::function<void(const QModelIndex&)> walk = [this, &walk](const QModelIndex& parent) {
        const int rows = model_->rowCount(parent);
        for (int row = 0; row < rows; ++row) {
            const QModelIndex child = model_->index(row, 0, parent);
            if (model_->rowCount(child) == 0) {
                continue;
            }
            if (proxy_->hasMatchingDescendant(child)) {
                const QModelIndex proxyIndex = proxyIndexForPath(child.data(PathRole).toString());
                if (proxyIndex.isValid()) {
                    tree_->expand(proxyIndex);
                }
            }
            walk(child);
        }
    };
    walk(QModelIndex());
}

void LayersPanel::applyFilter(const LayerFilter& filter)
{
    if (!proxy_) {
        return;
    }
    proxy_->setFilter(filter);
    if (filter.enabled && proxy_->hasActiveCriteria()) {
        expandMatchingGroups();
    } else {
        for (const QString& path : expandedPaths_) {
            const QModelIndex index = proxyIndexForPath(path);
            if (index.isValid()) {
                tree_->expand(index);
            }
        }
    }
    syncControls();
}

} // namespace pictura
