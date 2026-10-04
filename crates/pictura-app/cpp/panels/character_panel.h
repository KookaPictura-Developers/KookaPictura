#pragma once

#include <QtCore/QSize>
#include <QtCore/QString>
#include <QtGui/QColor>
#include <QtWidgets/QWidget>

class QCheckBox;
class QComboBox;
class QDoubleSpinBox;
class QFontComboBox;
class QSpinBox;
class QToolButton;

namespace pictura {

class PictureView;

// The Character panel (CS6 `Window > Character`, `Type > Panels > Character`):
// a live attribute editor over the active type layer's `CharacterAttrs`. With no
// type layer active it shows the model defaults and disables editing.
class CharacterPanel : public QWidget {
    Q_OBJECT

public:
    explicit CharacterPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    void refresh();

    // The dense form's natural minimum width would raise the panel column's
    // shared floor; the column supplies the width, so report none.
    QSize minimumSizeHint() const override;

    QDoubleSpinBox* sizeFieldForTest() const { return size_; }
    QDoubleSpinBox* trackingFieldForTest() const { return tracking_; }
    QCheckBox* underlineForTest() const { return underline_; }
    bool editingEnabledForTest() const;
    void commitForTest();

private:
    void apply();
    bool hasTypeLayer() const;
    void updateColorIcon();

    PictureView* view_ = nullptr;
    QFontComboBox* family_ = nullptr;
    QDoubleSpinBox* size_ = nullptr;
    QComboBox* leadingMode_ = nullptr;
    QDoubleSpinBox* leading_ = nullptr;
    QComboBox* kerningMode_ = nullptr;
    QSpinBox* kerning_ = nullptr;
    QDoubleSpinBox* tracking_ = nullptr;
    QDoubleSpinBox* horizontalScale_ = nullptr;
    QDoubleSpinBox* verticalScale_ = nullptr;
    QDoubleSpinBox* baselineShift_ = nullptr;
    QComboBox* antiAlias_ = nullptr;
    QToolButton* colorButton_ = nullptr;
    QCheckBox* allCaps_ = nullptr;
    QCheckBox* smallCaps_ = nullptr;
    QCheckBox* superscript_ = nullptr;
    QCheckBox* subscript_ = nullptr;
    QCheckBox* underline_ = nullptr;
    QCheckBox* strikethrough_ = nullptr;
    QColor color_ = Qt::black;
};

} // namespace pictura
