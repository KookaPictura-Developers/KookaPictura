#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the Layers-panel smart-object checks. Four entry points rather than one
// because the checks are interleaved with unrelated Layers-panel checks inside
// runLayersControlsChecks, and the self-test token order must not change when
// the blocks move out of that translation unit. Each returns 0 when its check
// passes, otherwise the self-test failure code.
//
// lpr_smart_object_convert (277) converts a raster pixel layer into an embedded
// smart object that keeps its raster proxy, survives save→load, and refuses a
// group or the Background without history. lpr_smart_object_rasterize (278)
// consumes that object back into a plain pixel layer with the same composite,
// drops it across save→load, and refuses a non-smart layer without history.
// lpr_place_smart_object (279) places a written PSD as a channel-less embedded
// layer and refuses a malformed file. lpr_smart_object_replace (280) replaces
// an embedded smart object's source in one labelled history state and refuses a
// malformed file without a state.
int runLayersSmartObjectConvertChecks(PicturaMainWindow& frame);
int runLayersSmartObjectRasterizeChecks(PicturaMainWindow& frame);
int runLayersPlaceSmartObjectChecks(PicturaMainWindow& frame);
int runLayersSmartObjectReplaceChecks(PicturaMainWindow& frame);
} // namespace pictura
