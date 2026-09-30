#pragma once

#include <QtCore/QPointF>
#include <QtWidgets/QWidget>

class QFormLayout;
class QLabel;

namespace pictura {

class PictureView;

class InfoPanel : public QWidget {
    Q_OBJECT

public:
    explicit InfoPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    void setCursorPosition(const QPointF& imagePos);
    // The Ruler tool's mode: the panel shows A/L and points W/H at the ruler.
    void setRulerMode(bool on);
    bool rulerMode() const { return rulerMode_; }
    void refresh();

    QString samplerTextForTest() const;
    QString colorTextForTest() const;
    QString cmykTextForTest() const;
    QString sizeTextForTest() const;
    QString rulerTextForTest() const;

private:
    PictureView* view_ = nullptr;
    QPointF cursor_;
    bool rulerMode_ = false;
    QFormLayout* form_ = nullptr;
    QLabel* positionLabel_ = nullptr;
    QLabel* colorLabel_ = nullptr;
    QLabel* cmykLabel_ = nullptr;
    QLabel* selectionLabel_ = nullptr;
    QLabel* sizeLabel_ = nullptr;
    QLabel* rulerLabel_ = nullptr;
    QLabel* samplersLabel_ = nullptr;
};

} // namespace pictura
