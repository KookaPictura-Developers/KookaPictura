#pragma once

#include <QtWidgets/QWidget>

class QGridLayout;
class QScrollBar;
class QResizeEvent;

namespace pictura {

class ImageView;

// Thin host around an ImageView adding horizontal/vertical workspace
// scrollbars. The bars are a pure projection of the canvas offset and zoom;
// ImageView::offset() stays the single source of truth and dragging a bar calls
// ImageView::setOffset, which clamps through the shared reveal range.
class CanvasScrollBars : public QWidget {
    Q_OBJECT

public:
    explicit CanvasScrollBars(QWidget* parent = nullptr);

    void setView(ImageView* view);
    ImageView* view() const { return view_; }

    QScrollBar* horizontalBarForTest() const { return hbar_; }
    QScrollBar* verticalBarForTest() const { return vbar_; }

protected:
    void resizeEvent(QResizeEvent* event) override;

private:
    void syncFromView();

    ImageView* view_ = nullptr;
    QScrollBar* hbar_ = nullptr;
    QScrollBar* vbar_ = nullptr;
    QGridLayout* grid_ = nullptr;
    bool syncing_ = false;
};

} // namespace pictura
