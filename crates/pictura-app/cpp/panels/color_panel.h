#pragma once

#include <QtCore/QObject>
#include <QtGui/QColor>
#include <QtWidgets/QWidget>

#include <functional>

class QLabel;
class QLineEdit;
class QPushButton;
class QSlider;

namespace pictura {

// Application foreground/background colour shared by the toolbox, Color,
// Swatches, and Eyedropper.
class ColorState : public QObject {
    Q_OBJECT

public:
    explicit ColorState(QObject* parent = nullptr);

    QColor foreground() const { return foreground_; }
    QColor background() const { return background_; }
    void setForeground(const QColor& color);
    void setBackground(const QColor& color);

    bool foregroundActive() const { return foregroundActive_; }
    void setForegroundActive(bool foreground);

signals:
    void foregroundChanged(QColor color);
    void backgroundChanged(QColor color);
    void activeChanged(bool foreground);

private:
    QColor foreground_{Qt::black};
    QColor background_{Qt::white};
    bool foregroundActive_ = true;
};

// Horizontal hue spectrum; emits the hue under the pointer.
class HueSpectrum : public QWidget {
public:
    explicit HueSpectrum(QWidget* parent = nullptr);

    void setHuePicked(std::function<void(int)> callback);

protected:
    void paintEvent(QPaintEvent* event) override;
    void mousePressEvent(QMouseEvent* event) override;
    void mouseMoveEvent(QMouseEvent* event) override;

private:
    void pickAt(const QPointF& pos);

    std::function<void(int)> picked_;
};

class ColorPanel : public QWidget {
    Q_OBJECT

public:
    explicit ColorPanel(ColorState* state, QWidget* parent = nullptr);

private:
    void syncControls();
    void applyRgb();
    void applyHsb();
    void selectColor(const QColor& color);
    QColor activeColor() const;
    void paintSwatch(QPushButton* button, const QColor& color, bool active);

    ColorState* state_ = nullptr;
    bool activeForeground_ = true;
    QSlider* rgb_[3] = {nullptr, nullptr, nullptr};
    QSlider* hsb_[3] = {nullptr, nullptr, nullptr};
    QLabel* rgbValue_[3] = {nullptr, nullptr, nullptr};
    QLabel* hsbValue_[3] = {nullptr, nullptr, nullptr};
    QLineEdit* hex_ = nullptr;
    QPushButton* fgSwatch_ = nullptr;
    QPushButton* bgSwatch_ = nullptr;
    HueSpectrum* spectrum_ = nullptr;
};

} // namespace pictura
