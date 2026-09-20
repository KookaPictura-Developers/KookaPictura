#pragma once

namespace pictura {

class PicturaMainWindow;

// Batch 4 cursor/ring/Space-pan/hint-bar checks. Returns 0 when all pass,
// otherwise the self-test failure code. Invoked from runLayersControlsChecks so
// the self-test token order is unchanged (selftest.cpp does not grow).
int runToolCanvasChecks(PicturaMainWindow& frame);

} // namespace pictura
