#include "layers_panel.h"

#include "layers_filter_bar.h"
#include "layers_filter_proxy.h"
#include "layers_panel_internal.h"
#include "percent_field.h"

#include <QtCore/QMetaObject>
#include <QtCore/QModelIndex>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtGui/QAction>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QMenu>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QTreeView>

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

} // namespace pictura
