#pragma once

#include <QtCore/QPointF>
#include <QtCore/QStringList>
#include <QtWidgets/QWidget>

class QColor;
class QGridLayout;
class QLabel;
class QToolButton;
class QVBoxLayout;

namespace pictura {

class PictureView;

class InfoPanel : public QWidget {
    Q_OBJECT

public:
    explicit InfoPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    void setCursorPosition(const QPointF& imagePos);
    // The Ruler tool's mode: the panel shows A/L and points W/H at the ruler.
    void setRulerMode(bool on);
    bool rulerMode() const { return rulerMode_; }
    void refresh();

    QString samplerTextForTest() const;
    QString colorBlockTextForTest(int index = 0) const;
    QString positionTextForTest() const;
    QString sizeTextForTest() const;
    QString rulerTextForTest() const;
    QString docTextForTest() const;
    void setColorModeForTest(int index, const QString& mode);
    void setMeasurementUnitForTest(int index, const QString& unit);

private:
    enum class ColorReadout { Grayscale, Rgb, Hsb, Cmyk, Lab };
    enum class MeasureUnit { Pixels, Inches, Centimeters, Millimeters, Points, Picas, Percent };
    enum class MenuKind { None, Color, Unit };

    struct Readout {
        QWidget* widget = nullptr;
        QToolButton* button = nullptr;
        QVBoxLayout* keysHost = nullptr;
        QVBoxLayout* valuesHost = nullptr;
        QStringList keyNames;
        QList<QLabel*> values;
        QLabel* footer = nullptr;
        ColorReadout colorMode = ColorReadout::Rgb;
        MeasureUnit unit = MeasureUnit::Pixels;
    };

    Readout* addReadout(QGridLayout* grid, int row, int column, const QStringList& keys,
                        const QString& iconId, const QString& footer, MenuKind menu);
    void rebuildRows(Readout* readout, const QStringList& keys);
    void setValues(Readout* readout, const QStringList& values);
    void applyColorMode(Readout* readout, ColorReadout mode);
    void applyUnit(Readout* readout, MeasureUnit unit);
    void refreshColorBlock(Readout* readout, const QColor& color);
    void rebuildTopRight();
    QString blockText(const Readout* readout) const;
    QString formatMeasure(double value, MeasureUnit unit, double percentBase) const;

    PictureView* view_ = nullptr;
    QPointF cursor_;
    bool rulerMode_ = false;
    QGridLayout* grid_ = nullptr;
    Readout* topLeft_ = nullptr;
    Readout* topRight_ = nullptr;
    Readout* position_ = nullptr;
    Readout* size_ = nullptr;
    QLabel* docLabel_ = nullptr;
    QLabel* samplersLabel_ = nullptr;
};

} // namespace pictura
