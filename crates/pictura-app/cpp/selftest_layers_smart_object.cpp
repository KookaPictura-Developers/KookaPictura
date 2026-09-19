#include "selftest_layers_smart_object.h"
#include "selftest_report.h"

#include "frame.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QDir>
#include <QtCore/QFile>

int pictura::runLayersSmartObjectConvertChecks(pictura::PicturaMainWindow& frame)
{
    // lpr_smart_object_convert (277): converting a raster pixel layer
    // authors an embedded smart object while keeping its raster proxy, so
    // the composite is unchanged and the object survives save→load; a group
    // and the Background refuse without history.
    const bool socCreated = frame.newDocument(QStringLiteral("SmartObjectCtl"), 4, 4,
                                              QStringLiteral("rgb"), 8,
                                              QStringLiteral("white"));
    pictura::PictureView* socView = frame.activeView();
    if (!socCreated || !socView) {
        return pictura::selfTest().fail(277, "smart object fixture");
    }
    const int socDoc = frame.activeDocumentIndex();
    const QString socPath = socView->add_solid_fill(0xff2244aau);
    const bool socRaster = socView->rasterize_fill_content(socPath);
    const bool socCan = socView->layer_can_convert_to_smart_object(socPath);
    const int socBase = socView->history_count();
    const unsigned int socBefore = socView->sample_argb(0, 0);
    const bool socConverted = socView->convert_to_smart_object(socPath);
    const QString socState = socView->layer_smart_object_state(socPath);
    const bool socProxy = socView->sample_argb(0, 0) == socBefore;
    const bool socRecorded = socView->history_count() == socBase + 1;
    const bool socEmbedded = socState.startsWith(QStringLiteral("embedded:"))
        && socState.mid(9).toInt() > 0
        && !socView->layer_can_convert_to_smart_object(socPath);
    const QString socGroup = socView->add_group_in(QString());
    socView->background_from_layer(QStringLiteral("0"));
    const int socRefBase = socView->history_count();
    const bool socRefGroup = !socView->convert_to_smart_object(socGroup);
    const bool socRefBackground = !socView->convert_to_smart_object(QStringLiteral("0"));
    const bool socRefHistory = socView->history_count() == socRefBase;
    const QString socSavePath =
        QDir::tempPath() + QStringLiteral("/kooka-pictura-smart-object.psd");
    const bool socSaved = frame.saveActiveAs(socSavePath);
    const bool socReopened = frame.openPath(socSavePath);
    pictura::PictureView* socReload = frame.activeView();
    const int socReloadDoc = frame.activeDocumentIndex();
    const QString socReloadState =
        socReload ? socReload->layer_smart_object_state(socPath) : QString();
    const bool socRoundTrip = socReopened && socReload
        && socReloadState.startsWith(QStringLiteral("embedded:"))
        && socReloadState.mid(9).toInt() > 0;
    const bool socOk = socRaster && socCan && socConverted && socProxy && socRecorded
        && socEmbedded && socRefGroup && socRefBackground && socRefHistory && socSaved
        && socRoundTrip;
    ST_BEGIN("lpr_smart_object_convert");
    ST_PASS("lpr_smart_object_convert raster=%d can=%d convert=%d state=%s proxy=%d "
            "history=%d group=%d bg=%d saved=%d reload=%s",
            socRaster ? 1 : 0, socCan ? 1 : 0, socConverted ? 1 : 0, qPrintable(socState),
            socProxy ? 1 : 0, socRecorded ? 1 : 0, socRefGroup ? 1 : 0,
            socRefBackground ? 1 : 0, socSaved ? 1 : 0, qPrintable(socReloadState));
    if (!socOk) {
        return pictura::selfTest().fail(277, "smart object convert");
    }
    frame.closeDocument(socReloadDoc, false);
    frame.closeDocument(socDoc, false);
    return 0;
}

int pictura::runLayersSmartObjectRasterizeChecks(pictura::PicturaMainWindow& frame)
{
    // lpr_smart_object_rasterize (278): converting then rasterizing a raster
    // pixel layer restores a plain pixel layer with the same composite, no
    // smart object on save→load, and refuses a non-smart layer without a
    // history state.
    const bool rosCreated = frame.newDocument(QStringLiteral("SmartObjectRasterCtl"), 4, 4,
                                              QStringLiteral("rgb"), 8,
                                              QStringLiteral("white"));
    pictura::PictureView* rosView = frame.activeView();
    if (!rosCreated || !rosView) {
        return pictura::selfTest().fail(278, "smart object rasterize fixture");
    }
    const int rosDoc = frame.activeDocumentIndex();
    const QString rosPath = rosView->add_solid_fill(0xff2244aau);
    const bool rosRaster = rosView->rasterize_fill_content(rosPath);
    const unsigned int rosBefore = rosView->sample_argb(0, 0);
    const bool rosConverted = rosView->convert_to_smart_object(rosPath);
    const unsigned int rosAfterConvert = rosView->sample_argb(0, 0);
    const bool rosCanRaster = rosView->layer_can_rasterize_smart_object(rosPath);
    const int rosBase = rosView->history_count();
    const bool rosRasterized = rosView->rasterize_smart_object(rosPath);
    const QString rosState = rosView->layer_smart_object_state(rosPath);
    const unsigned int rosAfterRaster = rosView->sample_argb(0, 0);
    const bool rosRecorded = rosView->history_count() == rosBase + 1;
    const bool rosPixel = rosView->layer_kind(rosPath.toInt()) == QStringLiteral("pixel");
    const int rosBase2 = rosView->history_count();
    const bool rosRefuse = !rosView->rasterize_smart_object(rosPath);
    const bool rosRefuseHistory = rosView->history_count() == rosBase2;
    const QString rosSavePath =
        QDir::tempPath() + QStringLiteral("/kooka-pictura-smart-object-rasterize.psd");
    const bool rosSaved = frame.saveActiveAs(rosSavePath);
    const bool rosReopened = frame.openPath(rosSavePath);
    pictura::PictureView* rosReload = frame.activeView();
    const int rosReloadDoc = frame.activeDocumentIndex();
    const QString rosReloadState =
        rosReload ? rosReload->layer_smart_object_state(rosPath) : QString();
    const bool rosRoundTrip = rosReopened && rosReload && rosReloadState.isEmpty();
    const bool rosOk = rosRaster && rosConverted && rosCanRaster && rosRasterized
        && rosBefore == rosAfterConvert && rosBefore == rosAfterRaster && rosState.isEmpty()
        && rosRecorded && rosPixel && rosRefuse && rosRefuseHistory && rosSaved && rosRoundTrip;
    ST_BEGIN("lpr_smart_object_rasterize");
    ST_PASS("lpr_smart_object_rasterize raster=%d convert=%d can=%d rasterize=%d proxy=%d "
            "state=%s history=%d pixel=%d refuse=%d saved=%d reload=%s",
            rosRaster ? 1 : 0, rosConverted ? 1 : 0, rosCanRaster ? 1 : 0,
            rosRasterized ? 1 : 0, rosBefore == rosAfterRaster ? 1 : 0, qPrintable(rosState),
            rosRecorded ? 1 : 0, rosPixel ? 1 : 0, rosRefuse ? 1 : 0, rosSaved ? 1 : 0,
            qPrintable(rosReloadState));
    if (!rosOk) {
        return pictura::selfTest().fail(278, "smart object rasterize");
    }
    frame.closeDocument(rosReloadDoc, false);
    frame.closeDocument(rosDoc, false);
    return 0;
}

int pictura::runLayersPlaceSmartObjectChecks(pictura::PicturaMainWindow& frame)
{
    // lpr_place_smart_object (279): placing a PSD file appends a channel-less
    // embedded smart-object layer that renders the source, in one undo state;
    // a malformed file refuses without changing the document.
    const bool plSrcCreated = frame.newDocument(QStringLiteral("PlaceSource"), 4, 4,
                                                QStringLiteral("rgb"), 8,
                                                QStringLiteral("white"));
    pictura::PictureView* plSrcView = frame.activeView();
    if (!plSrcCreated || !plSrcView) {
        return pictura::selfTest().fail(279, "place source fixture");
    }
    const int plSrcDoc = frame.activeDocumentIndex();
    const QString plSourcePath =
        QDir::tempPath() + QStringLiteral("/kooka-pictura-place-source.psd");
    const bool plSourceSaved = frame.saveActiveAs(plSourcePath);
    frame.closeDocument(plSrcDoc, false);

    const bool plCreated = frame.newDocument(QStringLiteral("PlaceCtl"), 4, 4,
                                             QStringLiteral("rgb"), 8,
                                             QStringLiteral("transparent"));
    pictura::PictureView* plView = frame.activeView();
    if (!plCreated || !plView) {
        return pictura::selfTest().fail(279, "place fixture");
    }
    const int plDoc = frame.activeDocumentIndex();
    const int plBase = plView->history_count();
    const int plRows = plView->layer_row_count();
    const unsigned int plBefore = plView->sample_argb(0, 0);
    const QString plPath = plView->place_smart_object(plSourcePath);
    const bool plPlaced = !plPath.isEmpty() && plView->layer_row_count() == plRows + 1
        && plView->history_count() == plBase + 1
        && plView->sample_argb(0, 0) == 0xffffffffu
        && plView->sample_argb(0, 0) != plBefore
        && plView->layer_smart_object_state(plPath).startsWith(QStringLiteral("embedded:"));
    const QString plBadPath =
        QDir::tempPath() + QStringLiteral("/kooka-pictura-place-bad.psd");
    QFile plBad(plBadPath);
    if (plBad.open(QIODevice::WriteOnly)) {
        plBad.write("not a psd");
    }
    plBad.close();
    const int plBadBase = plView->history_count();
    const QString plRefused = plView->place_smart_object(plBadPath);
    const bool plRefusedOk = plRefused.isEmpty()
        && plView->history_count() == plBadBase
        && plView->layer_row_count() == plRows + 1;
    ST_BEGIN("lpr_place_smart_object");
    ST_PASS("lpr_place_smart_object saved=%d path=%s pixel=%08x before=%08x refuse=%d",
            plSourceSaved ? 1 : 0, qPrintable(plPath), plView->sample_argb(0, 0), plBefore,
            plRefusedOk ? 1 : 0);
    if (!plSourceSaved || !plPlaced || !plRefusedOk) {
        return pictura::selfTest().fail(279, "place smart object");
    }
    QFile::remove(plSourcePath);
    QFile::remove(plBadPath);
    frame.closeDocument(plDoc, false);
    return 0;
}

int pictura::runLayersSmartObjectReplaceChecks(pictura::PicturaMainWindow& frame)
{
    // lpr_smart_object_replace (280): replacing a smart object's embedded
    // contents swaps the composite to the new source in one "Replace Contents"
    // history state; a malformed file refuses without adding a state.
    frame.newDocument(QStringLiteral("ReplaceSource"), 2, 2, QStringLiteral("rgb"), 8, QStringLiteral("white"));
    pictura::PictureView* rpSrcView = frame.activeView();
    if (!rpSrcView) { return pictura::selfTest().fail(280, "replace source fixture"); }
    const QString rpSource = QDir::tempPath() + QStringLiteral("/kooka-pictura-replace-source.psd");
    rpSrcView->rasterize_fill_content(rpSrcView->add_solid_fill(0xff22cc44u));
    const bool rpSaved = frame.saveActiveAs(rpSource);
    frame.closeDocument(frame.activeDocumentIndex(), false);
    frame.newDocument(QStringLiteral("SmartObjectReplaceCtl"), 4, 4, QStringLiteral("rgb"), 8, QStringLiteral("white"));
    pictura::PictureView* rpView = frame.activeView();
    if (!rpView) { return pictura::selfTest().fail(280, "replace fixture"); }
    const int rpDoc = frame.activeDocumentIndex();
    const QString rpPath = rpView->add_solid_fill(0xff2244aau);
    rpView->rasterize_fill_content(rpPath);
    rpView->convert_to_smart_object(rpPath);
    const bool rpCan = rpView->layer_can_replace_smart_object_contents(rpPath);
    const unsigned int rpBefore = rpView->sample_argb(0, 0);
    const int rpBase = rpView->history_count();
    const bool rpReplaced = rpView->replace_smart_object_contents(rpPath, rpSource);
    const bool rpChanged = rpView->sample_argb(0, 0) == 0xff22cc44u && rpView->sample_argb(0, 0) != rpBefore;
    const QString rpState = rpView->layer_smart_object_state(rpPath);
    const bool rpSmart = rpState.startsWith(QStringLiteral("embedded:")) && rpState.mid(9).toInt() > 0;
    const bool rpLabelOk = rpView->history_label(rpBase) == QStringLiteral("Replace Contents");
    const QString rpBad = QDir::tempPath() + QStringLiteral("/kooka-pictura-replace-bad.psd");
    QFile rpBadFile(rpBad); if (rpBadFile.open(QIODevice::WriteOnly)) { rpBadFile.write("not a psd"); } rpBadFile.close();
    const int rpBadBase = rpView->history_count();
    const bool rpRefused = !rpView->replace_smart_object_contents(rpPath, rpBad) && rpView->history_count() == rpBadBase;
    const bool rpOk = rpSaved && rpCan && rpReplaced && rpChanged && rpView->history_count() == rpBase + 1 && rpSmart && rpLabelOk && rpRefused;
    ST_BEGIN("lpr_smart_object_replace");
    ST_PASS("lpr_smart_object_replace can=%d replace=%d pixel=%08x before=%08x state=%s history=%d smart=%d refuse=%d",
            rpCan ? 1 : 0, rpReplaced ? 1 : 0, rpView->sample_argb(0, 0), rpBefore, qPrintable(rpState),
            rpView->history_count() - rpBase, rpSmart ? 1 : 0, rpRefused ? 1 : 0);
    if (!rpOk) { return pictura::selfTest().fail(280, "replace smart object"); }
    QFile::remove(rpSource);
    QFile::remove(rpBad);
    frame.closeDocument(rpDoc, false);
    return 0;
}

int pictura::runLayersOpenSmartObjectChecks(pictura::PicturaMainWindow& frame)
{
    // lpr_open_as_smart_object (281): opening a written PSD as a smart object
    // adds one untitled tab holding exactly one embedded smart-object layer
    // whose composite is the source colour, in one "Open As Smart Object"
    // history state; a malformed file refuses and adds no tab.
    const bool ossSourceCreated = frame.newDocument(QStringLiteral("OpenSmartSource"), 4, 4,
                                                    QStringLiteral("rgb"), 8,
                                                    QStringLiteral("white"));
    pictura::PictureView* ossSrcView = frame.activeView();
    if (!ossSourceCreated || !ossSrcView) {
        return pictura::selfTest().fail(281, "open smart object source fixture");
    }
    ossSrcView->rasterize_fill_content(ossSrcView->add_solid_fill(0xff2244aau));
    const QString ossSourcePath =
        QDir::tempPath() + QStringLiteral("/kooka-pictura-open-smart-source.psd");
    const bool ossSourceSaved = frame.saveActiveAs(ossSourcePath);
    frame.closeDocument(frame.activeDocumentIndex(), false);

    const int ossBase = frame.documentCount();
    const bool ossOpened = frame.openAsSmartObjectPath(ossSourcePath);
    pictura::PictureView* ossView = frame.activeView();
    const bool ossAdded = ossOpened && frame.documentCount() == ossBase + 1 && ossView;
    const bool ossUntitled = ossView && ossView->file_path().isEmpty();
    const bool ossOneLayer = ossView && ossView->layer_row_count() == 1;
    const QString ossState =
        ossView ? ossView->layer_smart_object_state(QStringLiteral("0")) : QString();
    const bool ossSmart = ossState.startsWith(QStringLiteral("embedded:"))
        && ossState.mid(9).toInt() > 0;
    const bool ossPixel = ossView && ossView->sample_argb(0, 0) == 0xff2244aau;
    const bool ossHistory = ossView && ossView->history_count() == 1
        && ossView->history_label(0) == QStringLiteral("Open As Smart Object");

    const QString ossBadPath =
        QDir::tempPath() + QStringLiteral("/kooka-pictura-open-smart-bad.psd");
    QFile ossBad(ossBadPath);
    if (ossBad.open(QIODevice::WriteOnly)) {
        ossBad.write("not a psd");
    }
    ossBad.close();
    const int ossBadBase = frame.documentCount();
    const bool ossRefused = !frame.openAsSmartObjectPath(ossBadPath);
    const bool ossNoTab = frame.documentCount() == ossBadBase;

    const bool ossOk = ossSourceSaved && ossAdded && ossUntitled && ossOneLayer && ossSmart
        && ossPixel && ossHistory && ossRefused && ossNoTab;
    ST_BEGIN("lpr_open_as_smart_object");
    ST_PASS("lpr_open_as_smart_object opened=%d added=%d untitled=%d layers=%d state=%s "
            "pixel=%08x history=%d refused=%d notab=%d",
            ossOpened ? 1 : 0, ossAdded ? 1 : 0, ossUntitled ? 1 : 0,
            ossView ? ossView->layer_row_count() : -1, qPrintable(ossState),
            ossView ? ossView->sample_argb(0, 0) : 0, ossView ? ossView->history_count() : -1,
            ossRefused ? 1 : 0, ossNoTab ? 1 : 0);
    if (!ossOk) {
        return pictura::selfTest().fail(281, "open as smart object");
    }
    QFile::remove(ossSourcePath);
    QFile::remove(ossBadPath);
    frame.closeDocument(frame.activeDocumentIndex(), false);
    return 0;
}

int pictura::runLayersExportSmartObjectChecks(pictura::PicturaMainWindow& frame)
{
    // lpr_export_smart_object_contents (282): exporting a smart object's
    // embedded source writes the payload bytes to the chosen file and adds no
    // history state; a non-smart layer refuses and writes nothing.
    const bool expCreated = frame.newDocument(QStringLiteral("ExportSmartCtl"), 4, 4,
                                              QStringLiteral("rgb"), 8,
                                              QStringLiteral("white"));
    pictura::PictureView* expView = frame.activeView();
    if (!expCreated || !expView) {
        return pictura::selfTest().fail(282, "export smart object fixture");
    }
    const int expDoc = frame.activeDocumentIndex();
    const QString expPath = expView->add_solid_fill(0xff2244aau);
    expView->rasterize_fill_content(expPath);
    const bool expConverted = expView->convert_to_smart_object(expPath);
    const bool expCan = expView->layer_can_export_smart_object_contents(expPath);
    const int expBase = expView->history_count();
    const QString expDest =
        QDir::tempPath() + QStringLiteral("/kooka-pictura-smart-object-export.psd");
    const bool expExported = expView->export_smart_object_contents(expPath, expDest);
    const int expHistoryDelta = expView->history_count() - expBase;
    QFile expFile(expDest);
    bool expRead = expFile.open(QIODevice::ReadOnly);
    const QByteArray expBytes = expRead ? expFile.readAll() : QByteArray();
    expFile.close();
    const bool expPsd = expBytes.size() > 4 && expBytes.startsWith("8BPS");
    const bool expHistory = expHistoryDelta == 0;

    const QString expPlain = expView->add_solid_fill(0xff22cc44u);
    const bool expPlainCan = !expView->layer_can_export_smart_object_contents(expPlain);
    const int expPlainBase = expView->history_count();
    const QString expPlainDest =
        QDir::tempPath() + QStringLiteral("/kooka-pictura-smart-object-export-plain.psd");
    const bool expRefused = !expView->export_smart_object_contents(expPlain, expPlainDest)
        && !QFile::exists(expPlainDest) && expView->history_count() == expPlainBase;

    const bool expOk = expConverted && expCan && expExported && expPsd && expHistory
        && expPlainCan && expRefused;
    ST_BEGIN("lpr_export_smart_object_contents");
    ST_PASS("lpr_export_smart_object_contents convert=%d can=%d export=%d psd=%d bytes=%d "
            "history=%d plain=%d refuse=%d",
            expConverted ? 1 : 0, expCan ? 1 : 0, expExported ? 1 : 0, expPsd ? 1 : 0,
            expBytes.size(), expHistoryDelta, expPlainCan ? 1 : 0,
            expRefused ? 1 : 0);
    if (!expOk) {
        return pictura::selfTest().fail(282, "export smart object contents");
    }
    QFile::remove(expDest);
    QFile::remove(expPlainDest);
    frame.closeDocument(expDoc, false);
    return 0;
}

int pictura::runLayersEditSmartObjectChecks(pictura::PicturaMainWindow& frame)
{
    // lpr_edit_smart_object_contents (288): Edit Contents opens an editable
    // smart object's source as a new untitled editor tab without touching the
    // origin; saving that tab commits the edited source in exactly one
    // "Edit Contents" history state and leaves the editor open and clean;
    // discarding a second editor leaves the origin unchanged; a non-editable
    // target refuses and adds no tab.
    const bool editCreated = frame.newDocument(QStringLiteral("EditSmartCtl"), 4, 4,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
    pictura::PictureView* editView = frame.activeView();
    if (!editCreated || !editView) {
        return pictura::selfTest().fail(288, "edit smart object fixture");
    }
    const int editOriginDoc = frame.activeDocumentIndex();
    const QString editPath = editView->add_solid_fill(0xff2244aau);
    editView->rasterize_fill_content(editPath);
    const bool editConverted = editView->convert_to_smart_object(editPath);
    const bool editCan = editView->layer_can_edit_smart_object_contents(editPath);
    const int editOriginHistory = editView->history_count();
    const unsigned int editOriginPixel = editView->sample_argb(0, 0);
    const int editDocsBase = frame.documentCount();

    const bool editOpened = frame.editSmartObjectContents(editPath);
    pictura::PictureView* editEditor = frame.activeView();
    const bool editAdded = editOpened && frame.documentCount() == editDocsBase + 1
        && editEditor && editEditor != editView;
    const bool editUntitled = editEditor && frame.documentPath(editDocsBase).isEmpty();
    const bool editDecoded = editEditor && editEditor->has_document()
        && editEditor->layer_row_count() == 1
        && editEditor->layer_smart_object_state(QStringLiteral("0")).isEmpty();
    const bool editOriginUntouched = editView->history_count() == editOriginHistory
        && editView->sample_argb(0, 0) == editOriginPixel;

    const QString editFill = editEditor ? editEditor->add_solid_fill(0xff22cc44u) : QString();
    const bool editModified = editEditor && !editFill.isEmpty()
        && editEditor->rasterize_fill_content(editFill);
    const bool editSaved = frame.saveActive();
    const unsigned int editCommitPixel = editView->sample_argb(0, 0);
    const bool editCommitted = editCommitPixel == 0xff22cc44u;
    const bool editOneState = editView->history_count() == editOriginHistory + 1;
    const bool editLabel = editView->history_label(editOriginHistory)
        == QStringLiteral("Edit Contents");
    const bool editClean = editEditor && !editEditor->is_dirty();
    const bool editStillOpen = frame.documentCount() == editDocsBase + 1;

    frame.setActiveDocumentIndex(editOriginDoc);
    const int editHistoryAfterSave = editView->history_count();
    const unsigned int editPixelAfterSave = editView->sample_argb(0, 0);
    const bool editOpened2 = frame.editSmartObjectContents(editPath);
    pictura::PictureView* editEditor2 = frame.activeView();
    if (editOpened2 && editEditor2) {
        const QString editFill2 = editEditor2->add_solid_fill(0xff000000u);
        editEditor2->rasterize_fill_content(editFill2);
    }
    const bool editDiscarded = frame.closeDocument(frame.activeDocumentIndex(), false);
    const bool editDiscardNoop = editView->history_count() == editHistoryAfterSave
        && editView->sample_argb(0, 0) == editPixelAfterSave;

    frame.setActiveDocumentIndex(editOriginDoc);
    const QString editPlain = editView->add_solid_fill(0xffffffffu);
    const bool editPlainCan = !editView->layer_can_edit_smart_object_contents(editPlain);
    const int editRefuseDocs = frame.documentCount();
    const bool editRefused = !frame.editSmartObjectContents(editPlain);
    const bool editNoTab = frame.documentCount() == editRefuseDocs;

    const bool editOk = editConverted && editCan && editOpened && editAdded && editUntitled
        && editDecoded && editOriginUntouched && editModified && editSaved && editCommitted
        && editOneState && editLabel && editClean && editStillOpen && editDiscarded
        && editDiscardNoop && editPlainCan && editRefused && editNoTab;
    ST_BEGIN("lpr_edit_smart_object_contents");
    ST_PASS("lpr_edit_smart_object_contents can=%d open=%d added=%d untitled=%d decoded=%d "
            "origin=%d saved=%d pixel=%08x state=%d label=%s clean=%d discard=%d plain=%d "
            "refuse=%d notab=%d",
            editCan ? 1 : 0, editOpened ? 1 : 0, editAdded ? 1 : 0, editUntitled ? 1 : 0,
            editDecoded ? 1 : 0, editOriginUntouched ? 1 : 0, editSaved ? 1 : 0,
            editCommitPixel, editOneState ? 1 : 0,
            qPrintable(editView->history_label(editOriginHistory)), editClean ? 1 : 0,
            editDiscardNoop ? 1 : 0, editPlainCan ? 1 : 0, editRefused ? 1 : 0,
            editNoTab ? 1 : 0);
    if (!editOk) {
        return pictura::selfTest().fail(288, "edit smart object contents");
    }
    frame.closeDocument(editDocsBase, false);
    frame.closeDocument(editOriginDoc, false);
    return 0;
}

int pictura::runLayersEditSmartObjectSessionChecks(pictura::PicturaMainWindow& frame)
{
    // lpr_edit_smart_object_session (289): closing an Edit Contents editor
    // removes its session and temporary file; closing the origin first drops the
    // session temp and leaves the orphaned editor open as an untitled tab.
    const bool sesCreated = frame.newDocument(QStringLiteral("EditSessionCtl"), 4, 4,
                                              QStringLiteral("rgb"), 8,
                                              QStringLiteral("white"));
    pictura::PictureView* sesOrigin = frame.activeView();
    if (!sesCreated || !sesOrigin) {
        return pictura::selfTest().fail(289, "edit session fixture");
    }
    const int sesOriginDoc = frame.activeDocumentIndex();
    const QString sesPath = sesOrigin->add_solid_fill(0xff2244aau);
    sesOrigin->rasterize_fill_content(sesPath);
    const bool sesConverted = sesOrigin->convert_to_smart_object(sesPath);

    // Closing the editor drops its session and temp file.
    const int sesDocs = frame.documentCount();
    const bool sesOpened = frame.editSmartObjectContents(sesPath);
    pictura::PictureView* sesEditor = frame.activeView();
    const int sesEditorDoc = frame.activeDocumentIndex();
    const QString sesTemp = sesEditor ? sesEditor->file_path() : QString();
    const bool sesTempLive = !sesTemp.isEmpty() && QFile::exists(sesTemp);
    const bool sesClosed = frame.closeDocument(sesEditorDoc, false);
    const bool sesTempGone = !sesTemp.isEmpty() && !QFile::exists(sesTemp);
    const bool sesDocsBack = frame.documentCount() == sesDocs;

    // Closing the origin drops the session temp and orphans the editor as an
    // untitled tab with an empty document path.
    const bool sesOpened2 = frame.editSmartObjectContents(sesPath);
    pictura::PictureView* sesOrphan = frame.activeView();
    const QString sesOrphanTemp = sesOrphan ? sesOrphan->file_path() : QString();
    const bool sesOrphanUntitled =
        sesOrphan && frame.documentPath(frame.activeDocumentIndex()).isEmpty();
    const bool sesOrphanTempLive = !sesOrphanTemp.isEmpty() && QFile::exists(sesOrphanTemp);
    frame.setActiveDocumentIndex(sesOriginDoc);
    const bool sesOriginClosed = frame.closeDocument(sesOriginDoc, false);
    const int sesOrphanDoc = sesDocs - 1;
    const bool sesOrphanKept = frame.documentCount() == sesDocs
        && frame.viewAt(sesOrphanDoc) == sesOrphan
        && frame.documentPath(sesOrphanDoc).isEmpty();
    const bool sesOrphanTempGone = !sesOrphanTemp.isEmpty() && !QFile::exists(sesOrphanTemp);
    const bool sesOrphanClosed = frame.closeDocument(sesOrphanDoc, false);

    const bool sesOk = sesConverted && sesOpened && sesTempLive && sesClosed && sesTempGone
        && sesDocsBack && sesOpened2 && sesOrphanUntitled && sesOrphanTempLive
        && sesOriginClosed && sesOrphanKept && sesOrphanTempGone && sesOrphanClosed;
    ST_BEGIN("lpr_edit_smart_object_session");
    ST_PASS("lpr_edit_smart_object_session convert=%d open=%d temp=%d closed=%d tempgone=%d "
            "docsback=%d open2=%d untitled=%d originclosed=%d kept=%d orphantempgone=%d "
            "closed2=%d",
            sesConverted ? 1 : 0, sesOpened ? 1 : 0, sesTempLive ? 1 : 0, sesClosed ? 1 : 0,
            sesTempGone ? 1 : 0, sesDocsBack ? 1 : 0, sesOpened2 ? 1 : 0,
            sesOrphanUntitled ? 1 : 0, sesOriginClosed ? 1 : 0, sesOrphanKept ? 1 : 0,
            sesOrphanTempGone ? 1 : 0, sesOrphanClosed ? 1 : 0);
    if (!sesOk) {
        return pictura::selfTest().fail(289, "edit smart object session");
    }
    return 0;
}
