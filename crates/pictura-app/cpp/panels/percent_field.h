#pragma once

#include <QtCore/QPoint>
#include <QtWidgets/QWidget>

class QLineEdit;
class QSlider;
class QToolButton;

namespace pictura {

// A 0..100 percentage field: a text box plus an arrow that opens a slider
// popup. Pressing and dragging horizontally on the field scrubs the value.
// `valueChanged` is emitted for user input only; `setValue` never feeds back.
class PercentField : public QWidget {
    Q_OBJECT

public:
    explicit PercentField(QWidget* parent = nullptr);

    int value() const { return value_; }
    void setValue(int pct);

signals:
    void valueChanged(int pct);

protected:
    bool eventFilter(QObject* watched, QEvent* event) override;

private:
    void commitEdit();
    void applyUserValue(int pct);
    void showPopup();

    QLineEdit* edit_ = nullptr;
    QToolButton* arrow_ = nullptr;
    QWidget* popup_ = nullptr;
    QSlider* slider_ = nullptr;
    int value_ = 100;
    bool syncing_ = false;
    bool scrubbing_ = false;
    QPoint scrubOrigin_;
    int scrubStart_ = 0;
};

} // namespace pictura
