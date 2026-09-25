#include "selftest_pictura_raw.h"
#include "selftest_report.h"

#include "frame.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QStringList>

int pictura::runPicturaRawChecks(pictura::PicturaMainWindow& frame)
{
    // pictura_raw (528): applying Pictura Raw to a plain raster pixel layer
    // converts it to an embedded smart object, changes its pixels, and records
    // exactly one "Pictura Raw" history state with the settings attached and
    // readable; a second apply re-filters in exactly one more state; a group
    // refuses without recording.
    const bool rawCreated = frame.newDocument(QStringLiteral("PicturaRawCtl"), 8, 8,
                                              QStringLiteral("rgb"), 8,
                                              QStringLiteral("white"));
    pictura::PictureView* rawView = frame.activeView();
    if (!rawCreated || !rawView) {
        return pictura::selfTest().fail(528, "pictura raw fixture");
    }
    const int rawDoc = frame.activeDocumentIndex();
    const QString rawPath = rawView->add_solid_fill(0xff808080u);
    const bool rawRaster = rawView->rasterize_fill_content(rawPath);
    const bool rawCan = rawView->layer_can_convert_to_smart_object(rawPath);
    const int rawBase = rawView->history_count();
    const unsigned int rawBefore = rawView->sample_argb(0, 0);
    const bool rawApplied = rawView->apply_pictura_raw_filter(
        rawPath, 20.0, 0.0, 1.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    const QString rawState = rawView->layer_smart_object_state(rawPath);
    const bool rawSmart = rawState.startsWith(QStringLiteral("embedded:"))
        && rawState.mid(9).toInt() > 0;
    const bool rawChanged = rawView->sample_argb(0, 0) != rawBefore;
    const bool rawOneState = rawView->history_count() == rawBase + 1
        && rawView->history_label(rawBase) == QStringLiteral("Pictura Raw");
    const QStringList rawStored =
        rawView->layer_pictura_raw_settings(rawPath).split(QLatin1Char(' '), Qt::SkipEmptyParts);
    const bool rawSettings = rawStored.size() == 11
        && rawStored.at(0).toDouble() == 20.0
        && rawStored.at(2).toDouble() == 1.5;

    const int rawBase2 = rawView->history_count();
    const bool rawReapplied = rawView->apply_pictura_raw_filter(
        rawPath, -30.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    const QStringList rawStored2 =
        rawView->layer_pictura_raw_settings(rawPath).split(QLatin1Char(' '), Qt::SkipEmptyParts);
    const bool rawOneMore = rawReapplied && rawView->history_count() == rawBase2 + 1
        && rawView->history_label(rawBase2) == QStringLiteral("Pictura Raw")
        && rawStored2.size() == 11 && rawStored2.at(0).toDouble() == -30.0;

    const QString rawGroup = rawView->add_group_in(QString());
    const int rawGroupBase = rawView->history_count();
    const bool rawRefused = !rawView->apply_pictura_raw_filter(
        rawGroup, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    const bool rawRefuseHistory = rawView->history_count() == rawGroupBase;

    const bool rawOk = rawRaster && rawCan && rawApplied && rawSmart && rawChanged
        && rawOneState && rawSettings && rawOneMore && rawRefused && rawRefuseHistory;
    ST_BEGIN("pictura_raw");
    ST_PASS("pictura_raw raster=%d can=%d apply=%d state=%s pixel=%08x before=%08x "
            "history=%d settings=%d reapply=%d refuse=%d",
            rawRaster ? 1 : 0, rawCan ? 1 : 0, rawApplied ? 1 : 0, qPrintable(rawState),
            rawView->sample_argb(0, 0), rawBefore, rawView->history_count() - rawBase,
            rawSettings ? 1 : 0, rawOneMore ? 1 : 0, rawRefused ? 1 : 0);
    if (!rawOk) {
        return pictura::selfTest().fail(528, "pictura raw filter");
    }
    frame.closeDocument(rawDoc, false);
    return 0;
}
