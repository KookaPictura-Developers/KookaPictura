#pragma once

#include <QtWidgets/QWidget>

class QListWidget;
class QToolButton;

namespace pictura {

class PictureView;

// Type > Paragraph Styles / Window > Paragraph Styles: the document's named
// paragraph styles, applied to the active type layer by clicking one. Ported
// from photorust's ParagraphStylesPanel.
//
// The styles live on the document (`ParagraphStyle` / `TextStyleSheet`), so
// applying or editing one rides the history snapshot. The default
// `Basic Paragraph` style is always present and cannot be deleted. The footer
// creates a style from the options dialog and deletes the selected one;
// double-clicking a row edits it in place, which re-applies it to every layer
// that set it.
class ParagraphStylesPanel : public QWidget {
    Q_OBJECT

public:
    explicit ParagraphStylesPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    void refresh();

    QListWidget* listForTest() const { return list_; }
    // Apply the style at `row` to the active type layer, as a click would.
    void applyRowForTest(int row) { applyRow(row); }

    // The footer actions on the selected row; false when nothing happened.
    bool createStyle();
    bool editStyle(int row);
    bool deleteSelectedStyle();

private:
    // A row's Qt::UserRole value: false for the default style.
    int selectedRow() const;
    void applyRow(int row);

    PictureView* view_ = nullptr;
    QListWidget* list_ = nullptr;
    QToolButton* add_ = nullptr;
    QToolButton* remove_ = nullptr;
    // refresh() rewrites the list; its signals must not write back.
    bool updating_ = false;
};

} // namespace pictura
