#pragma once

// Per-filter dialog descriptors, ported from perfecto25/photorust's
// MainWindow applyFilterWith parameter table.
// Source: https://github.com/perfecto25/photorust

#include <QtCore/QList>
#include <QtCore/QString>
#include <QtCore/QStringList>

namespace pictura {

// The control a filter parameter slot is edited with. Every control maps to one
// or more consecutive slots in the ordered value list the Rust mapping expects.
enum class FilterControl {
    Slider,    // one slot
    Angle,     // one slot, degrees, with a wheel
    Choice,    // one slot, an index into `choiceValues`
    CheckBox,  // one slot, 0 or 1
    Color,     // three slots: red, green, blue (0..255)
    Placement, // two slots: x, y (normalized 0..1)
    BlurCenter, // display-only centre pad, contributes no slot
};

// One parameter's control identity and range. Ported from photorust's
// per-filter dialog descriptors.
struct FilterParamSpec {
    FilterControl control = FilterControl::Slider;
    QString label;
    double minimum = 0.0;
    double maximum = 100.0;
    double value = 0.0;
    int decimals = 0;
    QString suffix;
    QStringList choices;
    QList<double> choiceValues;
    // Defaults for multi-slot controls (Color: r,g,b; Placement: x,y).
    QList<double> initial;
};

// One implemented Filter-menu leaf: its menu path, the `kind` the bridge maps,
// and the ordered parameters its dialog edits.
struct FilterCommandSpec {
    QStringList path; // {"Filter", [<family>], <leaf label>}
    QString label;
    QString kind;
    QList<FilterParamSpec> params;
    bool previewPane = true; // Radial Blur uses a Blur Center instead
    // One wide column under the preview instead of spilling into a second
    // input column; checkboxes trail the column.
    bool stacked = false;
};

// Every Filter-menu leaf that has an engine kernel, in menu order.
const QList<FilterCommandSpec>& filterCommands();

// The definition for a Filter-menu leaf path, or nullptr when the leaf has no
// engine kernel (a disabled stub).
const FilterCommandSpec* filterCommandForPath(const QStringList& path);

// The definition for a bridge `kind`, or nullptr when unknown.
const FilterCommandSpec* filterCommandForKind(const QString& kind);

} // namespace pictura
