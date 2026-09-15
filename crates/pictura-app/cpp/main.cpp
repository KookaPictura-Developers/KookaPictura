#include <QtCore/QCoreApplication>
#include <QtCore/QDir>
#include <QtCore/QProcessEnvironment>
#include <QtCore/QSet>
#include <QtCore/QStringList>
#include <QtCore/QTimer>
#include <QtGui/QAction>
#include <QtGui/QImage>
#include <QtGui/QPalette>
#include <QtWidgets/QApplication>
#include <QtWidgets/QDockWidget>

#include <cstdint>
#include <cstdio>

#include "commands.h"
#include "frame.h"
#include "image_view.h"
#include "session.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include "interop.h"

int main(int argc, char* argv[])
{
    QApplication app(argc, argv);

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

    pictura::PictureView view;

    if (interopProbe) {
        const bool prepared = view.gpu_interop_prepare();
        if (!prepared) {
            std::fprintf(stderr, "pictura interop-probe: no Vulkan device\n");
            return 0;
        }
        const std::int32_t result = pictura_try_qrhi_import(view.gpu_vk_instance(),
                                                            view.gpu_vk_physical_device(),
                                                            view.gpu_vk_device(),
                                                            view.gpu_vk_queue_family(),
                                                            view.gpu_vk_image(),
                                                            view.gpu_image_width(),
                                                            view.gpu_image_height());
        std::fprintf(stderr, "pictura interop-probe: qrhi_import=%d\n", result);
        std::fflush(stderr);
        return 0;
    }

    const bool codecLoaded = view.open(psdPath);
    // M0.5: fall back to the offscreen GPU demo only when no document loaded
    // (a loaded PSD must keep its composited image). 0 = no GPU,
    // 1 = rendered non-blank, 2 = rendered blank.
    int gpu = 0;
    if (!codecLoaded) {
        gpu = view.render_gpu();
    }
    const QImage image = view.image();

    pictura::PicturaMainWindow frame(&view);
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
        if (image.isNull()) {
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
            const bool wand = view.magic_wand(2, 2, 10);
            const bool hasSelection = view.has_selection();
            const int selectedPx = view.selection_count();
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
            const bool maskedAdded = view.add_adjustment(QStringLiteral("invert"));
            const QImage masked = view.image();
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
            view.remove_layer(view.layer_count() - 1);
            view.deselect();
            if (view.has_selection() || view.selection_count() != 0) {
                std::fprintf(stderr, "pictura self-test: FAIL: deselect left a selection\n");
                return 16;
            }

            // M4-C: add an Invert adjustment layer over the stack and verify the
            // composite changed as expected (red -> cyan, blue -> yellow).
            const QImage beforeAdjust = view.image();
            const bool added = view.add_adjustment(QStringLiteral("invert"));
            const QImage adjusted = view.image();
            const int layerCount = view.layer_count();
            std::fprintf(stderr,
                         "pictura self-test: add_adjustment(invert)=%d layers=%d last_kind=%s\n",
                         added ? 1 : 0,
                         layerCount,
                         view.layer_kind(layerCount - 1).toLocal8Bit().constData());
            std::fflush(stderr);
            if (!added || layerCount != 3
                || view.layer_kind(layerCount - 1) != QStringLiteral("adjustment")) {
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
            if (!view.layer_visible(0)) {
                std::fprintf(stderr, "pictura self-test: FAIL: base layer not visible\n");
                return 12;
            }
            view.set_layer_visible(0, false);
            const QImage hidden = view.image();
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
            if (!differs || view.layer_visible(0)) {
                std::fprintf(stderr,
                             "pictura self-test: FAIL: visibility toggle did not change output\n");
                return 13;
            }

            // M6-C: a filter must confine its change to the active selection.
            // The topmost pixel layer is the bottom-right blue quadrant; wand
            // that quadrant, apply the fixed-seed Add Noise, and require the
            // selected quadrant to change while the rest is bit-identical.
            view.deselect();
            const bool filterWand = view.magic_wand(6, 6, 10);
            const bool filterSelected = view.has_selection();
            const int filterSelectedPx = view.selection_count();
            std::fprintf(stderr,
                         "pictura self-test: filter_wand=%d selected_px=%d\n",
                         filterWand ? 1 : 0,
                         filterSelectedPx);
            std::fflush(stderr);
            if (!filterWand || !filterSelected || filterSelectedPx <= 0
                || filterSelectedPx >= view.image().width() * view.image().height()) {
                std::fprintf(stderr, "pictura self-test: FAIL: filter selection wrong\n");
                return 17;
            }
            const QImage filterBefore = view.image();
            const bool filtered = view.apply_filter(QStringLiteral("add-noise"));
            const QImage filterAfter = view.image();
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
            view.deselect();

            // M13: document ops. Earlier checks mutated the stack (hidden
            // layer, active invert, noise), so assert the exact 90 deg CW
            // remap (x,y) -> (7-y,x) on captured pixels, plus that a
            // successful op clears the selection.
            const QImage preRotate = view.image();
            view.select_all();
            const bool rotated = view.rotate_doc(1);
            const QImage rotatedImg = view.image();
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
                         view.selection_count());
            std::fflush(stderr);
            if (!rotated || rotatedImg.width() != 8 || rotatedImg.height() != 8
                || rotTr != preRotate.pixel(2, 2) || rotBl != preRotate.pixel(6, 6)
                || qAlpha(rotatedImg.pixel(5, 6)) != 0 || view.has_selection()
                || view.selection_count() != 0) {
                std::fprintf(stderr, "pictura self-test: FAIL: rotate cw wrong\n");
                return 19;
            }

            // M13: invalid document ops must be rejected and leave pixels put.
            const bool badRotate = view.rotate_doc(0);
            const bool badRotateClean = view.image() == rotatedImg;
            const bool badResize = view.resize_image(QStringLiteral("bicubic"), 0, 8);
            const bool badResizeClean = view.image() == rotatedImg;
            const bool badCanvas = view.resize_canvas(QStringLiteral("nope"), 10, 10);
            const bool badCanvasClean = view.image() == rotatedImg;
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
            const bool restoredOk = view.rotate_doc(3);
            const QImage restored = view.image();
            if (!restoredOk || restored != preRotate) {
                std::fprintf(stderr, "pictura self-test: FAIL: rotate ccw did not restore\n");
                return 21;
            }
            const bool grown = view.resize_canvas(QStringLiteral("bottom-right"), 10, 12);
            const QImage grownImg = view.image();
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
            const QImage preUndo = view.image();
            const int depthBefore = view.history_depth();
            const bool historyRotated = view.rotate_doc(1);
            const int depthAfterRotate = view.history_depth();
            const bool rotatedCanUndo = view.can_undo();
            const QImage postRotate = view.image();
            const bool undone = view.undo();
            const bool undoIdentical = view.image() == preUndo;
            const bool redone = view.redo();
            const bool redoIdentical = view.image() == postRotate;
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
            const bool flipped = view.flip_doc(true);
            const bool redoInvalidated = flipped && !view.can_redo();
            const bool reopened = view.open(psdPath);
            const QImage reopenedImg = view.image();
            const bool noUndoAfterOpen = !view.can_undo();
            const bool boundaryUndone = view.undo();
            const bool boundaryIdentical = view.image() == reopenedImg;
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
            const QImage preClouds = view.image();
            const bool clouded = view.apply_filter(QStringLiteral("clouds"));
            const QImage cloudedImg = view.image();
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
            const bool reapplied = view.apply_filter(QStringLiteral("clouds"));
            const bool reapplyIdentical = reapplied && view.image() == cloudedImg;
            const bool flared = view.apply_filter(QStringLiteral("lens-flare"));
            const QImage flaredImg = view.image();
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
            const bool selected = view.has_selection();
            const bool unknownInert = !registry->dispatch(QStringLiteral("no.such.command"));
            view.deselect();
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
            view.open(QStringLiteral("/nonexistent-kooka-pictura.psd"));
            registry->refresh();
            QAction* rotateAction =
                registry->action(QString::fromLatin1(pictura::command_ids::ImageRotate90Cw));
            QAction* openAction =
                registry->action(QString::fromLatin1(pictura::command_ids::FileOpen));
            const bool docDisabled = rotateAction && !rotateAction->isEnabled();
            const bool openEnabled = openAction && openAction->isEnabled();
            view.open(psdPath);
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
        }
        pictura::ImageView* canvas = frame.imageView();
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
        QTimer::singleShot(2000, &app, &QCoreApplication::quit);
    }

    return app.exec();
}
