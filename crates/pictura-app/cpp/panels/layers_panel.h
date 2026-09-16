#pragma once

#include <QtCore/QObject>
#include <QtCore/QPoint>
#include <QtWidgets/QDockWidget>

class QComboBox;
class QSpinBox;
class QToolButton;
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

    /// Bottom-first index of the selected row, or -1 when nothing is selected.
    int currentLayer() const;

private:
    void syncControls();
    void selectLayer(int index);
    void showColorMenu(const QPoint& pos);

    PictureView* view_ = nullptr;
    LayersModel* model_ = nullptr;
    QTreeView* tree_ = nullptr;
    QComboBox* blend_ = nullptr;
    QSpinBox* opacity_ = nullptr;
    QSpinBox* fill_ = nullptr;
    QToolButton* lockTransparency_ = nullptr;
    QToolButton* lockPixels_ = nullptr;
    QToolButton* lockPosition_ = nullptr;
    QToolButton* lockAll_ = nullptr;
    bool syncing_ = false;
    QMetaObject::Connection viewConnection_;
};

} // namespace pictura
