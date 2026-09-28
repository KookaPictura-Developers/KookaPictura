#pragma once

namespace pictura {
class PicturaMainWindow;

// Batch 5 persistence and import-identity checks:
//   lpr_quit_save (317)      emitting the app's quit signal writes the session
//   lpr_width_restart (318)  every column width is saved and reapplied after a
//                            simulated restart
//   lpr_v7_width (319)       a v6 store loads a default width, seeded from the
//                            legacy `railWidth` for the primary column
//   ldt_mode_bits (320)      an opened raster's tab reads `base (RGB/8)`
//   ldt_untitled (321)       a nameless document still reads `Untitled-N`
//   lim_opaque_background (322) an opaque import is a locked Background
//   lim_transparent_layer (323) a non-opaque import stays a regular alpha layer
//   lpr_raster_export (532)  Save writes the path's format; Export As/Quick
//                            Export flatten without mutating the document
//   lpr_dialog_last_dir (534) an empty-directory dialog reuses Qt's persisted
//                            last-visited folder (stored as a file:// URL)
// Returns 0 when all pass, otherwise the self-test failure code.
int runSessionChecks(PicturaMainWindow& frame);
} // namespace pictura
