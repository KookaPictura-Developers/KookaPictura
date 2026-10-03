#include "paths_panel.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/paths.cxxqt.h"

#include <QtGui/QFont>
#include <QtGui/QPainter>
#include <QtGui/QPainterPath>
#include <QtGui/QPixmap>
#include <QtWidgets/QLabel>
#include <QtWidgets/QListWidget>
#include <QtWidgets/QVBoxLayout>

#include <string>

namespace pictura {

namespace {

constexpr int kThumbHeight = 32;

// The targeted path as one QPainterPath in document pixels.
QPainterPath targetPath(const PictureView& v)
{
    QPainterPath out;
    const int subpaths = path_subpath_count(v);
    for (int sp = 0; sp < subpaths; ++sp) {
        const int count = path_point_count(v, sp);
        const int segments = path_subpath_closed(v, sp) ? count : count - 1;
        ::rust::Vec<double> a = path_point(v, sp, 0);
        if (a.size() != 9) {
            continue;
        }
        out.moveTo(a[0], a[1]);
        for (int i = 0; i < segments; ++i) {
            const ::rust::Vec<double> b = path_point(v, sp, (i + 1) % count);
            if (b.size() != 9) {
                break;
            }
            // A missing handle is reported at its anchor, so this is a plain
            // cubic either way.
            out.cubicTo(a[6], a[7], b[3], b[4], b[0], b[1]);
            a = b;
        }
    }
    return out;
}

// CS6's path thumbnail: the canvas as a white card with the path drawn on it.
QPixmap thumbnail(const PictureView& v, const QPainterPath& path)
{
    const double w = qMax(1, v.document_width());
    const double h = qMax(1, v.document_height());
    const double scale = kThumbHeight / h;
    QPixmap pixmap(qMax(1, qRound(w * scale)), kThumbHeight);
    pixmap.fill(Qt::white);
    QPainter painter(&pixmap);
    painter.setRenderHint(QPainter::Antialiasing, true);
    painter.scale(scale, scale);
    painter.setPen(QPen(Qt::black, 1.0 / scale));
    painter.drawPath(path);
    return pixmap;
}

} // namespace

PathsPanel::PathsPanel(QWidget* parent)
    : QWidget(parent)
{
    setWindowTitle(tr("Paths"));
    auto* layout = new QVBoxLayout(this);
    list_ = new QListWidget(this);
    list_->setObjectName(QStringLiteral("pathsList"));
    list_->setIconSize(QSize(kThumbHeight * 2, kThumbHeight));
    layout->addWidget(list_, 1);
    empty_ = new QLabel(tr("No Paths"), this);
    empty_->setAlignment(Qt::AlignCenter);
    empty_->setEnabled(false);
    layout->addWidget(empty_, 1, Qt::AlignCenter);
    refresh();
}

void PathsPanel::setView(PictureView* view) { view_ = view; }

// The active shape layer's "<Layer> Shape Path" (as CS6 lists a vector mask),
// then the Work Path. The path calls are pointed at each in turn and restored.
void PathsPanel::refresh()
{
    list_->clear();
    if (view_ && view_->has_document()) {
        PictureView& v = *view_;
        const bool layerTarget = path_layer_target(v);
        path_set_layer_target(v, true);
        if (path_target_is_layer(v)) {
            const QString name = QString::fromStdString(std::string(path_target_name(v)));
            auto* item = new QListWidgetItem(QIcon(thumbnail(v, targetPath(v))),
                                             tr("%1 Shape Path").arg(name), list_);
            item->setData(Qt::UserRole, QStringLiteral("shape"));
            list_->setCurrentItem(item);
        }
        path_set_layer_target(v, false);
        if (path_subpath_count(v) > 0) {
            // CS6 sets the temporary Work Path's name in italics.
            auto* item = new QListWidgetItem(QIcon(thumbnail(v, targetPath(v))),
                                             tr("Work Path"), list_);
            QFont font = item->font();
            font.setItalic(true);
            item->setFont(font);
            if (!list_->currentItem()) {
                list_->setCurrentItem(item);
            }
        }
        path_set_layer_target(v, layerTarget);
    }
    const bool has = list_->count() > 0;
    list_->setVisible(has);
    empty_->setVisible(!has);
}

} // namespace pictura
