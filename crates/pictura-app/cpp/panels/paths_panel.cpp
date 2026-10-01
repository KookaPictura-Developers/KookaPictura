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

namespace pictura {

namespace {

constexpr int kThumbHeight = 32;

// The Work Path as one QPainterPath in document pixels.
QPainterPath workPath(const PictureView& v)
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

void PathsPanel::refresh()
{
    list_->clear();
    const bool has = view_ && view_->has_document() && path_subpath_count(*view_) > 0;
    if (has) {
        // CS6 sets the temporary Work Path's name in italics.
        auto* item = new QListWidgetItem(QIcon(thumbnail(*view_, workPath(*view_))),
                                         tr("Work Path"), list_);
        QFont font = item->font();
        font.setItalic(true);
        item->setFont(font);
        list_->setCurrentItem(item);
    }
    list_->setVisible(has);
    empty_->setVisible(!has);
}

} // namespace pictura
