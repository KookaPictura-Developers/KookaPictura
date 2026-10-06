#include "font_combo.h"

#include <QtCore/QHash>
#include <QtGui/QIcon>
#include <QtGui/QPainter>
#include <QtGui/QRawFont>
#include <QtWidgets/QAbstractItemView>
#include <QtWidgets/QFontComboBox>
#include <QtWidgets/QStyledItemDelegate>

namespace pictura {

namespace {

class FastFontPreviewDelegate : public QStyledItemDelegate {
public:
    FastFontPreviewDelegate(QAbstractItemDelegate* base, QObject* parent)
        : QStyledItemDelegate(parent)
        , base_(base)
        , scalable_(
              QStringLiteral(":/qt-project.org/styles/commonstyle/images/fonttruetype-16.png"))
    {
        base_->setParent(this);
    }

    void paint(QPainter* painter, const QStyleOptionViewItem& option,
               const QModelIndex& index) const override
    {
        const QString family = index.data(Qt::DisplayRole).toString();
        if (!isEmojiFace(family)) {
            base_->paint(painter, option, index);
            return;
        }
        // Laid out like Qt's preview row (icon, then name) so the names align.
        painter->save();
        if (option.state & QStyle::State_Selected) {
            painter->fillRect(option.rect, option.palette.highlight());
            painter->setPen(option.palette.highlightedText().color());
        } else {
            painter->setPen(option.palette.text().color());
        }
        QRect rect = option.rect;
        const QSize iconSize = scalable_.actualSize(rect.size());
        scalable_.paint(painter, rect, Qt::AlignLeft | Qt::AlignVCenter);
        rect.setLeft(rect.left() + iconSize.width() + 4);
        painter->setFont(option.font);
        painter->drawText(rect, Qt::AlignVCenter | Qt::TextSingleLine, family);
        painter->restore();
    }

    QSize sizeHint(const QStyleOptionViewItem& option, const QModelIndex& index) const override
    {
        return base_->sizeHint(option, index);
    }

private:
    bool isEmojiFace(const QString& family) const
    {
        auto it = isEmojiFace_.constFind(family);
        if (it == isEmojiFace_.constEnd()) {
            const QRawFont raw = QRawFont::fromFont(QFont(family));
            // Emoji faces carry digits and '#', so test for a letter and a
            // pictograph rather than for any Latin coverage.
            it = isEmojiFace_.insert(family, raw.isValid()
                                                 && !raw.supportsCharacter(QLatin1Char('A'))
                                                 && raw.supportsCharacter(0x1F600U));
        }
        return *it;
    }

    QAbstractItemDelegate* base_;
    QIcon scalable_;
    mutable QHash<QString, bool> isEmojiFace_;
};

} // namespace

void useFastFontPreviews(QFontComboBox* combo)
{
    // Through the view: QComboBox::setItemDelegate may delete the delegate this
    // one wraps.
    QAbstractItemView* view = combo->view();
    view->setItemDelegate(new FastFontPreviewDelegate(view->itemDelegate(), combo));
}

} // namespace pictura
