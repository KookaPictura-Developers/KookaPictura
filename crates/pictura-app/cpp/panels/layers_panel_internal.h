#pragma once

#include "layers_panel.h"

#include "icons.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QAbstractItemModel>
#include <QtCore/QHash>
#include <QtCore/QModelIndex>
#include <QtCore/QRect>
#include <QtCore/QSize>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtCore/QVariant>
#include <QtCore/QVector>
#include <QtGui/QColor>
#include <QtGui/QFont>
#include <QtGui/QIcon>
#include <QtGui/QImage>
#include <QtGui/QPainter>
#include <QtGui/QPalette>
#include <QtGui/QPen>
#include <QtGui/QPixmap>
#include <QtWidgets/QApplication>
#include <QtWidgets/QStyle>
#include <QtWidgets/QStyledItemDelegate>
#include <QtWidgets/QStyleOptionViewItem>
#include <QtWidgets/QTreeView>

#include <array>
#include <memory>
#include <vector>

namespace pictura {

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

QString layerTooltip(const LayerRow& layer);

QPixmap labelSwatch(int label);

// Panel Options thumbnail sizes by enum order (None/Small/Medium/Large).
constexpr std::array<int, 4> kThumbSizePx{0, 16, 24, 32};

// The 27 CS6 blend entries: 4-byte PSD key plus display name. Shared by the
// blend combo and the filter bar's Mode dimension.
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

        // Two-pass: materialize every node first, then attach by parent path.
        // A forward reference (child before parent) or a partial list therefore
        // cannot silently drop the row to `root_` mid-build, and well-formed
        // pre-order input still attaches in encounter order.
        std::vector<std::unique_ptr<Node>> nodes;
        nodes.reserve(static_cast<size_t>(rows.size()));
        for (LayerRow& row : rows) {
            auto node = std::make_unique<Node>();
            node->row = std::move(row);
            byPath_.insert(node->row.path, node.get());
            nodes.push_back(std::move(node));
        }
        for (auto& node : nodes) {
            const int slash = node->row.path.lastIndexOf(QLatin1Char('/'));
            Node* parent = root_.get();
            if (slash >= 0) {
                if (Node* found = byPath_.value(node->row.path.left(slash), nullptr)) {
                    parent = found;
                }
            }
            node->parent = parent;
            node->rowInParent = static_cast<int>(parent->children.size());
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

// A QTreeView that draws no branch indicators: the row delegate owns the
// nesting indentation and the expand/collapse chevron, so the eye can stay
// anchored at the panel's left edge.
class LayersTreeView : public QTreeView {
public:
    explicit LayersTreeView(QWidget* parent = nullptr)
        : QTreeView(parent)
    {
    }

protected:
    void drawBranches(QPainter*, const QRect&, const QModelIndex&) const override {}
};

class LayerRowDelegate : public QStyledItemDelegate {
public:
    explicit LayerRowDelegate(QObject* parent = nullptr)
        : QStyledItemDelegate(parent)
    {
    }

    int thumbnailSize() const { return thumbnailSize_; }
    void setThumbnailSize(int size) { thumbnailSize_ = qMax(0, size); }

    // Fixed eye gutter and per-level content indent, in row-local pixels.
    static constexpr int kEyeColumn = 22;
    static constexpr int kIndent = 14;
    static constexpr int kChevronWidth = 14;

    /// The eye's hit-target inside a row's content rect (as painted).
    QRect eyeRect(const QRect& itemRect) const
    {
        const int width = qBound(12, itemRect.height(), 18);
        return QRect(itemRect.left() + 2, itemRect.top(), width, itemRect.height());
    }

    /// The expand/collapse chevron's hit-target for a row at `depth`.
    QRect chevronRect(const QRect& itemRect, int depth) const
    {
        const int side = qMin(kChevronWidth, itemRect.height());
        const int left = itemRect.left() + kEyeColumn + qMax(0, depth) * kIndent;
        return QRect(left, itemRect.top() + (itemRect.height() - side) / 2, side, side);
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

        const int depth = index.data(DepthRole).toInt();
        // Eye, anchored at the panel's left edge for every depth.
        const QRect eye = eyeRect(rect);
        paintEye(painter, eye, index.data(VisibleRole).toBool(), selected, palette);

        // Content (chevron, clipping glyph, thumbnail, name) is indented by
        // depth from the fixed eye gutter; the chevron slot is always reserved
        // so group and layer thumbnails align.
        const int thumb = qMax(0, thumbnailSize_);
        int x = rect.left() + kEyeColumn + qMax(0, depth) * kIndent;
        if (index.data(ExpandableRole).toBool()) {
            const auto* treeView = qobject_cast<const QTreeView*>(opt.widget);
            const bool expanded = treeView && treeView->isExpanded(index);
            paintChevron(painter, chevronRect(rect, depth), expanded,
                         selected ? palette.color(QPalette::HighlightedText)
                                  : palette.color(QPalette::Text));
        }
        x += kChevronWidth;
        if (index.data(ClippingRole).toBool()) {
            const int side = qMax(10, thumb > 0 ? thumb - 8 : 12);
            const QPixmap clip =
                pictura::icon(QStringLiteral("layers.clipMask")).pixmap(side, side);
            if (!clip.isNull()) {
                painter->drawPixmap(
                    QRect(x, rect.top() + (height - side) / 2, side, side), clip);
                x += side + 2;
            }
        }
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
    static void paintChevron(QPainter* painter, const QRect& rect, bool expanded,
                             const QColor& color)
    {
        painter->save();
        painter->setRenderHint(QPainter::Antialiasing, true);
        painter->setPen(QPen(color, 1.5, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
        const QPointF c = QRectF(rect).center();
        const qreal s = 3.0;
        if (expanded) {
            painter->drawLine(QPointF(c.x() - s, c.y() - s * 0.5),
                              QPointF(c.x(), c.y() + s * 0.5));
            painter->drawLine(QPointF(c.x(), c.y() + s * 0.5),
                              QPointF(c.x() + s, c.y() - s * 0.5));
        } else {
            painter->drawLine(QPointF(c.x() - s * 0.5, c.y() - s),
                              QPointF(c.x() + s * 0.5, c.y()));
            painter->drawLine(QPointF(c.x() + s * 0.5, c.y()),
                              QPointF(c.x() - s * 0.5, c.y() + s));
        }
        painter->restore();
    }

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

} // namespace pictura
