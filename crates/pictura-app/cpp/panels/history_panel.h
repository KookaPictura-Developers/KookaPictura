#pragma once

#include <QtCore/QObject>
#include <QtWidgets/QDockWidget>

class QListWidget;
class QListWidgetItem;
class QPushButton;

namespace pictura {

class PictureView;

class HistoryPanel : public QDockWidget {
    Q_OBJECT

public:
    explicit HistoryPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    void refresh();

private:
    void activate(QListWidgetItem* item);
    void createSnapshot();

    PictureView* view_ = nullptr;
    QListWidget* list_ = nullptr;
    QPushButton* snapshotButton_ = nullptr;
    QMetaObject::Connection viewConnection_;
};

} // namespace pictura
