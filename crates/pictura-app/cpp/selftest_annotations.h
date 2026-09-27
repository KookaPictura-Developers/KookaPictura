#pragma once

namespace pictura {
class PicturaMainWindow;

// annotation_tools (535): the Color Sampler places numbered samplers (one
// "Color Sampler" state each) that the Info panel reads out and every tool
// shows, refuses a fifth, moves one by dragging ("Move Color Sampler"),
// deletes one by Alt-click or by dragging it off the canvas ("Delete Color
// Sampler"), and Clear removes them all; undo restores them. The Ruler
// measures a drag (W/H/A/D1 in the options bar, no history), edits an end,
// drops a click-without-drag, and shows only while active. The Note tool adds
// a note ("New Note") and opens it in the Notes panel, whose text commits one
// "Edit Note" state; clicking a note reopens it and Alt-click deletes it.
int runAnnotationChecks(PicturaMainWindow& frame);
} // namespace pictura
