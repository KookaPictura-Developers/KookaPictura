#include "layers_panel.h"

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
#include <memory>
#include <utility>

namespace pictura {

namespace {

struct BlendEntry {
    const char* key;
    const char* name;
};

constexpr std::array<BlendEntry, 27> kBlends{{
    {"norm", "Normal"},
    {"diss", "Dissolve"},
    {"dark", "Darken"},
    {"mul ", "Multiply"},
    {"idiv", "Color Burn"},
    {"lbrn", "Linear Burn"},
    {"dkCl", "Darker Color"},
    {"lite", "Lighten"},
    {"scrn", "Screen"},
    {"div ", "Color Dodge"},
    {"lddg", "Linear Dodge (Add)"},
    {"lgCl", "Lighter Color"},
    {"over", "Overlay"},
    {"sLit", "Soft Light"},
    {"hLit", "Hard Light"},
    {"vLit", "Vivid Light"},
    {"lLit", "Linear Light"},
    {"pLit", "Pin Light"},
    {"hMix", "Hard Mix"},
    {"diff", "Difference"},
    {"smud", "Exclusion"},
    {"fsub", "Subtract"},
    {"fdiv", "Divide"},
    {"hue ", "Hue"},
    {"sat ", "Saturation"},
    {"col ", "Color"},
    {"lum ", "Luminosity"},
}};

// CS6 sheet colors (PSD `lclr`), index 1..7; 0 is no label.
QColor labelColor(int label)
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

} // namespace

QString layerTooltip(const LayerRow& layer)
{
    return QStringLiteral("%1 (%2)").arg(layer.name, layer.kind);
}

QPixmap labelSwatch(int label)
{
    const QColor color = labelColor(label);
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

    auto* controls = new QHBoxLayout();
    blend_ = new QComboBox(body);
    for (const BlendEntry& entry : kBlends) {
        blend_->addItem(QString::fromLatin1(entry.name), QString::fromLatin1(entry.key));
    }
    blend_->setEnabled(false);
    opacity_ = new PercentField(body);
    opacity_->setObjectName(QStringLiteral("layersOpacityField"));
    opacity_->setEnabled(false);
    opacity_->setToolTip(tr("Opacity"));
    fill_ = new PercentField(body);
    fill_->setObjectName(QStringLiteral("layersFillField"));
    fill_->setEnabled(false);
    fill_->setToolTip(tr("Fill"));
    panelMenu_ = new QToolButton(body);
    panelMenu_->setObjectName(QStringLiteral("layersPanelMenu"));
    panelMenu_->setPopupMode(QToolButton::InstantPopup);
    panelMenu_->setAutoRaise(true);
    panelMenu_->setIcon(QApplication::style()->standardIcon(QStyle::SP_TitleBarMenuButton));
    panelMenu_->setToolTip(tr("Layers Panel Menu"));
    controls->addWidget(blend_, 1);
    controls->addWidget(opacity_);
    controls->addWidget(fill_);
    controls->addWidget(panelMenu_);
    layout->addLayout(controls);

    model_ = new LayersModel(this);

    tree_ = new QTreeView(body);
    tree_->setModel(model_);
    delegate_ = new LayerRowDelegate(tree_);
    delegate_->setThumbnailSize(kThumbSizePx.at(thumbSizeIndex_));
    tree_->setItemDelegate(delegate_);
    tree_->setRootIsDecorated(true);
    tree_->setItemsExpandable(true);
    tree_->setExpandsOnDoubleClick(false);
    tree_->setAllColumnsShowFocus(true);
    tree_->setSelectionBehavior(QAbstractItemView::SelectRows);
    tree_->setSelectionMode(QAbstractItemView::ExtendedSelection);
    tree_->setUniformRowHeights(true);
    tree_->setDragEnabled(false);
    tree_->setHeaderHidden(true);
    tree_->header()->setSectionResizeMode(0, QHeaderView::Stretch);
    tree_->setContextMenuPolicy(Qt::CustomContextMenu);
    tree_->viewport()->installEventFilter(this);
    layout->addWidget(tree_, 1);

    // The lock strip sits below the blend/opacity/fill strip. Each icon is one
    // flag; "All" is the derived four-bit set. The panel reflects state, so
    // toggling an individual flag off naturally unchecks "All".
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
    lockAll_ = makeLock(QStringLiteral("layers.lockAll"), tr("Lock All"),
                        QStringLiteral("all"));
    layout->addLayout(locks);

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
    newGroupButton->setToolTip(tr("New Group"));
    auto* newLayerButton =
        stripIconButton(QStringLiteral("layersStripNewLayer"), QStringLiteral("layers.newLayer"));
    newLayerButton->setToolTip(tr("New Layer"));
    auto* deleteButton =
        stripIconButton(QStringLiteral("layersStripDelete"), QStringLiteral("layers.delete"));
    deleteButton->setToolTip(tr("Delete"));
    layout->addLayout(buttons);

    // Panel menu: only commands M39 actually wires (no disabled placeholders).
    auto* panelMenu = new QMenu(panelMenu_);
    QAction* optionsAction = panelMenu->addAction(tr("Panel Options…"));
    connect(optionsAction, &QAction::triggered, this, &LayersPanel::openPanelOptions);
    panelMenu->addSeparator();
    const auto addCommand = [this, panelMenu](const QString& text, auto fn) {
        QAction* action = panelMenu->addAction(text);
        connect(action, &QAction::triggered, this, fn);
    };
    addCommand(tr("New Layer"), [this] { addLayerAt(currentPath()); });
    addCommand(tr("New Group"), [this] { addGroupAt(currentPath()); });
    addCommand(tr("Duplicate Layer(s)"), [this] { duplicateSelection(); });
    addCommand(tr("Delete Layer(s)"), [this] { deleteSelection(); });
    addCommand(tr("Group Layers"), [this] { groupSelection(); });
    addCommand(tr("Ungroup Layers"), [this] { ungroupSelection(); });
    addCommand(tr("Move Layer Up"), [this] { moveCurrent(1); });
    addCommand(tr("Move Layer Down"), [this] { moveCurrent(-1); });
    panelMenu_->setMenu(panelMenu);

    connect(tree_->selectionModel(), &QItemSelectionModel::currentChanged, this,
            [this](const QModelIndex&, const QModelIndex&) { syncControls(); });
    connect(tree_->selectionModel(), &QItemSelectionModel::selectionChanged, this,
            [this](const QItemSelection&, const QItemSelection&) { syncControls(); });
    connect(tree_, &QTreeView::expanded, this, [this](const QModelIndex& index) {
        const QString path = model_->pathForIndex(index);
        if (!path.isEmpty()) {
            expandedPaths_.insert(path);
        }
    });
    connect(tree_, &QTreeView::collapsed, this, [this](const QModelIndex& index) {
        expandedPaths_.remove(model_->pathForIndex(index));
    });
    connect(blend_, &QComboBox::currentIndexChanged, this, [this](int index) {
        if (syncing_ || !view_) {
            return;
        }
        const QStringList paths = selectedPaths();
        if (!paths.isEmpty()) {
            view_->set_layers_blend(paths, blend_->itemData(index).toString());
        }
    });
    connect(opacity_, &PercentField::valueChanged, this, [this](int pct) {
        if (syncing_ || !view_) {
            return;
        }
        const QStringList paths = selectedPaths();
        if (!paths.isEmpty()) {
            view_->set_layers_opacity(paths, qRound(pct * 255.0 / 100.0));
        }
    });
    connect(fill_, &PercentField::valueChanged, this, [this](int pct) {
        if (syncing_ || !view_) {
            return;
        }
        const QStringList paths = selectedPaths();
        if (!paths.isEmpty()) {
            view_->set_layers_fill(paths, qRound(pct * 255.0 / 100.0));
        }
    });
    connect(tree_, &QTreeView::customContextMenuRequested, this,
            &LayersPanel::showContextMenu);
    connect(newGroupButton, &QToolButton::clicked, this, [this] { addGroupAt(currentPath()); });
    connect(newLayerButton, &QToolButton::clicked, this, [this] { addLayerAt(currentPath()); });
    connect(deleteButton, &QToolButton::clicked, this, [this] { deleteSelection(); });
}

void LayersPanel::setView(PictureView* view)
{
    if (viewConnection_) {
        QObject::disconnect(viewConnection_);
    }
    // A different view means a document switch: solo paths are stale.
    if (view_ != view) {
        clearSolo();
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
            row.thumbnail = view_->layer_row_thumbnail(i, thumbSize, thumbEntireDocument_);
            row.maskThumbnail = view_->layer_row_mask_thumbnail(i, thumbSize);
            rows.push_back(std::move(row));
        }
    }
    model_->setRows(std::move(rows));

    // Restore expansion by path; drop paths the structural change invalidated.
    // Iterate a copy: `expand()` emits `expanded`, which re-inserts into the set.
    const QSet<QString> previousExpansion = expandedPaths_;
    QSet<QString> live;
    for (const QString& path : previousExpansion) {
        const QModelIndex index = model_->indexForPath(path);
        if (index.isValid()) {
            tree_->expand(index);
            live.insert(path);
        }
    }
    expandedPaths_ = live;

    if (!selected.isEmpty()) {
        selectPaths(selected, selectedPath);
    }
    if (!tree_->currentIndex().isValid() && model_->rowCount() > 0) {
        tree_->setCurrentIndex(model_->index(0, 0));
    }
    if (wasEditing) {
        const QModelIndex editIndex = model_->indexForPath(selectedPath);
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
    return model_ ? model_->pathForIndex(tree_->currentIndex()) : QString();
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
    const QModelIndex index = model_->indexForPath(path);
    if (index.isValid()) {
        tree_->setCurrentIndex(index);
    }
}

void LayersPanel::selectPaths(const QStringList& paths, const QString& current)
{
    if (!model_ || paths.isEmpty()) {
        return;
    }
    QItemSelection selection;
    QModelIndex first;
    QModelIndex currentIndex;
    for (const QString& path : paths) {
        const QModelIndex index = model_->indexForPath(path);
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
        const QString path = model_->pathForIndex(index);
        if (!path.isEmpty()) {
            paths.push_back(path);
        }
    }
    return paths;
}

bool LayersPanel::eventFilter(QObject* watched, QEvent* event)
{
    if (watched == tree_->viewport() && event->type() == QEvent::MouseButtonPress) {
        auto* mouse = static_cast<QMouseEvent*>(event);
        if (mouse->button() == Qt::LeftButton) {
            const QModelIndex index = tree_->indexAt(mouse->position().toPoint());
            if (index.isValid()
                && delegate_->eyeRect(tree_->visualRect(index))
                       .contains(mouse->position().toPoint())) {
                const QString path = model_->pathForIndex(index);
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
    return QWidget::eventFilter(watched, event);
}

void LayersPanel::syncControls()
{
    const QModelIndex current = tree_->currentIndex();
    const QStringList paths = selectedPaths();
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

} // namespace pictura
