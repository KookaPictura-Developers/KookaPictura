#pragma once

// Lighting Effects' rig as the workspace edits it: up to sixteen lights plus
// the properties shared by all of them, flattened to the `lighting-effects`
// slots `filter_map.rs` reads (the rig's nine, then thirteen per light), and
// CS6's preset styles.

#include <QtCore/QList>
#include <QtCore/QPointF>
#include <QtCore/QString>
#include <QtGui/QColor>

namespace pictura {

constexpr int kLightingRigSlots = 9;
constexpr int kLightingLightSlots = 13;
constexpr int kMaxLights = 16;

// In the order of the Properties panel's light type list.
enum class LightKind { Spot = 0, Point = 1, Infinite = 2 };

// One light; positions are fractions of the picture, sizes fractions of its
// half-diagonal, angles screen degrees (y down). See `render/lighting.rs`.
struct LightSpec {
    LightKind kind = LightKind::Spot;
    bool on = true;
    QColor color = QColor(255, 255, 255);
    double intensity = 35.0;
    double hotspot = 69.0;
    QPointF center{0.5, 0.5};
    double angle = 325.0;
    double size = 0.6;
    double width = 0.32;
    double elevation = 50.0;
};

struct LightingRig {
    QColor colorize = QColor(255, 255, 255);
    double exposure = 0.0;
    double gloss = 0.0;
    double metallic = 0.0;
    double ambience = 0.0;
    int texture = 0; // None, Red, Green, Blue
    double height = 50.0;
    QList<LightSpec> lights;

    QList<double> toSlots() const;
    // False (and `out` untouched) unless `params` is a whole rig of 1..16 lights.
    static bool fromSlots(const QList<double>& params, LightingRig* out);
};

// A new light of `kind`, as the Lights buttons add it.
LightSpec defaultLight(LightKind kind);

// "Spot", "Point", "Infinite".
QString lightKindName(LightKind kind);

// "Spot Light 1", "Point Light 2", …: the Lights panel's name for each light,
// numbered per type in rig order.
QList<QString> lightNames(const LightingRig& rig);

struct LightingPreset {
    QString name;
    LightingRig rig;
};

// CS6's light styles, in its Presets menu order, `Default` among them.
// ponytail: Adobe publishes each style's colours, intensities, and focus, not
// where its lights sit; the placements are by eye.
const QList<LightingPreset>& lightingPresets();

// The rig the workspace opens on: CS6's Default style.
LightingRig defaultLightingRig();

// A Spot's hotspot ellipse in units of its outer ellipse: semi-axes as
// fractions of the outer semi-axes, and the centre's offset along the aim as
// a fraction of the semi-major axis. Mirrors `render::spot_hotspot`.
struct HotspotShape {
    double major = 0.0;
    double minor = 0.0;
    double offset = 0.0;
};
HotspotShape spotHotspot(double hotspot);

} // namespace pictura
