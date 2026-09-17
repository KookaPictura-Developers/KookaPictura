#pragma once

#include <QtCore/QPointF>
#include <QtCore/QRect>
#include <QtCore/QSize>
#include <QtGui/QImage>
#include <QtWidgets/QWidget>

#include <functional>

class QLabel;
class QSlider;

namespace pictura {

class ImageView;

// Paints the document thumbnail and the proxy rectangle for the region the
// canvas currently shows.
class NavigatorThumbnail : public QWidget {
public:
    explicit NavigatorThumbnail(QWidget* parent = nullptr);

    void setImage(const QImage& image);
    void setView(double zoom, const QPointF& offset, const QSize& viewport);
    void setPointPicked(std::function<void(const QPointF&)> callback);

protected:
    void paintEvent(QPaintEvent* event) override;
    void resizeEvent(QResizeEvent* event) override;
    void mousePressEvent(QMouseEvent* event) override;
    void mouseMoveEvent(QMouseEvent* event) override;

private:
    QRect imageRect() const;
    void pickAt(const QPointF& pos);

    QImage source_;
    QImage scaled_;
    double zoom_ = 1.0;
    QPointF offset_;
    QSize viewport_;
    std::function<void(const QPointF&)> picked_;
};

class NavigatorPanel : public QWidget {
    Q_OBJECT

public:
    explicit NavigatorPanel(QWidget* parent = nullptr);

    void setCanvas(ImageView* canvas);
    void refresh();

protected:
    bool eventFilter(QObject* watched, QEvent* event) override;

private:
    void syncFromCanvas();
    void centerOn(const QPointF& imagePoint);

    ImageView* canvas_ = nullptr;
    NavigatorThumbnail* thumbnail_ = nullptr;
    QSlider* slider_ = nullptr;
    QLabel* zoomLabel_ = nullptr;
    bool updating_ = false;
};

} // namespace pictura
