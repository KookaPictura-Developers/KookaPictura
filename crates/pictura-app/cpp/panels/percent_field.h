#pragma once

#include <QtCore/QPoint>
#include <QtWidgets/QWidget>

class QLabel;
class QLineEdit;
class QSlider;
class QToolButton;

namespace pictura {

// A 0..100 percentage field: a leading label, a text box with an inside `%`,
// and an arrow that opens a slider popup. Pressing and dragging horizontally on
// the label, the `%`, or the field scrubs the value. `valueChanged` previews
// user input; `valueCommitted` fires once when the edit finishes. `setValue`
// never feeds back.
class PercentField : public QWidget {
    Q_OBJECT

public:
    explicit PercentField(const QString& label, QWidget* parent = nullptr);

    int value() const { return value_; }
    void setValue(int pct);
    QString labelText() const;

signals:
    void valueChanged(int pct);
    void valueCommitted(int pct);

protected:
    bool eventFilter(QObject* watched, QEvent* event) override;

private:
    void commitEdit();
    void applyUserValue(int pct);
    void commitPending();
    void layoutSuffix();
    void showPopup();

    QLabel* label_ = nullptr;
    QLineEdit* edit_ = nullptr;
    QLabel* suffix_ = nullptr;
    QToolButton* arrow_ = nullptr;
    QWidget* popup_ = nullptr;
    QSlider* slider_ = nullptr;
    int value_ = 100;
    bool syncing_ = false;
    bool scrubbing_ = false;
    bool pending_ = false;
    QPoint scrubOrigin_;
    int scrubStart_ = 0;
};

} // namespace pictura
