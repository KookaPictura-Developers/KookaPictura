#pragma once

#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtGui/QColor>
#include <QtWidgets/QWidget>

#include <functional>
#include <vector>

class QComboBox;
class QFormLayout;
class QLabel;
class QStackedWidget;
class QTimer;
class QToolButton;
class QVBoxLayout;

namespace pictura {

class CurveWidget;
class PictureView;

// Window > Properties (PAN-006). For an adjustment layer it builds the
// adjustment's controls from the engine's per-kind descriptor
// (`adjustment_page`), edits live on the canvas, and records one "Modify …
// Layer" state per gesture (a slider release, a field or toggle change, or a
// pause in a curve drag); the footer carries Clip to Layer, Reset, Toggle
// Visibility, and Delete. Any other layer gets a read-only summary (kind, size,
// position, blend, opacity, fill, mask, locks) — a post-CS6 page photorust
// has, ported on request (#70). With no layer it reads No Properties.
// Ported from photorust's PropertiesPanel; Kooka's panel column hosts it.
// ponytail: no adjustment Presets menu, mask page, Previous State toggle, or
// Auto-Select menu items; the controls an engine descriptor omits are absent.
class PropertiesPanel : public QWidget {
    Q_OBJECT

public:
    explicit PropertiesPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    void refresh();

    // The header: the adjustment's name, "Layer Properties", or "No Properties".
    QString messageForTest() const;
    // The control editing `key`, for the Qt Test.
    QWidget* controlForTest(const QString& key) const;
    QString pathForTest() const { return path_; }
    // Commit any pending live edit now (as the commit timer would).
    void commitForTest() { commit(); }

private:
    struct Row {
        QString key;
        int group = -1;
        QWidget* widget = nullptr;      // the row container, shown per group
        QWidget* control = nullptr;     // the slider / check / combo / colour button
        std::function<void(double)> load;
    };

    // The selected layer's path and its Layers-panel row, or empty / -1.
    QString activePath(int* row) const;
    void showNothing();
    void showLayer(int row);
    void showAdjustment(const QStringList& page);
    void buildAdjustment(const QStringList& page);
    void loadAdjustment(const QStringList& page);
    void clearAdjustment();
    void addSlider(const QStringList& cells);
    void addCheck(const QStringList& cells);
    void addChoice(const QStringList& cells);
    void addColor(const QStringList& cells);
    void addCurves();
    void showGroup(int group);
    void loadCurve();
    // Write one parameter live, then (re)arm the commit.
    void push(const QString& key, double value);
    void edited();
    void commit();

    PictureView* view_ = nullptr;
    QString path_;
    QString builtFor_;
    QString title_;
    bool loading_ = false;
    bool dirty_ = false;

    QLabel* header_ = nullptr;
    QStackedWidget* stack_ = nullptr;
    QTimer* commitTimer_ = nullptr;

    // Adjustment page.
    QWidget* adjustmentPage_ = nullptr;
    QVBoxLayout* controls_ = nullptr;
    QComboBox* groups_ = nullptr;
    QLabel* note_ = nullptr;
    std::vector<Row> rows_;
    CurveWidget* curve_ = nullptr;
    QComboBox* curveChannel_ = nullptr;
    QWidget* footer_ = nullptr;
    QToolButton* clip_ = nullptr;
    QToolButton* visible_ = nullptr;

    // Layer page.
    QFormLayout* info_ = nullptr;
    QWidget* layerPage_ = nullptr;
};

} // namespace pictura
