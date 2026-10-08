#pragma once

#include <QtCore/QByteArray>
#include <QtCore/QList>
#include <QtCore/QPair>
#include <QtCore/QString>
#include <QtGui/QImage>
#include <QtWidgets/QDialog>

#include "filter_commands.h"

class QComboBox;
class QLabel;
class QListWidget;
class QListWidgetItem;
class QScrollArea;
class QShowEvent;
class QTimer;
class QToolButton;
class QVBoxLayout;
class QWidget;

namespace pictura {

class FilterParamControls;
class PictureView;

// CS6's Filter Gallery (docs/06-filters/artistic-filters.md, FILT-080): a
// large preview, the gallery filters as thumbnails by category, the selected
// effect's options, and a stack of effect layers. Effects stack like layers —
// the bottom row is applied first — and each can be hidden with its eye,
// dragged to reorder, or deleted. The stack previews on the canvas as it
// changes; OK commits it as one "Filter Gallery" history state, Cancel
// restores the layer.
class FilterGalleryDialog : public QDialog {
    Q_OBJECT

public:
    struct Effect {
        QString kind;
        QList<double> values;
        bool visible = true;
    };

    using Category = QPair<QString, QList<const FilterCommandSpec*>>;

    enum class CommitResult { Applied, NothingVisible, Refused };

    explicit FilterGalleryDialog(PictureView* view, QWidget* parent = nullptr);

    // The gallery's filters by category, in CS6 order. A gallery filter with no
    // engine kernel yet is left out.
    static QList<Category> categories();

    // The picture as a thumbnail sees it: scaled to cover the thumbnail size
    // and cropped to its centre, in RGBA8888.
    static QImage thumbnailSample(const QImage& picture);

    // The stack, first-applied first.
    QList<Effect> effects() const { return effects_; }

    // Index into effects() of the effect whose options are shown.
    int selectedEffect() const { return selected_; }

    // What clicking a thumbnail does: the selected effect becomes `kind`, on
    // its defaults.
    void selectFilter(const QString& kind);

    // New effect layer: a copy of the selected effect, above it.
    void addEffect();

    // Delete the selected effect; the last one cannot be deleted.
    void deleteEffect();

    // Show or hide the effect at `index` (the eye).
    void setEffectVisible(int index, bool visible);

    // Commit the visible effects as one history state. With every effect
    // hidden nothing is committed, which is not a refusal.
    CommitResult commit();

protected:
    void showEvent(QShowEvent* event) override;
    bool eventFilter(QObject* watched, QEvent* event) override;

private:
    QWidget* buildThumbnails();
    QWidget* buildPreview();
    void selectEffect(int index);
    void rebuildList();
    void showVisibility(QListWidgetItem* item, bool visible);
    void effectMoved(int from, int to);
    void schedulePreview();
    void runPreview();
    void showPreviewImage();
    void setZoom(double zoom);
    double fitZoom() const;
    void renderNextThumbnail();
    void discardPreview();

    PictureView* view_ = nullptr;
    QList<Effect> effects_;
    int selected_ = 0;
    bool previewShown_ = false;
    double zoom_ = 1.0;

    QScrollArea* previewArea_ = nullptr;
    QLabel* previewLabel_ = nullptr;
    QLabel* zoomLabel_ = nullptr;
    QWidget* thumbnailPane_ = nullptr;
    QList<QToolButton*> thumbnails_;
    QComboBox* filterCombo_ = nullptr;
    QVBoxLayout* optionsLayout_ = nullptr;
    QWidget* options_ = nullptr;
    FilterParamControls* controls_ = nullptr;
    QListWidget* list_ = nullptr;
    QTimer* previewTimer_ = nullptr;
    QImage thumbnailSample_;
    QByteArray thumbnailRgba_;
    int nextThumbnail_ = 0;
};

} // namespace pictura
