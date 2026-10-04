#pragma once

#include <QtCore/QSize>
#include <QtCore/QString>
#include <QtWidgets/QWidget>

class QCheckBox;
class QComboBox;
class QDoubleSpinBox;

namespace pictura {

class PictureView;

// The Paragraph panel (CS6 `Window > Paragraph`, `Type > Panels > Paragraph`): a
// live attribute editor over the active type layer's `ParagraphAttrs`. With no
// type layer active it shows the model defaults and disables editing.
class ParagraphPanel : public QWidget {
    Q_OBJECT

public:
    explicit ParagraphPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    void refresh();

    // The dense form's natural minimum width would raise the panel column's
    // shared floor; the column supplies the width, so report none.
    QSize minimumSizeHint() const override;

    QComboBox* justifyFieldForTest() const { return justify_; }
    QDoubleSpinBox* firstLineIndentFieldForTest() const { return firstLineIndent_; }
    bool editingEnabledForTest() const;
    void commitForTest();

private:
    void apply();
    bool hasTypeLayer() const;

    PictureView* view_ = nullptr;
    QComboBox* justify_ = nullptr;
    QDoubleSpinBox* startIndent_ = nullptr;
    QDoubleSpinBox* endIndent_ = nullptr;
    QDoubleSpinBox* firstLineIndent_ = nullptr;
    QDoubleSpinBox* spaceBefore_ = nullptr;
    QDoubleSpinBox* spaceAfter_ = nullptr;
    QCheckBox* hanging_ = nullptr;
    QCheckBox* hyphenate_ = nullptr;
    QComboBox* composer_ = nullptr;
};

} // namespace pictura
