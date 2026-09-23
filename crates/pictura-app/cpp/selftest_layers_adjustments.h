#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the adjustment-layer checks: lpr_photo_filter (283) adds a photo-filter
// kind, confirms it is reported as an adjustment, and that it warms the
// composite; adjustments_photo_filter_menu (284) checks the Adjustments panel
// offers the Photo Filter row; lpr_gradient_map (285) maps a painted backdrop
// through a black-to-white gradient and checks the panel row; lpr_color_balance
// (293) adds the neutral `color-balance` kind and checks it leaves the
// composite unchanged; lpr_channel_mixer (294) adds the neutral `channel-mixer`
// kind and checks the same neutrality plus its panel row; lpr_color_lookup (455)
// adds the neutral `color-lookup` kind (an identity cube) and checks the same
// neutrality plus its panel row; color_mode_open (297)
// writes a minimal flat CMYK PSD and opens it, checking the view reports the
// CMYK conversion notice and reads as an RGB document; lab_mode_open (518)
// writes a minimal flat 8-bit Lab PSD and opens it, checking the view reports
// the Lab-preserving save notice; lab16_mode_open (519) writes a 16-bit Lab PSD
// and checks the view reports an RGB save notice instead; depth_open (298) writes
// a minimal flat depth-16 RGB PSD and opens it, checking the 16-bit conversion
// notice and the `v >> 8` narrowing. color_settings_policy (461) checks the
// incoming-profile policy default/round-trip and the Color Settings dialog
// mapping. Returns 0 when all pass, otherwise the self-test failure code.
int runLayersAdjustmentChecks(PicturaMainWindow& frame);
} // namespace pictura
