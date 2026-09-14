#include <QtCore/QCoreApplication>
#include <QtCore/QDebug>
#include <QtCore/QSet>
#include <QtCore/QTimer>
#include <QtGui/QImage>
#include <QtGui/QMouseEvent>
#include <QtGui/QPainter>
#include <QtGui/QWheelEvent>
#include <QtWidgets/QApplication>
#include <QtWidgets/QWidget>

#include <cmath>
#include <cstdint>
#include <cstdio>

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include "interop.h"

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

    // Surface Qt's own diagnostics (QRhi logs through qWarning) on stderr so
    // the interop probe can capture them under xvfb/offscreen.
    qInstallMessageHandler([](QtMsgType type, const QMessageLogContext&, const QString& message) {
        std::fprintf(stderr, "qt[%d]: %s\n", static_cast<int>(type), message.toLocal8Bit().constData());
        std::fflush(stderr);
    });

    const QStringList args = app.arguments();
    bool selfTest = false;
    bool interopProbe = false;
    QString psdPath;
    for (int i = 1; i < args.size(); ++i) {
        if (args.at(i) == QStringLiteral("--self-test")) {
            selfTest = true;
        } else if (args.at(i) == QStringLiteral("--interop-probe")) {
            interopProbe = true;
        } else if (!args.at(i).startsWith(QLatin1Char('-'))) {
            psdPath = args.at(i);
        }
    }

    pictura::PictureView view;

    if (interopProbe) {
        const bool prepared = view.gpu_interop_prepare();
        if (!prepared) {
            std::fprintf(stderr, "pictura interop-probe: no Vulkan device\n");
            return 0;
        }
        const std::int32_t result = pictura_try_qrhi_import(view.gpu_vk_instance(),
                                                            view.gpu_vk_physical_device(),
                                                            view.gpu_vk_device(),
                                                            view.gpu_vk_queue_family(),
                                                            view.gpu_vk_image(),
                                                            view.gpu_image_width(),
                                                            view.gpu_image_height());
        std::fprintf(stderr, "pictura interop-probe: qrhi_import=%d\n", result);
        std::fflush(stderr);
        return 0;
    }

    const bool codecLoaded = view.open(psdPath);
    // M0.5: fall back to the offscreen GPU demo only when no document loaded
    // (a loaded PSD must keep its composited image). 0 = no GPU,
    // 1 = rendered non-blank, 2 = rendered blank.
    int gpu = 0;
    if (!codecLoaded) {
        gpu = view.render_gpu();
    }
    const QImage image = view.image();

    ImageView window;
    window.setImage(image);
    window.resize(900, 650);
    window.show();

    if (selfTest) {
        std::fprintf(stderr,
                     "pictura self-test: image=%dx%d codec_loaded=%d gpu=%d\n",
                     image.width(),
                     image.height(),
                     codecLoaded ? 1 : 0,
                     gpu);
        std::fflush(stderr);
        if (image.isNull()) {
            std::fprintf(stderr, "pictura self-test: FAIL: null image\n");
            return 2;
        }
        if (gpu == 2) {
            std::fprintf(stderr, "pictura self-test: FAIL: GPU render was blank\n");
            return 4;
        }
        if (!codecLoaded) {
            // The generated image (GPU or CPU fallback) must not be blank.
            QSet<QRgb> seen;
            for (int gy = 0; gy < 8; ++gy) {
                for (int gx = 0; gx < 8; ++gx) {
                    const int x = image.width() * gx / 8 + image.width() / 16;
                    const int y = image.height() * gy / 8 + image.height() / 16;
                    seen.insert(image.pixel(x, y));
                }
            }
            std::fprintf(stderr,
                         "pictura self-test: nonblank=%d distinct=%d\n",
                         seen.size() >= 2 ? 1 : 0,
                         seen.size());
            std::fflush(stderr);
            if (seen.size() < 2) {
                std::fprintf(stderr, "pictura self-test: FAIL: blank render\n");
                return 5;
            }
        }
        if (codecLoaded) {
            // two_layers.psd is an 8x8 layer stack: a red top-left quadrant and
            // a blue bottom-right quadrant; the other quadrants are uncovered.
            QSet<QRgb> seen;
            for (int y = 0; y < image.height(); ++y) {
                for (int x = 0; x < image.width(); ++x) {
                    seen.insert(image.pixel(x, y));
                }
            }
            std::fprintf(stderr, "pictura self-test: layered distinct=%d\n", seen.size());
            std::fflush(stderr);
            if (seen.size() < 2) {
                std::fprintf(stderr, "pictura self-test: FAIL: blank composition\n");
                return 6;
            }
            if (image.width() >= 8 && image.height() >= 8) {
                const QRgb tl = image.pixel(2, 2);
                const QRgb br = image.pixel(6, 6);
                const QRgb tr = image.pixel(6, 2);
                const QRgb bl = image.pixel(2, 6);
                std::fprintf(stderr,
                             "pictura self-test: tl=(%d,%d,%d,a%d) br=(%d,%d,%d,a%d) "
                             "tr_a=%d bl_a=%d\n",
                             qRed(tl),
                             qGreen(tl),
                             qBlue(tl),
                             qAlpha(tl),
                             qRed(br),
                             qGreen(br),
                             qBlue(br),
                             qAlpha(br),
                             qAlpha(tr),
                             qAlpha(bl));
                std::fflush(stderr);
                const bool red = qRed(tl) > 200 && qGreen(tl) < 60 && qBlue(tl) < 60
                                 && qAlpha(tl) == 255;
                const bool blue = qBlue(br) > 200 && qRed(br) < 60 && qGreen(br) < 60
                                  && qAlpha(br) == 255;
                if (!red || !blue) {
                    std::fprintf(stderr, "pictura self-test: FAIL: composited quadrants wrong\n");
                    return 7;
                }
                // Uncovered quadrants must be transparent: this proves the layer
                // stack was composited, not the opaque embedded PSD composite.
                if (qAlpha(tr) != 0 || qAlpha(bl) != 0) {
                    std::fprintf(stderr, "pictura self-test: FAIL: layer stack not composited\n");
                    return 8;
                }
            }
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
