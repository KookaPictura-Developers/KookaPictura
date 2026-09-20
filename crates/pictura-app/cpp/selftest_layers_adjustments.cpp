#include "selftest_layers_adjustments.h"
#include "selftest_report.h"

#include "frame.h"
#include "panels/panel_column.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QFile>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtCore/QTemporaryDir>
#include <QtGui/QImage>

int pictura::runLayersAdjustmentChecks(pictura::PicturaMainWindow& frame)
{
        // lpr_photo_filter (283): a photo-filter adjustment layer is reported
        // as an adjustment and warms the composite (red exceeds blue at a
        // sampled pixel).
        {
            const bool pflCreated = frame.newDocument(QStringLiteral("PhotoFilterCtl"), 4, 4,
                                                      QStringLiteral("rgb"), 8,
                                                      QStringLiteral("white"));
            pictura::PictureView* pflView = frame.activeView();
            if (!pflCreated || !pflView) {
                return pictura::selfTest().fail(283, "photo filter fixture");
            }
            const int pflDoc = frame.activeDocumentIndex();
            const unsigned int pflBefore = pflView->sample_argb(1, 1);
            const bool pflAdded = pflView->add_adjustment(QStringLiteral("photo-filter"));
            const bool pflAdjustment =
                pflView->layer_kind(pflView->layer_count() - 1) == QStringLiteral("adjustment");
            const unsigned int pflAfter = pflView->sample_argb(1, 1);
            const bool pflWarm = qRed(pflAfter) > qBlue(pflAfter);
            const bool pflChanged = pflAfter != pflBefore;
            ST_BEGIN("lpr_photo_filter");
            ST_PASS("lpr_photo_filter added=%d adjustment=%d before=%08x after=%08x warm=%d",
                    pflAdded ? 1 : 0, pflAdjustment ? 1 : 0, pflBefore, pflAfter,
                    pflWarm ? 1 : 0);
            if (!pflAdded || !pflAdjustment || !pflChanged || !pflWarm) {
                return pictura::selfTest().fail(283, "photo filter adjustment");
            }
            frame.closeDocument(pflDoc, false);
        }

        // adjustments_photo_filter_menu (284): the Adjustments panel menu offers
        // the `Photo Filter` row wired to `adjustment:photo-filter`.
        const pictura::PanelColumn* adjColumn = frame.panelColumn();
        const bool adjPF = adjColumn
            && adjColumn->widgetMenuTextsForTest(QStringLiteral("adjustmentsPanel"))
                   .contains(QStringLiteral("Photo Filter"));
        ST_BEGIN("adjustments_photo_filter_menu");
        ST_PASS("adjustments_photo_filter_menu row=%d", adjPF ? 1 : 0);
        if (!adjPF) {
            return pictura::selfTest().fail(284, "adjustments photo filter menu");
        }

        // lpr_gradient_map (285): a gradient-map adjustment layer is reported as
        // an adjustment, maps a non-uniform backdrop's luminance through the
        // black-to-white gradient, and the Adjustments panel menu offers it.
        {
            const bool gmCreated = frame.newDocument(QStringLiteral("GradientMapCtl"), 8, 8,
                                                     QStringLiteral("rgb"), 8,
                                                     QStringLiteral("white"));
            pictura::PictureView* gmView = frame.activeView();
            if (!gmCreated || !gmView) {
                return pictura::selfTest().fail(285, "gradient map fixture");
            }
            const int gmDoc = frame.activeDocumentIndex();
            bool gmPainted = gmView->begin_paint(0xFFFF0000u, 0xFFFFFFFFu, 4, 100, 100, 0,
                                                 100, 100, 25, QStringLiteral("normal"), false,
                                                 false);
            gmPainted = gmPainted && gmView->paint_dab(2, 2, 1.0) && gmView->paint_dab(5, 5, 1.0)
                && gmView->end_paint();
            const unsigned int gmBefore = gmView->sample_argb(2, 2);
            const bool gmAdded = gmView->add_adjustment(QStringLiteral("gradient-map"));
            const bool gmAdjustment =
                gmView->layer_kind(gmView->layer_count() - 1) == QStringLiteral("adjustment");
            const unsigned int gmAfter = gmView->sample_argb(2, 2);
            const pictura::PanelColumn* gmColumn = frame.panelColumn();
            const bool gmMenu = gmColumn
                && gmColumn->widgetMenuTextsForTest(QStringLiteral("adjustmentsPanel"))
                       .contains(QStringLiteral("Gradient Map"));
            ST_BEGIN("lpr_gradient_map");
            ST_PASS("lpr_gradient_map added=%d adjustment=%d changed=%d menu=%d before=%08x "
                    "after=%08x",
                    gmAdded ? 1 : 0, gmAdjustment ? 1 : 0, gmAfter != gmBefore ? 1 : 0,
                    gmMenu ? 1 : 0, gmBefore, gmAfter);
            if (!gmPainted || !gmAdded || !gmAdjustment || gmAfter == gmBefore || !gmMenu) {
                return pictura::selfTest().fail(285, "gradient map adjustment");
            }
            frame.closeDocument(gmDoc, false);
        }

        // lpr_color_balance (293): the neutral `color-balance` kind becomes an
        // adjustment layer that leaves the composite unchanged, and the
        // Adjustments panel offers a `Color Balance` row.
        {
            const bool cbCreated = frame.newDocument(QStringLiteral("ColorBalanceCtl"), 4, 4,
                                                     QStringLiteral("rgb"), 8,
                                                     QStringLiteral("white"));
            pictura::PictureView* cbView = frame.activeView();
            if (!cbCreated || !cbView) {
                return pictura::selfTest().fail(293, "color balance fixture");
            }
            const int cbDoc = frame.activeDocumentIndex();
            const unsigned int cbBefore = cbView->sample_argb(1, 1);
            const bool cbAdded = cbView->add_adjustment(QStringLiteral("color-balance"));
            const bool cbAdjustment =
                cbView->layer_kind(cbView->layer_count() - 1) == QStringLiteral("adjustment");
            const unsigned int cbAfter = cbView->sample_argb(1, 1);
            const bool cbNeutral = cbAfter == cbBefore;
            const pictura::PanelColumn* cbColumn = frame.panelColumn();
            const bool cbMenu = cbColumn
                && cbColumn->widgetMenuTextsForTest(QStringLiteral("adjustmentsPanel"))
                       .contains(QStringLiteral("Color Balance"));
            ST_BEGIN("lpr_color_balance");
            ST_PASS("lpr_color_balance added=%d adjustment=%d neutral=%d menu=%d",
                    cbAdded ? 1 : 0, cbAdjustment ? 1 : 0, cbNeutral ? 1 : 0, cbMenu ? 1 : 0);
            if (!cbAdded || !cbAdjustment || !cbNeutral || !cbMenu) {
                return pictura::selfTest().fail(293, "color balance adjustment");
            }
            frame.closeDocument(cbDoc, false);
        }

        // lpr_channel_mixer (294): the neutral `channel-mixer` kind becomes an
        // adjustment layer that leaves the composite unchanged, and the
        // Adjustments panel offers a `Channel Mixer` row.
        {
            const bool cmCreated = frame.newDocument(QStringLiteral("ChannelMixerCtl"), 4, 4,
                                                     QStringLiteral("rgb"), 8,
                                                     QStringLiteral("white"));
            pictura::PictureView* cmView = frame.activeView();
            if (!cmCreated || !cmView) {
                return pictura::selfTest().fail(294, "channel mixer fixture");
            }
            const int cmDoc = frame.activeDocumentIndex();
            const unsigned int cmBefore = cmView->sample_argb(1, 1);
            const bool cmAdded = cmView->add_adjustment(QStringLiteral("channel-mixer"));
            const bool cmAdjustment =
                cmView->layer_kind(cmView->layer_count() - 1) == QStringLiteral("adjustment");
            const unsigned int cmAfter = cmView->sample_argb(1, 1);
            const bool cmNeutral = cmAfter == cmBefore;
            const pictura::PanelColumn* cmColumn = frame.panelColumn();
            const bool cmMenu = cmColumn
                && cmColumn->widgetMenuTextsForTest(QStringLiteral("adjustmentsPanel"))
                       .contains(QStringLiteral("Channel Mixer"));
            ST_BEGIN("lpr_channel_mixer");
            ST_PASS("lpr_channel_mixer added=%d adjustment=%d neutral=%d menu=%d",
                    cmAdded ? 1 : 0, cmAdjustment ? 1 : 0, cmNeutral ? 1 : 0, cmMenu ? 1 : 0);
            if (!cmAdded || !cmAdjustment || !cmNeutral || !cmMenu) {
                return pictura::selfTest().fail(294, "channel mixer adjustment");
            }
            frame.closeDocument(cmDoc, false);
        }

        // lpr_selective_color (296): the neutral `selective-color` kind becomes
        // an adjustment layer that leaves the composite unchanged, and the
        // Adjustments panel offers a `Selective Color` row.
        {
            const bool selCreated = frame.newDocument(QStringLiteral("SelectiveColorCtl"), 4, 4,
                                                      QStringLiteral("rgb"), 8,
                                                      QStringLiteral("white"));
            pictura::PictureView* selView = frame.activeView();
            if (!selCreated || !selView) {
                return pictura::selfTest().fail(296, "selective color fixture");
            }
            const int selDoc = frame.activeDocumentIndex();
            const unsigned int selBefore = selView->sample_argb(1, 1);
            const bool selAdded = selView->add_adjustment(QStringLiteral("selective-color"));
            const bool selAdjustment =
                selView->layer_kind(selView->layer_count() - 1) == QStringLiteral("adjustment");
            const unsigned int selAfter = selView->sample_argb(1, 1);
            const bool selNeutral = selAfter == selBefore;
            const pictura::PanelColumn* selColumn = frame.panelColumn();
            const bool selMenu = selColumn
                && selColumn->widgetMenuTextsForTest(QStringLiteral("adjustmentsPanel"))
                       .contains(QStringLiteral("Selective Color"));
            ST_BEGIN("lpr_selective_color");
            ST_PASS("lpr_selective_color added=%d adjustment=%d neutral=%d menu=%d",
                    selAdded ? 1 : 0, selAdjustment ? 1 : 0, selNeutral ? 1 : 0,
                    selMenu ? 1 : 0);
            if (!selAdded || !selAdjustment || !selNeutral || !selMenu) {
                return pictura::selfTest().fail(296, "selective color adjustment");
            }
            frame.closeDocument(selDoc, false);
        }

        // color_mode_open (297): a minimal flat CMYK PSD opens as a normalized
        // RGB document and the view reports the CMYK conversion notice.
        {
            QTemporaryDir cmDir;
            const QString cmPath = cmDir.filePath(QStringLiteral("cmyk.psd"));
            QByteArray cmBytes;
            const auto cmAppend16 = [&cmBytes](unsigned short value) {
                cmBytes.append(char((value >> 8) & 0xff));
                cmBytes.append(char(value & 0xff));
            };
            const auto cmAppend32 = [&cmBytes](unsigned int value) {
                cmBytes.append(char((value >> 24) & 0xff));
                cmBytes.append(char((value >> 16) & 0xff));
                cmBytes.append(char((value >> 8) & 0xff));
                cmBytes.append(char(value & 0xff));
            };
            cmBytes.append("8BPS", 4);
            cmAppend16(1);              // version
            cmBytes.append(6, char(0)); // reserved
            cmAppend16(4);              // channels
            cmAppend32(1);              // height
            cmAppend32(1);              // width
            cmAppend16(8);              // depth
            cmAppend16(4);              // color mode CMYK
            cmAppend32(0);              // color mode data
            cmAppend32(0);              // image resources
            cmAppend32(0);              // layer/mask section
            cmAppend16(0);              // raw compression
            cmBytes.append(char(128));  // C
            cmBytes.append(char(64));   // M
            cmBytes.append(char(32));   // Y
            cmBytes.append(char(200));  // K
            QFile cmFile(cmPath);
            const bool cmWritten = cmFile.open(QIODevice::WriteOnly)
                && cmFile.write(cmBytes) == cmBytes.size();
            cmFile.close();

            const int cmDocs = frame.documentCount();
            const bool cmOpened = cmWritten && frame.openPath(cmPath);
            pictura::PictureView* cmView = frame.activeView();
            const bool cmOk = cmOpened && frame.documentCount() == cmDocs + 1 && cmView
                && cmView->mode_notice() == QStringLiteral("Converted from CMYK")
                && cmView->document_mode() == QStringLiteral("rgb")
                && cmView->sample_argb(0, 0) == 0xff643219u;
            ST_BEGIN("color_mode_open");
            ST_PASS("color_mode_open open=%d notice=%s mode=%s pixel=%08x",
                    cmOpened ? 1 : 0, cmView ? qPrintable(cmView->mode_notice()) : "-",
                    cmView ? qPrintable(cmView->document_mode()) : "-",
                    cmView ? cmView->sample_argb(0, 0) : 0u);
            if (cmView) {
                frame.closeDocument(frame.activeDocumentIndex(), false);
            }
            if (!cmOk) {
                return pictura::selfTest().fail(297, "color mode open");
            }
        }

    return 0;
}
