#pragma once

// CS6's shared filter dialog, ported from perfecto25/photorust's
// shell/src/dialogs/FilterPreviewDialog.{h,cpp}.
// Source: https://github.com/perfecto25/photorust

#include <QtCore/QList>
#include <QtCore/QPointF>
#include <QtCore/QRectF>
#include <QtWidgets/QDialog>

#include "filter_commands.h"

class QCheckBox;
class QComboBox;
class QDoubleSpinBox;
class QLabel;
class QPushButton;
class QResizeEvent;
class QShowEvent;
class QSlider;

namespace pictura {

class FilterParamControls;
class PictureView;

// The canvas state a filter dialog previews against: the document-space rect
// the user is looking at and its zoom, so the thumbnail shows that section at a
// comparable scale and the live preview only filters the visible area.
struct FilterPreviewView {
    QRectF visible;
    double canvasZoom = 1.0;
};

// CS6's shared filter dialog: a preview thumbnail with its own zoom and a
// Preview checkbox, plus one control per filter parameter. Radial Blur lays out
// without the thumbnail; every other filter re-renders a non-committing preview
// on the canvas as a control changes. Ported from photorust's FilterPreviewDialog.
class FilterPreviewDialog : public QDialog {
    Q_OBJECT

public:
    FilterPreviewDialog(PictureView* view, const FilterCommandSpec& spec,
                        const FilterPreviewView& previewView = {}, QWidget* parent = nullptr);

    // The parameter slot values in mapping order.
    QList<double> values() const;

    // Run the dialog modally, committing `out` on OK. On cancel the canvas
    // preview is discarded.
    static bool get(PictureView* view, const FilterCommandSpec& spec,
                    const FilterPreviewView& previewView, const QList<double>& initial,
                    QList<double>* out, QWidget* parent);

private:
    void applyInitial(const QList<double>& initial);
    void valuesChanged();
    void discardPreview();
    void updateThumbnail();

protected:
    void resizeEvent(QResizeEvent* event) override;
    void showEvent(QShowEvent* event) override;

private:
    PictureView* view_ = nullptr;
    const FilterCommandSpec spec_;
    FilterParamControls* controls_ = nullptr;
    QCheckBox* preview_ = nullptr;
    QLabel* thumbnail_ = nullptr;
    QLabel* zoomLabel_ = nullptr;
    int zoom_ = 2;
    bool previewShown_ = false;
    bool shownOnce_ = false;
    QRectF previewVisible_;
    double canvasZoom_ = 1.0;
};

} // namespace pictura
