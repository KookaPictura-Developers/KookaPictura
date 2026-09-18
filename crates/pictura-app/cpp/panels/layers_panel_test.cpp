#include "layers_panel.h"

#include "layers_filter_bar.h"
#include "layers_filter_proxy.h"
#include "layers_panel_internal.h"
#include "percent_field.h"

#include <QtCore/QCoreApplication>
#include <QtCore/QMetaObject>
#include <QtCore/QMimeData>
#include <QtCore/QModelIndex>
#include <QtCore/QPoint>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtGui/QAction>
#include <QtGui/QDragEnterEvent>
#include <QtGui/QDropEvent>
#include <QtGui/QMouseEvent>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QMenu>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QTreeView>

#include <array>

#include <functional>

namespace pictura {

namespace {

void setPercentForTest(PercentField* field, int pct)
{
    if (!field) {
        return;
    }
    auto* edit = field->findChild<QLineEdit*>();
    if (!edit) {
        return;
    }
    edit->setText(QString::number(pct));
    QMetaObject::invokeMethod(edit, "editingFinished");
}

} // namespace

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
    const QModelIndex index = proxyIndexForPath(path);
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

int LayersPanel::lockButtonCountForTest() const
{
    return 5;
}

int LayersPanel::opacityPercentForTest() const
{
    return opacity_ ? opacity_->value() : 0;
}

int LayersPanel::fillPercentForTest() const
{
    return fill_ ? fill_->value() : 0;
}

void LayersPanel::setOpacityPercentForTest(int pct)
{
    setPercentForTest(opacity_, pct);
}

void LayersPanel::setFillPercentForTest(int pct)
{
    setPercentForTest(fill_, pct);
}

QStringList LayersPanel::visiblePathsForTest() const
{
    QStringList paths;
    if (!proxy_ || !tree_) {
        return paths;
    }
    std::function<void(const QModelIndex&)> walk = [this, &walk, &paths](
                                                      const QModelIndex& parent) {
        const int rows = proxy_->rowCount(parent);
        for (int row = 0; row < rows; ++row) {
            const QModelIndex index = proxy_->index(row, 0, parent);
            paths.push_back(pathForProxyIndex(index));
            if (tree_->isExpanded(index)) {
                walk(index);
            }
        }
    };
    walk(QModelIndex());
    return paths;
}

void LayersPanel::setFilterNameForTest(const QString& name, bool enabled)
{
    LayerFilter filter = filterBar_ ? filterBar_->filter() : LayerFilter{};
    filter.enabled = enabled;
    filter.name = name;
    if (filterBar_) {
        filterBar_->setFilter(filter);
    }
    applyFilter(filter);
}

void LayersPanel::setFilterKindForTest(const QStringList& kinds, bool enabled)
{
    LayerFilter filter = filterBar_ ? filterBar_->filter() : LayerFilter{};
    filter.enabled = enabled;
    filter.kinds.clear();
    for (const QString& kind : kinds) {
        filter.kinds.insert(kind);
    }
    if (filterBar_) {
        filterBar_->setFilter(filter);
    }
    applyFilter(filter);
}

void LayersPanel::setFilterModeForTest(const QString& key, bool enabled)
{
    LayerFilter filter = filterBar_ ? filterBar_->filter() : LayerFilter{};
    filter.enabled = enabled;
    filter.mode = key;
    if (filterBar_) {
        filterBar_->setFilter(filter);
    }
    applyFilter(filter);
}

void LayersPanel::setFilterColorForTest(int label, bool enabled)
{
    LayerFilter filter = filterBar_ ? filterBar_->filter() : LayerFilter{};
    filter.enabled = enabled;
    filter.color = label;
    if (filterBar_) {
        filterBar_->setFilter(filter);
    }
    applyFilter(filter);
}

void LayersPanel::setFilterAttributeForTest(const QString& attr, bool enabled)
{
    LayerFilter filter = filterBar_ ? filterBar_->filter() : LayerFilter{};
    filter.enabled = enabled;
    filter.attribute = attr;
    if (filterBar_) {
        filterBar_->setFilter(filter);
    }
    applyFilter(filter);
}

int LayersPanel::filterDimensionForTest() const
{
    return filterBar_ ? filterBar_->dimensionIndexForTest() : -1;
}

bool LayersPanel::headerOrderOkForTest() const
{
    const auto y = [this](const QWidget* widget) {
        return widget ? widget->mapTo(this, QPoint(0, 0)).y() : -1;
    };
    const int filterY = y(filterBar_);
    const int opacityY = y(opacity_);
    const int locksY = y(lockTransparency_);
    const int treeY = y(tree_);
    return filterY >= 0 && filterY < opacityY && opacityY < locksY && locksY < treeY;
}

bool LayersPanel::opacityLabelPresentForTest() const
{
    return opacity_ && !opacity_->labelText().isEmpty();
}

bool LayersPanel::fillLabelPresentForTest() const
{
    return fill_ && !fill_->labelText().isEmpty();
}

bool LayersPanel::hasPanelMenuButtonForTest() const
{
    return findChild<QToolButton*>(QStringLiteral("layersPanelMenu")) != nullptr;
}

bool LayersPanel::filterToggleOnForTest() const
{
    return filterBar_ && filterBar_->toggleOnForTest();
}

bool LayersPanel::filterToggleHasIconForTest() const
{
    return filterBar_ && filterBar_->toggleHasIconForTest();
}

int LayersPanel::eyeLeftForTest(const QString& path) const
{
    const QModelIndex index = proxyIndexForPath(path);
    if (!index.isValid() || !delegate_ || !tree_) {
        return -1;
    }
    return delegate_->eyeRect(tree_->visualRect(index)).left();
}

bool LayersPanel::chevronClickExpandsForTest(const QString& path)
{
    const QModelIndex index = proxyIndexForPath(path);
    if (!index.isValid() || !index.data(ExpandableRole).toBool() || !delegate_ || !tree_) {
        return false;
    }
    const bool before = tree_->isExpanded(index);
    const QPoint pos =
        delegate_->chevronRect(tree_->visualRect(index), index.data(DepthRole).toInt()).center();
    QMouseEvent press(QEvent::MouseButtonPress, pos, tree_->viewport()->mapToGlobal(pos),
                      Qt::LeftButton, Qt::LeftButton, Qt::NoModifier);
    QCoreApplication::sendEvent(tree_->viewport(), &press);
    return tree_->isExpanded(index) != before;
}

QString LayersPanel::opacityLabelTextForTest() const
{
    return opacity_ ? opacity_->labelText() : QString();
}

bool LayersPanel::opacitySuffixPresentForTest() const
{
    return opacity_ && opacity_->findChild<QLabel*>(QStringLiteral("percentSuffix")) != nullptr;
}

bool LayersPanel::lockIconsPresentForTest() const
{
    const std::array<QToolButton*, 5> buttons = {
        lockTransparency_, lockPixels_, lockPosition_, lockNesting_, lockAll_};
    for (QToolButton* button : buttons) {
        if (!button || button->icon().isNull()) {
            return false;
        }
    }
    return true;
}

bool LayersPanel::treeDragEnabledForTest() const
{
    return tree_ && tree_->dragEnabled() && tree_->acceptDrops();
}

bool LayersPanel::moveForTest(const QString& path, const QString& target, int mode)
{
    return view_ && view_->move_layer_to(path, target, mode);
}

bool LayersPanel::dropOnStripButtonForTest(const QString& buttonName, const QStringList& paths)
{
    auto* button = findChild<QToolButton*>(buttonName);
    if (!button) {
        return false;
    }
    QMimeData mime;
    mime.setData(kLayerMimeType, paths.join(QLatin1Char('\n')).toUtf8());
    const QPointF local(5, 5);
    QDragEnterEvent enter(local.toPoint(), Qt::MoveAction, &mime, Qt::LeftButton, Qt::NoModifier);
    QCoreApplication::sendEvent(button, &enter);
    if (!enter.isAccepted()) {
        return false;
    }
    QDropEvent drop(local, Qt::MoveAction, &mime, Qt::LeftButton, Qt::NoModifier);
    QCoreApplication::sendEvent(button, &drop);
    return drop.isAccepted();
}

} // namespace pictura
