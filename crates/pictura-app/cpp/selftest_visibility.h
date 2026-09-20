#pragma once

namespace pictura {

class PicturaMainWindow;

// Batch 3 layer-visibility checks: a visibility toggle must take the region
// fast path (and not a full recomposite), while an invisible active layer
// refuses paint but still allows Move and selection/copy. Returns 0 when all
// pass, otherwise the failure code. Invoked from runLayersControlsChecks so
// the self-test token order is unchanged.
int runVisibilityChecks(PicturaMainWindow& frame);

} // namespace pictura
