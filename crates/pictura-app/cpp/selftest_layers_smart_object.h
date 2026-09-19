#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the Layers-panel smart-object checks. Several entry points rather than
// one because the checks are interleaved with unrelated Layers-panel checks
// inside runLayersControlsChecks, and the self-test token order must not change
// when the blocks move out of that translation unit. Each returns 0 when its
// check passes, otherwise the self-test failure code.
//
// lpr_smart_object_convert (277) converts a raster pixel layer into an embedded
// smart object that keeps its raster proxy, survives save→load, and refuses a
// group or the Background without history. lpr_smart_object_rasterize (278)
// consumes that object back into a plain pixel layer with the same composite,
// drops it across save→load, and refuses a non-smart layer without history.
// lpr_place_smart_object (279) places a written PSD as a channel-less embedded
// layer and refuses a malformed file. lpr_smart_object_replace (280) replaces
// an embedded smart object's source in one labelled history state and refuses a
// malformed file without a state. lpr_open_as_smart_object (281) opens a
// written PSD as an untitled document holding exactly one embedded smart-object
// layer and refuses a malformed file without adding a tab.
// lpr_export_smart_object_contents (282) writes an embedded smart object's
// source to a file byte-for-byte without adding history and refuses a non-smart
// layer. lpr_edit_smart_object_contents (288) opens an editable source as a new
// untitled editor tab, commits the edited source in one "Edit Contents" state on
// Save, leaves a discarded editor's origin unchanged, and refuses a non-smart
// layer without adding a tab. lpr_edit_smart_object_session (289) closes an
// editor and checks its session temp file is gone, then closes the origin and
// checks the orphaned editor stays open as an untitled tab with its session
// temp dropped.
int runLayersSmartObjectConvertChecks(PicturaMainWindow& frame);
int runLayersSmartObjectRasterizeChecks(PicturaMainWindow& frame);
int runLayersPlaceSmartObjectChecks(PicturaMainWindow& frame);
int runLayersSmartObjectReplaceChecks(PicturaMainWindow& frame);
int runLayersOpenSmartObjectChecks(PicturaMainWindow& frame);
int runLayersExportSmartObjectChecks(PicturaMainWindow& frame);
int runLayersEditSmartObjectChecks(PicturaMainWindow& frame);
int runLayersEditSmartObjectSessionChecks(PicturaMainWindow& frame);
} // namespace pictura
