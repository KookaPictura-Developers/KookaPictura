#pragma once

namespace pictura {

class PicturaMainWindow;

// Large-document interactive checks: a selection-only edit must not run the
// compositor, and painting on a large document must present from the view
// pyramid. Returns 0 when all pass, otherwise the failure code.
int runLargeDocumentChecks(PicturaMainWindow& frame);

} // namespace pictura
