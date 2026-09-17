#include "layers_panel.h"

#include "icons.h"
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
#include <QtWidgets/QSpinBox>
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

// One bridge row, as read by refresh(). The model owns a tree of these.
struct LayerRow {
    QString path;
    int depth = 0;
    QString name;
    QString kind;
    bool visible = true;
    QString blend;
    int opacity = 255;
    int fill = 255;
    int lockBits = 0;
    int color = 0;
    bool clipping = false;
    bool hasMask = false;
    bool hasAdjustment = false;
    bool expandable = false;
    int childCount = 0;
    QImage thumbnail;
    QImage maskThumbnail;
};

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

QString blendName(const QString& key)
{
    for (const BlendEntry& entry : kBlends) {
        if (key == QLatin1String(entry.key)) {
            return QString::fromLatin1(entry.name);
        }
    }
    return key;
}

QString layerTooltip(const LayerRow& layer)
{
    return QStringLiteral("%1 (%2)").arg(layer.name, layer.kind);
}

// Panel Options thumbnail sizes by enum order (None/Small/Medium/Large).
constexpr std::array<int, 4> kThumbSizePx{0, 16, 24, 32};

} // namespace

// Frozen per-row roles (see `docs/dev/m39-panel-anatomy.md` §3.6). ClipBaseRole
// is panel-local: a row whose sibling displayed immediately above it is clipped
// underlines its name as the clipping base.
enum LayerRole {
    PathRole = Qt::UserRole + 1,
    DepthRole,
    KindRole,
    VisibleRole,
    BlendRole,
    OpacityRole,
    FillRole,
    LockRole,
    ColorRole,
    ClippingRole,
    ClipBaseRole,
    HasMaskRole,
    HasAdjustmentRole,
    ExpandableRole,
    ChildCountRole,
    ThumbnailRole,
    MaskThumbnailRole,
};

struct Node {
    LayerRow row;
    bool clipBase = false;
    Node* parent = nullptr;
    int rowInParent = 0;
    std::vector<std::unique_ptr<Node>> children;
};

class LayersModel : public QAbstractItemModel {
public:
    explicit LayersModel(QObject* parent = nullptr)
        : QAbstractItemModel(parent)
        , root_(std::make_unique<Node>())
    {
    }

    void setView(PictureView* view) { view_ = view; }

    void setRows(QVector<LayerRow> rows)
    {
        beginResetModel();
        root_ = std::make_unique<Node>();
        byPath_.clear();
        for (LayerRow& row : rows) {
            auto node = std::make_unique<Node>();
            node->row = std::move(row);
            const int slash = node->row.path.lastIndexOf(QLatin1Char('/'));
            Node* parent = root_.get();
            if (slash >= 0) {
                parent = byPath_.value(node->row.path.left(slash), root_.get());
            }
            node->parent = parent;
            node->rowInParent = static_cast<int>(parent->children.size());
            Node* raw = node.get();
            byPath_.insert(node->row.path, raw);
            parent->children.push_back(std::move(node));
        }
        for (Node* node : byPath_) {
            const Node* parent = node->parent;
            if (parent && node->rowInParent > 0
                && parent->children[node->rowInParent - 1]->row.clipping) {
                node->clipBase = true;
            }
        }
        endResetModel();
    }

    QModelIndex index(int row, int column,
                      const QModelIndex& parent = QModelIndex()) const override
    {
        if (!hasIndex(row, column, parent)) {
            return {};
        }
        Node* node = parentNode(parent);
        return createIndex(row, column, node->children[row].get());
    }

    QModelIndex parent(const QModelIndex& child) const override
    {
        if (!child.isValid()) {
            return {};
        }
        Node* node = static_cast<Node*>(child.internalPointer());
        if (!node || !node->parent || node->parent == root_.get()) {
            return {};
        }
        return createIndex(node->parent->rowInParent, 0, node->parent);
    }

    int rowCount(const QModelIndex& parent = QModelIndex()) const override
    {
        if (parent.column() > 0) {
            return 0;
        }
        const Node* node = parentNode(parent);
        return node ? static_cast<int>(node->children.size()) : 0;
    }

    int columnCount(const QModelIndex& parent = QModelIndex()) const override
    {
        return parent.column() > 0 ? 0 : 1;
    }

    QVariant data(const QModelIndex& index, int role) const override
    {
        const Node* node = nodeFor(index);
        if (!node) {
            return {};
        }
        const LayerRow& row = node->row;
        switch (role) {
        case Qt::DisplayRole:
        case Qt::EditRole:
            return row.name;
        case Qt::ToolTipRole:
            return layerTooltip(row);
        case Qt::CheckStateRole:
            return row.visible ? Qt::Checked : Qt::Unchecked;
        case PathRole:
            return row.path;
        case DepthRole:
            return row.depth;
        case KindRole:
            return row.kind;
        case VisibleRole:
            return row.visible;
        case BlendRole:
            return row.blend;
        case OpacityRole:
            return row.opacity;
        case FillRole:
            return row.fill;
        case LockRole:
            return row.lockBits;
        case ColorRole:
            return row.color;
        case ClippingRole:
            return row.clipping;
        case ClipBaseRole:
            return node->clipBase;
        case HasMaskRole:
            return row.hasMask;
        case HasAdjustmentRole:
            return row.hasAdjustment;
        case ExpandableRole:
            return row.expandable;
        case ChildCountRole:
            return row.childCount;
        case ThumbnailRole:
            return row.thumbnail;
        case MaskThumbnailRole:
            return row.maskThumbnail;
        default:
            return {};
        }
    }

    Qt::ItemFlags flags(const QModelIndex& index) const override
    {
        if (!index.isValid()) {
            return Qt::NoItemFlags;
        }
        return Qt::ItemIsEnabled | Qt::ItemIsSelectable | Qt::ItemIsEditable;
    }

    bool setData(const QModelIndex& index, const QVariant& value, int role) override
    {
        Node* node = index.isValid() ? static_cast<Node*>(index.internalPointer()) : nullptr;
        if (!node || !view_) {
            return false;
        }
        if (role == Qt::CheckStateRole) {
            const bool visible = value.toInt() == Qt::Checked;
            if (visible == node->row.visible) {
                return false;
            }
            return view_->set_layers_visible(QStringList{node->row.path}, visible) > 0;
        }
        if (role == Qt::EditRole) {
            const QString name = value.toString().trimmed();
            if (name.isEmpty() || name == node->row.name) {
                return false;
            }
            return view_->set_layer_name_path(node->row.path, name);
        }
        return false;
    }

    QVariant headerData(int section, Qt::Orientation orientation, int role) const override
    {
        if (orientation != Qt::Horizontal || role != Qt::DisplayRole) {
            return {};
        }
        return section == 0 ? QStringLiteral("Name") : QVariant();
    }

    QString pathForIndex(const QModelIndex& index) const
    {
        const Node* node = nodeFor(index);
        return node ? node->row.path : QString();
    }

    QModelIndex indexForPath(const QString& path) const
    {
        Node* node = byPath_.value(path, nullptr);
        if (!node || node == root_.get()) {
            return {};
        }
        return createIndex(node->rowInParent, 0, node);
    }

    /// Every node's path and visibility, for the solo snapshot.
    QHash<QString, bool> visibilityByPath() const
    {
        QHash<QString, bool> result;
        for (auto it = byPath_.cbegin(); it != byPath_.cend(); ++it) {
            result.insert(it.key(), it.value()->row.visible);
        }
        return result;
    }

    QStringList paths() const { return byPath_.keys(); }

private:
    Node* parentNode(const QModelIndex& parent) const
    {
        return parent.isValid() ? static_cast<Node*>(parent.internalPointer()) : root_.get();
    }

    const Node* nodeFor(const QModelIndex& index) const
    {
        if (!index.isValid()) {
            return nullptr;
        }
        return static_cast<const Node*>(index.internalPointer());
    }

    PictureView* view_ = nullptr;
    std::unique_ptr<Node> root_;
    QHash<QString, Node*> byPath_;
};

class LayerRowDelegate : public QStyledItemDelegate {
public:
    explicit LayerRowDelegate(QObject* parent = nullptr)
        : QStyledItemDelegate(parent)
    {
    }

    int thumbnailSize() const { return thumbnailSize_; }
    void setThumbnailSize(int size) { thumbnailSize_ = qMax(0, size); }

    /// The eye's hit-target inside a row's content rect (as painted).
    QRect eyeRect(const QRect& itemRect) const
    {
        const int width = qBound(12, itemRect.height(), 18);
        return QRect(itemRect.left() + 2, itemRect.top(), width, itemRect.height());
    }

    QSize sizeHint(const QStyleOptionViewItem& option, const QModelIndex& index) const override
    {
        Q_UNUSED(option);
        Q_UNUSED(index);
        return QSize(200, qMax(24, thumbnailSize_ + 8));
    }

    void paint(QPainter* painter, const QStyleOptionViewItem& option,
               const QModelIndex& index) const override
    {
        // Let the style paint the row background/selection only, then draw the
        // CS6 anatomy ourselves on top.
        QStyleOptionViewItem opt = option;
        initStyleOption(&opt, index);
        opt.text.clear();
        opt.icon = QIcon();
        const QWidget* widget = opt.widget;
        QStyle* style = widget ? widget->style() : QApplication::style();
        style->drawControl(QStyle::CE_ItemViewItem, &opt, painter, widget);

        painter->save();
        const QRect rect = option.rect;
        const int height = rect.height();
        const bool selected = option.state & QStyle::State_Selected;
        const QPalette& palette = option.palette;

        // Eye.
        const QRect eye = eyeRect(rect);
        paintEye(painter, eye, index.data(VisibleRole).toBool(), selected, palette);

        // Thumbnail (pixel layer) or folder glyph (group).
        const int thumb = qMax(0, thumbnailSize_);
        int x = eye.right() + 3;
        if (thumb > 0) {
            const QRect thumbRect(x, rect.top() + (height - thumb) / 2, thumb, thumb);
            if (index.data(KindRole).toString() == QLatin1String("group")) {
                const QPixmap glyph =
                    pictura::icon(QStringLiteral("layers.group")).pixmap(thumb, thumb);
                if (!glyph.isNull()) {
                    painter->drawPixmap(thumbRect, glyph);
                }
            } else {
                const QImage image = index.data(ThumbnailRole).value<QImage>();
                if (!image.isNull()) {
                    painter->drawImage(thumbRect, image);
                }
            }
        }
        x += thumb + 4;

        // Right-aligned badges: fx at the far edge, the mask thumbnail to its
        // left. A null icon/thumbnail is omitted.
        int right = rect.right() - 3;
        const QIcon fxIcon = pictura::icon(QStringLiteral("layers.fx"));
        const int badge = qMax(12, thumb > 0 ? thumb : 16);
        if (index.data(HasAdjustmentRole).toBool() && !fxIcon.isNull()) {
            const QPixmap badgePix = fxIcon.pixmap(badge, badge);
            if (!badgePix.isNull()) {
                painter->drawPixmap(
                    QRect(right - badge, rect.top() + (height - badge) / 2, badge, badge),
                    badgePix);
                right -= badge + 3;
            }
        }
        const QImage mask = index.data(MaskThumbnailRole).value<QImage>();
        if (!mask.isNull() && thumb > 0) {
            painter->drawImage(
                QRect(right - thumb, rect.top() + (height - thumb) / 2, thumb, thumb), mask);
            right -= thumb + 3;
        }

        // Name, with the extra clipping indent and the base underline.
        const int nameLeft = x + (index.data(ClippingRole).toBool() ? 12 : 0);
        const int nameRight = qMax(nameLeft, right - 4);
        QFont font = option.font;
        font.setUnderline(index.data(ClipBaseRole).toBool());
        painter->setFont(font);
        painter->setPen(selected ? palette.color(QPalette::HighlightedText)
                                 : palette.color(QPalette::Text));
        const QString elided =
            painter->fontMetrics().elidedText(index.data(Qt::DisplayRole).toString(),
                                              Qt::ElideRight, nameRight - nameLeft);
        painter->drawText(QRect(nameLeft, rect.top(), nameRight - nameLeft, height),
                          Qt::AlignVCenter | Qt::AlignLeft, elided);

        // Color-label swatch, right after the name when it fits.
        const QPixmap swatch = labelSwatch(index.data(ColorRole).toInt());
        if (!swatch.isNull()) {
            const int side = 10;
            const int sx = nameLeft + painter->fontMetrics().horizontalAdvance(elided) + 6;
            if (sx + side <= right) {
                painter->drawPixmap(
                    QRect(sx, rect.top() + (height - side) / 2, side, side), swatch);
            }
        }
        painter->restore();
    }

private:
    static void paintEye(QPainter* painter, const QRect& rect, bool visible, bool selected,
                         const QPalette& palette)
    {
        painter->save();
        painter->setRenderHint(QPainter::Antialiasing, true);
        QColor color =
            selected ? palette.color(QPalette::HighlightedText) : palette.color(QPalette::Text);
        if (!visible) {
            color.setAlpha(90);
        }
        painter->setPen(QPen(color, 1.4));
        painter->setBrush(Qt::NoBrush);
        const QRectF eye = QRectF(rect).adjusted(2, rect.height() * 0.28, -2,
                                                 -rect.height() * 0.28);
        painter->drawEllipse(eye);
        if (visible) {
            const qreal radius = qMin(eye.width(), eye.height()) * 0.25;
            painter->setBrush(color);
            painter->drawEllipse(eye.center(), radius, radius);
        } else {
            painter->drawLine(eye.topLeft(), eye.bottomRight());
        }
        painter->restore();
    }

    int thumbnailSize_ = 24;
};

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
    opacity_ = new QSpinBox(body);
    opacity_->setRange(0, 255);
    opacity_->setEnabled(false);
    opacity_->setToolTip(tr("Opacity"));
    fill_ = new QSpinBox(body);
    fill_->setRange(0, 255);
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

    // The lock strip sits below the blend/opacity/fill strip. Each button is one
    // flag; "All" is the derived three-bit set. The panel reflects state, so
    // toggling an individual flag off naturally unchecks "All".
    auto* locks = new QHBoxLayout();
    const auto makeLock = [this, body, locks](const QString& text, const QString& flag) {
        auto* button = new QToolButton(body);
        button->setText(text);
        button->setCheckable(true);
        button->setEnabled(false);
        button->setToolButtonStyle(Qt::ToolButtonTextOnly);
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
    lockTransparency_ = makeLock(tr("Transparency"), QStringLiteral("transparency"));
    lockPixels_ = makeLock(tr("Pixels"), QStringLiteral("pixels"));
    lockPosition_ = makeLock(tr("Position"), QStringLiteral("position"));
    lockAll_ = makeLock(tr("All"), QStringLiteral("all"));
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
    connect(opacity_, &QSpinBox::valueChanged, this, [this](int value) {
        if (syncing_ || !view_) {
            return;
        }
        const QStringList paths = selectedPaths();
        if (!paths.isEmpty()) {
            view_->set_layers_opacity(paths, value);
        }
    });
    connect(fill_, &QSpinBox::valueChanged, this, [this](int value) {
        if (syncing_ || !view_) {
            return;
        }
        const QStringList paths = selectedPaths();
        if (!paths.isEmpty()) {
            view_->set_layers_fill(paths, value);
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
    const std::array<QToolButton*, 4> lockButtons = {
        lockTransparency_, lockPixels_, lockPosition_, lockAll_};
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
        opacity_->setValue(valueIndex.data(OpacityRole).toInt());
        fill_->setValue(valueIndex.data(FillRole).toInt());
        const int lockBits = valueIndex.data(LockRole).toInt();
        lockTransparency_->setChecked((lockBits & 0x01) != 0);
        lockPixels_->setChecked((lockBits & 0x02) != 0);
        lockPosition_->setChecked((lockBits & 0x04) != 0);
        lockAll_->setChecked((lockBits & 0x07) == 0x07);

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
            const bool fullLock = (index.data(LockRole).toInt() & 0x07) == 0x07;
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
    const QString path = model_->pathForIndex(index);
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

// Shared with the row-menu self-test hook so the check exercises the real menu.
void LayersPanel::populateRowMenu(QMenu& menu, const QString& path, int color)
{
    QAction* rename = menu.addAction(tr("Rename"));
    connect(rename, &QAction::triggered, this, [this, path] {
        const QModelIndex target = model_->indexForPath(path);
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
    const QString path = model_->pathForIndex(index);
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

// --- M39 self-test hooks ----------------------------------------------------

bool LayersPanel::rowHasMaskForTest(const QString& path) const
{
    return model_ && model_->indexForPath(path).data(HasMaskRole).toBool();
}

bool LayersPanel::rowHasAdjustmentForTest(const QString& path) const
{
    return model_ && model_->indexForPath(path).data(HasAdjustmentRole).toBool();
}

bool LayersPanel::rowClippingForTest(const QString& path) const
{
    return model_ && model_->indexForPath(path).data(ClippingRole).toBool();
}

bool LayersPanel::rowClipBaseForTest(const QString& path) const
{
    return model_ && model_->indexForPath(path).data(ClipBaseRole).toBool();
}

bool LayersPanel::rowExpandableForTest(const QString& path) const
{
    return model_ && model_->indexForPath(path).data(ExpandableRole).toBool();
}

QString LayersPanel::rowToolTipForTest(const QString& path) const
{
    return model_ ? model_->indexForPath(path).data(Qt::ToolTipRole).toString() : QString();
}

bool LayersPanel::beginRenameForTest(const QString& path)
{
    const QModelIndex index = model_ ? model_->indexForPath(path) : QModelIndex();
    if (!index.isValid()) {
        return false;
    }
    tree_->setCurrentIndex(index);
    tree_->edit(index);
    return true;
}

QObject* LayersPanel::itemDelegateForTest() const
{
    return delegate_;
}

void LayersPanel::toggleSoloForTest(const QString& path)
{
    toggleSolo(path);
}

int LayersPanel::thumbSizeIndexForTest() const
{
    return thumbSizeIndex_;
}

int LayersPanel::thumbContentsForTest() const
{
    return thumbContents_;
}

bool LayersPanel::expandNewEffectsForTest() const
{
    return expandNewEffects_;
}

void LayersPanel::setOptionsForTest(int size, int contents, bool expand)
{
    thumbSizeIndex_ = qBound(0, size, 3);
    thumbContents_ = qBound(0, contents, 1);
    expandNewEffects_ = expand;
    thumbEntireDocument_ = thumbContents_ == 0;
    if (delegate_) {
        delegate_->setThumbnailSize(kThumbSizePx.at(thumbSizeIndex_));
    }
    persistOptions();
}

QStringList LayersPanel::panelMenuTextsForTest() const
{
    QStringList texts;
    QMenu* menu = panelMenu_ ? panelMenu_->menu() : nullptr;
    if (!menu) {
        return texts;
    }
    for (QAction* action : menu->actions()) {
        if (!action->isSeparator()) {
            texts.push_back(action->text());
        }
    }
    return texts;
}

QStringList LayersPanel::rowMenuTextsForTest()
{
    QMenu menu;
    populateRowMenu(menu, QString(), 0);
    QStringList texts;
    for (QAction* action : menu.actions()) {
        if (!action->isSeparator()) {
            texts.push_back(action->text());
        }
    }
    return texts;
}

QStringList LayersPanel::colorLabelTextsForTest()
{
    QMenu menu;
    addColorLabelActions(&menu, 0);
    QStringList texts;
    for (QAction* action : menu.actions()) {
        texts.push_back(action->text());
    }
    return texts;
}

} // namespace pictura
