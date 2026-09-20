#pragma once

namespace pictura {
class PicturaMainWindow;

// Batch 2 interaction checks:
//   lpr_drag_sibling        (328) a sibling gap resolves to an above/below drop
//                                 and reorders in one undo step
//   lpr_drag_invalid        (329) an invalid target shows no indicator and
//                                 changes nothing
//   lpr_background_dblclick (330) a double-click outside the name converts the
//                                 Background to a normal unlocked layer
//   lpr_background_drop     (331) dropping the Background on New Layer converts
//                                 it in place instead of cloning a copy
//   lpr_thumbnail_select    (332) Ctrl+clicking a thumbnail selects the layer's
//                                 opaque pixels and starts no drag or editor
//   lpr_drag_nesting        (333) a nesting-locked group refuses a reparent drop
// Returns 0 when all pass, otherwise the self-test failure code.
int runLayersInteractionsChecks(PicturaMainWindow& frame);
} // namespace pictura
