#include "selftest_ports.h"

#include "selftest_annotations.h"
#include "selftest_click_deselect.h"
#include "selftest_clipboard.h"
#include "selftest_crop_group.h"
#include "selftest_crop_tool.h"
#include "selftest_erasers.h"
#include "selftest_healing.h"
#include "selftest_magnetic_lasso.h"
#include "selftest_paint_tools.h"
#include "selftest_recent_files.h"

#include <initializer_list>

int pictura::runPortChecks(pictura::PicturaMainWindow& frame)
{
    for (const auto run : {runCropToolChecks, runRecentFilesChecks, runCropGroupChecks,
                           runClickDeselectChecks, runMagneticLassoChecks, runClipboardChecks,
                           runAnnotationChecks, runHealingChecks, runPatchChecks,
                           runContentAwareMoveChecks, runRedEyeChecks,
                           runColorReplacementChecks, runMixerBrushChecks, runCloneStampChecks,
                           runPatternStampChecks, runHistoryBrushChecks, runBrushPanelChecks,
                           runCloneSourcePanelChecks, runEraserChecks,
                           runArtHistoryBrushChecks, runBrushPresetPickerChecks,
                           runBackgroundEraserChecks, runMagicEraserChecks}) {
        if (const int code = run(frame); code != 0) {
            return code;
        }
    }
    return 0;
}
