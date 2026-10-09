#include "lighting_rig.h"

#include <QtCore/QtMath>

#include <algorithm>
#include <tuple>

namespace pictura {

namespace {

LightSpec spot(const QColor& color, double intensity, double hotspot, QPointF center,
               double angle, double size, double width)
{
    LightSpec light;
    light.color = color;
    light.intensity = intensity;
    light.hotspot = hotspot;
    light.center = center;
    light.angle = angle;
    light.size = size;
    light.width = width;
    return light;
}

LightSpec point(const QColor& color, double intensity, QPointF center, double size)
{
    LightSpec light = defaultLight(LightKind::Point);
    light.color = color;
    light.intensity = intensity;
    light.center = center;
    light.size = size;
    return light;
}

LightSpec infinite(const QColor& color, double intensity, double angle, double elevation)
{
    LightSpec light = defaultLight(LightKind::Infinite);
    light.color = color;
    light.intensity = intensity;
    light.angle = angle;
    light.elevation = elevation;
    return light;
}

LightingRig rigOf(QList<LightSpec> lights)
{
    LightingRig rig;
    rig.lights = std::move(lights);
    return rig;
}

const QColor kWhite(255, 255, 255);
const QColor kYellow(255, 236, 150);
const QColor kBlue(90, 120, 255);
const QColor kRed(255, 60, 50);
const QColor kGreen(60, 220, 80);

} // namespace

QList<double> LightingRig::toSlots() const
{
    QList<double> params = {double(colorize.red()), double(colorize.green()),
                           double(colorize.blue()), exposure, gloss, metallic, ambience,
                           double(texture), height};
    for (const LightSpec& light : lights) {
        params << double(int(light.kind)) << (light.on ? 1.0 : 0.0) << double(light.color.red())
              << double(light.color.green()) << double(light.color.blue()) << light.intensity
              << light.hotspot << light.center.x() << light.center.y() << light.angle
              << light.size << light.width << light.elevation;
    }
    return params;
}

bool LightingRig::fromSlots(const QList<double>& params, LightingRig* out)
{
    const qsizetype lightSlots = params.size() - kLightingRigSlots;
    const qsizetype count = lightSlots / kLightingLightSlots;
    if (lightSlots <= 0 || lightSlots % kLightingLightSlots != 0 || count > kMaxLights) {
        return false;
    }
    auto channel = [](double v) { return std::clamp(qRound(v), 0, 255); };
    LightingRig rig;
    rig.colorize = QColor(channel(params[0]), channel(params[1]), channel(params[2]));
    rig.exposure = params[3];
    rig.gloss = params[4];
    rig.metallic = params[5];
    rig.ambience = params[6];
    rig.texture = std::clamp(qRound(params[7]), 0, 3);
    rig.height = params[8];
    for (qsizetype i = 0; i < count; ++i) {
        const double* s = params.constData() + kLightingRigSlots + i * kLightingLightSlots;
        LightSpec light;
        light.kind = LightKind(std::clamp(qRound(s[0]), 0, 2));
        light.on = s[1] >= 0.5;
        light.color = QColor(channel(s[2]), channel(s[3]), channel(s[4]));
        light.intensity = s[5];
        light.hotspot = s[6];
        light.center = QPointF(s[7], s[8]);
        light.angle = s[9];
        light.size = s[10];
        light.width = s[11];
        light.elevation = s[12];
        rig.lights.append(light);
    }
    *out = rig;
    return true;
}

LightSpec defaultLight(LightKind kind)
{
    LightSpec light;
    light.kind = kind;
    switch (kind) {
    case LightKind::Spot:
        break;
    case LightKind::Point:
        light.size = 0.35;
        break;
    case LightKind::Infinite:
        // Up and to the right, well above the picture.
        light.angle = 315.0;
        light.elevation = 55.0;
        break;
    }
    return light;
}

QString lightKindName(LightKind kind)
{
    switch (kind) {
    case LightKind::Point:
        return QStringLiteral("Point");
    case LightKind::Infinite:
        return QStringLiteral("Infinite");
    case LightKind::Spot:
        break;
    }
    return QStringLiteral("Spot");
}

QList<QString> lightNames(const LightingRig& rig)
{
    int counts[3] = {0, 0, 0};
    QList<QString> names;
    for (const LightSpec& light : rig.lights) {
        const int n = ++counts[int(light.kind)];
        names.append(QStringLiteral("%1 Light %2").arg(lightKindName(light.kind)).arg(n));
    }
    return names;
}

LightingRig defaultLightingRig()
{
    return rigOf({LightSpec()});
}

const QList<LightingPreset>& lightingPresets()
{
    // Colours, intensities, and focus (hotspot) are the values CS6's Help gives
    // for each style.
    static const QList<LightingPreset> presets = [] {
        QList<LightingPreset> list;
        list.append({QStringLiteral("2 o'clock Spotlight"),
                     rigOf({spot(kYellow, 17, 91, {0.55, 0.45}, 325, 0.7, 0.4)})});
        list.append({QStringLiteral("Blue Omni"), rigOf({point(kBlue, 85, {0.5, 0.5}, 0.7)})});
        list.append({QStringLiteral("Circle Of Light"),
                     rigOf({spot(kWhite, 100, 8, {0.5, 0.3}, 0, 0.32, 0.16),
                            spot(kYellow, 88, 3, {0.7, 0.5}, 90, 0.32, 0.16),
                            spot(kRed, 50, 0, {0.5, 0.7}, 180, 0.32, 0.16),
                            spot(kBlue, 100, 25, {0.3, 0.5}, 270, 0.32, 0.16)})});
        list.append({QStringLiteral("Crossing"),
                     rigOf({spot(kWhite, 35, 69, {0.5, 0.5}, 20, 0.7, 0.3)})});
        list.append({QStringLiteral("Crossing Down"),
                     rigOf({spot(kWhite, 35, 100, {0.35, 0.5}, 60, 0.55, 0.25),
                            spot(kWhite, 35, 100, {0.65, 0.5}, 120, 0.55, 0.25)})});
        list.append({QStringLiteral("Default"), defaultLightingRig()});
        for (const auto& [name, y, aim] :
             {std::tuple{QStringLiteral("Five Lights Down"), 0.4, 90.0},
              std::tuple{QStringLiteral("Five Lights Up"), 0.6, 270.0}}) {
            QList<LightSpec> lights;
            for (double x : {0.1, 0.3, 0.5, 0.7, 0.9}) {
                lights.append(spot(kWhite, 100, 60, {x, y}, aim, 0.35, 0.12));
            }
            list.append({name, rigOf(lights)});
        }
        list.append({QStringLiteral("Flashlight"), rigOf({point(kYellow, 46, {0.5, 0.5}, 0.3)})});
        list.append({QStringLiteral("Flood Light"),
                     rigOf({spot(kWhite, 35, 69, {0.5, 0.5}, 325, 0.95, 0.65)})});
        list.append({QStringLiteral("Parallel Directional"),
                     rigOf({infinite(kBlue, 98, 180, 45)})});
        list.append({QStringLiteral("RGB Lights"),
                     rigOf({spot(kRed, 60, 96, {0.4, 0.45}, 210, 0.45, 0.3),
                            spot(kBlue, 60, 96, {0.6, 0.45}, 330, 0.45, 0.3),
                            spot(kGreen, 60, 96, {0.5, 0.6}, 90, 0.45, 0.3)})});
        list.append({QStringLiteral("Soft Direct Lights"),
                     rigOf({infinite(kWhite, 20, 225, 45), infinite(kBlue, 67, 315, 45)})});
        list.append({QStringLiteral("Soft Omni"), rigOf({point(kWhite, 50, {0.5, 0.5}, 0.8)})});
        list.append({QStringLiteral("Soft Spotlight"),
                     rigOf({spot(kWhite, 98, 100, {0.5, 0.5}, 325, 0.8, 0.5)})});
        list.append({QStringLiteral("Three Down"),
                     rigOf({spot(kWhite, 35, 96, {0.25, 0.4}, 90, 0.45, 0.2),
                            spot(kWhite, 35, 96, {0.5, 0.4}, 90, 0.45, 0.2),
                            spot(kWhite, 35, 96, {0.75, 0.4}, 90, 0.45, 0.2)})});
        list.append({QStringLiteral("Triple Spotlight"),
                     rigOf({spot(kWhite, 35, 100, {0.4, 0.45}, 210, 0.45, 0.3),
                            spot(kWhite, 35, 100, {0.6, 0.45}, 330, 0.45, 0.3),
                            spot(kWhite, 35, 100, {0.5, 0.6}, 90, 0.45, 0.3)})});
        return list;
    }();
    return presets;
}

HotspotShape spotHotspot(double hotspot)
{
    const double h = (std::clamp(hotspot, -100.0, 100.0) + 100.0) / 200.0;
    const double major = 0.6 * h * h;
    return {major, 0.7 * h, 0.9 * (1.0 - major)};
}

} // namespace pictura
