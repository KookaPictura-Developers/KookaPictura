#include "history_panel.h"

#include "icons.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/paint_tools.cxxqt.h"

#include <QtCore/QSize>
#include <QtCore/QTimer>
#include <QtCore/QVariant>
#include <QtGui/QFont>
#include <QtGui/QMouseEvent>
#include <QtGui/QPixmap>
#include <QtWidgets/QInputDialog>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QListWidget>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QWidget>

namespace pictura {

namespace {

constexpr int kKindRole = Qt::UserRole;
constexpr int kIndexRole = Qt::UserRole + 1;
constexpr int kStateKind = 0;
constexpr int kSnapshotKind = 1;
constexpr int kSourceIconSize = 16;
// The left column that chooses the History Brush source: the icon plus the
// item's leading margin.
constexpr int kSourceColumnWidth = kSourceIconSize + 6;

// The History Brush marker on the source row, or a blank of the same size so
// every label lines up.
QIcon sourceIcon(bool source)
{
    if (source) {
        return pictura::icon(QStringLiteral("tool.historybrush"));
    }
    QPixmap blank(kSourceIconSize, kSourceIconSize);
    blank.fill(Qt::transparent);
    return QIcon(blank);
}

} // namespace

HistoryPanel::HistoryPanel(QWidget* parent)
    : QWidget(parent)
{
    QWidget* body = this;
    auto* layout = new QVBoxLayout(body);
    list_ = new QListWidget(body);
    list_->setIconSize(QSize(kSourceIconSize, kSourceIconSize));
    list_->viewport()->installEventFilter(this);
    layout->addWidget(list_, 1);
    snapshotButton_ = new QPushButton(tr("Create Snapshot"), body);
    snapshotButton_->setObjectName(QStringLiteral("snapshotButton"));
    snapshotButton_->setIcon(pictura::icon(QStringLiteral("history.snapshot")));
    snapshotButton_->setIconSize(QSize(20, 20));
    snapshotButton_->setToolTip(tr("Create New Snapshot"));
    layout->addWidget(snapshotButton_);

    connect(list_, &QListWidget::itemClicked, this, [this](QListWidgetItem* item) { activate(item); });
    connect(snapshotButton_, &QPushButton::clicked, this, [this] { createSnapshot(); });
}

void HistoryPanel::setView(PictureView* view)
{
    if (viewConnection_) {
        QObject::disconnect(viewConnection_);
    }
    view_ = view;
    if (view_) {
        viewConnection_ = connect(view_, &PictureView::changed, this, [this] {
            QTimer::singleShot(0, this, [this] { refresh(); });
        });
    }
    refresh();
}

void HistoryPanel::refresh()
{
    list_->clear();
    if (!view_) {
        return;
    }

    const int sourceSnapshot = history_brush_source_snapshot(*view_);
    const int sourceState = history_brush_source_state(*view_);
    const int snapshots = view_->history_snapshot_count();
    for (int i = 0; i < snapshots; ++i) {
        auto* item = new QListWidgetItem(sourceIcon(i == sourceSnapshot),
                                         view_->history_snapshot_label(i), list_);
        item->setData(kKindRole, kSnapshotKind);
        item->setData(kIndexRole, i);
        item->setToolTip(tr("Snapshot"));
    }

    const int count = view_->history_count();
    const int current = view_->history_index();
    QListWidgetItem* currentItem = nullptr;
    for (int i = 0; i < count; ++i) {
        const bool isCurrent = i == current;
        auto* item = new QListWidgetItem(
            sourceIcon(i == sourceState),
            (isCurrent ? QStringLiteral("> ") : QString()) + view_->history_label(i), list_);
        item->setData(kKindRole, kStateKind);
        item->setData(kIndexRole, i);
        if (isCurrent) {
            QFont font = item->font();
            font.setBold(true);
            item->setFont(font);
            currentItem = item;
        }
    }
    if (currentItem) {
        list_->setCurrentItem(currentItem);
    }
}

void HistoryPanel::activate(QListWidgetItem* item)
{
    if (!view_ || !item) {
        return;
    }
    const int index = item->data(kIndexRole).toInt();
    if (item->data(kKindRole).toInt() == kSnapshotKind) {
        view_->history_restore_snapshot(index);
    } else {
        view_->history_jump(index);
    }
}

bool HistoryPanel::eventFilter(QObject* watched, QEvent* event)
{
    if (watched == list_->viewport() && event->type() == QEvent::MouseButtonPress && view_) {
        const auto* press = static_cast<QMouseEvent*>(event);
        const QPoint pos = press->position().toPoint();
        QListWidgetItem* item = list_->itemAt(pos);
        if (item && pos.x() - list_->visualItemRect(item).left() < kSourceColumnWidth) {
            set_history_brush_source(*view_, item->data(kKindRole).toInt() == kSnapshotKind,
                                     item->data(kIndexRole).toInt());
            refresh();
            return true;
        }
    }
    return QWidget::eventFilter(watched, event);
}

bool HistoryPanel::performPanelMenuAction(const QString& actionId)
{
    if (actionId == QLatin1String("stepForward")) {
        if (view_) {
            view_->redo();
        }
    } else if (actionId == QLatin1String("stepBackward")) {
        if (view_) {
            view_->undo();
        }
    } else if (actionId == QLatin1String("newSnapshot")) {
        createSnapshot();
    } else {
        return false;
    }
    return true;
}

void HistoryPanel::createSnapshot()
{
    if (!view_) {
        return;
    }
    bool accepted = false;
    const QString label = QInputDialog::getText(this, tr("Create Snapshot"), tr("Label:"),
                                                QLineEdit::Normal, tr("Snapshot"), &accepted);
    if (!accepted || label.isEmpty()) {
        return;
    }
    view_->history_add_snapshot(label);
}

} // namespace pictura
