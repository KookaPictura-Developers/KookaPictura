#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the Layers-panel control checks: lpc_percent (199) covers the percent
// Opacity field, its five lock toggles, and history-free sync; lpc_nesting (200)
// covers the nesting lock bit, Group Layers refusal, and in-container reorder;
// lpc_preview (214) covers preview-without-history and one commit per edit;
// lpr_percent (215), the inside-box `%`; lpc_lockbadge (216), the right-side lock
// badge; lpr_eye (217), the row's lack of a check state; lpr_slider (218), the
// groove-jump/track opacity drag. lpr_merge_down (219) merges two pixel layers
// and checks lower-layer inheritance; lpr_merge_visible (220) leaves a hidden
// layer in place; lpr_flatten (221) yields one opaque white Background and
// discards hidden layers; lpr_merge_clip (222) refuses a non-clipping base.
// lpr_new_layer_dialog (223) creates a configured Multiply-neutral layer;
// lpr_new_group_dialog (224) creates a group that is never clipped;
// lpr_new_neutral_option (225) disables the neutral fill for Normal, hides
// clipping for a group, and creates a transparent layer for a missing neutral.
// lpr_background_roundtrip (226) saves and re-reads the Background flag;
// lpr_background_independence (227) shows the flag ignores name and position;
// lpr_background_convert (228) converts both directions, filling transparency
// and moving to the bottom, then unlocking and clearing the flag.
// lpr_via_copy (229) extracts the selection into a new layer above the source
// and leaves the source intact; lpr_via_cut (230) also clears the selected
// pixels from the source; lpr_via_refuse (231) records no history without a
// selection.
// lpr_select_similar (232) selects every layer of the active kind/attributes;
// lpr_link_linked_unlink (233) joins two layers into one set, selects its
// members, then drops one by unlinking; lpr_delete_hidden (234) removes only
// the hidden layers and records nothing when none are hidden; lpr_hide_layers
// (235) hides the selection in one undo state; lpr_solid_fill (236) creates and
// composites a solid fill layer; lpr_rasterize_fill (237) bakes a fill into
// pixels, clears its data, and refuses a second run; lpr_rasterize_refuse (238)
// refuses a plain layer with no history and keeps the kind-less Rasterize
// variants disabled. lpr_drop_out (239) reparents a nested layer to the root
// when dropped on the empty viewport; lpr_drop_rules (240) refuses a drop into
// a non-group, onto a descendant, or onto self, all without history.
// lpr_group_from_layers (241) groups two pixel layers under one group carrying
// the dialog's name/color/blend/opacity in one undo step.
// lpr_smart_object_convert (277) converts a raster pixel layer into an embedded
// smart object that keeps its raster proxy, survives save→load, and refuses a
// group or the Background without history. lpr_smart_object_rasterize (278)
// consumes that object back into a plain pixel layer with the same composite,
// drops it across save→load, and refuses a non-smart layer without history.
// Returns 0 when all pass, otherwise the self-test failure code.
int runLayersControlsChecks(PicturaMainWindow& frame);
} // namespace pictura
