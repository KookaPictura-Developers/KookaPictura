#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the selection-tool drag checks: tsc_ellipse (242) commits an ellipse
// through the Elliptical Marquee drag path and proves the corners are excluded;
// tsc_feather (243) applies a tool-time Feather and observes a partial-coverage
// edge; tsc_fixed_size (244) honours the Fixed Size style; tsc_polygon (245)
// commits a clicked triangle through the Polygonal Lasso press path;
// tsc_polygon_cancel (246) proves Escape discards an in-progress path without a
// selection or history change; tsc_polygon_feather (247) softens a polygon edge;
// tsc_wand_contiguous (248) selects only the connected uniform region;
// tsc_wand_global (249) includes a disconnected same-colour patch; tsc_wand_add
// (250) unions the wand result with an existing selection. The Select menu
// checks continue: tsc_select_inverse (251) complements coverage;
// tsc_select_reselect (252) restores a deselected selection; tsc_select_modify
// (253) applies each Modify op; tsc_select_grow (254) follows connectivity;
// tsc_select_similar (255) reaches disconnected patches; tsc_save_load (256)
// round-trips a channel exactly; tsc_select_refusal (257) proves refusals record
// nothing; tsc_layer_select_commands (258) selects/clears panel rows.
// tsc_selection_ants (259) mirrors a committed marquee contour onto the canvas;
// tsc_selection_ants_clear (260) proves Deselect clears it; and
// tsc_selection_edges_toggle (261) hides/shows the edges without losing data;
// tsc_marquee_preview (262) proves the live rubber band is shown during the drag
// and replaced by the committed contour on release; tsc_ellipse_preview (263)
// proves the elliptical rubber band is a multi-point ellipse, not a rectangle.
// Returns 0 when all pass, otherwise the self-test failure code.
int runToolsSelectionChecks(PicturaMainWindow& frame);
} // namespace pictura
