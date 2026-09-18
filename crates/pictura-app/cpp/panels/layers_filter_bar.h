#pragma once

#include "layers_filter_proxy.h"

#include <QtWidgets/QWidget>

#include <array>

class QComboBox;
class QLineEdit;
class QStackedWidget;
class QToolButton;

namespace pictura {

class LayerFilterBar : public QWidget {
    Q_OBJECT

public:
    explicit LayerFilterBar(QWidget* parent = nullptr);

    void setFilter(const LayerFilter& filter);
    LayerFilter filter() const { return filter_; }

    int dimensionIndexForTest() const;
    void setDimensionForTest(const QString& key);

signals:
    void filterChanged(const LayerFilter& filter);

private:
    LayerFilter buildFilter() const;
    QString activeDimension() const;
    void userChanged();

    QComboBox* dimension_ = nullptr;
    QStackedWidget* stack_ = nullptr;
    QToolButton* toggle_ = nullptr;
    QLineEdit* name_ = nullptr;
    std::array<QToolButton*, 4> kindButtons_{};
    QComboBox* effect_ = nullptr;
    QComboBox* mode_ = nullptr;
    QComboBox* attribute_ = nullptr;
    QComboBox* color_ = nullptr;
    LayerFilter filter_;
    bool syncing_ = false;
};

} // namespace pictura
