#pragma once

namespace pictura {
class PicturaMainWindow;

// red_eye_tool (539): the Red Eye bar carries Pupil Size / Darken Amount (50 /
// 50); a box dragged over a red pupil neutralises it in one "Red Eye Tool"
// state and leaves the skin alone; a click on skin records nothing.
int runRedEyeChecks(PicturaMainWindow& frame);

// color_replacement_tool (540): the bar's Mode / Sampling / Limits default to
// Color / Continuous / Contiguous; a stroke over a blue field repaints it with
// the foreground in one "Color Replacement Tool" state, keeps the far (yellow)
// field, and the size ring follows the brush.
int runColorReplacementChecks(PicturaMainWindow& frame);

// mixer_brush_tool (541): a preset sets Wet / Load / Mix and greys Load / Mix
// on a dry canvas; a wet stroke drags black into white in one "Mixer Brush
// Tool" state and carries the pickup on the brush; Clean After Stroke empties
// it; Alt-click loads the brush from the image.
int runMixerBrushChecks(PicturaMainWindow& frame);

// clone_stamp_tool (542): Sample defaults to Current Layer and shows Ignore
// Adjustment Layers only with All Layers; a stroke before Alt-click is refused
// without history; after it, a stroke copies the source in one "Clone Stamp"
// state, and an Aligned second stroke keeps the offset.
int runCloneStampChecks(PicturaMainWindow& frame);

// pattern_stamp_tool (543): the bar lists the built-in patterns with
// Impressionist disabled; an Aligned stroke paints the checkerboard pinned to
// the document in one "Pattern Stamp" state; unaligned pins it to the stroke.
int runPatternStampChecks(PicturaMainWindow& frame);

// history_brush_tool (544): after a Brush stroke, a History Brush stroke paints
// the oldest state back in one "History Brush" state; a press in the History
// panel's left column makes the Brush state the source, and painting brings
// the stroke back.
int runHistoryBrushChecks(PicturaMainWindow& frame);

// brush_panel (545): the Clone Stamp bar's Toggle the Brush panel shows and
// hides the Brush panel; its Roundness field reaches the controller and the
// engine preview, and a Brush stroke with a flattened tip is wider than tall.
int runBrushPanelChecks(PicturaMainWindow& frame);

// clone_source_panel (546): Toggle the Clone Source panel shows the panel; each
// slot keeps its own source; the Offset reads destination minus source after a
// stroke; Flip Horizontal mirrors the cloned pixels about the stroke start.
int runCloneSourcePanelChecks(PicturaMainWindow& frame);
} // namespace pictura
