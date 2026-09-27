#pragma once

namespace pictura {
class PicturaMainWindow;

// edit_clipboard (529): through the Edit command handlers, Copy records no
// state and exports the clip to the system clipboard; Cut, Paste in Place,
// Paste Into, Clear, and Paste each record exactly one labelled state; Paste in
// Place restores the cut pixels at their source position and Paste Into masks
// and deselects; another application's image pastes in place at the origin;
// and Purge > Clipboard empties the clipboard, removes our export, and
// disables Paste.
int runClipboardChecks(PicturaMainWindow& frame);
} // namespace pictura
