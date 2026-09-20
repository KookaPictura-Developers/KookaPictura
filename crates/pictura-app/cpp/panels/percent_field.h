#pragma once

#include "numeric_field.h"

namespace pictura {

// A 0..100 integer percentage field: a `NumericField` configured with a `%`
// suffix inside the value box and a slider popup. Kept as a named configuration
// so the Layers panel Opacity and Fill fields read as before.
class PercentField : public NumericField {
    Q_OBJECT

public:
    explicit PercentField(const QString& label, QWidget* parent = nullptr);
};

} // namespace pictura
