#pragma once

namespace pictura {
class PicturaMainWindow;

// edit_clipboard (529): through the Edit command handlers, Copy records no
// state, Cut/Clear/Paste Into/Paste each record exactly one labelled state, a
// cut region reads the layer below, Paste Into restores it masked to the
// selection and deselects, and Purge > Clipboard empties the clipboard and
// disables Paste.
int runClipboardChecks(PicturaMainWindow& frame);
} // namespace pictura
