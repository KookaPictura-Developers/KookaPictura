#pragma once

// CS6's Lens Flare dialog: the whole picture with a draggable flare-centre
// crosshair, Brightness, and the Lens Type radio group. The parameter model
// (brightness, centre, lens) is a struct, not a run of independent slots, so it
// does not fit the generic FilterPreviewDialog.

#include <QtCore/QByteArray>
#include <QtCore/QList>
#include <QtCore/QPointF>
#include <QtGui/QImage>
#include <QtWidgets/QDialog>

#include "filter_commands.h"

class QButtonGroup;
class QCheckBox;
class QSlider;
class QSpinBox;

namespace pictura {

class FlareCenterPad;
class PictureView;

class LensFlareDialog : public QDialog {
    Q_OBJECT

public:
    LensFlareDialog(PictureView* view, const FilterCommandSpec& spec, QWidget* parent = nullptr);

    // The `lens-flare` slots: brightness, centre x, centre y, lens index.
    QList<double> values() const;
    // Silent: emits no preview.
    void setValues(const QList<double>& values);

    // The flare centre as fractions of the picture.
    QPointF center() const;
    void setCenter(const QPointF& center);

    // The pad's picture: the proxy with the current flare thrown on it.
    QImage padImage() const;

    // Run the dialog modally, committing `out` on OK. On cancel the canvas
    // preview is discarded.
    static bool get(PictureView* view, const FilterCommandSpec& spec,
                    const QList<double>& initial, QList<double>* out, QWidget* parent);

protected:
    void showEvent(QShowEvent* event) override;

private:
    void valuesChanged(bool canvas);
    void discardPreview();

    PictureView* view_ = nullptr;
    const FilterCommandSpec spec_;
    FlareCenterPad* pad_ = nullptr;
    QSpinBox* brightness_ = nullptr;
    QSlider* slider_ = nullptr;
    QButtonGroup* lens_ = nullptr;
    QCheckBox* preview_ = nullptr;
    // The picture shrunk to the pad, taken before any canvas preview so the
    // flare is never thrown twice.
    QImage proxy_;
    QByteArray proxyRgba_;
    bool previewShown_ = false;
    bool shownOnce_ = false;
};

} // namespace pictura
