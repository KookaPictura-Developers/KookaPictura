#pragma once

#include <QtCore/QPointF>
#include <QtWidgets/QDockWidget>

class QLabel;

namespace pictura {

class PictureView;

class InfoPanel : public QDockWidget {
    Q_OBJECT

public:
    explicit InfoPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    void setCursorPosition(const QPointF& imagePos);
    void refresh();

private:
    PictureView* view_ = nullptr;
    QPointF cursor_;
    QLabel* positionLabel_ = nullptr;
    QLabel* colorLabel_ = nullptr;
    QLabel* selectionLabel_ = nullptr;
    QLabel* sizeLabel_ = nullptr;
};

} // namespace pictura
