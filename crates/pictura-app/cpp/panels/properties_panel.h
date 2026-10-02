#pragma once

#include <QtCore/QString>
#include <QtWidgets/QWidget>

class QLabel;

namespace pictura {

class PictureView;

class PropertiesPanel : public QWidget {
    Q_OBJECT

public:
    explicit PropertiesPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    void refresh();

    QString messageForTest() const;

private:
    PictureView* view_ = nullptr;
    QLabel* message_ = nullptr;
};

} // namespace pictura
