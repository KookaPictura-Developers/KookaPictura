#pragma once

namespace pictura {

class PicturaMainWindow;

// Batch 5 paint-latency checks: a dab's region refresh must cover that dab only,
// not the accumulated stroke, and the scaled present cache must be patched
// rather than rebuilt. Returns 0 when all pass, otherwise the failure code.
// Invoked from runLayersControlsChecks so the self-test token order is unchanged.
int runPaintPerfChecks(PicturaMainWindow& frame);

} // namespace pictura
