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
#include <QtGui/QCursor>
#include <QtGui/QFont>
#include <QtGui/QIcon>
#include <QtGui/QImage>
#include <QtGui/QPainter>
#include <QtGui/QPalette>
#include <QtGui/QPen>
#include <QtGui/QPixmap>
#include <QtGui/QRegion>
#include <QtWidgets/QApplication>
#include <QtWidgets/QStyle>
#include <QtCore/QMimeData>
#include <QtGui/QDrag>
#include <QtGui/QDragEnterEvent>
#include <QtGui/QDragLeaveEvent>
#include <QtGui/QDropEvent>
#include <QtGui/QPaintEvent>
#include <QtWidgets/QStyledItemDelegate>
#include <QtWidgets/QStyleOptionViewItem>
#include <QtWidgets/QTreeView>

#include <array>
#include <functional>
#include <memory>
#include <vector>

namespace pictura {

/// MIME type carrying dragged layer paths between the tree and the strip buttons.
inline constexpr char kLayerMimeType[] = "application/x-pictura-layer";

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
    bool linked = false;
    bool placed = false;
    QImage thumbnail;
    QImage maskThumbnail;
    int documentWidth = 0;
    int documentHeight = 0;
};

QString layerTooltip(const LayerRow& layer);

// CS6 sheet color for a label index (1..7); an invalid color for 0/unknown.
QColor layerLabelColor(int label);

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
    {"colr", "Color"},
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
    LayerRowLinkedRole,
    LayerRowPlacedRole,
    DocumentWidthRole,
    DocumentHeightRole,
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
        case LayerRowLinkedRole:
            return row.linked;
        case LayerRowPlacedRole:
            return row.placed;
        case DocumentWidthRole:
            return row.documentWidth;
        case DocumentHeightRole:
            return row.documentHeight;
        default:
            return {};
        }
    }

    Qt::ItemFlags flags(const QModelIndex& index) const override
    {
        if (!index.isValid()) {
            // The invalid parent is the top-level drop surface; without the
            // flag Qt computes no AboveItem/BelowItem indicator for root rows.
            return Qt::ItemIsDropEnabled;
        }
        return Qt::ItemIsEnabled | Qt::ItemIsSelectable | Qt::ItemIsEditable
            | Qt::ItemIsDragEnabled | Qt::ItemIsDropEnabled;
    }

    // Drag-and-drop capability virtuals. Without these the view's private
    // `canDrop()` is false, so `QTreeView::dragMoveEvent` never computes a real
    // `dropIndicatorPosition` and every drop reads the stale position.
    QStringList mimeTypes() const override
    {
        return {QString::fromLatin1(kLayerMimeType)};
    }

    Qt::DropActions supportedDropActions() const override { return Qt::MoveAction; }

    bool canDropMimeData(const QMimeData*, Qt::DropAction, int, int,
                         const QModelIndex&) const override
    {
        return true;
    }

    bool setData(const QModelIndex& index, const QVariant& value, int role) override
    {
        Node* node = index.isValid() ? static_cast<Node*>(index.internalPointer()) : nullptr;
        if (!node || !view_) {
            return false;
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

// A QTreeView that draws no branch indicators (the row delegate owns the
// nesting indentation and the disclosure icon, so the eye stays anchored at the
// panel's left edge) and that owns a self-contained layer drag/drop gesture: a
// drag carries the current row's path and a drop resolves the row under the
// cursor to a target path plus a mode (0 above, 1 below, 2 into).
class LayersTreeView : public QTreeView {
public:
    explicit LayersTreeView(QWidget* parent = nullptr)
        : QTreeView(parent)
    {
        setDragEnabled(true);
        setAcceptDrops(true);
        // The view draws the CS6 drop indicator itself; Qt's stock primitive is
        // off so only the custom line/outline shows.
        setDropIndicatorShown(false);
        setDragDropMode(QAbstractItemView::DragDrop);
        setDefaultDropAction(Qt::MoveAction);
    }

    void setDragPathsProvider(std::function<QStringList()> provider)
    {
        dragPaths_ = std::move(provider);
    }
    void setPathResolver(std::function<QString(const QModelIndex&)> resolver)
    {
        pathForIndex_ = std::move(resolver);
    }
    void setDropHandler(std::function<bool(const QString&, const QString&, int)> handler)
    {
        dropHandler_ = std::move(handler);
    }
    // Dry-run predicate consulted before a drop is highlighted or committed.
    // Returns true when the (dragged, target, mode) move is acceptable.
    void setDropValidator(std::function<bool(const QString&, const QString&, int)> validator)
    {
        dropValidator_ = std::move(validator);
    }

    // Closed-hand cursor for the duration of a drag, restored when it ends.
    void enterDragCursor()
    {
        dragCursor_ = viewport()->cursor();
        viewport()->setCursor(Qt::ClosedHandCursor);
    }
    void leaveDragCursor() { viewport()->setCursor(dragCursor_); }
    int cursorShapeForTest() const { return static_cast<int>(viewport()->cursor().shape()); }

    // Self-test hooks: state() is protected on QAbstractItemView, and a model
    // reset can leave the view in EditingState with no live editor, which wedges
    // the next edit.
    int editStateForTest() const { return static_cast<int>(state()); }
    void resetEditStateForTest() { setState(NoState); }
    int dropIndicatorForTest() const { return static_cast<int>(dropIndicatorPosition()); }
    /// Whether the custom CS6 indicator currently has a valid target.
    bool dropIndicatorShownForTest() const { return dropMode_ >= 0; }
    /// 0 = none, 1 = sibling line, 2 = drop-into outline.
    int dropIndicatorKindForTest() const
    {
        if (dropMode_ < 0 || dropRect_.isEmpty()) {
            return 0;
        }
        return dropMode_ == 2 ? 2 : 1;
    }
    int dropModeAtForTest(const QPoint& pos) const
    {
        int mode = 0;
        dropTargetFor(pos, &mode);
        return mode;
    }

protected:
    void drawBranches(QPainter*, const QRect&, const QModelIndex&) const override {}

    // The CS6 drop indicator: a thin blue line at a sibling edge, or a thin
    // blue outline around a group row for a drop-into. Drawn over the base
    // paint so Qt's own primitive (disabled in the ctor) never competes.
    void paintEvent(QPaintEvent* event) override
    {
        QTreeView::paintEvent(event);
        if (dropMode_ < 0 || dropRect_.isEmpty()) {
            return;
        }
        QPainter painter(viewport());
        painter.setPen(QPen(QColor(0x33, 0x99, 0xDD), 1));
        painter.setBrush(Qt::NoBrush);
        if (dropMode_ == 2) {
            painter.drawRect(dropRect_.adjusted(0, 0, -1, -1));
        } else {
            const int y = dropMode_ == 0 ? dropRect_.top() : dropRect_.bottom();
            painter.drawLine(dropRect_.left(), y, dropRect_.right(), y);
        }
    }

    void startDrag(Qt::DropActions) override
    {
        if (!dragPaths_) {
            return;
        }
        const QStringList paths = dragPaths_();
        if (paths.isEmpty()) {
            return;
        }
        auto* mime = new QMimeData();
        mime->setData(kLayerMimeType, paths.join(QLatin1Char('\n')).toUtf8());
        auto* drag = new QDrag(this);
        drag->setMimeData(mime);
        enterDragCursor();
        drag->exec(Qt::MoveAction);
        leaveDragCursor();
    }

    void dragEnterEvent(QDragEnterEvent* event) override
    {
        if (event->mimeData()->hasFormat(kLayerMimeType)) {
            // Enter the dragging state so the drop indicator paints (the base
            // implementation would do this, but we handle the drag ourselves).
            setState(QAbstractItemView::DraggingState);
            event->acceptProposedAction();
        } else {
            QTreeView::dragEnterEvent(event);
        }
    }

    void dragLeaveEvent(QDragLeaveEvent* event) override
    {
        clearDropIndicator();
        QTreeView::dragLeaveEvent(event);
    }

    void dragMoveEvent(QDragMoveEvent* event) override
    {
        if (!event->mimeData()->hasFormat(kLayerMimeType) || !dropValidator_ || !pathForIndex_) {
            QTreeView::dragMoveEvent(event);
            return;
        }
        // Let Qt resolve the row under the cursor and its drop position first;
        // then veto invalid targets so the indicator only marks a legal drop.
        QTreeView::dragMoveEvent(event);
        int mode = 0;
        const QPoint pos = event->position().toPoint();
        const QString target = dropTargetFor(pos, &mode);
        const QStringList dragged =
            QString::fromUtf8(event->mimeData()->data(kLayerMimeType))
                .split(QLatin1Char('\n'), Qt::SkipEmptyParts);
        if (!dragged.isEmpty() && dropValidator_(dragged.first(), target, mode)) {
            const QModelIndex index = indexAt(pos);
            dropRect_ = index.isValid() ? visualRect(index)
                                        : QRect(0, pos.y(), viewport()->width(), 1);
            dropMode_ = mode;
            viewport()->update();
            event->acceptProposedAction();
        } else {
            clearDropIndicator();
            event->ignore();
        }
    }

    void dropEvent(QDropEvent* event) override
    {
        clearDropIndicator();
        if (!event->mimeData()->hasFormat(kLayerMimeType) || !dropHandler_ || !pathForIndex_) {
            QTreeView::dropEvent(event);
            return;
        }
        int mode = 0;
        const QString target = dropTargetFor(event->position().toPoint(), &mode);
        const QStringList dragged =
            QString::fromUtf8(event->mimeData()->data(kLayerMimeType))
                .split(QLatin1Char('\n'), Qt::SkipEmptyParts);
        // ponytail: only the current row is dragged; a multi-row drag would need
        // path re-sequencing as each move shifts the survivors.
        if (dragged.isEmpty() || (dropValidator_ && !dropValidator_(dragged.first(), target, mode))) {
            event->ignore();
            return;
        }
        dropHandler_(dragged.first(), target, mode);
        event->setDropAction(Qt::MoveAction);
        event->accept();
    }

private:
    // Resolve the drop row under `pos` to a target path plus mode (0 above,
    // 1 below, 2 into an item); an invalid index is the empty-root target.
    QString dropTargetFor(const QPoint& pos, int* mode) const
    {
        const QModelIndex index = indexAt(pos);
        *mode = dropPositionFor(pos, index);
        return index.isValid() ? pathForIndex_(index) : QString();
    }

    // Mirror Qt's 2 px AboveItem/BelowItem margin rule ourselves: the stock
    // indicator is disabled, and Qt leaves `dropIndicatorPosition` stale then.
    int dropPositionFor(const QPoint& pos, const QModelIndex& index) const
    {
        if (!index.isValid()) {
            return 1;
        }
        const QRect rect = visualRect(index);
        if (pos.y() - rect.top() < 2) {
            return 0;
        }
        if (rect.bottom() - pos.y() < 2) {
            return 1;
        }
        if (rect.contains(pos, true)) {
            return 2;
        }
        return 1;
    }

    void clearDropIndicator()
    {
        if (dropMode_ < 0 && dropRect_.isNull()) {
            return;
        }
        dropMode_ = -1;
        dropRect_ = QRect();
        viewport()->update();
    }

    std::function<QStringList()> dragPaths_;
    std::function<QString(const QModelIndex&)> pathForIndex_;
    std::function<bool(const QString&, const QString&, int)> dropHandler_;
    std::function<bool(const QString&, const QString&, int)> dropValidator_;
    QCursor dragCursor_;
    QRect dropRect_;
    int dropMode_ = -1;
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
    static constexpr int kEyeColumn = 26;
    static constexpr int kIndent = 14;
    static constexpr int kChevronWidth = 16;

    /// The eye's hit-target inside a row's content rect (as painted): a square
    /// glyph centred in the fixed gutter with equal left/right padding.
    QRect eyeRect(const QRect& itemRect) const
    {
        const int width = qBound(14, itemRect.height(), 20);
        return QRect(itemRect.left() + (kEyeColumn - width) / 2, itemRect.top(), width,
                     itemRect.height());
    }

    /// The lock badge's rect at a row's right edge (as painted).
    QRect lockRect(const QRect& itemRect) const
    {
        const int side = qMax(12, thumbnailSize() > 0 ? thumbnailSize() : 16);
        return QRect(itemRect.right() - 3 - side,
                     itemRect.top() + (itemRect.height() - side) / 2, side, side);
    }

    /// The x where a row's content begins: the eye gutter plus the per-depth
    /// indent, plus the chevron slot only for expandable rows. Shared by
    /// thumbRect, nameRect, and paint() so the three never disagree.
    int contentLeft(const QRect& itemRect, const QModelIndex& index) const
    {
        const int depth = index.data(DepthRole).toInt();
        int x = itemRect.left() + kEyeColumn + qMax(0, depth) * kIndent;
        if (index.data(ExpandableRole).toBool()) {
            x += kChevronWidth;
        }
        return x;
    }

    /// The expand/collapse chevron's hit-target for a row at `depth`.
    QRect chevronRect(const QRect& itemRect, int depth) const
    {
        const int side = qMin(kChevronWidth, itemRect.height());
        const int left = itemRect.left() + kEyeColumn + qMax(0, depth) * kIndent;
        return QRect(left, itemRect.top() + (itemRect.height() - side) / 2, side, side);
    }

    /// Letterbox a square thumbnail `box` to the document's aspect ratio, so a
    /// wide or tall document is not stretched. A group (folder glyph) or a
    /// zero-size document keeps the square box.
    QRect letterboxedThumb(const QRect& box, const QModelIndex& index) const
    {
        if (box.isEmpty() || index.data(KindRole).toString() == QLatin1String("group")) {
            return box;
        }
        const int documentWidth = index.data(DocumentWidthRole).toInt();
        const int documentHeight = index.data(DocumentHeightRole).toInt();
        if (documentWidth <= 0 || documentHeight <= 0) {
            return box;
        }
        int width = box.width();
        int height = box.height();
        if (static_cast<qint64>(documentWidth) * height
            > static_cast<qint64>(documentHeight) * width) {
            height = qMax(1, qRound(static_cast<double>(box.height()) * documentHeight
                                    / documentWidth));
        } else {
            width = qMax(1, qRound(static_cast<double>(box.width()) * documentWidth
                                   / documentHeight));
        }
        return QRect(box.left() + (box.width() - width) / 2,
                     box.top() + (box.height() - height) / 2, width, height);
    }

    /// The thumbnail's hit-target, mirroring the x/y math paint() lays out, as
    /// the letterboxed (document-aspect) rect. Empty when thumbnails are off.
    QRect thumbRect(const QRect& itemRect, const QModelIndex& index) const
    {
        const int thumb = qMax(0, thumbnailSize_);
        if (thumb <= 0) {
            return {};
        }
        int x = contentLeft(itemRect, index);
        if (index.data(ClippingRole).toBool()) {
            x += qMax(10, thumb - 8) + 2;
        }
        const QRect box(x, itemRect.top() + (itemRect.height() - thumb) / 2, thumb, thumb);
        return letterboxedThumb(box, index);
    }

    /// The name text's hit-target, mirroring the geometry paint() lays out: the
    /// clipping glyph and thumbnail advance, the +4 gap, the clipping indent,
    /// and the right-edge lock/fx/mask caps. Floored to a non-zero width so a
    /// click in an empty label area still resolves to the name.
    QRect nameRect(const QRect& itemRect, const QModelIndex& index) const
    {
        const int thumb = qMax(0, thumbnailSize_);
        int x = contentLeft(itemRect, index);
        const bool clipping = index.data(ClippingRole).toBool();
        if (clipping) {
            const int side = qMax(10, thumb > 0 ? thumb - 8 : 12);
            if (!pictura::icon(QStringLiteral("layers.clipMask")).pixmap(side, side).isNull()) {
                x += side + 2;
            }
        }
        if (thumb > 0) {
            x += thumb;
        }
        x += 4;
        int right = itemRect.right() - 3;
        const int badge = qMax(12, thumb > 0 ? thumb : 16);
        if (index.data(LockRole).toInt() != 0
            && !pictura::icon(QStringLiteral("layers.lockAll")).pixmap(badge, badge).isNull()) {
            right -= badge + 3;
        }
        if (index.data(HasAdjustmentRole).toBool()
            && !pictura::icon(QStringLiteral("layers.fx")).pixmap(badge, badge).isNull()) {
            right -= badge + 3;
        }
        if (thumb > 0 && !index.data(MaskThumbnailRole).value<QImage>().isNull()) {
            right -= thumb + 3;
        }
        const int nameLeft = x + (clipping ? 12 : 0);
        const int nameRight = qMax(nameLeft, right - 4);
        return QRect(nameLeft, itemRect.top(), qMax(1, nameRight - nameLeft), itemRect.height());
    }

    /// One named row height (floor 32 px) shared by sizeHint and centring; a
    /// Medium (24 px) thumbnail row lands at 36 px.
    static constexpr int kRowHeightFloor = 32;
    int rowHeight() const { return qMax(kRowHeightFloor, thumbnailSize_ + 12); }

    /// The row name's font: Background italic, linked/placed (and clip-base)
    /// underlined. Shared by paint and the self-test so one rule is asserted.
    QFont nameFont(const QFont& base, const QModelIndex& index) const
    {
        QFont font = base;
        font.setItalic(index.data(KindRole).toString() == QLatin1String("background"));
        font.setUnderline(index.data(ClipBaseRole).toBool()
                          || index.data(LayerRowLinkedRole).toBool()
                          || index.data(LayerRowPlacedRole).toBool());
        return font;
    }

    QSize sizeHint(const QStyleOptionViewItem& option, const QModelIndex& index) const override
    {
        Q_UNUSED(option);
        Q_UNUSED(index);
        return QSize(200, rowHeight());
    }

    void paint(QPainter* painter, const QStyleOptionViewItem& option,
               const QModelIndex& index) const override
    {
        // Let the style paint the row background/selection only, then draw the
        // CS6 anatomy ourselves on top. The selection highlight is clipped away
        // from the eye column and that column is repainted with the base colour,
        // so the visibility toggle stays legible.
        QStyleOptionViewItem opt = option;
        initStyleOption(&opt, index);
        opt.text.clear();
        opt.icon = QIcon();
        const QWidget* widget = opt.widget;
        QStyle* style = widget ? widget->style() : QApplication::style();

        const QRect rect = option.rect;
        const int height = qMax(kRowHeightFloor, rect.height());
        const bool selected = option.state & QStyle::State_Selected;
        const QRect eye = eyeRect(rect);
        if (selected) {
            painter->save();
            painter->setClipRegion(QRegion(rect).subtracted(QRegion(eye)));
            style->drawControl(QStyle::CE_ItemViewItem, &opt, painter, widget);
            painter->restore();
            painter->fillRect(eye, option.palette.color(QPalette::Base));
        } else {
            style->drawControl(QStyle::CE_ItemViewItem, &opt, painter, widget);
        }

        painter->save();
        const QPalette& palette = option.palette;
        const int depth = index.data(DepthRole).toInt();
        // A non-None color label tints the eye toggle's own background; the
        // alpha keeps the eye glyph legible.
        QColor label = layerLabelColor(index.data(ColorRole).toInt());
        if (label.isValid()) {
            label.setAlpha(90);
            painter->fillRect(eye, label);
        }
        // Eye icon, anchored at the panel's left edge for every depth.
        paintAsset(painter, eye,
                   index.data(VisibleRole).toBool() ? QStringLiteral("layers.eyeOn")
                                                    : QStringLiteral("layers.eyeOff"));
        // 1 px darker-grey separator at the gutter's right edge, between the
        // eye and the row content.
        const QColor separator = palette.color(QPalette::Base).darker(115);
        painter->fillRect(QRect(rect.left() + kEyeColumn - 1, rect.top(), 1, height), separator);

        // Content (disclosure, clipping glyph, thumbnail, name) starts after the
        // gutter; the chevron slot is reserved only for expandable rows.
        const int thumb = qMax(0, thumbnailSize_);
        int x = contentLeft(rect, index);
        QRect thumbBox;
        if (index.data(ExpandableRole).toBool()) {
            const auto* treeView = qobject_cast<const QTreeView*>(opt.widget);
            const bool expanded = treeView && treeView->isExpanded(index);
            paintAsset(painter, chevronRect(rect, depth),
                       expanded ? QStringLiteral("layers.disclosureDown")
                                : QStringLiteral("layers.disclosureRight"));
        }
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
            const QRect box(x, rect.top() + (height - thumb) / 2, thumb, thumb);
            const QRect shaped = letterboxedThumb(box, index);
            thumbBox = shaped;
            if (index.data(KindRole).toString() == QLatin1String("group")) {
                const QPixmap glyph =
                    pictura::icon(QStringLiteral("layers.group")).pixmap(thumb, thumb);
                if (!glyph.isNull()) {
                    painter->drawPixmap(box, glyph);
                }
            } else {
                painter->drawTiledPixmap(shaped, checkerTile());
                const QImage image = index.data(ThumbnailRole).value<QImage>();
                if (!image.isNull()) {
                    painter->drawImage(shaped, image);
                }
            }
            // 1 px black outline around the letterboxed thumbnail.
            painter->setPen(QPen(Qt::black, 1));
            painter->setBrush(Qt::NoBrush);
            painter->drawRect(shaped.adjusted(0, 0, -1, -1));
        }
        x += thumb + 4;

        // Right-aligned badges: the lock badge at the far edge, then fx, then
        // the mask thumbnail. A null icon/thumbnail is omitted.
        int right = rect.right() - 3;
        const QIcon fxIcon = pictura::icon(QStringLiteral("layers.fx"));
        const int badge = qMax(12, thumb > 0 ? thumb : 16);
        if (index.data(LockRole).toInt() != 0) {
            const QPixmap lockPix =
                pictura::icon(QStringLiteral("layers.lockAll")).pixmap(badge, badge);
            if (!lockPix.isNull()) {
                painter->drawPixmap(
                    QRect(right - badge, rect.top() + (height - badge) / 2, badge, badge),
                    lockPix);
                right -= badge + 3;
            }
        }
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
        painter->setFont(nameFont(option.font, index));
        painter->setPen(selected ? palette.color(QPalette::HighlightedText)
                                 : palette.color(QPalette::Text));
        const QString elided =
            painter->fontMetrics().elidedText(index.data(Qt::DisplayRole).toString(),
                                              Qt::ElideRight, nameRight - nameLeft);
        painter->drawText(QRect(nameLeft, rect.top(), nameRight - nameLeft, height),
                          Qt::AlignVCenter | Qt::AlignLeft, elided);

        // White 1 px corner brackets one pixel outside the outline, only for the
        // singular active layer.
        if (thumb > 0 && singularActive(selected, opt.widget, index)) {
            paintBrackets(painter, thumbBox.adjusted(-1, -1, 1, 1));
        }
        painter->restore();
    }

private:
    static bool singularActive(bool selected, const QWidget* widget, const QModelIndex& index)
    {
        if (!selected) {
            return false;
        }
        const auto* treeView = qobject_cast<const QTreeView*>(widget);
        return treeView && treeView->selectionModel()
            && treeView->selectionModel()->selectedRows(0).size() == 1;
    }

    static void paintBrackets(QPainter* painter, const QRect& r)
    {
        painter->setPen(QPen(Qt::white, 1));
        const int len = qMax(2, qMin(r.width(), r.height()) / 3);
        painter->drawLine(r.left(), r.top(), r.left() + len, r.top());
        painter->drawLine(r.left(), r.top(), r.left(), r.top() + len);
        painter->drawLine(r.right() - len, r.top(), r.right(), r.top());
        painter->drawLine(r.right(), r.top(), r.right(), r.top() + len);
        painter->drawLine(r.left(), r.bottom() - len, r.left(), r.bottom());
        painter->drawLine(r.left(), r.bottom(), r.left() + len, r.bottom());
        painter->drawLine(r.right() - len, r.bottom(), r.right(), r.bottom());
        painter->drawLine(r.right(), r.bottom() - len, r.right(), r.bottom());
    }

    static const QPixmap& checkerTile()
    {
        static const QPixmap tile = [] {
            QPixmap pixmap(8, 8);
            pixmap.fill(QColor(0xC8, 0xC8, 0xC8));
            QPainter painter(&pixmap);
            painter.fillRect(0, 0, 4, 4, QColor(0xFF, 0xFF, 0xFF));
            painter.fillRect(4, 4, 4, 4, QColor(0xFF, 0xFF, 0xFF));
            return pixmap;
        }();
        return tile;
    }

    static void paintAsset(QPainter* painter, const QRect& rect, const QString& assetId)
    {
        // Render the SVG at the largest square that fits, at the painter's
        // device pixel ratio, so it stays crisp instead of being scaled from a
        // logical-size pixmap (or stretched into a non-square rect).
        const int side = qMin(rect.width(), rect.height());
        if (side <= 0) {
            return;
        }
        const qreal dpr = painter->device() ? painter->device()->devicePixelRatioF() : 1.0;
        const QPixmap pixmap = pictura::icon(assetId).pixmap(QSize(side, side), dpr);
        if (!pixmap.isNull()) {
            painter->drawPixmap(
                QRect(rect.left() + (rect.width() - side) / 2,
                      rect.top() + (rect.height() - side) / 2, side, side),
                pixmap);
        }
    }

    int thumbnailSize_ = 24;
};

} // namespace pictura
