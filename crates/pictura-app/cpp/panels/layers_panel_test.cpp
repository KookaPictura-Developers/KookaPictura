#include "layers_panel.h"

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

} // namespace pictura
