#pragma once

#include <QtCore/QVector>
#include <QtGui/QColor>
#include <QtWidgets/QDockWidget>
#include <QtWidgets/QWidget>

#include <array>

class QComboBox;

namespace pictura {

class PictureView;

// Custom-painted 256-bin histogram.
class HistogramView : public QWidget {
public:
    explicit HistogramView(QWidget* parent = nullptr);

    void setBins(const QVector<quint32>& bins, const QColor& color);
    void clear();

protected:
    void paintEvent(QPaintEvent* event) override;

private:
    QVector<quint32> bins_;
    QColor color_{Qt::white};
};

class HistogramPanel : public QDockWidget {
    Q_OBJECT

public:
    explicit HistogramPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    void refresh();

private:
    void recompute();
    void applyChannel();

    PictureView* view_ = nullptr;
    HistogramView* histogram_ = nullptr;
    QComboBox* channel_ = nullptr;
    std::array<std::array<quint32, 256>, 4> bins_{};
};

} // namespace pictura
