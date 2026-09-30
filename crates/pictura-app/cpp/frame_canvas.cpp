#include "frame_canvas.h"

#include "image_view.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QPointer>
#include <QtCore/QStringList>

namespace pictura {

void wireCanvasLevelProvider(PictureView* view, ImageView* canvas)
{
    if (!view || !canvas) {
        return;
    }
    // Present from the view pyramid: the canvas crops a level instead of
    // scaling the full-resolution image. The lambdas stay valid via QPointer.
    QPointer<PictureView> levelView = view;
    canvas->setLevelProvider(ImageView::LevelProvider{
        [levelView](int level, int x, int y, int w, int h) {
            return levelView ? levelView->display_image(level, x, y, w, h) : QImage();
        },
        [levelView]() { return levelView ? levelView->display_level_count() : 0; },
        [levelView](int level) {
            QSize size;
            if (levelView) {
                const QStringList parts = levelView->display_level_size(level).split(
                    QLatin1Char(' '), Qt::SkipEmptyParts);
                if (parts.size() == 2) {
                    size = QSize(parts.at(0).toInt(), parts.at(1).toInt());
                }
            }
            return size;
        },
        [levelView]() { return levelView ? levelView->canvas_revision() : quint64(0); },
        [levelView]() { return levelView && levelView->is_painting(); },
        [levelView]() { return levelView ? levelView->preview_present_level() : 0; }});
}

} // namespace pictura
