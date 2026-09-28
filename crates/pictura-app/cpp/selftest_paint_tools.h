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
} // namespace pictura
