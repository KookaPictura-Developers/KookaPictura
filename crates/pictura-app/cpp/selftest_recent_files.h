#pragma once

namespace pictura {
class PicturaMainWindow;

// recent_files (533): with an empty list File > Open Recent shows only the
// disabled "No Recent Files" row; opening an image puts it at the front, and
// the submenu (rebuilt as it opens) offers it by name with the full path as the
// tooltip plus Clear Recent File List; choosing it reopens the image; a missing
// file is dropped; Clear empties the list. The user's list is restored after.
int runRecentFilesChecks(PicturaMainWindow& frame);
} // namespace pictura
