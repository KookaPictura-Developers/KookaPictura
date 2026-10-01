#pragma once

#include <QtWidgets/QWidget>

class QLabel;
class QListWidget;

namespace pictura {

class PictureView;

// Window > Paths: the document's Work Path, listed with a thumbnail once the
// Pen tool group has drawn one, "No Paths" otherwise.
// ponytail: the Work Path only; no saved paths, fill / stroke / selection
// buttons, or panel menu yet.
class PathsPanel : public QWidget {
    Q_OBJECT

public:
    explicit PathsPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    void refresh();

    QListWidget* listForTest() const { return list_; }

private:
    PictureView* view_ = nullptr;
    QListWidget* list_ = nullptr;
    QLabel* empty_ = nullptr;
};

} // namespace pictura
