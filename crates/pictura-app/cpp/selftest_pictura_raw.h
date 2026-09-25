#pragma once

namespace pictura {
class PicturaMainWindow;

// pictura_raw (528): applying Pictura Raw to a plain raster pixel layer
// converts it to an embedded smart object, changes its pixels, and records
// exactly one "Pictura Raw" history state with the settings attached and
// readable; a second apply re-filters in exactly one more state; a group
// refuses without recording.
int runPicturaRawChecks(PicturaMainWindow& frame);
} // namespace pictura
