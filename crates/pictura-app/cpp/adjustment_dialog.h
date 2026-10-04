#pragma once

#include <QtCore/QByteArray>
#include <QtCore/QRect>
#include <QtWidgets/QDialog>

class QCheckBox;

namespace pictura {

class AdjustmentControls;
class PictureView;

// An Image > Adjustments dialog (Levels…, Curves…, Hue/Saturation…, and the
// rest the engine describes): the adjustment's controls, built from the same
// descriptor page as the Properties panel, a Preview checkbox that shows the
// result on the canvas while the dialog is open, and OK / Cancel. OK applies
// the adjustment to the active pixel layer within the selection as one state
// named for it; Cancel restores the layer exactly. Ported from photorust's
// per-adjustment dialogs (LevelsDialog, CurvesDialog, …).
// ponytail: no presets menu, Auto / eyedropper buttons, histograms, or
// Load / Save; the preview covers the visible canvas section.
class AdjustmentDialog : public QDialog {
    Q_OBJECT

public:
    // `block` is the opening adjustment (`image_adjustment_default`);
    // `visible` the canvas section the preview refreshes.
    AdjustmentDialog(PictureView* view, const QByteArray& block, const QRect& visible,
                     QWidget* parent = nullptr);
    ~AdjustmentDialog() override;

    QByteArray block() const { return block_; }
    QWidget* controlForTest(const QString& key) const;

    void accept() override;
    void reject() override;

private:
    void edited(const QByteArray& next);
    void preview();
    void cancelPreview();

    PictureView* view_ = nullptr;
    QByteArray block_;
    QRect visible_;
    QString title_;
    bool previewing_ = false;
    AdjustmentControls* controls_ = nullptr;
    QCheckBox* preview_ = nullptr;
};

// Open the `kind` dialog (`"levels"`, …) over `view`'s active layer; true when
// it was applied.
bool runAdjustmentDialog(QWidget* parent, PictureView* view, const QString& kind,
                         const QColor& foreground, const QColor& background, const QRect& visible);

} // namespace pictura
