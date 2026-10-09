#pragma once

// The Lighting Effects workspace's preview: the picture fitted on a dark
// pasteboard under CS6's on-canvas light controls. The selected light shows
// its whole widget — a Spot's ellipse with its four handles and hotspot, a
// Point's radius ring, an Infinite light's direction disc — and every light
// its centre handle inside an Intensity ring. Dragging edits the rig in place.

#include <QtCore/QPointF>
#include <QtCore/QRectF>
#include <QtCore/QSize>
#include <QtGui/QImage>
#include <QtWidgets/QWidget>

#include "lighting_rig.h"

namespace pictura {

class LightingCanvas : public QWidget {
    Q_OBJECT

public:
    // The part of a light a press lands on.
    enum class Part {
        None,
        Move,      // inside a Spot, Point, or Infinite disc
        Rotate,    // beyond the selected Spot's ellipse
        Major,     // a Spot's handle on its long axis
        Minor,     // a Spot's handle on its short axis
        Hotspot,   // a Spot's hotspot edge
        Radius,    // a Point's ring
        Direction, // an Infinite light's end handle
        Intensity, // the ring round any light's centre
    };

    explicit LightingCanvas(LightingRig* rig, QWidget* parent = nullptr);

    void setImage(const QImage& image);
    QImage image() const { return image_; }
    void setAspect(const QSize& aspect);

    int selected() const { return selected_; }
    void setSelected(int index);

    // Where the picture sits in the widget.
    QRectF imageRect() const;
    // Widget position of a picture fraction, and back.
    QPointF toWidget(const QPointF& fraction) const;
    QPointF toFraction(const QPointF& widget) const;
    // Widget position of a light's handle (`Major`, `Minor`, `Direction`,
    // `Intensity` — the top of the ring — or `Move`, the centre).
    QPointF handlePosition(int index, Part part) const;

    // What a press at `pos` would grab, and on which light; `index` is -1 when
    // nothing.
    Part hitTest(const QPointF& pos, int* index) const;

signals:
    // A drag changed the selected light.
    void rigEdited();
    void lightSelected(int index);
    // Alt-drag copied the selected light onto the end of the rig.
    void lightDuplicated(int index);
    // Delete / Backspace on the canvas.
    void deleteRequested();

protected:
    void paintEvent(QPaintEvent* event) override;
    void mousePressEvent(QMouseEvent* event) override;
    void mouseMoveEvent(QMouseEvent* event) override;
    void mouseReleaseEvent(QMouseEvent* event) override;
    void keyPressEvent(QKeyEvent* event) override;

private:
    double span() const;
    void drag(const QPointF& pos);

    LightingRig* rig_ = nullptr;
    QImage image_;
    QSize aspect_;
    int selected_ = 0;
    Part dragging_ = Part::None;
    QPointF grabOffset_;
};

} // namespace pictura
