#pragma once

#include <QtCore/QList>
#include <QtGui/QColor>
#include <QtWidgets/QWidget>

#include <array>

class QCheckBox;
class QComboBox;
class QLabel;

namespace pictura {

class GradientSwatch;
class PercentField;

// A Noise gradient's settings. `low` / `high` bound each colour component in
// percent of its range: R, G, B; H, S, B; or L, a, b by `model`.
struct NoiseSettings {
    enum Model { Rgb, Hsb, Lab };
    quint32 seed = 0x2545f491u;
    int roughness = 50;
    int model = Rgb;
    std::array<int, 3> low{0, 0, 0};
    std::array<int, 3> high{100, 100, 100};
    bool restrictColors = false;
    bool transparency = false;
};

// One component's range: a ramp of the component with a low handle (black)
// and a high handle (white) under it, 0..100 %. Dragging moves the nearer.
class ChannelRangeSlider : public QWidget {
    Q_OBJECT

public:
    explicit ChannelRangeSlider(QWidget* parent = nullptr);

    int low() const { return low_; }
    int high() const { return high_; }
    void setRange(int low, int high);
    void setRamp(const QList<QColor>& colors);

    QSize sizeHint() const override;

signals:
    void rangeChanged(int low, int high);

protected:
    void paintEvent(QPaintEvent* event) override;
    void mousePressEvent(QMouseEvent* event) override;
    void mouseMoveEvent(QMouseEvent* event) override;

private:
    QRect grooveRect() const;
    int valueAt(int x) const;
    int handleX(int value) const;

    int low_ = 0;
    int high_ = 100;
    int dragging_ = -1;
    QList<QColor> ramp_;
};

// The Gradient Editor's Noise page: Roughness, the resulting ramp, Color Model
// with a range per component, and the Restrict Colors / Add Transparency /
// Randomize options. Every edit emits `changed`.
class GradientNoisePage : public QWidget {
    Q_OBJECT

public:
    explicit GradientNoisePage(QWidget* parent = nullptr);

    NoiseSettings settings() const { return settings_; }
    void setSettings(const NoiseSettings& settings);
    // A new seed, as the Randomize button picks.
    void randomize();

signals:
    void changed();

private:
    void edited();
    void showSettings();

    NoiseSettings settings_;
    PercentField* roughness_ = nullptr;
    GradientSwatch* strip_ = nullptr;
    QComboBox* model_ = nullptr;
    std::array<QLabel*, 3> labels_{};
    std::array<ChannelRangeSlider*, 3> ranges_{};
    QCheckBox* restrict_ = nullptr;
    QCheckBox* transparency_ = nullptr;
    bool showing_ = false;
};

} // namespace pictura
