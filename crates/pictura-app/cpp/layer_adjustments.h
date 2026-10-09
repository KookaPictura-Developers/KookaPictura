#pragma once

namespace pictura {

// The sixteen CS6 Layer > New Adjustment Layer kinds, in menu order. `ellipsis`
// follows the Image > Adjustments split: Invert has no options, so it creates
// silently; every other kind is labelled with an ellipsis. Shared by the Layer
// menu and the Layers panel New Fill / Adjustment menu so the two creation
// surfaces cannot drift.
struct LayerAdjustment {
    const char* kind;
    const char* leaf;
    bool ellipsis;
};

inline constexpr LayerAdjustment kLayerAdjustments[] = {
    {"brightness-contrast", "Brightness/Contrast", true},
    {"levels", "Levels", true},
    {"curves", "Curves", true},
    {"exposure", "Exposure", true},
    {"vibrance", "Vibrance", true},
    {"hue-saturation", "Hue/Saturation", true},
    {"color-balance", "Color Balance", true},
    {"black-white", "Black & White", true},
    {"photo-filter", "Photo Filter", true},
    {"channel-mixer", "Channel Mixer", true},
    {"color-lookup", "Color Lookup", true},
    {"invert", "Invert", false},
    {"posterize", "Posterize", true},
    {"threshold", "Threshold", true},
    {"gradient-map", "Gradient Map", true},
    {"selective-color", "Selective Color", true},
};

} // namespace pictura
