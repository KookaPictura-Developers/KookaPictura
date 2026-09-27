#pragma once

#include <QtCore/QPointF>
#include <QtWidgets/QWidget>

class QLabel;

namespace pictura {

class PictureView;

class InfoPanel : public QWidget {
    Q_OBJECT

public:
    explicit InfoPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    void setCursorPosition(const QPointF& imagePos);
    void refresh();

    // The color-sampler readouts, one "#n" line per sampler (self-test hook).
    QString samplerTextForTest() const;

private:
    PictureView* view_ = nullptr;
    QPointF cursor_;
    QLabel* positionLabel_ = nullptr;
    QLabel* colorLabel_ = nullptr;
    QLabel* selectionLabel_ = nullptr;
    QLabel* sizeLabel_ = nullptr;
    QLabel* samplersLabel_ = nullptr;
};

} // namespace pictura
