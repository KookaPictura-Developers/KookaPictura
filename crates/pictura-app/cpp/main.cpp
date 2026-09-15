#include <QtCore/QCoreApplication>
#include <QtCore/QDir>
#include <QtCore/QProcessEnvironment>
#include <QtCore/QSet>
#include <QtCore/QStringList>
#include <QtCore/QTemporaryDir>
#include <QtCore/QTimer>
#include <QtGui/QAction>
#include <QtGui/QIcon>
#include <QtGui/QImage>
#include <QtGui/QMouseEvent>
#include <QtGui/QPalette>
#include <QtWidgets/QApplication>
#include <QtWidgets/QDockWidget>
#include <QtWidgets/QLabel>
#include <QtWidgets/QToolBar>
#include <QtWidgets/QToolButton>

#include <cmath>
#include <cstdint>
#include <cstdio>
#include <optional>

#include "commands.h"
#include "dialogs.h"
#include "frame.h"
#include "icons.h"
#include "image_view.h"
#include "session.h"
#include "theme.h"
#include "toolbox.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include "interop.h"

int main(int argc, char* argv[])
{
    QApplication app(argc, argv);
    QApplication::setWindowIcon(pictura::icon(QStringLiteral("app")));

    // Surface Qt's own diagnostics (QRhi logs through qWarning) on stderr so
    // the interop probe can capture them under xvfb/offscreen.
    qInstallMessageHandler([](QtMsgType type, const QMessageLogContext&, const QString& message) {
        std::fprintf(stderr, "qt[%d]: %s\n", static_cast<int>(type), message.toLocal8Bit().constData());
        std::fflush(stderr);
    });

    const QStringList args = app.arguments();
    bool selfTest = false;
    bool interopProbe = false;
    QString psdPath;
    for (int i = 1; i < args.size(); ++i) {
        if (args.at(i) == QStringLiteral("--self-test")) {
            selfTest = true;
        } else if (args.at(i) == QStringLiteral("--interop-probe")) {
            interopProbe = true;
        } else if (!args.at(i).startsWith(QLatin1Char('-'))) {
            psdPath = args.at(i);
        }
    }

    // Self-test must not read the user's saved layout: isolate the session store
    // (session.cpp only reads XDG_STATE_HOME) before the frame restores state.
    std::optional<QTemporaryDir> selfTestStateDir;
    if (selfTest) {
        selfTestStateDir.emplace();
        if (selfTestStateDir->isValid()) {
            qputenv("XDG_STATE_HOME", selfTestStateDir->path().toUtf8());
        } else {
            selfTestStateDir.reset();
        }
    }

    if (interopProbe) {
        pictura::PictureView probe;
        const bool prepared = probe.gpu_interop_prepare();
        if (!prepared) {
            std::fprintf(stderr, "pictura interop-probe: no Vulkan device\n");
            return 0;
        }
        const std::int32_t result = pictura_try_qrhi_import(probe.gpu_vk_instance(),
                                                            probe.gpu_vk_physical_device(),
                                                            probe.gpu_vk_device(),
                                                            probe.gpu_vk_queue_family(),
                                                            probe.gpu_vk_image(),
                                                            probe.gpu_image_width(),
                                                            probe.gpu_image_height());
        std::fprintf(stderr, "pictura interop-probe: qrhi_import=%d\n", result);
        std::fflush(stderr);
        return 0;
    }

    pictura::PicturaMainWindow frame;

    bool codecLoaded = false;
    if (!psdPath.isEmpty()) {
        codecLoaded = frame.openPath(psdPath);
    }
    // M0.5: fall back to a scratch document with the offscreen GPU demo only
    // when no document loaded. 0 = no GPU, 1 = rendered non-blank, 2 = blank.
    int gpu = 0;
    if (!codecLoaded) {
        frame.newDocument(QStringLiteral("Untitled"), 512, 512, QStringLiteral("rgb"), 8,
                          QStringLiteral("white"));
        if (pictura::PictureView* scratch = frame.activeView()) {
            gpu = scratch->render_gpu();
            frame.refresh();
        }
    }
    pictura::PictureView* view = frame.activeView();
    const QImage image = view ? view->image() : QImage();

    frame.resize(1100, 700);
    frame.show();

    if (selfTest) {
        std::fprintf(stderr,
                     "pictura self-test: image=%dx%d codec_loaded=%d gpu=%d\n",
                     image.width(),
                     image.height(),
                     codecLoaded ? 1 : 0,
                     gpu);
        std::fflush(stderr);
        if (!view || image.isNull()) {
            std::fprintf(stderr, "pictura self-test: FAIL: null image\n");
            return 2;
        }
        if (gpu == 2) {
            std::fprintf(stderr, "pictura self-test: FAIL: GPU render was blank\n");
            return 4;
        }
        if (!codecLoaded) {
            // The generated image (GPU or CPU fallback) must not be blank.
            QSet<QRgb> seen;
            for (int gy = 0; gy < 8; ++gy) {
                for (int gx = 0; gx < 8; ++gx) {
                    const int x = image.width() * gx / 8 + image.width() / 16;
                    const int y = image.height() * gy / 8 + image.height() / 16;
                    seen.insert(image.pixel(x, y));
                }
            }
            std::fprintf(stderr,
                         "pictura self-test: nonblank=%d distinct=%d\n",
                         seen.size() >= 2 ? 1 : 0,
                         seen.size());
            std::fflush(stderr);
            if (seen.size() < 2) {
                std::fprintf(stderr, "pictura self-test: FAIL: blank render\n");
                return 5;
            }
        }
        if (codecLoaded) {
            // two_layers.psd is an 8x8 layer stack: a red top-left quadrant and
            // a blue bottom-right quadrant; the other quadrants are uncovered.
            QSet<QRgb> seen;
            for (int y = 0; y < image.height(); ++y) {
                for (int x = 0; x < image.width(); ++x) {
                    seen.insert(image.pixel(x, y));
                }
            }
            std::fprintf(stderr, "pictura self-test: layered distinct=%d\n", seen.size());
            std::fflush(stderr);
            if (seen.size() < 2) {
                std::fprintf(stderr, "pictura self-test: FAIL: blank composition\n");
                return 6;
            }
            if (image.width() >= 8 && image.height() >= 8) {
                const QRgb tl = image.pixel(2, 2);
                const QRgb br = image.pixel(6, 6);
                const QRgb tr = image.pixel(6, 2);
                const QRgb bl = image.pixel(2, 6);
                std::fprintf(stderr,
                             "pictura self-test: tl=(%d,%d,%d,a%d) br=(%d,%d,%d,a%d) "
                             "tr_a=%d bl_a=%d\n",
                             qRed(tl),
                             qGreen(tl),
                             qBlue(tl),
                             qAlpha(tl),
                             qRed(br),
                             qGreen(br),
                             qBlue(br),
                             qAlpha(br),
                             qAlpha(tr),
                             qAlpha(bl));
                std::fflush(stderr);
                const bool red = qRed(tl) > 200 && qGreen(tl) < 60 && qBlue(tl) < 60
                                 && qAlpha(tl) == 255;
                const bool blue = qBlue(br) > 200 && qRed(br) < 60 && qGreen(br) < 60
                                  && qAlpha(br) == 255;
                if (!red || !blue) {
                    std::fprintf(stderr, "pictura self-test: FAIL: composited quadrants wrong\n");
                    return 7;
                }
                // Uncovered quadrants must be transparent: this proves the layer
                // stack was composited, not the opaque embedded PSD composite.
                if (qAlpha(tr) != 0 || qAlpha(bl) != 0) {
                    std::fprintf(stderr, "pictura self-test: FAIL: layer stack not composited\n");
                    return 8;
                }
            }

            // M5-C2: a wand selection must confine an adjustment to the
            // selected quadrant. Wand the red top-left, Invert it, and require
            // the blue bottom-right to be untouched. Clean up afterwards so the
            // full-frame checks below see the original stack.
            const bool wand = view->magic_wand(2, 2, 10);
            const bool hasSelection = view->has_selection();
            const int selectedPx = view->selection_count();
            std::fprintf(stderr,
                         "pictura self-test: magic_wand=%d has_selection=%d selected_px=%d\n",
                         wand ? 1 : 0,
                         hasSelection ? 1 : 0,
                         selectedPx);
            std::fflush(stderr);
            if (!wand || !hasSelection || selectedPx <= 0
                || selectedPx >= image.width() * image.height()) {
                std::fprintf(stderr, "pictura self-test: FAIL: wand selection wrong\n");
                return 14;
            }
            const bool maskedAdded = view->add_adjustment(QStringLiteral("invert"));
            const QImage masked = view->image();
            const QRgb mtl = masked.pixel(2, 2);
            const QRgb mbr = masked.pixel(6, 6);
            std::fprintf(stderr,
                         "pictura self-test: masked_adjustment=%d tl=(%d,%d,%d,a%d) "
                         "br=(%d,%d,%d,a%d)\n",
                         maskedAdded ? 1 : 0,
                         qRed(mtl),
                         qGreen(mtl),
                         qBlue(mtl),
                         qAlpha(mtl),
                         qRed(mbr),
                         qGreen(mbr),
                         qBlue(mbr),
                         qAlpha(mbr));
            std::fflush(stderr);
            const bool maskedCyan = qRed(mtl) < 60 && qGreen(mtl) > 200 && qBlue(mtl) > 200;
            const bool maskedBlue = qBlue(mbr) > 200 && qRed(mbr) < 60 && qGreen(mbr) < 60;
            if (!maskedAdded || !maskedCyan || !maskedBlue) {
                std::fprintf(stderr, "pictura self-test: FAIL: masked adjustment not confined\n");
                return 15;
            }
            view->remove_layer(view->layer_count() - 1);
            view->deselect();
            if (view->has_selection() || view->selection_count() != 0) {
                std::fprintf(stderr, "pictura self-test: FAIL: deselect left a selection\n");
                return 16;
            }

            // M4-C: add an Invert adjustment layer over the stack and verify the
            // composite changed as expected (red -> cyan, blue -> yellow).
            const QImage beforeAdjust = view->image();
            const bool added = view->add_adjustment(QStringLiteral("invert"));
            const QImage adjusted = view->image();
            const int layerCount = view->layer_count();
            std::fprintf(stderr,
                         "pictura self-test: add_adjustment(invert)=%d layers=%d last_kind=%s\n",
                         added ? 1 : 0,
                         layerCount,
                         view->layer_kind(layerCount - 1).toLocal8Bit().constData());
            std::fflush(stderr);
            if (!added || layerCount != 3
                || view->layer_kind(layerCount - 1) != QStringLiteral("adjustment")) {
                std::fprintf(stderr, "pictura self-test: FAIL: invert adjustment not added\n");
                return 9;
            }
            const QRgb atl = adjusted.pixel(2, 2);
            const QRgb abr = adjusted.pixel(6, 6);
            std::fprintf(stderr,
                         "pictura self-test: adjusted tl=(%d,%d,%d,a%d) br=(%d,%d,%d,a%d)\n",
                         qRed(atl),
                         qGreen(atl),
                         qBlue(atl),
                         qAlpha(atl),
                         qRed(abr),
                         qGreen(abr),
                         qBlue(abr),
                         qAlpha(abr));
            std::fflush(stderr);
            const bool cyan = qRed(atl) < 60 && qGreen(atl) > 200 && qBlue(atl) > 200
                              && qAlpha(atl) == 255;
            const bool yellow = qRed(abr) > 200 && qGreen(abr) > 200 && qBlue(abr) < 60
                                && qAlpha(abr) == 255;
            if (!cyan || !yellow) {
                std::fprintf(stderr, "pictura self-test: FAIL: invert composite wrong\n");
                return 10;
            }
            if (beforeAdjust == adjusted) {
                std::fprintf(stderr, "pictura self-test: FAIL: adjustment did not change image\n");
                return 11;
            }

            // Toggle the bottom pixel layer's visibility: the output must change.
            if (!view->layer_visible(0)) {
                std::fprintf(stderr, "pictura self-test: FAIL: base layer not visible\n");
                return 12;
            }
            view->set_layer_visible(0, false);
            const QImage hidden = view->image();
            bool differs = false;
            for (int y = 0; y < hidden.height() && !differs; ++y) {
                for (int x = 0; x < hidden.width(); ++x) {
                    if (hidden.pixel(x, y) != adjusted.pixel(x, y)) {
                        differs = true;
                        break;
                    }
                }
            }
            std::fprintf(stderr, "pictura self-test: visibility_change=%d\n", differs ? 1 : 0);
            std::fflush(stderr);
            if (!differs || view->layer_visible(0)) {
                std::fprintf(stderr,
                             "pictura self-test: FAIL: visibility toggle did not change output\n");
                return 13;
            }

            // M6-C: a filter must confine its change to the active selection.
            // The topmost pixel layer is the bottom-right blue quadrant; wand
            // that quadrant, apply the fixed-seed Add Noise, and require the
            // selected quadrant to change while the rest is bit-identical.
            view->deselect();
            const bool filterWand = view->magic_wand(6, 6, 10);
            const bool filterSelected = view->has_selection();
            const int filterSelectedPx = view->selection_count();
            std::fprintf(stderr,
                         "pictura self-test: filter_wand=%d selected_px=%d\n",
                         filterWand ? 1 : 0,
                         filterSelectedPx);
            std::fflush(stderr);
            if (!filterWand || !filterSelected || filterSelectedPx <= 0
                || filterSelectedPx >= view->image().width() * view->image().height()) {
                std::fprintf(stderr, "pictura self-test: FAIL: filter selection wrong\n");
                return 17;
            }
            const QImage filterBefore = view->image();
            const bool filtered = view->apply_filter(QStringLiteral("add-noise"));
            const QImage filterAfter = view->image();
            int insideChanged = 0;
            int outsideChanged = 0;
            for (int y = 0; y < filterAfter.height(); ++y) {
                for (int x = 0; x < filterAfter.width(); ++x) {
                    const bool inside = x >= 4 && x < 8 && y >= 4 && y < 8;
                    if (filterAfter.pixel(x, y) == filterBefore.pixel(x, y)) {
                        continue;
                    }
                    if (inside) {
                        ++insideChanged;
                    } else {
                        ++outsideChanged;
                    }
                }
            }
            std::fprintf(stderr,
                         "pictura self-test: filter_change=%d changed_inside=%d "
                         "changed_outside=%d\n",
                         filtered ? 1 : 0,
                         insideChanged,
                         outsideChanged);
            std::fflush(stderr);
            if (!filtered || insideChanged == 0 || outsideChanged != 0) {
                std::fprintf(stderr, "pictura self-test: FAIL: filter not confined\n");
                return 18;
            }
            view->deselect();

            // M13: document ops. Earlier checks mutated the stack (hidden
            // layer, active invert, noise), so assert the exact 90 deg CW
            // remap (x,y) -> (7-y,x) on captured pixels, plus that a
            // successful op clears the selection.
            const QImage preRotate = view->image();
            view->select_all();
            const bool rotated = view->rotate_doc(1);
            const QImage rotatedImg = view->image();
            const QRgb rotTr = rotatedImg.pixel(5, 2);
            const QRgb rotBl = rotatedImg.pixel(1, 6);
            std::fprintf(stderr,
                         "pictura self-test: rotate_cw=%d size=%dx%d "
                         "map_tl=%d map_br=%d corner_a=%d sel=%d\n",
                         rotated ? 1 : 0,
                         rotatedImg.width(),
                         rotatedImg.height(),
                         rotTr == preRotate.pixel(2, 2) ? 1 : 0,
                         rotBl == preRotate.pixel(6, 6) ? 1 : 0,
                         qAlpha(rotatedImg.pixel(5, 6)),
                         view->selection_count());
            std::fflush(stderr);
            if (!rotated || rotatedImg.width() != 8 || rotatedImg.height() != 8
                || rotTr != preRotate.pixel(2, 2) || rotBl != preRotate.pixel(6, 6)
                || qAlpha(rotatedImg.pixel(5, 6)) != 0 || view->has_selection()
                || view->selection_count() != 0) {
                std::fprintf(stderr, "pictura self-test: FAIL: rotate cw wrong\n");
                return 19;
            }

            // M13: invalid document ops must be rejected and leave pixels put.
            const bool badRotate = view->rotate_doc(0);
            const bool badRotateClean = view->image() == rotatedImg;
            const bool badResize = view->resize_image(QStringLiteral("bicubic"), 0, 8);
            const bool badResizeClean = view->image() == rotatedImg;
            const bool badCanvas = view->resize_canvas(QStringLiteral("nope"), 10, 10);
            const bool badCanvasClean = view->image() == rotatedImg;
            std::fprintf(stderr,
                         "pictura self-test: reject rotate0=%d resize_w0=%d "
                         "canvas_bad_anchor=%d unchanged=%d\n",
                         badRotate ? 1 : 0,
                         badResize ? 1 : 0,
                         badCanvas ? 1 : 0,
                         badRotateClean && badResizeClean && badCanvasClean ? 1 : 0);
            std::fflush(stderr);
            if (badRotate || badResize || badCanvas || !badRotateClean || !badResizeClean
                || !badCanvasClean) {
                std::fprintf(stderr, "pictura self-test: FAIL: invalid doc op accepted\n");
                return 20;
            }

            // M13: CCW must undo CW bit-exactly, then growing the canvas to
            // 10x12 with the bottom-right anchor maps old (x,y) to (x+2,y+4)
            // and leaves the new top-left area transparent.
            const bool restoredOk = view->rotate_doc(3);
            const QImage restored = view->image();
            if (!restoredOk || restored != preRotate) {
                std::fprintf(stderr, "pictura self-test: FAIL: rotate ccw did not restore\n");
                return 21;
            }
            const bool grown = view->resize_canvas(QStringLiteral("bottom-right"), 10, 12);
            const QImage grownImg = view->image();
            std::fprintf(stderr,
                         "pictura self-test: canvas_grow=%d size=%dx%d "
                         "map_tl=%d map_br=%d corner_a=%d\n",
                         grown ? 1 : 0,
                         grownImg.width(),
                         grownImg.height(),
                         grownImg.pixel(4, 6) == restored.pixel(2, 2) ? 1 : 0,
                         grownImg.pixel(8, 10) == restored.pixel(6, 6) ? 1 : 0,
                         qAlpha(grownImg.pixel(0, 0)));
            std::fflush(stderr);
            if (!grown || grownImg.width() != 10 || grownImg.height() != 12
                || grownImg.pixel(4, 6) != restored.pixel(2, 2)
                || grownImg.pixel(8, 10) != restored.pixel(6, 6)
                || qAlpha(grownImg.pixel(0, 0)) != 0 || qAlpha(grownImg.pixel(1, 1)) != 0) {
                std::fprintf(stderr, "pictura self-test: FAIL: canvas growth wrong\n");
                return 21;
            }

            // M14: mutating ops capture history; undo/redo must round-trip the
            // pixels bit-exactly.
            const QImage preUndo = view->image();
            const int depthBefore = view->history_depth();
            const bool historyRotated = view->rotate_doc(1);
            const int depthAfterRotate = view->history_depth();
            const bool rotatedCanUndo = view->can_undo();
            const QImage postRotate = view->image();
            const bool undone = view->undo();
            const bool undoIdentical = view->image() == preUndo;
            const bool redone = view->redo();
            const bool redoIdentical = view->image() == postRotate;
            std::fprintf(stderr,
                         "pictura self-test: history rotate=%d depth=%d undo=%d "
                         "undo_ident=%d redo=%d redo_ident=%d\n",
                         historyRotated ? 1 : 0,
                         depthAfterRotate,
                         undone ? 1 : 0,
                         undoIdentical ? 1 : 0,
                         redone ? 1 : 0,
                         redoIdentical ? 1 : 0);
            std::fflush(stderr);
            if (!historyRotated || depthAfterRotate != depthBefore + 1
                || !rotatedCanUndo || postRotate.width() != 12
                || postRotate.height() != 10 || !undone || !undoIdentical
                || !redone || !redoIdentical) {
                std::fprintf(stderr, "pictura self-test: FAIL: undo/redo round-trip wrong\n");
                return 22;
            }

            // M14: a fresh op invalidates redo; reopening the fixture resets
            // history and an empty-stack undo fails without touching pixels.
            const bool flipped = view->flip_doc(true);
            const bool redoInvalidated = flipped && !view->can_redo();
            const bool reopened = view->open(psdPath);
            const QImage reopenedImg = view->image();
            const bool noUndoAfterOpen = !view->can_undo();
            const bool boundaryUndone = view->undo();
            const bool boundaryIdentical = view->image() == reopenedImg;
            const bool openReset = reopened && noUndoAfterOpen && boundaryIdentical;
            std::fprintf(stderr,
                         "pictura self-test: history redo_invalid=%d open_reset=%d "
                         "boundary_undo=%d\n",
                         redoInvalidated ? 1 : 0,
                         openReset ? 1 : 0,
                         boundaryUndone ? 1 : 0);
            std::fflush(stderr);
            if (!redoInvalidated || !openReset || boundaryUndone) {
                std::fprintf(stderr, "pictura self-test: FAIL: history invalidation wrong\n");
                return 23;
            }

            // M15: render filters. The topmost pixel layer is the blue
            // quadrant, so a full-frame apply must change pixels only inside
            // the blue rect (4,4)-(8,8); the fixed seed must make a re-apply
            // bit-identical.
            const QImage preClouds = view->image();
            const bool clouded = view->apply_filter(QStringLiteral("clouds"));
            const QImage cloudedImg = view->image();
            int cloudsInside = 0;
            int cloudsOutside = 0;
            for (int y = 0; y < cloudedImg.height(); ++y) {
                for (int x = 0; x < cloudedImg.width(); ++x) {
                    if (cloudedImg.pixel(x, y) == preClouds.pixel(x, y)) {
                        continue;
                    }
                    if (x >= 4 && x < 8 && y >= 4 && y < 8) {
                        ++cloudsInside;
                    } else {
                        ++cloudsOutside;
                    }
                }
            }
            const bool reapplied = view->apply_filter(QStringLiteral("clouds"));
            const bool reapplyIdentical = reapplied && view->image() == cloudedImg;
            const bool flared = view->apply_filter(QStringLiteral("lens-flare"));
            const QImage flaredImg = view->image();
            int flareOutsideChanged = 0;
            for (int y = 0; y < flaredImg.height(); ++y) {
                for (int x = 0; x < flaredImg.width(); ++x) {
                    if (!(x >= 4 && x < 8 && y >= 4 && y < 8)
                        && flaredImg.pixel(x, y) != cloudedImg.pixel(x, y)) {
                        ++flareOutsideChanged;
                    }
                }
            }
            std::fprintf(stderr,
                         "pictura self-test: clouds=%d confined=%d inside=%d "
                         "reapply_ident=%d flare=%d\n",
                         clouded ? 1 : 0,
                         cloudsOutside == 0 && flareOutsideChanged == 0 ? 1 : 0,
                         cloudsInside,
                         reapplyIdentical ? 1 : 0,
                         flared && flaredImg != cloudedImg ? 1 : 0);
            std::fflush(stderr);
            if (!clouded || cloudedImg == preClouds || cloudsInside == 0
                || cloudsOutside != 0 || !reapplyIdentical || !flared
                || flaredImg == cloudedImg || flareOutsideChanged != 0) {
                std::fprintf(stderr, "pictura self-test: FAIL: clouds/flare wrong\n");
                return 24;
            }

            // M16: the frame exposes the ten documented menus in order.
            const QStringList expectedMenus = {QStringLiteral("File"),
                                               QStringLiteral("Edit"),
                                               QStringLiteral("Image"),
                                               QStringLiteral("Layer"),
                                               QStringLiteral("Type"),
                                               QStringLiteral("Select"),
                                               QStringLiteral("Filter"),
                                               QStringLiteral("View"),
                                               QStringLiteral("Window"),
                                               QStringLiteral("Help")};
            const QStringList actualMenus = frame.topLevelMenuTitles();
            std::fprintf(stderr,
                         "pictura self-test: menus=%d first=%s last=%s\n",
                         actualMenus.size(),
                         actualMenus.isEmpty() ? "-" : actualMenus.first().toLocal8Bit().constData(),
                         actualMenus.isEmpty() ? "-" : actualMenus.last().toLocal8Bit().constData());
            std::fflush(stderr);
            if (actualMenus != expectedMenus) {
                std::fprintf(stderr, "pictura self-test: FAIL: menu bar wrong\n");
                return 25;
            }

            // M16: dispatch a registered command and prove an unknown id is inert.
            pictura::CommandRegistry* registry = frame.registry();
            const bool dispatched =
                registry->dispatch(QString::fromLatin1(pictura::command_ids::SelectAll));
            const bool selected = view->has_selection();
            const bool unknownInert = !registry->dispatch(QStringLiteral("no.such.command"));
            view->deselect();
            std::fprintf(stderr,
                         "pictura self-test: dispatch=%d selected=%d unknown_inert=%d\n",
                         dispatched ? 1 : 0,
                         selected ? 1 : 0,
                         unknownInert ? 1 : 0);
            std::fflush(stderr);
            if (!dispatched || !selected || !unknownInert) {
                std::fprintf(stderr, "pictura self-test: FAIL: command dispatch wrong\n");
                return 26;
            }

            // M16: document-required commands disable with no document, while
            // File > Open stays enabled.
            view->open(QStringLiteral("/nonexistent-kooka-pictura.psd"));
            registry->refresh();
            QAction* rotateAction =
                registry->action(QString::fromLatin1(pictura::command_ids::ImageRotate90Cw));
            QAction* openAction =
                registry->action(QString::fromLatin1(pictura::command_ids::FileOpen));
            const bool docDisabled = rotateAction && !rotateAction->isEnabled();
            const bool openEnabled = openAction && openAction->isEnabled();
            view->open(psdPath);
            registry->refresh();
            std::fprintf(stderr,
                         "pictura self-test: no_doc_disable=%d open_enable=%d\n",
                         docDisabled ? 1 : 0,
                         openEnabled ? 1 : 0);
            std::fflush(stderr);
            if (!docDisabled || !openEnabled) {
                std::fprintf(stderr, "pictura self-test: FAIL: command enablement wrong\n");
                return 27;
            }

            // M16: brightness levels apply and differ (theme is the source of truth).
            frame.setBrightnessLevel(0);
            const QColor darkWindow = qApp->palette().color(QPalette::Window);
            frame.setBrightnessLevel(3);
            const QColor lightWindow = qApp->palette().color(QPalette::Window);
            const bool brightOk =
                frame.brightnessLevel() == 3 && darkWindow != lightWindow;
            frame.setBrightnessLevel(1);
            std::fprintf(stderr,
                         "pictura self-test: brightness ok=%d dark=%s light=%s\n",
                         brightOk ? 1 : 0,
                         darkWindow.name().toLocal8Bit().constData(),
                         lightWindow.name().toLocal8Bit().constData());
            std::fflush(stderr);
            if (!brightOk) {
                std::fprintf(stderr, "pictura self-test: FAIL: brightness wrong\n");
                return 28;
            }

            // M16: screen modes cycle forward and backward.
            using ScreenMode = pictura::PicturaMainWindow::ScreenMode;
            frame.setScreenMode(ScreenMode::Standard);
            frame.cycleScreenMode(true);
            const ScreenMode mode1 = frame.screenMode();
            frame.cycleScreenMode(true);
            const ScreenMode mode2 = frame.screenMode();
            frame.cycleScreenMode(false);
            const ScreenMode mode3 = frame.screenMode();
            const bool modesOk = mode1 == ScreenMode::FullWithMenuBar
                                 && mode2 == ScreenMode::Full
                                 && mode3 == ScreenMode::FullWithMenuBar;
            frame.setScreenMode(ScreenMode::Standard);
            std::fprintf(stderr, "pictura self-test: screen_modes=%d\n", modesOk ? 1 : 0);
            std::fflush(stderr);
            if (!modesOk) {
                std::fprintf(stderr, "pictura self-test: FAIL: screen mode cycle wrong\n");
                return 29;
            }

            // M16: layout + brightness persist atomically under a temp XDG state dir.
            const QString tmpState = QDir::tempPath()
                                     + QStringLiteral("/kooka-pictura-selftest-")
                                     + QString::number(QCoreApplication::applicationPid());
            qputenv("XDG_STATE_HOME", tmpState.toUtf8());
            frame.setBrightnessLevel(2);
            frame.saveSession();
            const pictura::SessionState loaded = pictura::loadSession();
            const bool sessionOk = loaded.brightnessLevel == 2
                                   && loaded.layout == frame.saveState();
            std::fprintf(stderr,
                         "pictura self-test: session_ok=%d bytes=%d level=%d\n",
                         sessionOk ? 1 : 0,
                         loaded.layout.size(),
                         loaded.brightnessLevel);
            std::fflush(stderr);
            if (!sessionOk) {
                std::fprintf(stderr, "pictura self-test: FAIL: session round-trip wrong\n");
                return 30;
            }

            // M16: duplicate panel objectNames are rejected.
            auto* duplicate = new QDockWidget(QStringLiteral("Duplicate"), &frame);
            duplicate->setObjectName(QStringLiteral("layersPanel"));
            const bool duplicateRejected = !frame.registerPanel(duplicate, Qt::LeftDockWidgetArea);
            delete duplicate;
            std::fprintf(stderr, "pictura self-test: dup_panel_rejected=%d\n", duplicateRejected ? 1 : 0);
            std::fflush(stderr);
            if (!duplicateRejected) {
                std::fprintf(stderr, "pictura self-test: FAIL: duplicate panel accepted\n");
                return 31;
            }

            // M16: Tab hides and restores all panels.
            auto* layersPanel = frame.findChild<QDockWidget*>(QStringLiteral("layersPanel"));
            frame.setPanelsHidden(true);
            const bool panelsHidden = layersPanel && !layersPanel->isVisible();
            frame.setPanelsHidden(false);
            const bool panelsShown = layersPanel && layersPanel->isVisible();
            std::fprintf(stderr,
                         "pictura self-test: hide_all=%d restore=%d\n",
                         panelsHidden ? 1 : 0,
                         panelsShown ? 1 : 0);
            std::fflush(stderr);
            if (!panelsHidden || !panelsShown) {
                std::fprintf(stderr, "pictura self-test: FAIL: hide-all wrong\n");
                return 32;
            }

            // M17: New creates an untitled document with the requested size and
            // a white background.
            const int docsBeforeNew = frame.documentCount();
            const bool created = frame.newDocument(QStringLiteral("Scratch"), 4, 3,
                                                   QStringLiteral("rgb"), 8,
                                                   QStringLiteral("white"));
            pictura::PictureView* fresh = frame.activeView();
            const bool freshOk = created && frame.documentCount() == docsBeforeNew + 1
                                 && fresh && fresh->has_document()
                                 && fresh->image().width() == 4 && fresh->image().height() == 3;
            const QRgb whitePixel = fresh ? fresh->image().pixel(1, 1) : 0;
            const bool whiteOk = qRed(whitePixel) == 255 && qGreen(whitePixel) == 255
                                 && qBlue(whitePixel) == 255 && qAlpha(whitePixel) == 255;
            std::fprintf(stderr, "pictura self-test: new_doc=%d white=%d\n",
                         freshOk ? 1 : 0, whiteOk ? 1 : 0);
            std::fflush(stderr);
            if (!freshOk || !whiteOk) {
                std::fprintf(stderr, "pictura self-test: FAIL: new document wrong\n");
                return 33;
            }

            // M17: Save As then open round-trips the new document's pixels.
            const QString savePath =
                QDir::tempPath() + QStringLiteral("/kooka-pictura-m17-roundtrip.psd");
            const QImage scratchImg = fresh->image();
            const bool saved = frame.saveActiveAs(savePath);
            const bool reopenedM17 = frame.openPath(savePath);
            pictura::PictureView* reloaded = frame.activeView();
            const bool roundtrip = saved && reopenedM17 && reloaded && reloaded->has_document()
                                   && reloaded->image() == scratchImg;
            std::fprintf(stderr, "pictura self-test: roundtrip=%d\n", roundtrip ? 1 : 0);
            std::fflush(stderr);
            if (!roundtrip) {
                std::fprintf(stderr, "pictura self-test: FAIL: save/open round-trip wrong\n");
                return 34;
            }

            // M17: a mutating command sets dirty; save clears it.
            const bool cleanAfterOpen = !reloaded->is_dirty();
            reloaded->select_all();
            const bool dirtyAfterMutate = reloaded->is_dirty();
            const bool resaved = frame.saveActive();
            const bool cleanAfterSave = !reloaded->is_dirty();
            std::fprintf(stderr, "pictura self-test: dirty clean=%d set=%d resave=%d cleared=%d\n",
                         cleanAfterOpen ? 1 : 0, dirtyAfterMutate ? 1 : 0,
                         resaved ? 1 : 0, cleanAfterSave ? 1 : 0);
            std::fflush(stderr);
            if (!cleanAfterOpen || !dirtyAfterMutate || !resaved || !cleanAfterSave) {
                std::fprintf(stderr, "pictura self-test: FAIL: dirty state wrong\n");
                return 35;
            }

            // M17: multiple documents become multiple tabs; switching targets.
            const int totalDocs = frame.documentCount();
            frame.setActiveDocumentIndex(0);
            const bool firstActive = frame.activeDocumentIndex() == 0
                                     && frame.imageView() == frame.canvasAt(0);
            frame.setActiveDocumentIndex(totalDocs - 1);
            const bool lastActive = frame.activeDocumentIndex() == totalDocs - 1;
            std::fprintf(stderr, "pictura self-test: tabs=%d first=%d last=%d\n",
                         totalDocs, firstActive ? 1 : 0, lastActive ? 1 : 0);
            std::fflush(stderr);
            if (totalDocs < 2 || !firstActive || !lastActive) {
                std::fprintf(stderr, "pictura self-test: FAIL: document tabs wrong\n");
                return 36;
            }

            // M17: closing a modified document honours the prompt: Cancel keeps
            // it, Discard closes it.
            pictura::setUnsavedPromptInteractive(false);
            pictura::setNonInteractiveUnsavedChoice(pictura::UnsavedChoice::Cancel);
            frame.setActiveDocumentIndex(frame.documentCount() - 1);
            frame.activeView()->select_all();
            const int beforeClose = frame.documentCount();
            const bool cancelled = !frame.closeDocument(frame.activeDocumentIndex(), true)
                                   && frame.documentCount() == beforeClose;
            pictura::setNonInteractiveUnsavedChoice(pictura::UnsavedChoice::Discard);
            const bool discarded = frame.closeDocument(frame.activeDocumentIndex(), true)
                                   && frame.documentCount() == beforeClose - 1;
            pictura::setUnsavedPromptInteractive(true);
            std::fprintf(stderr, "pictura self-test: close_cancel=%d close_discard=%d\n",
                         cancelled ? 1 : 0, discarded ? 1 : 0);
            std::fflush(stderr);
            if (!cancelled || !discarded) {
                std::fprintf(stderr, "pictura self-test: FAIL: unsaved close prompt wrong\n");
                return 37;
            }

            // M18: the frame's active tool round-trips through the controller.
            frame.setActiveTool(pictura::ToolId::Marquee);
            const bool marqueeOn = frame.activeTool() == pictura::ToolId::Marquee;
            frame.setActiveTool(pictura::ToolId::Lasso);
            const bool lassoOn = frame.activeTool() == pictura::ToolId::Lasso;
            frame.setActiveTool(pictura::ToolId::Eyedropper);
            const bool eyedropOn = frame.activeTool() == pictura::ToolId::Eyedropper;
            frame.setActiveTool(pictura::ToolId::Move);
            const bool moveOn = frame.activeTool() == pictura::ToolId::Move;
            std::fprintf(stderr,
                         "pictura self-test: tool_switch marquee=%d lasso=%d eyedropper=%d "
                         "move=%d active=%s\n",
                         marqueeOn ? 1 : 0,
                         lassoOn ? 1 : 0,
                         eyedropOn ? 1 : 0,
                         moveOn ? 1 : 0,
                         pictura::toolInfo(frame.activeTool()).label);
            std::fflush(stderr);
            if (!marqueeOn || !lassoOn || !eyedropOn || !moveOn) {
                std::fprintf(stderr, "pictura self-test: FAIL: tool switch wrong\n");
                return 39;
            }

            // M18: marquee rect/ellipse counts and pixel membership. The 4x4
            // ellipse rasterizes 12 px (pictura-select ellipse test).
            frame.newDocument(QStringLiteral("ToolTest"), 8, 8, QStringLiteral("rgb"), 8,
                              QStringLiteral("white"));
            pictura::PictureView* toolView = frame.activeView();
            const bool rectSel =
                toolView && toolView->select_rect(0, 0, 4, 4, QStringLiteral("new"));
            const int rectPx = toolView ? toolView->selection_count() : -1;
            const bool ellipseSel =
                toolView && toolView->select_ellipse(0, 0, 4, 4, QStringLiteral("new"));
            const int ellipsePx = toolView ? toolView->selection_count() : -1;
            if (toolView) {
                toolView->select_ellipse(0, 0, 4, 4, QStringLiteral("add"));
            }
            const bool addIdempotent = toolView && toolView->selection_count() == ellipsePx;
            if (toolView) {
                toolView->select_ellipse(0, 0, 4, 4, QStringLiteral("new"));
                toolView->select_rect(2, 2, 1, 1, QStringLiteral("intersect"));
            }
            const bool centreInside = toolView && toolView->selection_count() == 1;
            if (toolView) {
                toolView->select_ellipse(0, 0, 4, 4, QStringLiteral("new"));
                toolView->select_rect(0, 0, 1, 1, QStringLiteral("intersect"));
            }
            const bool cornerOutside = toolView && toolView->selection_count() == 0;
            std::fprintf(stderr,
                         "pictura self-test: marquee rect=%d(%d) ellipse=%d(%d) add_same=%d "
                         "centre=%d corner=%d\n",
                         rectSel ? 1 : 0,
                         rectPx,
                         ellipseSel ? 1 : 0,
                         ellipsePx,
                         addIdempotent ? 1 : 0,
                         centreInside ? 1 : 0,
                         cornerOutside ? 1 : 0);
            std::fflush(stderr);
            if (!rectSel || rectPx != 16 || !ellipseSel || ellipsePx != 12 || !addIdempotent
                || !centreInside || !cornerOutside) {
                std::fprintf(stderr, "pictura self-test: FAIL: marquee selection wrong\n");
                return 40;
            }

            // M18: combine modes union/subtract/intersect on the 8x8 canvas.
            if (toolView) {
                toolView->select_rect(0, 0, 4, 4, QStringLiteral("new"));
            }
            const int newPx = toolView ? toolView->selection_count() : -1;
            if (toolView) {
                toolView->select_rect(2, 2, 4, 4, QStringLiteral("add"));
            }
            const int addPx = toolView ? toolView->selection_count() : -1;
            if (toolView) {
                toolView->select_rect(0, 0, 4, 4, QStringLiteral("new"));
                toolView->select_rect(1, 1, 2, 2, QStringLiteral("subtract"));
            }
            const int subPx = toolView ? toolView->selection_count() : -1;
            if (toolView) {
                toolView->select_rect(0, 0, 4, 4, QStringLiteral("new"));
                toolView->select_rect(2, 2, 4, 4, QStringLiteral("intersect"));
            }
            const int interPx = toolView ? toolView->selection_count() : -1;
            std::fprintf(stderr,
                         "pictura self-test: combine new=%d add=%d subtract=%d intersect=%d\n",
                         newPx,
                         addPx,
                         subPx,
                         interPx);
            std::fflush(stderr);
            if (newPx != 16 || addPx != 28 || subPx != 12 || interPx != 4) {
                std::fprintf(stderr, "pictura self-test: FAIL: combine modes wrong\n");
                return 41;
            }

            // M18: lasso fills a 5x5 square, and a two-point path is rejected
            // without disturbing the existing selection.
            const bool lassoStarted =
                toolView && toolView->begin_lasso(QStringLiteral("new"));
            if (toolView) {
                toolView->lasso_add_point(1, 1);
                toolView->lasso_add_point(6, 1);
                toolView->lasso_add_point(6, 6);
                toolView->lasso_add_point(1, 6);
            }
            const bool lassoEnded = toolView && toolView->end_lasso();
            const int lassoPx = toolView ? toolView->selection_count() : -1;
            const bool shortStarted =
                toolView && toolView->begin_lasso(QStringLiteral("new"));
            if (toolView) {
                toolView->lasso_add_point(1, 1);
                toolView->lasso_add_point(6, 1);
            }
            const bool shortEnded = toolView && toolView->end_lasso();
            const bool shortUnchanged = toolView && toolView->selection_count() == lassoPx;
            std::fprintf(stderr,
                         "pictura self-test: lasso start=%d end=%d px=%d short_end=%d "
                         "unchanged=%d\n",
                         lassoStarted ? 1 : 0,
                         lassoEnded ? 1 : 0,
                         lassoPx,
                         shortEnded ? 1 : 0,
                         shortUnchanged ? 1 : 0);
            std::fflush(stderr);
            if (!lassoStarted || !lassoEnded || lassoPx != 25 || !shortStarted || shortEnded
                || !shortUnchanged) {
                std::fprintf(stderr, "pictura self-test: FAIL: lasso selection wrong\n");
                return 42;
            }

            // M18: quick selection wands a white region and leaves a selection.
            if (toolView) {
                toolView->deselect();
            }
            const bool quickOk =
                toolView && toolView->quick_select(4, 4, 32, QStringLiteral("new"));
            const bool quickSelected = toolView && toolView->has_selection();
            const int quickPx = toolView ? toolView->selection_count() : -1;
            std::fprintf(stderr, "pictura self-test: quick_select=%d has=%d px=%d\n",
                         quickOk ? 1 : 0, quickSelected ? 1 : 0, quickPx);
            std::fflush(stderr);
            if (!quickOk || !quickSelected || quickPx <= 0) {
                std::fprintf(stderr, "pictura self-test: FAIL: quick selection wrong\n");
                return 43;
            }

            // M18: crop the fixture to its blue bottom-right quadrant.
            frame.openPath(psdPath);
            pictura::PictureView* cropView = frame.activeView();
            const bool cropOk = cropView && cropView->crop(4, 4, 4, 4);
            const QImage cropImg = cropView ? cropView->image() : QImage();
            const QRgb cropPx = cropImg.isNull() ? 0 : cropImg.pixel(1, 1);
            const bool cropBlue = qBlue(cropPx) > 200 && qRed(cropPx) < 60 && qGreen(cropPx) < 60;
            std::fprintf(stderr, "pictura self-test: crop=%d size=%dx%d blue=%d\n",
                         cropOk ? 1 : 0, cropImg.width(), cropImg.height(), cropBlue ? 1 : 0);
            std::fflush(stderr);
            if (!cropOk || cropImg.width() != 4 || cropImg.height() != 4 || !cropBlue) {
                std::fprintf(stderr, "pictura self-test: FAIL: crop wrong\n");
                return 44;
            }

            // M18: move the topmost (blue) layer over the red quadrant.
            frame.openPath(psdPath);
            pictura::PictureView* moveView = frame.activeView();
            const bool moved = moveView && moveView->translate_layer(-4, -4);
            const QImage moveImg = moveView ? moveView->image() : QImage();
            const QRgb movePx = moveImg.isNull() ? 0 : moveImg.pixel(2, 2);
            const bool moveBlue = qBlue(movePx) > 200 && qRed(movePx) < 60 && qGreen(movePx) < 60;
            std::fprintf(stderr, "pictura self-test: translate=%d blue=%d\n",
                         moved ? 1 : 0, moveBlue ? 1 : 0);
            std::fflush(stderr);
            if (!moved || !moveBlue) {
                std::fprintf(stderr, "pictura self-test: FAIL: layer move wrong\n");
                return 45;
            }

            // M18: eyedropper samples the composited red and rejects out-of-bounds.
            frame.openPath(psdPath);
            pictura::PictureView* eyeView = frame.activeView();
            const quint32 redSample = eyeView ? eyeView->sample_argb(2, 2) : 0u;
            const quint32 outSample = eyeView ? eyeView->sample_argb(-1, -1) : 1u;
            if (eyeView) {
                eyeView->deselect();
            }
            const bool eyeDeselected = eyeView && !eyeView->has_selection();
            std::fprintf(stderr,
                         "pictura self-test: eyedropper red=%08x out=%08x deselected=%d\n",
                         static_cast<unsigned>(redSample),
                         static_cast<unsigned>(outSample),
                         eyeDeselected ? 1 : 0);
            std::fflush(stderr);
            if (redSample != 0xFFFF0000u || outSample != 0u || !eyeDeselected) {
                std::fprintf(stderr, "pictura self-test: FAIL: eyedropper wrong\n");
                return 46;
            }

            // M20: layer property getters/setters round-trip and mark dirty.
            frame.openPath(psdPath);
            pictura::PictureView* propView = frame.activeView();
            const int m20Count = propView ? propView->layer_count() : -1;
            const int m20BaseOpacity = propView ? propView->layer_opacity(0) : -1;
            const bool renameOk =
                propView && propView->set_layer_name(0, QStringLiteral("Renamed"));
            const bool renamedOk =
                propView && propView->layer_name(0) == QStringLiteral("Renamed");
            const bool dirtyAfterRename = propView && propView->is_dirty();
            const bool blendOk =
                propView && propView->set_layer_blend(0, QStringLiteral("mul "));
            const bool blendRoundTrip =
                propView && propView->layer_blend(0) == QStringLiteral("mul ");
            const bool badBlendRejected =
                propView && !propView->set_layer_blend(0, QStringLiteral("zzzz"))
                && propView->layer_blend(0) == QStringLiteral("mul ");
            const bool opacityOk = propView && propView->set_layer_opacity(0, 128);
            const int roundTripOpacity = propView ? propView->layer_opacity(0) : -1;
            const bool clampOk = propView && propView->set_layer_opacity(0, 9999);
            const int clampedOpacity = propView ? propView->layer_opacity(0) : -1;
            std::fprintf(stderr,
                         "pictura self-test: m20_layer count=%d name=%d blend=%d badblend=%d "
                         "opacity=%d dirty=%d\n",
                         m20Count,
                         renameOk && renamedOk ? 1 : 0,
                         blendOk && blendRoundTrip ? 1 : 0,
                         badBlendRejected ? 1 : 0,
                         roundTripOpacity,
                         dirtyAfterRename ? 1 : 0);
            std::fflush(stderr);
            if (m20Count != 2 || !renameOk || !renamedOk || !dirtyAfterRename || !blendOk
                || !blendRoundTrip || !badBlendRejected || !opacityOk || roundTripOpacity != 128
                || !clampOk || clampedOpacity != 255) {
                std::fprintf(stderr, "pictura self-test: FAIL: M20 layer properties wrong\n");
                return 50;
            }

            // M20: labeled history, jump restore, and named snapshots.
            const int histBefore = propView ? propView->history_count() : -1;
            const bool histOpenLabel =
                propView && histBefore >= 1 && !propView->history_label(0).isEmpty();
            const bool histMutated = propView && propView->set_layer_opacity(0, 64);
            const int histAfter = propView ? propView->history_count() : -1;
            const bool histGrew = histAfter == histBefore + 1;
            const bool histTopLabel =
                propView && histAfter >= 1 && !propView->history_label(histAfter - 1).isEmpty();
            const bool histAtTop = propView && propView->history_index() == histAfter - 1;
            const bool histJump = propView && propView->history_jump(0);
            const bool histAtZero = propView && propView->history_index() == 0;
            const int restoredOpacity = propView ? propView->layer_opacity(0) : -1;
            const bool opacityRestored = restoredOpacity == m20BaseOpacity;
            const bool histJumpBack = propView && propView->history_jump(histAfter - 1);
            const bool snapAdded =
                propView && propView->history_add_snapshot(QStringLiteral("Checkpoint"));
            const int snapCount = propView ? propView->history_snapshot_count() : -1;
            const bool snapLabelOk =
                propView && propView->history_snapshot_label(0) == QStringLiteral("Checkpoint");
            const bool snapRestored = propView && propView->history_restore_snapshot(0);
            std::fprintf(stderr,
                         "pictura self-test: m20_history states=%d open_label=%d grew=%d "
                         "jump=%d snapshot=%d\n",
                         histBefore,
                         histOpenLabel ? 1 : 0,
                         histGrew ? 1 : 0,
                         histJump && histAtZero && opacityRestored && histJumpBack ? 1 : 0,
                         snapAdded && snapCount >= 1 && snapLabelOk && snapRestored ? 1 : 0);
            std::fflush(stderr);
            if (!histOpenLabel || !histMutated || !histGrew || !histTopLabel || !histAtTop
                || !histJump || !histAtZero || !opacityRestored || !histJumpBack || !snapAdded
                || snapCount < 1 || !snapLabelOk || !snapRestored) {
                std::fprintf(stderr, "pictura self-test: FAIL: M20 history wrong\n");
                return 51;
            }
        }

        // M19: every documented asset id resolves from the Qt resource
        // bundle; unknown ids must be inert rather than crash. Assets are
        // document-independent, so these run with or without a loaded PSD.
        const QStringList expectedIcons = {
            QStringLiteral("app"),
            QStringLiteral("tool.move"),
            QStringLiteral("tool.marquee"),
            QStringLiteral("tool.lasso"),
            QStringLiteral("tool.quickselection"),
            QStringLiteral("tool.crop"),
            QStringLiteral("tool.eyedropper"),
            QStringLiteral("tool.hand"),
            QStringLiteral("tool.zoom"),
            QStringLiteral("file.new"),
            QStringLiteral("file.open"),
            QStringLiteral("file.save"),
            QStringLiteral("file.saveAs"),
            QStringLiteral("file.revert"),
            QStringLiteral("file.close"),
            QStringLiteral("file.closeAll"),
            QStringLiteral("file.exit"),
            QStringLiteral("edit.undo"),
            QStringLiteral("edit.redo"),
            QStringLiteral("edit.stepForward"),
            QStringLiteral("edit.stepBackward"),
            QStringLiteral("image.rotate90cw"),
            QStringLiteral("image.rotate90ccw"),
            QStringLiteral("image.rotate180"),
            QStringLiteral("image.flipHorizontal"),
            QStringLiteral("image.flipVertical"),
            QStringLiteral("image.crop"),
            QStringLiteral("select.all"),
            QStringLiteral("select.deselect"),
            QStringLiteral("view.zoomIn"),
            QStringLiteral("view.zoomOut"),
            QStringLiteral("view.fitOnScreen"),
            QStringLiteral("view.actualPixels"),
            QStringLiteral("view.screenMode.standard"),
            QStringLiteral("view.screenMode.fullWithMenuBar"),
            QStringLiteral("view.screenMode.full"),
            QStringLiteral("view.options"),
            QStringLiteral("window.panels.layers"),
            QStringLiteral("window.panels.tools"),
            QStringLiteral("help.about"),
        };
        int iconsResolved = 0;
        QString missingIcon;
        for (const QString& id : expectedIcons) {
            if (pictura::icon(id).isNull()) {
                if (missingIcon.isEmpty()) {
                    missingIcon = id;
                }
            } else {
                ++iconsResolved;
            }
        }
        const bool unknownIconNull = pictura::icon(QStringLiteral("no.such.icon")).isNull();
        std::fprintf(stderr,
                     "pictura self-test: icons=%d unknown_null=%d\n",
                     iconsResolved,
                     unknownIconNull ? 1 : 0);
        std::fflush(stderr);
        if (iconsResolved != expectedIcons.size() || !unknownIconNull) {
            std::fprintf(stderr,
                         "pictura self-test: FAIL: icon missing=%s unknown_null=%d\n",
                         missingIcon.toLocal8Bit().constData(),
                         unknownIconNull ? 1 : 0);
            return 47;
        }

        // M19: the eight tool cursors must render; an unknown cursor id must
        // collapse to the default cursor without crashing.
        const QStringList expectedCursors = {
            QStringLiteral("tool.move"),
            QStringLiteral("tool.marquee"),
            QStringLiteral("tool.lasso"),
            QStringLiteral("tool.quickselection"),
            QStringLiteral("tool.crop"),
            QStringLiteral("tool.eyedropper"),
            QStringLiteral("tool.hand"),
            QStringLiteral("tool.zoom"),
        };
        int cursorsResolved = 0;
        QString missingCursor;
        for (const QString& id : expectedCursors) {
            if (pictura::cursor(id).pixmap().isNull()) {
                if (missingCursor.isEmpty()) {
                    missingCursor = id;
                }
            } else {
                ++cursorsResolved;
            }
        }
        pictura::cursor(QStringLiteral("no.such.cursor"));
        std::fprintf(stderr, "pictura self-test: cursors=%d\n", cursorsResolved);
        std::fflush(stderr);
        if (cursorsResolved != expectedCursors.size()) {
            std::fprintf(stderr,
                         "pictura self-test: FAIL: cursor missing=%s\n",
                         missingCursor.toLocal8Bit().constData());
            return 48;
        }

        // M19: the window icon must be set from the app asset.
        const bool windowIconSet = !QApplication::windowIcon().isNull();
        std::fprintf(stderr, "pictura self-test: window_icon=%d\n", windowIconSet ? 1 : 0);
        std::fflush(stderr);
        if (!windowIconSet) {
            std::fprintf(stderr, "pictura self-test: FAIL: window icon not set\n");
            return 49;
        }

        // M20: the seven panel docks are registered and their menu actions
        // toggle them on and off.
        const QStringList expectedPanelDocks = {
            QStringLiteral("layersPanel"),
            QStringLiteral("historyPanel"),
            QStringLiteral("navigatorPanel"),
            QStringLiteral("colorPanel"),
            QStringLiteral("swatchesPanel"),
            QStringLiteral("infoPanel"),
            QStringLiteral("histogramPanel"),
        };
        const QSet<QString>& registeredPanels = frame.panelObjectNames();
        int panelsRegistered = 0;
        for (const QString& name : expectedPanelDocks) {
            if (registeredPanels.contains(name)) {
                ++panelsRegistered;
            }
        }

        const QStringList panelCommands = {
            QStringLiteral("window.panels.layers"),
            QStringLiteral("window.panels.navigator"),
            QStringLiteral("window.panels.history"),
            QStringLiteral("window.panels.color"),
            QStringLiteral("window.panels.swatches"),
            QStringLiteral("window.panels.info"),
            QStringLiteral("window.panels.histogram"),
        };
        int panelsToggled = 0;
        for (const QString& id : panelCommands) {
            QAction* action = frame.registry()->action(id);
            const QString objectName =
                id.section(QLatin1Char('.'), -1) + QStringLiteral("Panel");
            QDockWidget* dock = frame.findChild<QDockWidget*>(objectName);
            if (!action || !dock) {
                continue;
            }
            const bool before = dock->isVisible();
            action->setChecked(!before);
            frame.registry()->dispatch(id);
            const bool toggled = dock->isVisible() != before;
            action->setChecked(!dock->isVisible());
            frame.registry()->dispatch(id);
            const bool restored = dock->isVisible() == before;
            if (toggled && restored) {
                ++panelsToggled;
            }
        }
        std::fprintf(stderr,
                     "pictura self-test: m20_panels registered=%d toggled=%d\n",
                     panelsRegistered,
                     panelsToggled);
        std::fflush(stderr);
        if (panelsRegistered != expectedPanelDocks.size()
            || panelsToggled != panelCommands.size()) {
            std::fprintf(stderr, "pictura self-test: FAIL: M20 panels wrong\n");
            return 52;
        }

        // M21: painting. A fresh transparent document keeps these checks
        // independent of any loaded PSD; every behaviour is read from pixels.
        const bool paintDoc = frame.newDocument(QStringLiteral("Paint"), 32, 32,
                                                QStringLiteral("rgb"), 8,
                                                QStringLiteral("transparent"));
        pictura::PictureView* pv = frame.activeView();
        if (!paintDoc || !pv) {
            std::fprintf(stderr, "pictura self-test: FAIL: M21 paint document\n");
            std::fflush(stderr);
            return 53;
        }

        // 53: a stroke marks pixels, dirties the document, and adds one state.
        const int m21HistBefore = pv->history_count();
        pv->begin_paint(0xFFFF0000u, 0xFFFFFFFFu, 8, 100, 100, 0, 100, 100, 25,
                        QStringLiteral("normal"), false, false);
        pv->paint_dab(6, 16, 1.0);
        pv->paint_dab(12, 16, 1.0);
        pv->paint_dab(18, 16, 1.0);
        pv->paint_dab(24, 16, 1.0);
        const bool m21Ended = pv->end_paint();
        const QImage m21Stroke = pv->image();
        int m21Painted = 0;
        for (int y = 0; y < m21Stroke.height(); ++y) {
            for (int x = 0; x < m21Stroke.width(); ++x) {
                if (qAlpha(m21Stroke.pixel(x, y)) > 0) {
                    ++m21Painted;
                }
            }
        }
        std::fprintf(stderr,
                     "pictura self-test: m21_stroke ended=%d painted=%d dirty=%d hist=%d\n",
                     m21Ended ? 1 : 0,
                     m21Painted,
                     pv->is_dirty() ? 1 : 0,
                     pv->history_count());
        std::fflush(stderr);
        if (!m21Ended || !pv->is_dirty() || pv->history_count() != m21HistBefore + 1
            || m21Painted < 20) {
            std::fprintf(stderr, "pictura self-test: FAIL: M21 stroke wrong\n");
            return 53;
        }

        // 54: opacity caps one stroke and a second stroke adds coverage. A fresh
        // transparent canvas keeps the sampled point clear of the 53 stroke.
        const bool opacityDoc = frame.newDocument(QStringLiteral("PaintOpacity"), 32, 32,
                                                  QStringLiteral("rgb"), 8,
                                                  QStringLiteral("transparent"));
        pv = frame.activeView();
        if (!opacityDoc || !pv) {
            std::fprintf(stderr, "pictura self-test: FAIL: M21 opacity document\n");
            return 54;
        }
        pv->begin_paint(0xFF00FF00u, 0xFFFFFFFFu, 12, 100, 100, 0, 33, 100, 0,
                        QStringLiteral("normal"), false, false);
        for (int i = 0; i < 40; ++i) {
            pv->paint_dab(16, 16, 1.0);
        }
        pv->end_paint();
        const int m21A1 = qAlpha(pv->sample_argb(16, 16));
        pv->begin_paint(0xFF00FF00u, 0xFFFFFFFFu, 12, 100, 100, 0, 33, 100, 0,
                        QStringLiteral("normal"), false, false);
        for (int i = 0; i < 40; ++i) {
            pv->paint_dab(16, 16, 1.0);
        }
        pv->end_paint();
        const int m21A2 = qAlpha(pv->sample_argb(16, 16));
        std::fprintf(stderr, "pictura self-test: m21_opacity a1=%d a2=%d\n", m21A1, m21A2);
        std::fflush(stderr);
        if (!(m21A1 >= 78 && m21A1 <= 92) || !(m21A2 > m21A1 && m21A2 < 255)) {
            std::fprintf(stderr, "pictura self-test: FAIL: M21 opacity wrong\n");
            return 54;
        }

        // 55: Pencil edges are aliased; Brush edges are anti-aliased.
        const bool pencilDoc = frame.newDocument(QStringLiteral("PaintPencil"), 32, 32,
                                                 QStringLiteral("rgb"), 8,
                                                 QStringLiteral("transparent"));
        pictura::PictureView* pencilView = frame.activeView();
        const bool pencilBegun =
            pencilDoc && pencilView
            && pencilView->begin_paint(0xFFFF0000u, 0xFFFFFFFFu, 10, 100, 100, 0, 100, 100, 25,
                                       QStringLiteral("normal"), true, false);
        if (pencilView) {
            for (int x = 4; x <= 28; x += 2) {
                pencilView->paint_dab(x, 16, 1.0);
            }
        }
        const bool pencilEnded = pencilView && pencilView->end_paint();
        const QImage pencilImg = pencilView ? pencilView->image() : QImage();
        bool pencilBinary = !pencilImg.isNull();
        int pencilCovered = 0;
        for (int y = 0; y < pencilImg.height() && pencilBinary; ++y) {
            for (int x = 0; x < pencilImg.width(); ++x) {
                const int a = qAlpha(pencilImg.pixel(x, y));
                if (a == 0) {
                    continue;
                }
                ++pencilCovered;
                if (a != 255) {
                    pencilBinary = false;
                    break;
                }
            }
        }

        const bool brushDoc = frame.newDocument(QStringLiteral("PaintBrush"), 32, 32,
                                                QStringLiteral("rgb"), 8,
                                                QStringLiteral("transparent"));
        pictura::PictureView* brushView = frame.activeView();
        const bool brushBegun =
            brushDoc && brushView
            && brushView->begin_paint(0xFFFF0000u, 0xFFFFFFFFu, 10, 100, 100, 0, 100, 100, 25,
                                      QStringLiteral("normal"), false, false);
        if (brushView) {
            for (int x = 4; x <= 28; x += 2) {
                brushView->paint_dab(x, 16, 1.0);
            }
        }
        const bool brushEnded = brushView && brushView->end_paint();
        const QImage brushImg = brushView ? brushView->image() : QImage();
        bool brushSoft = false;
        int brushCovered = 0;
        for (int y = 0; y < brushImg.height() && !brushSoft; ++y) {
            for (int x = 0; x < brushImg.width(); ++x) {
                const int a = qAlpha(brushImg.pixel(x, y));
                if (a > 0) {
                    ++brushCovered;
                    if (a < 255) {
                        brushSoft = true;
                        break;
                    }
                }
            }
        }
        const bool pencilOk =
            pencilBegun && pencilEnded && pencilBinary && pencilCovered > 0;
        const bool brushOk = brushBegun && brushEnded && brushCovered > 0 && brushSoft;
        std::fprintf(stderr, "pictura self-test: m21_aliased pencil_ok=%d brush_aa=%d\n",
                     pencilOk ? 1 : 0, brushOk ? 1 : 0);
        std::fflush(stderr);
        if (!pencilOk || !brushOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M21 aliasing wrong\n");
            return 55;
        }

        // 56: undo restores the touched pixels of the brush document.
        const QImage m21Pre = brushView->image();
        brushView->begin_paint(0xFF0000FFu, 0xFFFFFFFFu, 10, 100, 100, 0, 100, 100, 25,
                               QStringLiteral("normal"), false, false);
        for (int x = 4; x <= 28; x += 2) {
            brushView->paint_dab(x, 20, 1.0);
        }
        const bool m21Changed = brushView->end_paint() && brushView->image() != m21Pre;
        const bool m21Restored = brushView->undo() && brushView->image() == m21Pre;
        std::fprintf(stderr, "pictura self-test: m21_undo changed=%d restored=%d\n",
                     m21Changed ? 1 : 0, m21Restored ? 1 : 0);
        std::fflush(stderr);
        if (!m21Changed || !m21Restored) {
            std::fprintf(stderr, "pictura self-test: FAIL: M21 undo wrong\n");
            return 56;
        }

        // M22: the 15 Artistic filters. A fresh white document exercises each
        // mapping end to end, and the seeded ones must reproduce bit-for-bit.
        const bool artDoc = frame.newDocument(QStringLiteral("Art"), 24, 24,
                                              QStringLiteral("rgb"), 8,
                                              QStringLiteral("white"));
        pictura::PictureView* av = frame.activeView();
        if (!artDoc || !av) {
            std::fprintf(stderr, "pictura self-test: FAIL: M22 artistic document\n");
            std::fflush(stderr);
            return 57;
        }

        // 57: every Artistic kind maps, applies, and changes the image.
        const QStringList artisticKinds = {
            QStringLiteral("colored-pencil"),
            QStringLiteral("cutout"),
            QStringLiteral("dry-brush"),
            QStringLiteral("film-grain"),
            QStringLiteral("fresco"),
            QStringLiteral("neon-glow"),
            QStringLiteral("paint-daubs"),
            QStringLiteral("palette-knife"),
            QStringLiteral("plastic-wrap"),
            QStringLiteral("poster-edges"),
            QStringLiteral("rough-pastels"),
            QStringLiteral("smudge-stick"),
            QStringLiteral("sponge"),
            QStringLiteral("underpainting"),
            QStringLiteral("watercolor"),
        };
        int m22Applied = 0;
        for (const QString& kind : artisticKinds) {
            const QImage before = av->image();
            const bool ok = av->apply_filter(kind);
            if (ok && av->image() != before) {
                ++m22Applied;
            }
        }
        std::fprintf(stderr, "pictura self-test: m22_applied applied=%d/%d\n", m22Applied,
                     static_cast<int>(artisticKinds.size()));
        std::fflush(stderr);
        if (m22Applied != artisticKinds.size()) {
            std::fprintf(stderr, "pictura self-test: FAIL: M22 artistic apply wrong\n");
            return 57;
        }

        // 58: a fixed seed makes a stochastic Artistic filter deterministic
        // across undo and reapply.
        while (av->can_undo()) {
            av->undo();
        }
        av->apply_filter(QStringLiteral("film-grain"));
        const QImage m22First = av->image();
        av->undo();
        av->apply_filter(QStringLiteral("film-grain"));
        const bool m22Deterministic = av->image() == m22First;
        std::fprintf(stderr, "pictura self-test: m22_deterministic=%d\n",
                     m22Deterministic ? 1 : 0);
        std::fflush(stderr);
        if (!m22Deterministic) {
            std::fprintf(stderr, "pictura self-test: FAIL: M22 artistic determinism wrong\n");
            return 58;
        }

        // M25: the 29 new filter kinds. A clouds-filled document gives every
        // filter structured content; a pure-white source is a no-op for some.
        const bool m25FilterDoc = frame.newDocument(QStringLiteral("M25Filters"), 32, 32,
                                                    QStringLiteral("rgb"), 8,
                                                    QStringLiteral("white"));
        pictura::PictureView* m25av = frame.activeView();
        if (!m25FilterDoc || !m25av || !m25av->apply_filter(QStringLiteral("clouds"))) {
            std::fprintf(stderr, "pictura self-test: FAIL: M25 filter document\n");
            std::fflush(stderr);
            return 68;
        }

        // 68: every M25 kind maps, applies, and changes the image.
        const QStringList m25Kinds = {
            QStringLiteral("accented-edges"),
            QStringLiteral("angled-strokes"),
            QStringLiteral("crosshatch"),
            QStringLiteral("dark-strokes"),
            QStringLiteral("ink-outlines"),
            QStringLiteral("spatter"),
            QStringLiteral("sprayed-strokes"),
            QStringLiteral("sumi-e"),
            QStringLiteral("bas-relief"),
            QStringLiteral("chalk-charcoal"),
            QStringLiteral("charcoal"),
            QStringLiteral("chrome"),
            QStringLiteral("conte-crayon"),
            QStringLiteral("graphic-pen"),
            QStringLiteral("halftone-pattern"),
            QStringLiteral("note-paper"),
            QStringLiteral("photocopy"),
            QStringLiteral("plaster"),
            QStringLiteral("reticulation"),
            QStringLiteral("stamp"),
            QStringLiteral("torn-edges"),
            QStringLiteral("water-paper"),
            QStringLiteral("craquelure"),
            QStringLiteral("grain"),
            QStringLiteral("mosaic-tiles"),
            QStringLiteral("patchwork"),
            QStringLiteral("stained-glass"),
            QStringLiteral("texturizer"),
            QStringLiteral("oil-paint"),
        };
        int m25Applied = 0;
        for (const QString& kind : m25Kinds) {
            const QImage before = m25av->image();
            const bool ok = m25av->apply_filter(kind);
            if (ok && m25av->image() != before) {
                ++m25Applied;
            }
            // Undo so every kind is compared against the same structured source
            // and the scratch history stays well under its 20-state cap.
            m25av->undo();
        }
        std::fprintf(stderr, "pictura self-test: m25_applied applied=%d/%d\n", m25Applied,
                     static_cast<int>(m25Kinds.size()));
        std::fflush(stderr);
        if (m25Applied != m25Kinds.size()) {
            std::fprintf(stderr, "pictura self-test: FAIL: M25 filter apply wrong\n");
            return 68;
        }

        // 69: a fixed seed makes the seeded M25 filter deterministic across
        // undo and reapply.
        m25av->apply_filter(QStringLiteral("grain"));
        const QImage m25First = m25av->image();
        m25av->undo();
        m25av->apply_filter(QStringLiteral("grain"));
        const bool m25Deterministic = m25av->image() == m25First;
        std::fprintf(stderr, "pictura self-test: m25_deterministic=%d\n",
                     m25Deterministic ? 1 : 0);
        std::fflush(stderr);
        if (!m25Deterministic) {
            std::fprintf(stderr, "pictura self-test: FAIL: M25 filter determinism wrong\n");
            return 69;
        }

        // M23: CS6 chrome. 59 stylesheet, 60 toolbox, 61 default dock groups.
        int m23Levels = 0;
        for (int level = 0; level < pictura::Theme::kLevelCount; ++level) {
            if (!pictura::Theme::styleSheet(level).isEmpty()) {
                ++m23Levels;
            }
        }
        const bool m23Distinct =
            pictura::Theme::styleSheet(0) != pictura::Theme::styleSheet(3);
        std::fprintf(stderr,
                     "pictura self-test: m23_stylesheet applied=%d levels=%d distinct=%d\n",
                     qApp->styleSheet().isEmpty() ? 0 : 1,
                     m23Levels,
                     m23Distinct ? 1 : 0);
        std::fflush(stderr);
        if (qApp->styleSheet().isEmpty() || m23Levels != pictura::Theme::kLevelCount
            || !m23Distinct) {
            std::fprintf(stderr, "pictura self-test: FAIL: M23 stylesheet wrong\n");
            return 59;
        }

        QDockWidget* m23Tools = frame.findChild<QDockWidget*>(QStringLiteral("toolsPanel"));
        const int m23Buttons =
            m23Tools ? static_cast<int>(m23Tools->findChildren<QToolButton*>().size()) : 0;
        const bool m23Fgbg = m23Tools
            && m23Tools->findChild<pictura::ForegroundBackgroundWidget*>() != nullptr;
        std::fprintf(stderr,
                     "pictura self-test: m23_toolbox dock=%d buttons=%d fgbg=%d\n",
                     m23Tools ? 1 : 0,
                     m23Buttons,
                     m23Fgbg ? 1 : 0);
        std::fflush(stderr);
        if (!m23Tools || m23Buttons < 10 || !m23Fgbg) {
            std::fprintf(stderr, "pictura self-test: FAIL: M23 toolbox wrong\n");
            return 60;
        }

        // M24: CS6 right side. 61 default dock groups, 62 the new panels,
        // 63 the icon rail toggling a collapsed panel.
        auto m24Dock = [&frame](const char* name) {
            return frame.findChild<QDockWidget*>(QString::fromLatin1(name));
        };
        QDockWidget* m24Color = m24Dock("colorPanel");
        QDockWidget* m24Swatch = m24Dock("swatchesPanel");
        QDockWidget* m24Gradients = m24Dock("gradientsPanel");
        QDockWidget* m24Patterns = m24Dock("patternsPanel");
        QDockWidget* m24Properties = m24Dock("propertiesPanel");
        QDockWidget* m24Adjustments = m24Dock("adjustmentsPanel");
        QDockWidget* m24Libraries = m24Dock("librariesPanel");
        QDockWidget* m24Layers = m24Dock("layersPanel");
        QDockWidget* m24Channels = m24Dock("channelsPanel");
        QDockWidget* m24Paths = m24Dock("pathsPanel");
        auto m24Tabbed = [&frame](QDockWidget* base, QDockWidget* member) {
            return base && member && frame.tabifiedDockWidgets(base).contains(member);
        };
        int m24Groups = 0;
        m24Groups += m24Tabbed(m24Color, m24Swatch) ? 1 : 0;
        m24Groups += m24Tabbed(m24Color, m24Gradients) ? 1 : 0;
        m24Groups += m24Tabbed(m24Color, m24Patterns) ? 1 : 0;
        m24Groups += (m24Color && frame.tabifiedDockWidgets(m24Color).size() == 3) ? 1 : 0;
        m24Groups += m24Tabbed(m24Properties, m24Adjustments) ? 1 : 0;
        m24Groups += m24Tabbed(m24Properties, m24Libraries) ? 1 : 0;
        m24Groups += m24Tabbed(m24Layers, m24Channels) ? 1 : 0;
        m24Groups += m24Tabbed(m24Layers, m24Paths) ? 1 : 0;
        std::fprintf(stderr, "pictura self-test: m24_groups grouped=%d/8\n", m24Groups);
        std::fflush(stderr);
        if (m24Groups != 8) {
            std::fprintf(stderr, "pictura self-test: FAIL: M24 dock grouping wrong\n");
            return 61;
        }

        const QStringList m24PanelNames = {
            QStringLiteral("gradientsPanel"),
            QStringLiteral("patternsPanel"),
            QStringLiteral("propertiesPanel"),
            QStringLiteral("adjustmentsPanel"),
            QStringLiteral("librariesPanel"),
            QStringLiteral("channelsPanel"),
            QStringLiteral("pathsPanel"),
            QStringLiteral("actionsPanel"),
        };
        int m24Found = 0;
        for (const QString& name : m24PanelNames) {
            if (frame.findChild<QDockWidget*>(name)) {
                ++m24Found;
            }
        }
        int m24PropsEmpty = 0;
        if (m24Properties) {
            for (QLabel* label : m24Properties->findChildren<QLabel*>()) {
                if (label->text().contains(QStringLiteral("No Properties"))) {
                    m24PropsEmpty = 1;
                    break;
                }
            }
        }
        std::fprintf(stderr,
                     "pictura self-test: m24_panels found=%d/8 properties_empty=%d\n",
                     m24Found,
                     m24PropsEmpty);
        std::fflush(stderr);
        if (m24Found != 8 || m24PropsEmpty != 1) {
            std::fprintf(stderr, "pictura self-test: FAIL: M24 panels wrong\n");
            return 62;
        }

        QToolBar* m24Rail = frame.findChild<QToolBar*>(QStringLiteral("panelRail"));
        int m24RailActions = 0;
        if (m24Rail) {
            for (QAction* action : m24Rail->actions()) {
                if (!action->isSeparator()) {
                    ++m24RailActions;
                }
            }
        }
        QDockWidget* m24Actions = m24Dock("actionsPanel");
        int m24Toggled = 0;
        int m24Synced = 0;
        if (m24Rail && m24Actions) {
            m24Actions->hide();
            const QString command = QStringLiteral("window.panels.actions");
            QAction* railAction = nullptr;
            for (QAction* action : m24Rail->actions()) {
                if (action->data().toString() == command) {
                    railAction = action;
                    break;
                }
            }
            QAction* commandAction = frame.registry()->action(command);
            if (railAction && commandAction) {
                commandAction->setChecked(true);
                frame.registry()->dispatch(command);
                const bool shown = m24Actions->isVisible();
                const bool railShown = railAction->isChecked();
                commandAction->setChecked(false);
                frame.registry()->dispatch(command);
                const bool hidden = !m24Actions->isVisible();
                const bool railHidden = !railAction->isChecked();
                m24Toggled = (shown && hidden) ? 1 : 0;
                m24Synced = (railShown && railHidden) ? 1 : 0;
            }
        }
        std::fprintf(stderr,
                     "pictura self-test: m24_rail actions=%d toggled=%d synced=%d\n",
                     m24RailActions,
                     m24Toggled,
                     m24Synced);
        std::fflush(stderr);
        if (!m24Rail || m24RailActions < 5 || m24Toggled != 1 || m24Synced != 1) {
            std::fprintf(stderr, "pictura self-test: FAIL: M24 rail wrong\n");
            return 63;
        }

        pictura::ImageView* canvas = frame.imageView();
        if (!canvas) {
            std::fprintf(stderr, "pictura self-test: FAIL: M25 no active canvas\n");
            return 64;
        }

        // 64: a freshly set image is centred, not pinned to the top-left.
        canvas->setImage(canvas->image());
        const double m25Zoom = canvas->zoom();
        const double m25Vw = canvas->width();
        const double m25Vh = canvas->height();
        const double m25Iw = canvas->image().width();
        const double m25Ih = canvas->image().height();
        const QPointF m25Expected((m25Vw - m25Iw * m25Zoom) / 2.0,
                                  (m25Vh - m25Ih * m25Zoom) / 2.0);
        const QPointF m25Actual = canvas->offset();
        const bool m25Centred = std::abs(m25Actual.x() - m25Expected.x()) < 1e-6
                                && std::abs(m25Actual.y() - m25Expected.y()) < 1e-6;
        const bool m25Smaller = m25Iw < m25Vw && m25Ih < m25Vh;
        const bool m25NotTopLeft = !m25Smaller || m25Actual != QPointF(0, 0);
        std::fprintf(stderr,
                     "pictura self-test: canvas_centre offset=(%g,%g) zoom=%g\n",
                     m25Actual.x(),
                     m25Actual.y(),
                     m25Zoom);
        std::fflush(stderr);
        if (!m25Centred || !m25NotTopLeft || m25Zoom <= 0.0) {
            std::fprintf(stderr, "pictura self-test: FAIL: M25 centre wrong\n");
            return 64;
        }

        // 65: middle-button drag pans regardless of the active tool.
        const QPointF m25Before = canvas->offset();
        QMouseEvent m25Press(QEvent::MouseButtonPress,
                             QPointF(200, 150),
                             canvas->mapToGlobal(QPoint(200, 150)),
                             Qt::MiddleButton,
                             Qt::MiddleButton,
                             Qt::NoModifier);
        QApplication::sendEvent(canvas, &m25Press);
        QMouseEvent m25Move(QEvent::MouseMove,
                            QPointF(230, 165),
                            canvas->mapToGlobal(QPoint(230, 165)),
                            Qt::NoButton,
                            Qt::MiddleButton,
                            Qt::NoModifier);
        QApplication::sendEvent(canvas, &m25Move);
        QMouseEvent m25Release(QEvent::MouseButtonRelease,
                               QPointF(230, 165),
                               canvas->mapToGlobal(QPoint(230, 165)),
                               Qt::MiddleButton,
                               Qt::NoButton,
                               Qt::NoModifier);
        QApplication::sendEvent(canvas, &m25Release);
        const QPointF m25Delta = canvas->offset() - m25Before;
        const bool m25Panned = std::abs(m25Delta.x() - 30.0) < 1e-6
                               && std::abs(m25Delta.y() - 15.0) < 1e-6;
        std::fprintf(stderr,
                     "pictura self-test: canvas_middle_pan delta=(%g,%g)\n",
                     m25Delta.x(),
                     m25Delta.y());
        std::fflush(stderr);
        if (!m25Panned) {
            std::fprintf(stderr, "pictura self-test: FAIL: M25 middle pan wrong\n");
            return 65;
        }

        // 66: a live move preview is transient; commit adds exactly one state.
        pictura::PictureView* m25View = frame.activeView();
        if (!m25View) {
            std::fprintf(stderr, "pictura self-test: FAIL: M25 move preview no document\n");
            return 66;
        }
        const int m25HistBefore = m25View->history_count();
        const QImage m25Pre = m25View->image();
        const bool m25Previewed = m25View->move_preview(3, 2);
        const bool m25PreviewChanged = m25View->image() != m25Pre;
        const bool m25PreviewNoHistory = m25View->history_count() == m25HistBefore;
        const bool m25Committed = m25View->commit_move();
        const bool m25CommitHistory = m25View->history_count() == m25HistBefore + 1;
        const bool m25Dirty = m25View->is_dirty();
        m25View->undo();
        const bool m25Undone = m25View->image() == m25Pre;
        std::fprintf(stderr,
                     "pictura self-test: canvas_move preview=%d hist=%d undo=%d\n",
                     m25Previewed ? 1 : 0,
                     m25CommitHistory ? 1 : 0,
                     m25Undone ? 1 : 0);
        std::fflush(stderr);
        if (!m25Previewed || !m25PreviewChanged || !m25PreviewNoHistory
            || !m25Committed || !m25CommitHistory || !m25Dirty || !m25Undone) {
            std::fprintf(stderr, "pictura self-test: FAIL: M25 move preview wrong\n");
            return 66;
        }

        // 67: the drag-start cache (base + layer) must not touch history.
        const int m25CacheHist = m25View->history_count();
        const bool m25Began = m25View->begin_move_preview();
        const bool m25BaseOk = !m25View->move_preview_base().isNull();
        const bool m25LayerOk = !m25View->move_preview_layer().isNull();
        const bool m25CacheHistOk = m25View->history_count() == m25CacheHist;
        m25View->end_move_preview();
        const bool m25HistUnchanged = m25View->history_count() == m25CacheHist;
        std::fprintf(stderr,
                     "pictura self-test: canvas_preview_cache began=%d base=%d layer=%d "
                     "hist_unchanged=%d\n",
                     m25Began ? 1 : 0,
                     m25BaseOk ? 1 : 0,
                     m25LayerOk ? 1 : 0,
                     (m25CacheHistOk && m25HistUnchanged) ? 1 : 0);
        std::fflush(stderr);
        if (!m25Began || !m25BaseOk || !m25LayerOk || !m25CacheHistOk || !m25HistUnchanged) {
            std::fprintf(stderr, "pictura self-test: FAIL: M25 preview cache wrong\n");
            return 67;
        }

        // Re-acquire for the trailing transform check.
        canvas = frame.imageView();
        if (!canvas) {
            std::fprintf(stderr, "pictura self-test: FAIL: no active canvas\n");
            return 38;
        }
        const QPointF center(canvas->width() / 2.0, canvas->height() / 2.0);
        canvas->zoomAt(center, 120);
        const QPointF afterZoom = canvas->offset();
        canvas->panBy(QPointF(10.0, 5.0));
        if (canvas->zoom() <= 1.0 || canvas->offset() != afterZoom + QPointF(10.0, 5.0)) {
            std::fprintf(stderr, "pictura self-test: FAIL: zoom/pan transform wrong\n");
            return 3;
        }
        std::fprintf(stderr,
                     "pictura self-test: zoom=%.3f pan_ok=1\n",
                     canvas->zoom());
        std::fflush(stderr);
        // The self-test leaves dirty documents behind; headless shutdown closes
        // the window and must discard them without opening a modal prompt.
        pictura::setUnsavedPromptInteractive(false);
        pictura::setNonInteractiveUnsavedChoice(pictura::UnsavedChoice::Discard);
        QTimer::singleShot(2000, &app, &QCoreApplication::quit);
    }

    return app.exec();
}
