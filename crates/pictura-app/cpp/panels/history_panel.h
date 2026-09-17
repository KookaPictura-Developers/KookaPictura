#pragma once

#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtWidgets/QWidget>

class QListWidget;
class QListWidgetItem;
class QPushButton;

namespace pictura {

class PictureView;

class HistoryPanel : public QWidget {
    Q_OBJECT

public:
    explicit HistoryPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    void refresh();

    // Phase D: run a wired History per-widget menu entry by its action id.
    // Returns false for ids this panel does not own.
    bool performPanelMenuAction(const QString& actionId);

private:
    void activate(QListWidgetItem* item);
    void createSnapshot();

    PictureView* view_ = nullptr;
    QListWidget* list_ = nullptr;
    QPushButton* snapshotButton_ = nullptr;
    QMetaObject::Connection viewConnection_;
};

} // namespace pictura
