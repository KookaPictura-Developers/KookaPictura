#include <QtCore/QCoreApplication>
#include <QtCore/QTimer>
#include <QtGui/QImage>
#include <QtGui/QMouseEvent>
#include <QtGui/QPainter>
#include <QtGui/QWheelEvent>
#include <QtWidgets/QApplication>
#include <QtWidgets/QWidget>

#include <cmath>
#include <cstdio>

#include "pictura_app/src/cxxqt_object.cxxqt.h"

class ImageView : public QWidget
{
    Q_OBJECT

public:
    explicit ImageView(QWidget* parent = nullptr)
        : QWidget(parent)
    {
        setWindowTitle(QStringLiteral("Kooka Pictura - M0 walking skeleton"));
        setMinimumSize(640, 480);
    }

    void setImage(const QImage& image)
    {
        image_ = image;
        zoom_ = 1.0;
        offset_ = QPointF(0.0, 0.0);
        update();
    }

    // Zoom about a cursor position so the point under the cursor stays put.
    void zoomAt(const QPointF& cursor, int angleDelta)
    {
        const double factor = std::pow(1.0015, angleDelta);
        offset_ = cursor - (cursor - offset_) * factor;
        zoom_ *= factor;
        update();
    }

    void panBy(const QPointF& delta)
    {
        offset_ += delta;
        update();
    }

    double zoom() const { return zoom_; }
    QPointF offset() const { return offset_; }

protected:
    void paintEvent(QPaintEvent*) override
    {
        QPainter painter(this);
        painter.fillRect(rect(), Qt::darkGray);
        if (image_.isNull()) {
            return;
        }
        painter.translate(offset_);
        painter.scale(zoom_, zoom_);
        painter.drawImage(QPointF(0.0, 0.0), image_);
    }

    void wheelEvent(QWheelEvent* event) override
    {
        zoomAt(event->position(), event->angleDelta().y());
    }

    void mousePressEvent(QMouseEvent* event) override
    {
        if (event->button() == Qt::LeftButton) {
            last_ = event->position();
        }
    }

    void mouseMoveEvent(QMouseEvent* event) override
    {
        if (event->buttons() & Qt::LeftButton) {
            panBy(event->position() - last_);
            last_ = event->position();
        }
    }

private:
    QImage image_;
    double zoom_ = 1.0;
    QPointF offset_;
    QPointF last_;
};

int main(int argc, char* argv[])
{
    QApplication app(argc, argv);

    const QStringList args = app.arguments();
    bool selfTest = false;
    QString psdPath;
    for (int i = 1; i < args.size(); ++i) {
        if (args.at(i) == QStringLiteral("--self-test")) {
            selfTest = true;
        } else if (!args.at(i).startsWith(QLatin1Char('-'))) {
            psdPath = args.at(i);
        }
    }

    pictura::PictureView view;
    const bool codecLoaded = view.open(psdPath);
    const QImage image = view.image();

    ImageView window;
    window.setImage(image);
    window.resize(900, 650);
    window.show();

    if (selfTest) {
        std::fprintf(stderr,
                     "pictura self-test: image=%dx%d codec_loaded=%d\n",
                     image.width(),
                     image.height(),
                     codecLoaded ? 1 : 0);
        std::fflush(stderr);
        if (image.isNull()) {
            std::fprintf(stderr, "pictura self-test: FAIL: null image\n");
            return 2;
        }
        const QPointF center(window.width() / 2.0, window.height() / 2.0);
        window.zoomAt(center, 120);
        const QPointF afterZoom = window.offset();
        window.panBy(QPointF(10.0, 5.0));
        if (window.zoom() <= 1.0 || window.offset() != afterZoom + QPointF(10.0, 5.0)) {
            std::fprintf(stderr, "pictura self-test: FAIL: zoom/pan transform wrong\n");
            return 3;
        }
        std::fprintf(stderr,
                     "pictura self-test: zoom=%.3f pan_ok=1\n",
                     window.zoom());
        std::fflush(stderr);
        QTimer::singleShot(2000, &app, &QCoreApplication::quit);
    }

    return app.exec();
}

#include "main.moc"
