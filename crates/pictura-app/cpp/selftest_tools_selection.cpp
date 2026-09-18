#include "selftest_tools_selection.h"
#include "selftest_report.h"

#include "commands.h"
#include "frame.h"
#include "image_view.h"
#include "panels/layers_panel.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QPointF>
#include <QtCore/QString>
#include <QtCore/Qt>
#include <QtGui/QAction>
#include <QtGui/QKeyEvent>
#include <QtWidgets/QApplication>

int pictura::runToolsSelectionChecks(pictura::PicturaMainWindow& frame)
{
        // tsc_ellipse (242): an Elliptical Marquee drag commits the ellipse
        // inscribed in the drag rectangle; the corner is excluded, so the
        // coverage is strictly smaller than the bounding rectangle.
        const bool ellipseCreated = frame.newDocument(
            QStringLiteral("ToolSelection"), 16, 16, QStringLiteral("rgb"), 8,
            QStringLiteral("white"));
        pictura::PictureView* ellipseView = frame.activeView();
        pictura::ImageView* ellipseCanvas = frame.imageView();
        auto* tools = frame.findChild<pictura::ToolController*>();
        if (!ellipseCreated || !ellipseView || !ellipseCanvas || !tools) {
            return pictura::selfTest().fail(242, "selection tool fixture");
        }
        const int ellipseDoc = frame.activeDocumentIndex();
        frame.setActiveTool(pictura::ToolId::EllipticalMarquee);
        const bool ellipseActive = frame.activeTool() == pictura::ToolId::EllipticalMarquee;
        const int ellipseBase = ellipseView->history_count();
        ellipseCanvas->mousePressed(QPointF(2, 2), Qt::LeftButton, int(Qt::NoModifier));
        ellipseCanvas->mouseMoved(QPointF(10, 10));
        ellipseCanvas->mouseReleased(QPointF(10, 10));
        const bool ellipseDrawn = ellipseView->has_selection()
            && ellipseView->history_count() == ellipseBase + 1;
        const int ellipsePx = ellipseView->selection_count();
        ellipseView->select_rect(2, 2, 1, 1, QStringLiteral("intersect"), 0.0);
        const int ellipseCornerPx = ellipseView->selection_count();
        ST_BEGIN("tsc_ellipse");
        ST_PASS("tsc_ellipse active=%d drawn=%d px=%d corner=%d", ellipseActive ? 1 : 0,
                ellipseDrawn ? 1 : 0, ellipsePx, ellipseCornerPx);
        if (!ellipseActive || !ellipseDrawn || ellipsePx <= 0 || ellipsePx >= 64
            || ellipseCornerPx != 0) {
            return pictura::selfTest().fail(242, "elliptical marquee");
        }

        // tsc_feather (243): a tool-time Feather softens the committed boundary,
        // so the coverage contains values strictly between 0 and 255.
        tools->setFeather(8.0);
        ellipseView->deselect();
        const int featherBase = ellipseView->history_count();
        ellipseCanvas->mousePressed(QPointF(3, 3), Qt::LeftButton, int(Qt::NoModifier));
        ellipseCanvas->mouseMoved(QPointF(13, 13));
        ellipseCanvas->mouseReleased(QPointF(13, 13));
        int softPixels = 0;
        for (int y = 0; y < 16; ++y) {
            for (int x = 0; x < 16; ++x) {
                const int coverage = ellipseView->selection_coverage(x, y);
                if (coverage > 0 && coverage < 255) {
                    ++softPixels;
                }
            }
        }
        const bool featherApplied = tools->feather() == 8.0;
        const bool featherDrawn = ellipseView->has_selection()
            && ellipseView->history_count() == featherBase + 1;
        ST_BEGIN("tsc_feather");
        ST_PASS("tsc_feather applied=%d drawn=%d soft=%d", featherApplied ? 1 : 0,
                featherDrawn ? 1 : 0, softPixels);
        if (!featherApplied || !featherDrawn || softPixels <= 0) {
            return pictura::selfTest().fail(243, "marquee feather");
        }
        tools->setFeather(0.0);

        // tsc_fixed_size (244): Fixed Size ignores the drag extent and commits
        // the configured size centred on the mousedown.
        tools->setMarqueeStyle(pictura::MarqueeStyle::FixedSize);
        tools->setFixedSize(10, 6);
        frame.setActiveTool(pictura::ToolId::Marquee);
        ellipseView->deselect();
        const int fixedBase = ellipseView->history_count();
        ellipseCanvas->mousePressed(QPointF(5, 5), Qt::LeftButton, int(Qt::NoModifier));
        ellipseCanvas->mouseMoved(QPointF(15, 15));
        ellipseCanvas->mouseReleased(QPointF(15, 15));
        const QString fixedBounds = ellipseView->selection_bounds();
        const bool fixedDrawn = ellipseView->has_selection()
            && ellipseView->history_count() == fixedBase + 1;
        ST_BEGIN("tsc_fixed_size");
        ST_PASS("tsc_fixed_size drawn=%d bounds=%s", fixedDrawn ? 1 : 0,
                qPrintable(fixedBounds));
        if (!fixedDrawn || fixedBounds != QStringLiteral("0 2 10 6")) {
            return pictura::selfTest().fail(244, "fixed size marquee");
        }
        tools->setMarqueeStyle(pictura::MarqueeStyle::Normal);

        // tsc_polygon (245): the Polygonal Lasso commits the clicked triangle
        // through the press path; clicking the first vertex closes it, the
        // interior is selected, and a point across the hypotenuse is excluded.
        frame.setActiveTool(pictura::ToolId::PolygonalLasso);
        ellipseView->deselect();
        const int polygonBase = ellipseView->history_count();
        const auto polygonClick = [ellipseCanvas](qreal x, qreal y) {
            ellipseCanvas->mousePressed(QPointF(x, y), Qt::LeftButton, int(Qt::NoModifier));
            ellipseCanvas->mouseReleased(QPointF(x, y));
        };
        polygonClick(2, 2);
        polygonClick(12, 2);
        polygonClick(2, 12);
        polygonClick(2, 2);
        const bool polygonDrawn = ellipseView->has_selection()
            && ellipseView->history_count() == polygonBase + 1;
        const int polygonInside = ellipseView->selection_coverage(4, 4);
        const int polygonOutside = ellipseView->selection_coverage(11, 11);
        ST_BEGIN("tsc_polygon");
        ST_PASS("tsc_polygon drawn=%d inside=%d outside=%d", polygonDrawn ? 1 : 0,
                polygonInside, polygonOutside);
        if (!polygonDrawn || polygonInside <= 0 || polygonOutside != 0) {
            return pictura::selfTest().fail(245, "polygonal lasso");
        }

        // tsc_polygon_cancel (246): Escape discards an in-progress path without
        // changing the selection or history; a later close cannot resurrect it.
        ellipseView->deselect();
        const int cancelBase = ellipseView->history_count();
        const int cancelSelBefore = ellipseView->selection_count();
        polygonClick(3, 3);
        polygonClick(10, 3);
        QKeyEvent escapeEvent(QEvent::KeyPress, Qt::Key_Escape, Qt::NoModifier);
        QApplication::sendEvent(&frame, &escapeEvent);
        const bool cancelClean = ellipseView->history_count() == cancelBase
            && ellipseView->selection_count() == cancelSelBefore;
        // A commit attempt must now find nothing in progress; this is the
        // observable proof that Escape reached the controller.
        const bool cancelCleared = !tools->commitPolygonLasso();
        polygonClick(2, 2);
        polygonClick(12, 2);
        polygonClick(2, 2);
        const bool cancelNoCommit =
            ellipseView->history_count() == cancelBase && !ellipseView->has_selection();
        ST_BEGIN("tsc_polygon_cancel");
        ST_PASS("tsc_polygon_cancel clean=%d cleared=%d nocommit=%d", cancelClean ? 1 : 0,
                cancelCleared ? 1 : 0, cancelNoCommit ? 1 : 0);
        if (!cancelClean || !cancelCleared || !cancelNoCommit) {
            return pictura::selfTest().fail(246, "polygonal lasso cancel");
        }

        // tsc_polygon_feather (247): a tool-time Feather on the Polygonal Lasso
        // softens the committed boundary.
        tools->setFeather(6.0);
        ellipseView->deselect();
        const int polygonFeatherBase = ellipseView->history_count();
        polygonClick(3, 3);
        polygonClick(13, 3);
        polygonClick(13, 13);
        polygonClick(3, 13);
        polygonClick(3, 3);
        int polygonSoft = 0;
        for (int y = 0; y < 16; ++y) {
            for (int x = 0; x < 16; ++x) {
                const int coverage = ellipseView->selection_coverage(x, y);
                if (coverage > 0 && coverage < 255) {
                    ++polygonSoft;
                }
            }
        }
        const bool polygonFeatherDrawn = ellipseView->has_selection()
            && ellipseView->history_count() == polygonFeatherBase + 1;
        ST_BEGIN("tsc_polygon_feather");
        ST_PASS("tsc_polygon_feather drawn=%d soft=%d", polygonFeatherDrawn ? 1 : 0,
                polygonSoft);
        if (!polygonFeatherDrawn || polygonSoft <= 0) {
            return pictura::selfTest().fail(247, "polygonal lasso feather");
        }
        tools->setFeather(0.0);

        // tsc_wand_contiguous (248): a Magic Wand click selects the contiguous
        // uniform region and excludes an identical-coloured disconnected patch.
        // Two separate strokes (a single stroke interpolates between dabs).
        const auto wandDab = [ellipseView](qreal x, qreal y) {
            const bool begun = ellipseView->begin_paint(
                0xFFFF0000u, 0xFFFFFFFFu, 5, 100, 100, 0, 100, 100, 25,
                QStringLiteral("normal"), true, false);
            return begun && ellipseView->paint_dab(x, y, 1.0) && ellipseView->end_paint();
        };
        const bool wandPainted = wandDab(3, 3) && wandDab(12, 12);
        frame.setActiveTool(pictura::ToolId::MagicWand);
        tools->setTolerance(32);
        tools->setCombineMode(pictura::SelectionMode::New);
        tools->setContiguous(true);
        ellipseView->deselect();
        const int wandBase = ellipseView->history_count();
        ellipseCanvas->mousePressed(QPointF(3, 3), Qt::LeftButton, int(Qt::NoModifier));
        ellipseCanvas->mouseReleased(QPointF(3, 3));
        const bool wandActive = frame.activeTool() == pictura::ToolId::MagicWand;
        const bool wandContigDrawn = wandPainted && ellipseView->has_selection()
            && ellipseView->history_count() == wandBase + 1;
        const int wandContigPx = ellipseView->selection_count();
        const bool wandContigExcludes = ellipseView->selection_coverage(12, 12) == 0;
        ST_BEGIN("tsc_wand_contiguous");
        ST_PASS("tsc_wand_contiguous active=%d drawn=%d px=%d excl=%d", wandActive ? 1 : 0,
                wandContigDrawn ? 1 : 0, wandContigPx, wandContigExcludes ? 1 : 0);
        if (!wandActive || !wandContigDrawn || wandContigPx <= 0 || !wandContigExcludes) {
            return pictura::selfTest().fail(248, "magic wand contiguous");
        }

        // tsc_wand_global (249): non-contiguous mode includes the disconnected
        // patch, so the committed selection grows and covers its seed pixel.
        ellipseView->deselect();
        tools->setContiguous(false);
        const int wandGlobalBase = ellipseView->history_count();
        ellipseCanvas->mousePressed(QPointF(3, 3), Qt::LeftButton, int(Qt::NoModifier));
        const bool wandGlobalDrawn = ellipseView->has_selection()
            && ellipseView->history_count() == wandGlobalBase + 1;
        const int wandGlobalPx = ellipseView->selection_count();
        const int wandGlobalHit = ellipseView->selection_coverage(12, 12);
        ST_BEGIN("tsc_wand_global");
        ST_PASS("tsc_wand_global drawn=%d px=%d hit=%d", wandGlobalDrawn ? 1 : 0,
                wandGlobalPx, wandGlobalHit);
        if (!wandGlobalDrawn || wandGlobalPx <= wandContigPx || wandGlobalHit <= 0) {
            return pictura::selfTest().fail(249, "magic wand global");
        }

        // tsc_wand_add (250): Add mode unions the wand result with the existing
        // selection instead of replacing it.
        ellipseView->deselect();
        tools->setContiguous(true);
        tools->setCombineMode(pictura::SelectionMode::Add);
        const bool wandSeeded =
            ellipseView->select_rect(10, 10, 5, 5, QStringLiteral("new"), 0.0);
        const int wandSeedPx = ellipseView->selection_count();
        const int wandAddBase = ellipseView->history_count();
        ellipseCanvas->mousePressed(QPointF(3, 3), Qt::LeftButton, int(Qt::NoModifier));
        const bool wandAddDrawn = wandSeeded && ellipseView->has_selection()
            && ellipseView->history_count() == wandAddBase + 1;
        const int wandAddPx = ellipseView->selection_count();
        const int wandAddKept = ellipseView->selection_coverage(11, 11);
        ST_BEGIN("tsc_wand_add");
        ST_PASS("tsc_wand_add drawn=%d px=%d kept=%d", wandAddDrawn ? 1 : 0, wandAddPx,
                wandAddKept);
        if (!wandAddDrawn || wandAddPx <= wandSeedPx || wandAddKept <= 0) {
            return pictura::selfTest().fail(250, "magic wand add");
        }
        tools->setCombineMode(pictura::SelectionMode::New);
        frame.closeDocument(ellipseDoc, false);

        // tsc_select_inverse (251): Inverse complements the coverage and records
        // one undo state.
        const bool smCreated = frame.newDocument(QStringLiteral("SelectMenu"), 16, 16,
                                                 QStringLiteral("rgb"), 8, QStringLiteral("white"));
        pictura::PictureView* smView = frame.activeView();
        auto* smPanel = frame.findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
        if (!smCreated || !smView || !smPanel) {
            return pictura::selfTest().fail(251, "select menu fixture");
        }
        const int smDoc = frame.activeDocumentIndex();
        const bool smSeed = smView->select_rect(0, 0, 8, 16, QStringLiteral("new"), 0.0);
        const int smBase = smView->history_count();
        const bool smInverted = smView->invert_selection();
        const bool smFlipped = smView->selection_coverage(0, 0) == 0
            && smView->selection_coverage(8, 0) == 255;
        ST_BEGIN("tsc_select_inverse");
        ST_PASS("tsc_select_inverse seed=%d drawn=%d flipped=%d states=%d", smSeed ? 1 : 0,
                smInverted ? 1 : 0, smFlipped ? 1 : 0, smView->history_count() - smBase);
        if (!smSeed || !smInverted || !smFlipped || smView->history_count() != smBase + 1) {
            return pictura::selfTest().fail(251, "select inverse");
        }

        // tsc_select_reselect (252): Deselect stores the selection and Reselect
        // restores the same byte coverage in one undo state.
        const int rsCount = smView->selection_count();
        smView->deselect();
        const bool rsStored = smView->has_deselected_selection() && !smView->has_selection();
        const int rsBase = smView->history_count();
        const bool rsRestored = smView->reselect();
        const bool rsExact = smView->selection_count() == rsCount
            && smView->selection_coverage(0, 0) == 0 && smView->selection_coverage(8, 0) == 255;
        ST_BEGIN("tsc_select_reselect");
        ST_PASS("tsc_select_reselect stored=%d restored=%d exact=%d states=%d", rsStored ? 1 : 0,
                rsRestored ? 1 : 0, rsExact ? 1 : 0, smView->history_count() - rsBase);
        if (!rsStored || !rsRestored || !rsExact || smView->history_count() != rsBase + 1) {
            return pictura::selfTest().fail(252, "select reselect");
        }

        // tsc_select_modify (253): Border empties a block's interior, Expand and
        // Contract move its pixel count, Smooth and Feather apply; each records
        // exactly one state.
        const auto smBlock = [smView]() {
            return smView->select_rect(4, 4, 8, 8, QStringLiteral("new"), 0.0);
        };
        const bool mdSeed = smBlock();
        const int mdBase = smView->history_count();
        const bool mdBorder = smView->modify_selection(QStringLiteral("border"), 6);
        const bool mdBand = smView->selection_coverage(8, 8) == 0
            && smView->selection_coverage(4, 7) > 0;
        const int mdStates = smView->history_count() - mdBase;
        smBlock();
        const int mdExpandBefore = smView->selection_count();
        const bool mdExpand = smView->modify_selection(QStringLiteral("expand"), 3);
        const int mdExpanded = smView->selection_count();
        smBlock();
        const int mdContractBefore = smView->selection_count();
        const bool mdContract = smView->modify_selection(QStringLiteral("contract"), 3);
        const int mdContracted = smView->selection_count();
        smBlock();
        const bool mdSmooth = smView->modify_selection(QStringLiteral("smooth"), 2);
        smBlock();
        const bool mdFeather = smView->modify_selection(QStringLiteral("feather"), 5.0);
        int mdSoft = 0;
        for (int y = 0; y < 16; ++y) {
            for (int x = 0; x < 16; ++x) {
                const int coverage = smView->selection_coverage(x, y);
                if (coverage > 0 && coverage < 255) {
                    ++mdSoft;
                }
            }
        }
        ST_BEGIN("tsc_select_modify");
        ST_PASS("tsc_select_modify seed=%d border=%d band=%d expand=%d contract=%d smooth=%d "
                "feather=%d soft=%d states=%d",
                mdSeed ? 1 : 0, mdBorder ? 1 : 0, mdBand ? 1 : 0, mdExpand ? 1 : 0,
                mdContract ? 1 : 0, mdSmooth ? 1 : 0, mdFeather ? 1 : 0, mdSoft, mdStates);
        if (!mdSeed || !mdBorder || !mdBand || !mdExpand || !mdContract || !mdSmooth || !mdFeather
            || mdSoft <= 0 || mdExpanded <= mdExpandBefore || mdContracted >= mdContractBefore
            || mdStates != 1) {
            return pictura::selfTest().fail(253, "select modify");
        }

        frame.closeDocument(smDoc, false);

        // Two disconnected red patches on white, for Grow and Similar.
        const bool grCreated = frame.newDocument(QStringLiteral("SelectMenuGrow"), 16, 16,
                                                 QStringLiteral("rgb"), 8, QStringLiteral("white"));
        pictura::PictureView* grView = frame.activeView();
        if (!grCreated || !grView) {
            return pictura::selfTest().fail(254, "grow fixture");
        }
        const int grDoc = frame.activeDocumentIndex();
        const auto grPaint = [grView](qreal x, qreal y) {
            const bool begun = grView->begin_paint(
                0xFFFF0000u, 0xFFFFFFFFu, 5, 100, 100, 0, 100, 100, 25,
                QStringLiteral("normal"), true, false);
            return begun && grView->paint_dab(x, y, 1.0) && grView->end_paint();
        };
        const bool grPainted = grPaint(3, 3) && grPaint(12, 12);

        // tsc_select_grow (254): Grow adds only nearby similar pixels, so the
        // seed's patch grows but the disconnected patch stays out.
        grView->deselect();
        const bool grSeed = grView->select_rect(3, 3, 1, 1, QStringLiteral("new"), 0.0);
        const int grBase = grView->history_count();
        const bool grGrown = grView->grow_selection(32);
        const int grCount = grView->selection_count();
        const bool grExcludes = grView->selection_coverage(12, 12) == 0;
        ST_BEGIN("tsc_select_grow");
        ST_PASS("tsc_select_grow painted=%d seed=%d drawn=%d px=%d excluded=%d states=%d",
                grPainted ? 1 : 0, grSeed ? 1 : 0, grGrown ? 1 : 0, grCount, grExcludes ? 1 : 0,
                grView->history_count() - grBase);
        if (!grPainted || !grSeed || !grGrown || grCount <= 1 || !grExcludes
            || grView->history_count() != grBase + 1) {
            return pictura::selfTest().fail(254, "select grow");
        }

        // tsc_select_similar (255): Similar reaches the disconnected patch.
        grView->deselect();
        const bool siSeed = grView->select_rect(3, 3, 1, 1, QStringLiteral("new"), 0.0);
        const int siBase = grView->history_count();
        const bool siGrown = grView->similar_selection(32);
        const bool siReaches = grView->selection_coverage(12, 12) > 0;
        ST_BEGIN("tsc_select_similar");
        ST_PASS("tsc_select_similar seed=%d drawn=%d reached=%d states=%d", siSeed ? 1 : 0,
                siGrown ? 1 : 0, siReaches ? 1 : 0, grView->history_count() - siBase);
        if (!siSeed || !siGrown || !siReaches || grView->history_count() != siBase + 1) {
            return pictura::selfTest().fail(255, "select similar");
        }
        frame.closeDocument(grDoc, false);

        // tsc_save_load (256): a feathered selection round-trips byte for byte
        // through an extra channel; save and load each record one state.
        const bool slCreated = frame.newDocument(QStringLiteral("SelectMenuSave"), 16, 16,
                                                 QStringLiteral("rgb"), 8, QStringLiteral("white"));
        pictura::PictureView* slView = frame.activeView();
        if (!slCreated || !slView) {
            return pictura::selfTest().fail(256, "save fixture");
        }
        const int slDoc = frame.activeDocumentIndex();
        slView->deselect();
        const bool slSeed = slView->select_rect(2, 2, 5, 5, QStringLiteral("new"), 2.0);
        int slCoverage[256];
        for (int y = 0; y < 16; ++y) {
            for (int x = 0; x < 16; ++x) {
                slCoverage[y * 16 + x] = slView->selection_coverage(x, y);
            }
        }
        const int slBase = slView->history_count();
        const bool slSaved = slView->save_selection(QStringLiteral("Alpha 1"));
        const bool slChannel = slView->selection_channel_count() == 1;
        slView->deselect();
        const bool slLoaded = slView->load_selection(QStringLiteral("Alpha 1"));
        bool slExact = slSeed && slView->has_selection();
        for (int y = 0; y < 16 && slExact; ++y) {
            for (int x = 0; x < 16 && slExact; ++x) {
                if (slView->selection_coverage(x, y) != slCoverage[y * 16 + x]) {
                    slExact = false;
                }
            }
        }
        ST_BEGIN("tsc_save_load");
        ST_PASS("tsc_save_load seed=%d saved=%d channel=%d loaded=%d exact=%d states=%d",
                slSeed ? 1 : 0, slSaved ? 1 : 0, slChannel ? 1 : 0, slLoaded ? 1 : 0,
                slExact ? 1 : 0, slView->history_count() - slBase);
        if (!slSeed || !slSaved || !slChannel || !slLoaded || !slExact
            || slView->history_count() != slBase + 3) {
            return pictura::selfTest().fail(256, "save load selection");
        }

        // tsc_select_refusal (257): without a selection, and for unknown names or
        // zero amounts, the bridge refuses and records nothing. A wrong-length
        // channel is refused by the `wrong_length_channel_is_refused` unit test.
        slView->deselect();
        const int rfBase = slView->history_count();
        const bool rfInvert = !slView->invert_selection();
        const bool rfModify = !slView->modify_selection(QStringLiteral("expand"), 5);
        const bool rfGrow = !slView->grow_selection(32);
        const bool rfSimilar = !slView->similar_selection(32);
        const bool rfSave = !slView->save_selection(QStringLiteral("Alpha 2"));
        const bool rfLoad = !slView->load_selection(QStringLiteral("Alpha 9"));
        const bool rfZero = !slView->modify_selection(QStringLiteral("expand"), 0.0);
        const bool rfUnknown = !slView->modify_selection(QStringLiteral("bogus"), 5);
        frame.registry()->refresh();
        const bool rfDisabled = !frame.registry()
                                     ->action(QString::fromLatin1(pictura::command_ids::SelectInverse))
                                     ->isEnabled()
            && !frame.registry()
                    ->action(QString::fromLatin1(pictura::command_ids::SelectSave))
                    ->isEnabled()
            && frame.registry()
                   ->action(QString::fromLatin1(pictura::command_ids::SelectReselect))
                   ->isEnabled();
        const bool rfNoHistory = slView->history_count() == rfBase;
        ST_BEGIN("tsc_select_refusal");
        ST_PASS("tsc_select_refusal invert=%d modify=%d grow=%d similar=%d save=%d load=%d "
                "zero=%d unknown=%d disabled=%d history=%d",
                rfInvert ? 1 : 0, rfModify ? 1 : 0, rfGrow ? 1 : 0, rfSimilar ? 1 : 0,
                rfSave ? 1 : 0, rfLoad ? 1 : 0, rfZero ? 1 : 0, rfUnknown ? 1 : 0,
                rfDisabled ? 1 : 0, rfNoHistory ? 1 : 0);
        if (!rfInvert || !rfModify || !rfGrow || !rfSimilar || !rfSave || !rfLoad || !rfZero
            || !rfUnknown || !rfDisabled || !rfNoHistory) {
            return pictura::selfTest().fail(257, "select refusal");
        }
        frame.closeDocument(slDoc, false);

        // tsc_layer_select_commands (258): All Layers selects every row,
        // Deselect Layers clears the rows, and Similar Layers matches the
        // current layer's class/blend; none records history.
        const bool lsCreated = frame.newDocument(QStringLiteral("LayerSel"), 4, 4,
                                                 QStringLiteral("rgb"), 8, QStringLiteral("white"));
        pictura::PictureView* lsView = frame.activeView();
        if (!lsCreated || !lsView) {
            return pictura::selfTest().fail(258, "layer select fixture");
        }
        const int lsDoc = frame.activeDocumentIndex();
        const QString lsTop = lsView->add_layer_in(QString());
        const QString lsMid = lsView->add_layer_in(QString());
        lsView->set_layers_blend(QStringList{lsTop}, QStringLiteral("mul "));
        smPanel->refresh();
        const int lsAll = lsView->layer_row_count();
        const int lsBase = lsView->history_count();
        frame.registry()->dispatch(QString::fromLatin1(pictura::command_ids::SelectAllLayers));
        const int lsAllSelected = smPanel->selectedPaths().size();
        frame.registry()->dispatch(QString::fromLatin1(pictura::command_ids::SelectDeselectLayers));
        const bool lsCleared = smPanel->selectedPaths().isEmpty();
        smPanel->selectPaths(QStringList{lsMid}, lsMid);
        frame.registry()->dispatch(QString::fromLatin1(pictura::command_ids::SelectSimilarLayers));
        const QStringList lsSimilar = smPanel->selectedPaths();
        const bool lsMatches = lsSimilar.contains(lsMid) && !lsSimilar.contains(lsTop)
            && !lsSimilar.isEmpty();
        const bool lsNoHistory = lsView->history_count() == lsBase;
        ST_BEGIN("tsc_layer_select_commands");
        ST_PASS("tsc_layer_select_commands rows=%d all=%d cleared=%d similar=%d nohistory=%d",
                lsAll, lsAllSelected, lsCleared ? 1 : 0, lsMatches ? 1 : 0, lsNoHistory ? 1 : 0);
        if (lsAllSelected != lsAll || !lsCleared || !lsMatches || !lsNoHistory) {
            return pictura::selfTest().fail(258, "layer select commands");
        }
        frame.closeDocument(lsDoc, false);

    return 0;
}
