#pragma once

#include <QtCore/QObject>
#include <QtWidgets/QDockWidget>

class QComboBox;
class QSpinBox;
class QTreeView;

namespace pictura {

class LayersModel;
class PictureView;

class LayersPanel : public QDockWidget {
    Q_OBJECT

public:
    explicit LayersPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    void refresh();

private:
    void syncControls();

    PictureView* view_ = nullptr;
    LayersModel* model_ = nullptr;
    QTreeView* tree_ = nullptr;
    QComboBox* blend_ = nullptr;
    QSpinBox* opacity_ = nullptr;
    bool syncing_ = false;
    QMetaObject::Connection viewConnection_;
};

} // namespace pictura
