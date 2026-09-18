#include <QtCore/QCoreApplication>
#include <QtCore/QDir>
#include <QtCore/QFile>
#include <QtCore/QJsonArray>
#include <QtCore/QJsonObject>
#include <QtCore/QProcessEnvironment>
#include <QtCore/QSet>
#include <QtCore/QStringList>
#include <QtCore/QTemporaryDir>
#include <QtCore/QTimer>
#include <QtGui/QAction>
#include <QtGui/QColor>
#include <QtGui/QIcon>
#include <QtGui/QImage>
#include <QtGui/QKeyEvent>
#include <QtGui/QMouseEvent>
#include <QtGui/QPalette>
#include <QtGui/QPixmap>
#include <QtWidgets/QAbstractButton>
#include <QtWidgets/QApplication>
#include <QtWidgets/QDockWidget>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QMenu>
#include <QtWidgets/QMenuBar>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSplitter>
#include <QtWidgets/QTabBar>
#include <QtWidgets/QTabWidget>
#include <QtWidgets/QToolBar>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QTreeView>

#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <optional>

#include "commands.h"
#include "dialogs.h"
#include "frame.h"
#include "icons.h"
#include "image_view.h"
#include "panels/layers_panel.h"
#include "panels/panel_column.h"
#include "panels/panel_group.h"
#include "preferences_dialog.h"
#include "session.h"
#include "theme.h"
#include "toolbox.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include "interop.h"

#include "selftest.h"

int runSelfTest(QApplication& app, bool headless, const QString& psdPath,
                pictura::PicturaMainWindow& frame, pictura::PictureView* view,
                const QImage& image, bool codecLoaded, int gpu)
{
        if (headless && qApp->platformName() != QStringLiteral("offscreen")) {
            std::fprintf(stderr,
                         "pictura self-test: FAIL: headless platform is %s, expected offscreen\n",
                         qPrintable(qApp->platformName()));
            return 152;
        }
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
            // E1: the scratch document is plain white and its first present must
            // be pure white across every pixel. The GPU smoke probe above must
            // not leak its gradient into the document's display image.
            bool freshWhite = true;
            for (int y = 0; y < image.height() && freshWhite; ++y) {
                for (int x = 0; x < image.width(); ++x) {
                    if (image.pixel(x, y) != 0xFFFFFFFFu) {
                        freshWhite = false;
                        break;
                    }
                }
            }
            std::fprintf(stderr,
                         "pictura self-test: fresh_white=%d\n",
                         freshWhite ? 1 : 0);
            std::fflush(stderr);
            if (!freshWhite) {
                std::fprintf(stderr, "pictura self-test: FAIL: fresh document not white\n");
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
            auto* layersPanel = frame.findChild<QWidget*>(QStringLiteral("layersPanel"));
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
        const bool docTabReorderOk = frame.reorderDocumentsForTest();
        std::fprintf(stderr, "pictura self-test: doc_tab_reorder aligned=%d\n",
                     docTabReorderOk ? 1 : 0);
        std::fflush(stderr);
        if (!docTabReorderOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: document tab reorder desync\n");
            return 196;
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
        auto* m20Column = frame.panelColumn();
        for (const QString& id : panelCommands) {
            QAction* action = frame.registry()->action(id);
            const QString objectName =
                id.section(QLatin1Char('.'), -1) + QStringLiteral("Panel");
            QWidget* panel = frame.findChild<QWidget*>(objectName);
            if (!action || !panel || !m20Column) {
                continue;
            }
            const bool before = m20Column->isPanelVisible(objectName);
            action->setChecked(!before);
            frame.registry()->dispatch(id);
            const bool toggled = m20Column->isPanelVisible(objectName) != before;
            action->setChecked(!m20Column->isPanelVisible(objectName));
            frame.registry()->dispatch(id);
            const bool restored = m20Column->isPanelVisible(objectName) == before;
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

        // M26: GPU-compute default/toggle and its persisted preference.
        // 70: default is on; the toggle flips to CPU and restores the backend.
        pictura::PictureView* m26av = frame.activeView();
        if (!m26av) {
            std::fprintf(stderr, "pictura self-test: FAIL: M26 no active view\n");
            std::fflush(stderr);
            return 70;
        }
        const bool m26Avail = m26av->gpu_available();
        const bool m26DefaultOn = m26av->gpu_compute();
        const QString m26OnBackend = m26av->active_backend();
        const bool m26OnOk = m26Avail ? m26OnBackend == QStringLiteral("GPU")
                                      : m26OnBackend == QStringLiteral("CPU (no GPU)");
        m26av->set_gpu_compute(false);
        const bool m26OffCpu = m26av->active_backend() == QStringLiteral("CPU");
        m26av->set_gpu_compute(true);
        const bool m26Restored = m26av->active_backend() == m26OnBackend;
        std::fprintf(stderr,
                     "pictura self-test: m26_gpu available=%d default_on=%d off_cpu=%d on_back=%d\n",
                     m26Avail ? 1 : 0,
                     m26DefaultOn ? 1 : 0,
                     m26OffCpu ? 1 : 0,
                     m26Restored ? 1 : 0);
        std::fflush(stderr);
        if (!m26DefaultOn || !m26OnOk || !m26OffCpu || !m26Restored) {
            std::fprintf(stderr, "pictura self-test: FAIL: M26 gpu backend wrong\n");
            return 70;
        }

        // 71: the GPU preference round-trips through the session store, and a
        // fresh SessionState defaults to on.
        const bool m26FramePref = m26av->gpu_compute();
        pictura::SessionState m26State = pictura::loadSession();
        m26State.gpuCompute = false;
        const bool m26SavedOff = pictura::saveSession(m26State);
        const bool m26OffRound = !pictura::loadSession().gpuCompute;
        m26State.gpuCompute = true;
        const bool m26SavedOn = pictura::saveSession(m26State);
        const bool m26OnRound = pictura::loadSession().gpuCompute;
        const bool m26Default = pictura::SessionState{}.gpuCompute;
        m26State.gpuCompute = m26FramePref;
        pictura::saveSession(m26State);
        std::fprintf(stderr,
                     "pictura self-test: m26_session gpu_off=%d gpu_on=%d default=%d\n",
                     (m26SavedOff && m26OffRound) ? 1 : 0,
                     (m26SavedOn && m26OnRound) ? 1 : 0,
                     m26Default ? 1 : 0);
        std::fflush(stderr);
        if (!m26SavedOff || !m26OffRound || !m26SavedOn || !m26OnRound || !m26Default) {
            std::fprintf(stderr, "pictura self-test: FAIL: M26 session persistence wrong\n");
            return 71;
        }

        // M27: the GPU filter path. A supported filter must produce the same
        // image through the GPU and CPU backends (the accelerated kernels are
        // byte-exact), and the gpuCompute preference is restored afterwards.
        // 72: gaussian-blur is byte-identical through both backends.
        pictura::PictureView* m27av = frame.activeView();
        if (!m27av) {
            std::fprintf(stderr, "pictura self-test: FAIL: M27 no active view\n");
            std::fflush(stderr);
            return 72;
        }
        m27av->set_gpu_compute(true);
        m27av->apply_filter(QStringLiteral("gaussian-blur"));
        const QImage m27Gpu = m27av->image();
        m27av->undo();
        m27av->set_gpu_compute(false);
        m27av->apply_filter(QStringLiteral("gaussian-blur"));
        const QImage m27Cpu = m27av->image();
        m27av->set_gpu_compute(true);
        const bool m27Identical = m27Gpu == m27Cpu;
        std::fprintf(stderr, "pictura self-test: m27_filter byte_identical=%d\n",
                     m27Identical ? 1 : 0);
        std::fflush(stderr);
        if (!m27Identical) {
            std::fprintf(stderr, "pictura self-test: FAIL: M27 filter byte-identity wrong\n");
            return 72;
        }

        // M28: the heavy deterministic kernels. Surface Blur and Median must
        // produce the same image through the GPU and CPU backends (the M28
        // kernels are byte-exact, like the M27 set).
        // 73: both kinds match across backends.
        pictura::PictureView* m28av = frame.activeView();
        if (!m28av) {
            std::fprintf(stderr, "pictura self-test: FAIL: M28 no active view\n");
            std::fflush(stderr);
            return 73;
        }
        const QStringList m28Kinds = {
            QStringLiteral("surface-blur"),
            QStringLiteral("median"),
        };
        m28av->set_gpu_compute(true);
        QList<QImage> m28Gpu;
        bool m28Heavy = true;
        for (const QString& kind : m28Kinds) {
            const QImage m28Before = m28av->image();
            const bool m28Ok = m28av->apply_filter(kind);
            const QImage m28After = m28av->image();
            m28Heavy = m28Heavy && m28Ok && m28After != m28Before;
            m28Gpu.append(m28After);
            m28av->undo();
        }
        m28av->set_gpu_compute(false);
        for (int i = 0; i < m28Kinds.size(); ++i) {
            const bool m28Ok = m28av->apply_filter(m28Kinds.at(i));
            const QImage m28After = m28av->image();
            m28Heavy = m28Heavy && m28Ok && m28After == m28Gpu.at(i);
            // Keep the last CPU result applied: a dangling redo state would
            // shrink `canvas_move`'s history count.
            if (i + 1 < m28Kinds.size()) {
                m28av->undo();
            }
        }
        m28av->set_gpu_compute(true);
        std::fprintf(stderr, "pictura self-test: m28_heavy byte_identical=%d\n",
                     m28Heavy ? 1 : 0);
        std::fflush(stderr);
        if (!m28Heavy) {
            std::fprintf(stderr, "pictura self-test: FAIL: M28 heavy byte-identity wrong\n");
            return 73;
        }

        // M30: canvas transparency display and document-rect clipping.
        // 74: an all-transparent document reveals the checkerboard, and a moved
        // layer is cropped to the document rect (no red on the canvas area).
        const bool m30Created = frame.newDocument(QStringLiteral("Alpha"), 64, 64,
                                                  QStringLiteral("rgb"), 8,
                                                  QStringLiteral("transparent"));
        pictura::ImageView* m30Canvas = frame.imageView();
        if (!m30Created || !m30Canvas) {
            std::fprintf(stderr, "pictura self-test: FAIL: M30 transparent document\n");
            return 74;
        }
        const int m30DocIndex = frame.activeDocumentIndex();
        if (m30Canvas->width() <= 0 || m30Canvas->height() <= 0) {
            frame.resize(800, 600);
            QApplication::processEvents();
        }

        const QColor m30CanvasColor = m30Canvas->canvasColor();
        const QColor m30A = pictura::ImageView::transparencyColorA();
        const QColor m30B = pictura::ImageView::transparencyColorB();
        const QRectF m30DocRect(m30Canvas->offset(),
                                QSizeF(64.0 * m30Canvas->zoom(), 64.0 * m30Canvas->zoom()));

        QImage m30Shot(m30Canvas->size(), QImage::Format_ARGB32);
        m30Canvas->render(&m30Shot);

        bool m30SawA = false;
        bool m30SawB = false;
        bool m30SawCanvas = false;
        const QRect m30DocPx = m30DocRect.toAlignedRect();
        for (int y = m30DocPx.top() + 2; y <= m30DocPx.bottom() - 2; y += 3) {
            for (int x = m30DocPx.left() + 2; x <= m30DocPx.right() - 2; x += 3) {
                if (x < 0 || y < 0 || x >= m30Shot.width() || y >= m30Shot.height()) {
                    continue;
                }
                const QColor c = m30Shot.pixelColor(x, y);
                if (c.rgb() == m30A.rgb()) {
                    m30SawA = true;
                } else if (c.rgb() == m30B.rgb()) {
                    m30SawB = true;
                } else if (c.rgb() == m30CanvasColor.rgb()) {
                    m30SawCanvas = true;
                }
            }
        }
        const QPoint m30OutsidePx = m30DocRect.topLeft().toPoint() - QPoint(4, 4);
        const bool m30OutsideOk = m30OutsidePx.x() >= 0 && m30OutsidePx.y() >= 0
                                  && m30OutsidePx.x() < m30Shot.width()
                                  && m30OutsidePx.y() < m30Shot.height()
                                  && m30Shot.pixelColor(m30OutsidePx).rgb() == m30CanvasColor.rgb();
        const bool m30Checker = m30SawA && m30SawB && !m30SawCanvas && m30OutsideOk;

        QImage m30Red(64, 64, QImage::Format_RGBA8888);
        m30Red.fill(QColor(255, 0, 0));
        m30Canvas->beginMovePreview(m30Canvas->image(), m30Red, QPointF(-32.0, -32.0), 1.0);
        QImage m30ClipShot(m30Canvas->size(), QImage::Format_ARGB32);
        m30Canvas->render(&m30ClipShot);
        m30Canvas->endMovePreview();

        // The layer at (-32,-32) covers the document's top-left quadrant; probe
        // the few pixels around the centre on the covered side.
        const QPoint m30Centre = m30DocRect.center().toPoint();
        bool m30CentreRed = false;
        for (int dy = -2; dy <= 0 && !m30CentreRed; ++dy) {
            for (int dx = -2; dx <= 0 && !m30CentreRed; ++dx) {
                const QPoint q = m30Centre + QPoint(dx, dy);
                if (q.x() >= 0 && q.y() >= 0 && q.x() < m30ClipShot.width()
                    && q.y() < m30ClipShot.height()
                    && m30ClipShot.pixelColor(q).rgb() == QColor(255, 0, 0).rgb()) {
                    m30CentreRed = true;
                }
            }
        }
        const QPoint m30OverflowPx = m30DocRect.topLeft().toPoint() - QPoint(4, 4);
        const bool m30OverflowOk = m30OverflowPx.x() >= 0 && m30OverflowPx.y() >= 0
                                   && m30OverflowPx.x() < m30ClipShot.width()
                                   && m30OverflowPx.y() < m30ClipShot.height()
                                   && m30ClipShot.pixelColor(m30OverflowPx).rgb()
                                          == m30CanvasColor.rgb();
        const bool m30Clipped = m30CentreRed && m30OverflowOk;

        std::fprintf(stderr, "pictura self-test: m30_canvas checker=%d clipped=%d\n",
                     m30Checker ? 1 : 0,
                     m30Clipped ? 1 : 0);
        std::fflush(stderr);
        if (!m30Checker || !m30Clipped) {
            std::fprintf(stderr, "pictura self-test: FAIL: M30 transparency/clipping wrong\n");
            return 74;
        }
        // Restore the previously active document so later checks are undisturbed.
        frame.closeDocument(m30DocIndex, false);

        // Full pixel-by-pixel equality, shared by the canvas checks.
        auto samePixels = [](const QImage& a, const QImage& b) {
            if (a.isNull() || b.isNull() || a.size() != b.size()) {
                return false;
            }
            for (int y = 0; y < a.height(); ++y) {
                for (int x = 0; x < a.width(); ++x) {
                    if (a.pixel(x, y) != b.pixel(x, y)) {
                        return false;
                    }
                }
            }
            return true;
        };

        // M31: dirty-region canvas refresh. Grow a seeded document so a small
        // white layer sits in a larger transparent canvas, then Move it by a
        // known offset. The layer's pixels must land at the new location, a
        // sample far outside the dirty union must be untouched, and undo must
        // restore the pre-move image.
        // 75: region move.
        const bool m31Created = frame.newDocument(QStringLiteral("Region"), 32, 32,
                                                  QStringLiteral("rgb"), 8,
                                                  QStringLiteral("white"));
        pictura::PictureView* m31View = frame.activeView();
        if (!m31Created || !m31View) {
            std::fprintf(stderr, "pictura self-test: FAIL: M31 document\n");
            std::fflush(stderr);
            return 75;
        }
        const int m31DocIndex = frame.activeDocumentIndex();
        const bool m31Grown =
            m31View->resize_canvas(QStringLiteral("top-left"), 128, 128);
        if (!m31Grown) {
            std::fprintf(stderr, "pictura self-test: FAIL: M31 canvas growth\n");
            std::fflush(stderr);
            return 75;
        }
        const QImage m31Before = m31View->image();
        const QRgb m31Src = m31Before.pixel(5, 5);         // inside the layer
        const QRgb m31NewBefore = m31Before.pixel(38, 38); // shape's new home
        const QRgb m31Far = m31Before.pixel(100, 100);     // outside dirty union
        const bool m31Preview = m31View->begin_move_preview();
        const bool m31Moved = m31View->commit_move(8, 8);
        const QImage m31After = m31View->image();
        const bool m31Appeared = m31After.pixel(38, 38) == m31Src
                                 && m31NewBefore != m31Src;
        const bool m31Vacated = m31After.pixel(2, 2) != m31Src;
        const bool m31Outside = m31After.pixel(100, 100) == m31Far;
        const bool m31Undone = m31View->undo() && m31View->image() == m31Before;
        std::fprintf(stderr,
                     "pictura self-test: m31_region moved=%d outside_unchanged=%d undo=%d\n",
                     (m31Moved && m31Appeared && m31Vacated) ? 1 : 0,
                     m31Outside ? 1 : 0,
                     m31Undone ? 1 : 0);
        std::fflush(stderr);
        if (!m31Preview || !m31Moved || !m31Appeared || !m31Vacated || !m31Outside
            || !m31Undone) {
            std::fprintf(stderr, "pictura self-test: FAIL: M31 region move wrong\n");
            return 75;
        }
        // Leave the frame as M30 did: close the scratch document.
        frame.closeDocument(m31DocIndex, false);

        // M31 large region: with the per-pixel blit budget gone, a canvas-sized
        // dirty union (a 1024² canvas moved by (1,1) makes `old ∪ new` the whole
        // canvas) takes the region-blit path in C++ and must NOT trigger a
        // full-document recomposite. `regionBlitted` fires; `changed` (emitted
        // only by a full recomposite) does not. The on-screen canvas must still
        // equal a full recomposite.
        const bool m31bCreated =
            frame.newDocument(QStringLiteral("RegionLarge"), 1024, 1024,
                              QStringLiteral("rgb"), 8, QStringLiteral("white"));
        pictura::PictureView* m31bView = frame.activeView();
        pictura::ImageView* m31bCanvas = frame.imageView();
        if (!m31bCreated || !m31bView || !m31bCanvas) {
            std::fprintf(stderr, "pictura self-test: FAIL: M31 large document\n");
            std::fflush(stderr);
            return 75;
        }
        const int m31bDocIndex = frame.activeDocumentIndex();
        const QImage m31bBefore = m31bView->image();
        const QRgb m31bOrigin = m31bBefore.pixel(0, 0);
        int m31bRegionBlits = 0;
        int m31bChanged = 0;
        auto m31bRegionConn = QObject::connect(
            m31bView, &pictura::PictureView::regionBlitted,
            [&m31bRegionBlits](const QImage&, int, int) { ++m31bRegionBlits; });
        auto m31bChangedConn = QObject::connect(
            m31bView, &pictura::PictureView::changed, [&m31bChanged]() { ++m31bChanged; });
        const bool m31bMoved = m31bView->commit_move(1, 1);
        const QImage m31bBlitted = m31bCanvas->image();
        QObject::disconnect(m31bRegionConn);
        QObject::disconnect(m31bChangedConn);
        // Force a full recomposite and compare the blitted canvas with it.
        m31bView->set_gpu_compute(m31bView->gpu_compute());
        const QImage m31bFull = m31bCanvas->image();
        const bool m31bVacated = m31bBlitted.pixel(0, 0) != m31bOrigin;
        const bool m31bRegionPath = m31bRegionBlits >= 1 && m31bChanged == 0;
        const bool m31bCanvasSame = samePixels(m31bBlitted, m31bFull);
        const bool m31bUndone = m31bView->undo() && m31bView->image() == m31bBefore;
        std::fprintf(stderr,
                     "pictura self-test: m31_region_large moved=%d vacated=%d undo=%d "
                     "region=%d recomposite=%d canvas=%d\n",
                     m31bMoved ? 1 : 0,
                     m31bVacated ? 1 : 0,
                     m31bUndone ? 1 : 0,
                     m31bRegionPath ? 1 : 0,
                     m31bChanged > 0 ? 1 : 0,
                     m31bCanvasSame ? 1 : 0);
        std::fflush(stderr);
        if (!m31bMoved || !m31bVacated || !m31bUndone || !m31bRegionPath || !m31bCanvasSame) {
            std::fprintf(stderr, "pictura self-test: FAIL: M31 large-region blit wrong\n");
            return 75;
        }
        frame.closeDocument(m31bDocIndex, false);

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

        auto* m23Toolbox = frame.findChild<pictura::Toolbox*>(QStringLiteral("toolsPanel"));
        QDockWidget* m23Tools = m23Toolbox;
        const QList<QToolButton*> m23SlotButtons =
            m23Toolbox ? m23Toolbox->slotButtons() : QList<QToolButton*>();
        const int m23Buttons =
            m23Tools ? static_cast<int>(m23Tools->findChildren<QToolButton*>().size()) : 0;
        QToolButton* m23Toggle = m23Tools
            ? m23Tools->findChild<QToolButton*>(QStringLiteral("toolsColumnToggle"))
            : nullptr;
        const bool m23ToggleDistinct =
            m23Toggle != nullptr && !m23SlotButtons.contains(m23Toggle);
        const bool m23Fgbg = m23Tools
            && m23Tools->findChild<pictura::ForegroundBackgroundWidget*>() != nullptr;
        std::fprintf(stderr,
                     "pictura self-test: m23_toolbox dock=%d buttons=%d toggle=%d fgbg=%d\n",
                     m23Tools ? 1 : 0,
                     m23Buttons,
                     m23ToggleDistinct ? 1 : 0,
                     m23Fgbg ? 1 : 0);
        std::fflush(stderr);
        if (!m23Tools || m23Buttons != 25 || !m23ToggleDistinct || !m23Fgbg) {
            std::fprintf(stderr, "pictura self-test: FAIL: M23 toolbox wrong\n");
            return 60;
        }

        // M24: CS6 right side. 61 default PanelColumn groups, 62 the new
        // panels, 63 the Window > Panels toggles (the icon rail was removed in
        // M41, so this now proves no PanelRail remains and the toggles work).
        auto* m24Column = frame.panelColumn();
        auto m24GroupOf = [m24Column](const char* name) {
            return m24Column ? m24Column->groupOfForTest(QString::fromLatin1(name)) : QString();
        };
        QWidget* m24Properties = frame.findChild<QWidget*>(QStringLiteral("propertiesPanel"));
        int m24Groups = 0;
        m24Groups += (!m24GroupOf("colorPanel").isEmpty()
                      && m24GroupOf("colorPanel") == m24GroupOf("swatchesPanel")) ? 1 : 0;
        m24Groups += (!m24GroupOf("colorPanel").isEmpty()
                      && m24GroupOf("colorPanel") == m24GroupOf("stylesPanel")) ? 1 : 0;
        m24Groups += (!m24GroupOf("layersPanel").isEmpty()
                      && m24GroupOf("layersPanel") == m24GroupOf("channelsPanel")) ? 1 : 0;
        m24Groups += (m24GroupOf("layersPanel") == m24GroupOf("pathsPanel")) ? 1 : 0;
        m24Groups += (!m24GroupOf("navigatorPanel").isEmpty()
                      && m24GroupOf("navigatorPanel") == m24GroupOf("histogramPanel")) ? 1 : 0;
        m24Groups += (m24GroupOf("navigatorPanel") == m24GroupOf("infoPanel")) ? 1 : 0;
        m24Groups += (!m24GroupOf("adjustmentsPanel").isEmpty()) ? 1 : 0;
        m24Groups += (!m24GroupOf("historyPanel").isEmpty()
                      && m24GroupOf("historyPanel") != m24GroupOf("actionsPanel")) ? 1 : 0;
        std::fprintf(stderr, "pictura self-test: m24_groups grouped=%d/8\n", m24Groups);
        std::fflush(stderr);
        if (m24Groups != 8) {
            std::fprintf(stderr, "pictura self-test: FAIL: M24 panel grouping wrong\n");
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
            if (frame.findChild<QWidget*>(name)) {
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

        const QStringList m24RailCommands = {
            QStringLiteral("window.panels.history"),
            QStringLiteral("window.panels.actions"),
            QStringLiteral("window.panels.info"),
            QStringLiteral("window.panels.navigator"),
            QStringLiteral("window.panels.histogram"),
        };
        int m24RailActions = 0;
        for (const QString& id : m24RailCommands) {
            if (frame.registry()->action(id)) {
                ++m24RailActions;
            }
        }
        int m24Toggled = 0;
        if (m24Column) {
            const QString command = QStringLiteral("window.panels.history");
            const QString panelName = QStringLiteral("historyPanel");
            QAction* action = frame.registry()->action(command);
            const bool before = m24Column->isPanelVisible(panelName);
            if (action) {
                action->setChecked(!before);
                frame.registry()->dispatch(command);
                const bool shown = m24Column->isPanelVisible(panelName) != before;
                action->setChecked(before);
                frame.registry()->dispatch(command);
                const bool hidden = m24Column->isPanelVisible(panelName) == before;
                m24Toggled = (shown && hidden) ? 1 : 0;
            }
        }
        const bool m24NoRail =
            frame.findChild<QToolBar*>(QStringLiteral("panelRail")) == nullptr;
        std::fprintf(stderr,
                     "pictura self-test: m24_rail actions=%d toggled=%d norail=%d\n",
                     m24RailActions,
                     m24Toggled,
                     m24NoRail ? 1 : 0);
        std::fflush(stderr);
        if (m24RailActions < 5 || m24Toggled != 1 || !m24NoRail) {
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

        // 76: the present cache reuses the scaled document across repaints at a
        // fixed zoom, rebuilds when the zoom changes, and its size is the
        // scaled document size.
        canvas = frame.imageView();
        if (!canvas || canvas->image().isNull()) {
            std::fprintf(stderr, "pictura self-test: FAIL: present cache no canvas\n");
            return 76;
        }
        const QImage pcSource = canvas->image();
        const double pcZoom = canvas->zoom();
        const QSize pcExpected(std::max(1, int(pcSource.width() * pcZoom)),
                               std::max(1, int(pcSource.height() * pcZoom)));
        QImage pcShot1(canvas->size(), QImage::Format_ARGB32);
        canvas->render(&pcShot1);
        const int pcRebuildsAfterFirst = canvas->presentCacheRebuildCount();
        QImage pcShot2(canvas->size(), QImage::Format_ARGB32);
        canvas->render(&pcShot2);
        const bool pcReused = !canvas->presentCacheRebuiltOnLastPaint()
                              && canvas->presentCacheRebuildCount() == pcRebuildsAfterFirst;
        const bool pcStable = pcShot1 == pcShot2;
        const QSize pcCachedAtZoom = canvas->presentCacheImageSize();
        canvas->setZoom(pcZoom * 2.0,
                        QPointF(canvas->width() / 2.0, canvas->height() / 2.0));
        QImage pcShot3(canvas->size(), QImage::Format_ARGB32);
        canvas->render(&pcShot3);
        const bool pcRebuiltOnZoom = canvas->presentCacheRebuiltOnLastPaint()
                                      && canvas->presentCacheRebuildCount() > pcRebuildsAfterFirst;
        const QSize pcCachedAfterZoom = canvas->presentCacheImageSize();
        const QSize pcExpectedAfterZoom(std::max(1, int(pcSource.width() * canvas->zoom())),
                                        std::max(1, int(pcSource.height() * canvas->zoom())));
        const bool pcSizeOk = pcCachedAtZoom == pcExpected
                              && pcCachedAfterZoom == pcExpectedAfterZoom;
        canvas->setPresentCacheEnabledForTest(false);
        QImage pcShot4(canvas->size(), QImage::Format_ARGB32);
        canvas->render(&pcShot4);
        canvas->setPresentCacheEnabledForTest(true);
        const bool pcIdentical = pcShot3 == pcShot4;
        std::fprintf(stderr,
                     "pictura self-test: present_cache reused=%d zoom_rebuild=%d src=%dx%d "
                     "z0=%g z1=%g size=%dx%d stable=%d identical=%d\n",
                     pcReused ? 1 : 0,
                     pcRebuiltOnZoom ? 1 : 0,
                     pcSource.width(),
                     pcSource.height(),
                     pcZoom,
                     canvas->zoom(),
                     pcCachedAfterZoom.width(),
                     pcCachedAfterZoom.height(),
                     pcStable ? 1 : 0,
                     pcIdentical ? 1 : 0);
        std::fflush(stderr);
        if (!pcReused || !pcRebuiltOnZoom || !pcSizeOk || !pcStable || !pcIdentical) {
            std::fprintf(stderr, "pictura self-test: FAIL: present cache wrong\n");
            return 76;
        }

        // M32: the interactive region paths. A 32x32 layer grown into a 64x64
        // canvas keeps a known sub-rectangle, so hiding it changes only that
        // rectangle.
        // 77: the move-preview base equals the canvas with the layer hidden.
        // 78: a raster visibility toggle equals a full recomposite.
        const bool m32Created = frame.newDocument(QStringLiteral("M32Region"), 32, 32,
                                                  QStringLiteral("rgb"), 8,
                                                  QStringLiteral("white"));
        pictura::PictureView* m32View = frame.activeView();
        if (!m32Created || !m32View) {
            std::fprintf(stderr, "pictura self-test: FAIL: M32 document\n");
            std::fflush(stderr);
            return 77;
        }
        const int m32DocIndex = frame.activeDocumentIndex();
        if (!m32View->resize_canvas(QStringLiteral("top-left"), 64, 64)) {
            std::fprintf(stderr, "pictura self-test: FAIL: M32 canvas growth\n");
            std::fflush(stderr);
            return 77;
        }
        const int m32k = m32View->topmost_pixel_layer_index();
        if (m32k < 0) {
            std::fprintf(stderr, "pictura self-test: FAIL: M32 no pixel layer\n");
            std::fflush(stderr);
            return 77;
        }
        // Full pixel-by-pixel equality over the small (64x64) canvas.
        const bool m32Began = m32View->begin_move_preview();
        const QImage m32Base = m32View->move_preview_base();
        m32View->set_layer_visible(m32k, false);
        const QImage m32RegionHidden = m32View->image();
        // Force a full recomposite of the same document state.
        m32View->set_gpu_compute(m32View->gpu_compute());
        const QImage m32FullHidden = m32View->image();
        const bool m32BaseSame = m32Began && !m32Base.isNull()
                                 && samePixels(m32Base, m32RegionHidden);
        const bool m32VisibilitySame = samePixels(m32RegionHidden, m32FullHidden);
        m32View->set_layer_visible(m32k, true);
        m32View->end_move_preview();
        frame.closeDocument(m32DocIndex, false);

        std::fprintf(stderr,
                     "pictura self-test: m32_region preview_base=%d visibility=%d\n",
                     m32BaseSame ? 1 : 0,
                     m32VisibilitySame ? 1 : 0);
        std::fflush(stderr);
        if (!m32BaseSame) {
            std::fprintf(stderr, "pictura self-test: FAIL: M32 preview base wrong\n");
            return 77;
        }
        if (!m32VisibilitySame) {
            std::fprintf(stderr, "pictura self-test: FAIL: M32 visibility region wrong\n");
            return 78;
        }

        // M34: composite coherence and cheap undo/redo. A region-path move must
        // leave the stored document composite equal to the displayed image;
        // undo/redo restore from the snapshot composite; and a save after the
        // edit must write that composite to disk.
        const bool m34Created = frame.newDocument(QStringLiteral("M34Coherent"), 32, 32,
                                                  QStringLiteral("rgb"), 8,
                                                  QStringLiteral("white"));
        pictura::PictureView* m34View = frame.activeView();
        if (!m34Created || !m34View) {
            std::fprintf(stderr, "pictura self-test: FAIL: M34 document\n");
            std::fflush(stderr);
            return 79;
        }
        const int m34DocIndex = frame.activeDocumentIndex();
        const QImage m34Pre = m34View->image();
        m34View->begin_move_preview();
        const bool m34Moved = m34View->commit_move(4, 4);
        const QImage m34Post = m34View->image();
        const bool m34Composite =
            m34Moved
            && static_cast<unsigned>(m34View->composite_argb(6, 6))
                   == static_cast<unsigned>(m34Post.pixel(6, 6))
            && static_cast<unsigned>(m34View->composite_argb(2, 2))
                   == static_cast<unsigned>(m34Post.pixel(2, 2));
        const bool m34Undo = m34View->undo() && m34View->image() == m34Pre
                             && m34View->redo() && m34View->image() == m34Post;

        // A full-recomposite edit (add noise) changes the layer's colour; the
        // save must write that edited composite, not the pre-edit merged image.
        const bool m34Filtered = m34View->apply_filter(QStringLiteral("add-noise"));
        const QImage m34Edited = m34View->image();
        const bool m34EditChanged = m34Edited != m34Post;

        const QString m34SavePath =
            QDir::tempPath() + QStringLiteral("/kooka-pictura-m34-roundtrip.psd");
        const bool m34Saved = frame.saveActiveAs(m34SavePath);
        const bool m34Reopened = frame.openPath(m34SavePath);
        pictura::PictureView* m34Reloaded = frame.activeView();
        const bool m34Save =
            m34Filtered && m34EditChanged && m34Saved && m34Reopened && m34Reloaded
            && m34Reloaded->has_document()
            && m34Reloaded->image() == m34Edited
            && static_cast<unsigned>(m34Reloaded->composite_argb(6, 6))
                   == static_cast<unsigned>(m34Edited.pixel(6, 6));

        std::fprintf(stderr,
                     "pictura self-test: m34_coherent composite=%d undo=%d save=%d\n",
                     m34Composite ? 1 : 0,
                     m34Undo ? 1 : 0,
                     m34Save ? 1 : 0);
        std::fflush(stderr);
        if (!m34Composite) {
            std::fprintf(stderr, "pictura self-test: FAIL: M34 composite coherence wrong\n");
            return 79;
        }
        if (!m34Undo) {
            std::fprintf(stderr, "pictura self-test: FAIL: M34 undo/redo wrong\n");
            return 80;
        }
        if (!m34Save) {
            std::fprintf(stderr, "pictura self-test: FAIL: M34 save round-trip wrong\n");
            return 81;
        }
        frame.closeDocument(frame.activeDocumentIndex(), false);
        frame.closeDocument(m34DocIndex, false);

        // M35: the C++ region-blit path. A region refresh emits regionBlitted
        // (not changed), ImageView::blitRegion overwrites the canvas, and the
        // on-screen canvas equals a full recomposite; image() rebuilds from the
        // composite while dirty; the present cache is invalidated then rebuilt.
        // 82: document; 83: blit equality; 84: present cache; 85: large region.
        const bool m35Created = frame.newDocument(QStringLiteral("M35RegionBlit"), 32, 32,
                                                  QStringLiteral("rgb"), 8,
                                                  QStringLiteral("white"));
        pictura::PictureView* m35View = frame.activeView();
        pictura::ImageView* m35Canvas = frame.imageView();
        if (!m35Created || !m35View || !m35Canvas) {
            std::fprintf(stderr, "pictura self-test: FAIL: M35 document\n");
            std::fflush(stderr);
            return 82;
        }
        const int m35DocIndex = frame.activeDocumentIndex();
        if (!m35View->resize_canvas(QStringLiteral("top-left"), 64, 64)) {
            std::fprintf(stderr, "pictura self-test: FAIL: M35 canvas growth\n");
            std::fflush(stderr);
            return 82;
        }
        // Warm the present cache; the second paint at the same zoom must reuse it.
        QImage m35Warm(m35Canvas->size(), QImage::Format_ARGB32);
        m35Canvas->render(&m35Warm);
        m35Canvas->render(&m35Warm);
        const bool m35ReuseBefore = !m35Canvas->presentCacheRebuiltOnLastPaint();

        int m35RegionBlits = 0;
        int m35Changed = 0;
        auto m35RegionConn = QObject::connect(
            m35View, &pictura::PictureView::regionBlitted,
            [&m35RegionBlits](const QImage&, int, int) { ++m35RegionBlits; });
        auto m35ChangedConn = QObject::connect(
            m35View, &pictura::PictureView::changed, [&m35Changed]() { ++m35Changed; });
        const bool m35Previewed = m35View->begin_move_preview();
        const bool m35Moved = m35View->commit_move(4, 4);
        const QImage m35Blitted = m35Canvas->image();
        const QImage m35Rebuilt = m35View->image();
        QObject::disconnect(m35RegionConn);
        QObject::disconnect(m35ChangedConn);

        QImage m35Shot(m35Canvas->size(), QImage::Format_ARGB32);
        m35Canvas->render(&m35Shot);
        const bool m35CacheRebuiltAfter = m35Canvas->presentCacheRebuiltOnLastPaint();

        // Force a full recomposite and compare both the blitted canvas and the
        // rebuilt image with it.
        m35View->set_gpu_compute(m35View->gpu_compute());
        const QImage m35Full = m35Canvas->image();
        const bool m35RegionPath = m35RegionBlits >= 1 && m35Changed == 0;
        const bool m35CanvasSame = samePixels(m35Blitted, m35Full);
        const bool m35RebuiltSame = samePixels(m35Rebuilt, m35Full);
        const bool m35CacheOk = m35ReuseBefore && m35CacheRebuiltAfter;
        std::fprintf(stderr,
                     "pictura self-test: m35_region_blit region=%d changed=%d canvas=%d "
                     "rebuilt=%d cache=%d\n",
                     m35RegionPath ? 1 : 0,
                     m35Changed > 0 ? 1 : 0,
                     m35CanvasSame ? 1 : 0,
                     m35RebuiltSame ? 1 : 0,
                     m35CacheOk ? 1 : 0);
        std::fflush(stderr);
        if (!m35Previewed || !m35Moved || !m35RegionPath || !m35CanvasSame || !m35RebuiltSame) {
            std::fprintf(stderr, "pictura self-test: FAIL: M35 region blit wrong\n");
            return 83;
        }
        if (!m35CacheOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M35 present cache wrong\n");
            return 84;
        }
        frame.closeDocument(m35DocIndex, false);

        // M35 large region: a canvas-sized dirty union takes the C++ blit path
        // and does not recomposite the whole 1024² document.
        const bool m35bCreated =
            frame.newDocument(QStringLiteral("M35RegionLarge"), 1024, 1024,
                              QStringLiteral("rgb"), 8, QStringLiteral("white"));
        pictura::PictureView* m35bView = frame.activeView();
        pictura::ImageView* m35bCanvas = frame.imageView();
        if (!m35bCreated || !m35bView || !m35bCanvas) {
            std::fprintf(stderr, "pictura self-test: FAIL: M35 large document\n");
            std::fflush(stderr);
            return 82;
        }
        const int m35bDocIndex = frame.activeDocumentIndex();
        // Run this pass on the CPU compositor to exercise the region-blit path
        // with the non-default backend (the earlier M35 pass used the default).
        m35bView->set_gpu_compute(false);
        int m35bRegionBlits = 0;
        int m35bChanged = 0;
        auto m35bRegionConn = QObject::connect(
            m35bView, &pictura::PictureView::regionBlitted,
            [&m35bRegionBlits](const QImage&, int, int) { ++m35bRegionBlits; });
        auto m35bChangedConn = QObject::connect(
            m35bView, &pictura::PictureView::changed, [&m35bChanged]() { ++m35bChanged; });
        const bool m35bMoved = m35bView->commit_move(1, 1);
        const QImage m35bBlitted = m35bCanvas->image();
        QObject::disconnect(m35bRegionConn);
        QObject::disconnect(m35bChangedConn);
        m35bView->set_gpu_compute(m35bView->gpu_compute());
        const QImage m35bFull = m35bCanvas->image();
        const bool m35bRegionPath = m35bRegionBlits >= 1 && m35bChanged == 0;
        const bool m35bCanvasSame = samePixels(m35bBlitted, m35bFull);
        std::fprintf(stderr,
                     "pictura self-test: m35_region_large region=%d recomposite=%d canvas=%d\n",
                     m35bRegionPath ? 1 : 0,
                     m35bChanged > 0 ? 1 : 0,
                     m35bCanvasSame ? 1 : 0);
        std::fflush(stderr);
        if (!m35bMoved || !m35bRegionPath || !m35bCanvasSame) {
            std::fprintf(stderr, "pictura self-test: FAIL: M35 large-region blit wrong\n");
            return 85;
        }
        frame.closeDocument(m35bDocIndex, false);

        // Move-preview base cache: warm prepare -> begin hit, base reused byte-for-byte, content change -> miss. 197: cache.
        const bool mpcCreated = frame.newDocument(QStringLiteral("MovePreviewCache"), 32, 32, QStringLiteral("rgb"), 8, QStringLiteral("white"));
        pictura::PictureView* mpcView = frame.activeView();
        const bool mpcHit = mpcCreated && mpcView && mpcView->prepare_move_preview() && mpcView->begin_move_preview() && mpcView->move_preview_cache_hit();
        const QImage mpcBase = mpcView ? mpcView->move_preview_base() : QImage();
        const bool mpcReuse = mpcView && mpcView->begin_move_preview() && mpcView->move_preview_cache_hit() && !mpcBase.isNull() && samePixels(mpcBase, mpcView->move_preview_base());
        if (mpcView) mpcView->set_layer_visible(0, false);
        const bool mpcMiss = mpcView && mpcView->begin_move_preview() && !mpcView->move_preview_cache_hit();
        if (mpcView) mpcView->end_move_preview();
        std::fprintf(stderr, "pictura self-test: move_preview_cache hit=%d reuse=%d miss=%d\n", mpcHit ? 1 : 0, mpcReuse ? 1 : 0, mpcMiss ? 1 : 0);
        std::fflush(stderr);
        if (!mpcHit || !mpcReuse || !mpcMiss) { std::fprintf(stderr, "pictura self-test: FAIL: move preview cache wrong\n"); return 197; }
        frame.closeDocument(frame.activeDocumentIndex(), false);
        // Document size accessors: match full-image dims, then track a resize. 198: size.
        const bool dsCreated = frame.newDocument(QStringLiteral("DocSize"), 20, 12, QStringLiteral("rgb"), 8, QStringLiteral("white"));
        pictura::PictureView* dsView = frame.activeView();
        const bool dsMatch = dsCreated && dsView && dsView->document_width() == dsView->image().width() && dsView->document_height() == dsView->image().height();
        const bool dsResized = dsView && dsView->resize_canvas(QStringLiteral("top-left"), 30, 18) && dsView->document_width() == 30 && dsView->document_height() == 18 && dsView->document_width() == dsView->image().width() && dsView->document_height() == dsView->image().height();
        std::fprintf(stderr, "pictura self-test: document_size match=%d resized=%d\n", dsMatch ? 1 : 0, dsResized ? 1 : 0); std::fflush(stderr);
        if (!dsMatch || !dsResized) { std::fprintf(stderr, "pictura self-test: FAIL: document size wrong\n"); return 198; }
        frame.closeDocument(frame.activeDocumentIndex(), false);
        // M36: layer attributes through the bridge — fill, lock, color, each one
        // history state and undoable. 86: fill; 87: lock; 88: color; 89: undo.
        const bool m36Created = frame.newDocument(QStringLiteral("M36Attrs"), 16, 16,
                                                  QStringLiteral("rgb"), 8,
                                                  QStringLiteral("white"));
        pictura::PictureView* m36View = frame.activeView();
        if (!m36Created || !m36View) {
            std::fprintf(stderr, "pictura self-test: FAIL: M36 document\n");
            std::fflush(stderr);
            return 86;
        }
        const int m36DocIndex = frame.activeDocumentIndex();
        const int m36HistBase = m36View->history_count();
        const int m36FillBefore = m36View->layer_fill(0);
        const bool m36FillSet = m36View->set_layer_fill(0, 128);
        const int m36FillAfter = m36View->layer_fill(0);
        const bool m36FillOk = m36FillBefore == 255 && m36FillSet && m36FillAfter == 128
                               && m36View->history_count() == m36HistBase + 1;
        const bool m36LockSet =
            m36View->set_layer_lock(0, QStringLiteral("transparency"), true);
        const int m36LockAfter = m36View->layer_lock(0);
        const bool m36LockOk = m36LockSet && (m36LockAfter & 0x01) != 0
                               && m36View->history_count() == m36HistBase + 2;
        const int m36ColorBefore = m36View->layer_color(0);
        const bool m36ColorSet = m36View->set_layer_color(0, 3);
        const int m36ColorAfter = m36View->layer_color(0);
        const bool m36ColorOk = m36ColorBefore == 0 && m36ColorSet && m36ColorAfter == 3
                                && m36View->history_count() == m36HistBase + 3;
        // Undo walks each edit back to its prior value; redo restores them.
        const bool m36Undo = m36View->undo() && m36View->layer_color(0) == 0
                             && m36View->undo() && m36View->layer_lock(0) == 0
                             && m36View->undo() && m36View->layer_fill(0) == 255
                             && m36View->redo() && m36View->redo() && m36View->redo()
                             && m36View->layer_fill(0) == 128 && m36View->layer_color(0) == 3;
        std::fprintf(stderr, "pictura self-test: m36_attrs fill=%d lock=%d color=%d undo=%d\n",
                     m36FillOk ? 1 : 0,
                     m36LockOk ? 1 : 0,
                     m36ColorOk ? 1 : 0,
                     m36Undo ? 1 : 0);
        std::fflush(stderr);
        if (!m36FillOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M36 fill\n");
            return 86;
        }
        if (!m36LockOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M36 lock\n");
            return 87;
        }
        if (!m36ColorOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M36 color\n");
            return 88;
        }
        if (!m36Undo) {
            std::fprintf(stderr, "pictura self-test: FAIL: M36 undo restore\n");
            return 89;
        }
        frame.closeDocument(m36DocIndex, false);
        // M37: layer creation and grouping. Each op is exactly one history
        // step; a transparent new layer leaves the composite unchanged, and
        // undo restores the original stack.
        const bool m37Created = frame.newDocument(QStringLiteral("M37Create"), 16, 16,
                                                  QStringLiteral("rgb"), 8,
                                                  QStringLiteral("white"));
        pictura::PictureView* m37View = frame.activeView();
        if (!m37Created || !m37View) {
            std::fprintf(stderr, "pictura self-test: FAIL: m37 no view\n");
            return 90;
        }
        const int m37DocIndex = frame.activeDocumentIndex();
        const int m37BaseCount = m37View->layer_count();
        const int m37HistBase = m37View->history_count();
        // (a) New Layer: count grows by one and the transparent layer is inert.
        const QImage m37Before = m37View->image();
        const int m37Added = m37View->add_layer(-1);
        const int m37AfterAdd = m37View->layer_count();
        const bool m37AddOk = m37Added == m37BaseCount && m37AfterAdd == m37BaseCount + 1
                              && m37View->image() == m37Before
                              && m37View->history_count() == m37HistBase + 1;
        // (b) New Group.
        const int m37Group = m37View->add_group(-1);
        const bool m37GroupOk = m37Group == m37AfterAdd
                                && m37View->layer_count() == m37AfterAdd + 1
                                && m37View->layer_kind(m37Group) == QStringLiteral("group")
                                && m37View->history_count() == m37HistBase + 2;
        // (c) Duplicate: count grows and the copy is named "<name> copy".
        const int m37Dup = m37View->duplicate_layer(m37Added);
        const QString m37DupName = m37Dup >= 0 ? m37View->layer_name(m37Dup) : QString();
        const bool m37DupOk = m37Dup == m37Added + 1
                              && m37View->layer_count() == m37AfterAdd + 2
                              && m37DupName.endsWith(QStringLiteral(" copy"))
                              && m37View->history_count() == m37HistBase + 3;

        // (d) Group then ungroup: the wrapped layer keeps its slot and the
        // ungrouped children splice back in order.
        const int m37Wrapped = m37View->group_layer(m37Added);
        const bool m37Grouped = m37Wrapped == m37Added
                                && m37View->layer_kind(m37Wrapped) == QStringLiteral("group");
        const bool m37Ungrouped = m37View->ungroup_layer(m37Wrapped)
                                  && m37View->layer_kind(m37Wrapped) == QStringLiteral("pixel");
        const bool m37UngroupOk = m37Grouped && m37Ungrouped
                                  && m37View->history_count() == m37HistBase + 5;

        // (e) Five undos restore the initial single-layer stack.
        bool m37UndoOk = true;
        for (int i = 0; i < 5; ++i) {
            m37UndoOk = m37UndoOk && m37View->undo();
        }
        m37UndoOk = m37UndoOk && m37View->layer_count() == m37BaseCount;

        std::fprintf(stderr,
                     "pictura self-test: m37_create new=%d group=%d duplicate=%d "
                     "ungroup=%d undo=%d\n",
                     m37AddOk ? 1 : 0,
                     m37GroupOk ? 1 : 0,
                     m37DupOk ? 1 : 0,
                     m37UngroupOk ? 1 : 0,
                     m37UndoOk ? 1 : 0);
        std::fflush(stderr);
        if (!m37AddOk) {
            return 90;
        }
        if (!m37GroupOk) {
            return 91;
        }
        if (!m37DupOk) {
            return 92;
        }
        if (!m37UngroupOk) {
            return 93;
        }
        if (!m37UndoOk) {
            return 94;
        }
        frame.closeDocument(m37DocIndex, false);
        // M38: the frozen 71-tool catalogue, its icons/cursors/hotspots, the
        // 23-slot toolbox, and the unimplemented-tool guard. Assets are
        // document-independent, so these run with or without a loaded PSD.
        const QList<pictura::ToolId> catalogue = pictura::allToolIds();
        QString missingToolIcon;
        QString missingToolCursor;
        int iconsOk = 1;
        int cursorsOk = 1;
        for (const pictura::ToolId id : catalogue) {
            const pictura::ToolInfo& info = pictura::toolInfo(id);
            const QString assetId = QStringLiteral("tool.") + pictura::toolIdName(id);
            if (pictura::icon(assetId).isNull()) {
                iconsOk = 0;
                if (missingToolIcon.isEmpty()) {
                    missingToolIcon = assetId;
                }
            }
            const bool hotspotOk = info.hotspotX >= 0 && info.hotspotX < 24
                                   && info.hotspotY >= 0 && info.hotspotY < 24;
            if (!hotspotOk
                || pictura::cursor(assetId, info.hotspotX, info.hotspotY).pixmap().isNull()) {
                cursorsOk = 0;
                if (missingToolCursor.isEmpty()) {
                    missingToolCursor = assetId;
                }
            }
        }

        auto* toolsDock = frame.findChild<pictura::Toolbox*>(QStringLiteral("toolsPanel"));
        int slotsOk = 0;
        if (toolsDock) {
            const QList<QToolButton*> slotButtons = toolsDock->slotButtons();
            slotsOk = slotButtons.size() == 23 ? 1 : 0;
            if (slotsOk) {
                for (int g = 1; g <= slotButtons.size(); ++g) {
                    bool anyImplemented = false;
                    for (const pictura::ToolId id : catalogue) {
                        if (pictura::toolInfo(id).group == g && pictura::toolImplemented(id)) {
                            anyImplemented = true;
                        }
                    }
                    if (slotButtons.at(g - 1)->isEnabled() != anyImplemented) {
                        slotsOk = 0;
                    }
                }
            }
        }

        pictura::ToolController probe;
        const pictura::ToolId guardBefore = probe.activeTool();
        probe.setActiveTool(pictura::ToolId::MagicWand);
        const bool guardOk = probe.activeTool() == guardBefore;

        std::fprintf(stderr,
                     "pictura self-test: m38_tools icons=%d cursors=%d slots=%d guard=%d\n",
                     iconsOk,
                     cursorsOk,
                     slotsOk,
                     guardOk ? 1 : 0);
        std::fflush(stderr);
        if (!iconsOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: m38 tool icon missing=%s\n",
                         missingToolIcon.toLocal8Bit().constData());
            return 95;
        }
        if (!cursorsOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: m38 tool cursor missing=%s\n",
                         missingToolCursor.toLocal8Bit().constData());
            return 96;
        }
        if (!slotsOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: m38 toolbox slots wrong\n");
            return 97;
        }
        if (!guardOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: m38 unimplemented tool activated\n");
            return 98;
        }
        // M38 panels: the former rail icon ids now live on the PanelColumn
        // group tabs, plus the Layers action strip and the History snapshot
        // button. Assets are document-independent, so these run with or without
        // a loaded PSD.
        auto sameIcon = [](const QIcon& actual, const QIcon& expected) {
            return !actual.isNull() && !expected.isNull()
                   && actual.pixmap(20, 20).toImage() == expected.pixmap(20, 20).toImage();
        };
        auto* m38Column = frame.panelColumn();
        const QStringList m38RailTitles = {
            QStringLiteral("History"),
            QStringLiteral("Actions"),
            QStringLiteral("Info"),
            QStringLiteral("Navigator"),
            QStringLiteral("Histogram"),
        };
        const QStringList m38RailIds = {
            QStringLiteral("window.panels.history"),
            QStringLiteral("window.panels.actions"),
            QStringLiteral("window.panels.info"),
            QStringLiteral("window.panels.navigator"),
            QStringLiteral("window.panels.histogram"),
        };
        int m38RailIcons = 0;
        QString m38RailMissing;
        for (int i = 0; i < m38RailTitles.size(); ++i) {
            bool matched = false;
            if (m38Column) {
                for (pictura::PanelGroup* group : m38Column->groups()) {
                    if (group
                        && sameIcon(group->titleIconForTest(m38RailTitles.at(i)),
                                    pictura::icon(m38RailIds.at(i)))) {
                        matched = true;
                        break;
                    }
                }
            }
            if (matched) {
                ++m38RailIcons;
            } else if (m38RailMissing.isEmpty()) {
                m38RailMissing = m38RailIds.at(i);
            }
        }
        const bool m38RailOk = m38Column && m38RailIcons == m38RailTitles.size();

        const QStringList m38StripIds = {
            QStringLiteral("link"),           QStringLiteral("fx"),
            QStringLiteral("mask"),           QStringLiteral("fillAdjustment"),
            QStringLiteral("group"),          QStringLiteral("newLayer"),
            QStringLiteral("delete")};
        const QStringList m38StripObjectNames = {
            QStringLiteral("layersStripLink"),   QStringLiteral("layersStripFx"),
            QStringLiteral("layersStripMask"),   QStringLiteral("layersStripFillAdjustment"),
            QStringLiteral("layersStripGroup"),  QStringLiteral("layersStripNewLayer"),
            QStringLiteral("layersStripDelete")};
        // link/fx/mask have icons but no behaviour yet.
        const QSet<QString> m38StripDisabled = {
            QStringLiteral("link"), QStringLiteral("fx"), QStringLiteral("mask")};
        QWidget* m38Layers = frame.findChild<QWidget*>(QStringLiteral("layersPanel"));
        int m38StripOk = 0;
        QString m38StripWrong;
        for (int i = 0; i < m38StripIds.size(); ++i) {
            auto* button = m38Layers
                ? m38Layers->findChild<QToolButton*>(m38StripObjectNames.at(i))
                : nullptr;
            const QIcon expected =
                pictura::icon(QStringLiteral("layers.") + m38StripIds.at(i));
            const bool disabled = m38StripDisabled.contains(m38StripIds.at(i));
            if (button && sameIcon(button->icon(), expected)
                && button->isEnabled() != disabled) {
                ++m38StripOk;
            } else if (m38StripWrong.isEmpty()) {
                m38StripWrong = m38StripIds.at(i);
            }
        }
        const bool m38StripPass = m38StripOk == m38StripIds.size();

        QWidget* m38History = frame.findChild<QWidget*>(QStringLiteral("historyPanel"));
        auto* m38Snapshot =
            m38History ? m38History->findChild<QPushButton*>(QStringLiteral("snapshotButton"))
                       : nullptr;
        const bool m38HistoryOk =
            m38Snapshot
            && sameIcon(m38Snapshot->icon(), pictura::icon(QStringLiteral("history.snapshot")));

        std::fprintf(stderr, "pictura self-test: m38_panels rail=%d strip=%d history=%d\n",
                     m38RailOk ? 1 : 0,
                     m38StripPass ? 1 : 0,
                     m38HistoryOk ? 1 : 0);
        std::fflush(stderr);
        if (!m38RailOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: m38 rail icon missing=%s\n",
                         m38RailMissing.toLocal8Bit().constData());
            return 99;
        }
        if (!m38StripPass) {
            std::fprintf(stderr, "pictura self-test: FAIL: m38 strip button wrong=%s\n",
                         m38StripWrong.toLocal8Bit().constData());
            return 100;
        }
        if (!m38HistoryOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: m38 snapshot icon wrong\n");
            return 101;
        }
        // M39: the layers panel's tree projection, multi-selection batches,
        // solo visibility, Tab rename, Panel Options, badges, menus, tooltips,
        // and the seven-button strip. A deterministic fixture is built through
        // the bridge; the panel is read through its M39 test hooks.
        auto* m39Panel = frame.findChild<pictura::LayersPanel*>(QStringLiteral("layersPanel"));
        const bool m39Created = frame.newDocument(QStringLiteral("M39"), 16, 16,
                                                  QStringLiteral("rgb"), 8,
                                                  QStringLiteral("white"));
        pictura::PictureView* m39View = frame.activeView();
        if (!m39Created || !m39View || !m39Panel) {
            std::fprintf(stderr, "pictura self-test: FAIL: M39 document/panel\n");
            std::fflush(stderr);
            return 102;
        }
        const int m39DocIndex = frame.activeDocumentIndex();

        // Fixture: a Background, a group "1" with two pixel children and a
        // nested group "1/2", and a masked adjustment layer "2" on top.
        m39View->set_layer_name_path(QStringLiteral("0"), QStringLiteral("Background"));
        const QString m39Group = m39View->add_group_in(QString());
        const QString m39ChildA = m39View->add_layer_in(m39Group);
        const QString m39ChildB = m39View->add_layer_in(m39Group);
        const QString m39Nested = m39View->add_group_in(m39Group);
        const QString m39NestedChild = m39View->add_layer_in(m39Nested);
        m39View->select_all();
        m39View->add_adjustment(QStringLiteral("invert"));
        m39View->deselect();
        m39Panel->setView(m39View);
        m39Panel->refresh();

        auto m39RowOf = [m39View](const QString& path) {
            for (int i = 0; i < m39View->layer_row_count(); ++i) {
                if (m39View->layer_row_path(i) == path) {
                    return i;
                }
            }
            return -1;
        };

        // 8.1 m39_tree (102-103): the depth-first, topmost-first projection,
        // the tree-aware add-inside-group paths, and a nested rename by path.
        const QStringList m39TreePaths = {
            QStringLiteral("2"), QStringLiteral("1"), QStringLiteral("1/2"),
            QStringLiteral("1/2/0"), QStringLiteral("1/1"), QStringLiteral("1/0"),
            QStringLiteral("0")};
        const int m39TreeDepths[] = {0, 0, 1, 2, 1, 1, 0};
        const QStringList m39TreeKinds = {
            QStringLiteral("adjustment"), QStringLiteral("group"), QStringLiteral("group"),
            QStringLiteral("pixel"), QStringLiteral("pixel"), QStringLiteral("pixel"),
            QStringLiteral("background")};
        bool m39OrderOk = m39View->layer_row_count() == m39TreePaths.size();
        for (int i = 0; m39OrderOk && i < m39TreePaths.size(); ++i) {
            m39OrderOk = m39View->layer_row_path(i) == m39TreePaths.at(i)
                         && m39View->layer_row_depth(i) == m39TreeDepths[i]
                         && m39View->layer_row_kind(i) == m39TreeKinds.at(i);
        }
        const bool m39AddOk =
            m39ChildA == QStringLiteral("1/0") && m39ChildB == QStringLiteral("1/1")
            && m39Nested == QStringLiteral("1/2")
            && m39NestedChild == QStringLiteral("1/2/0")
            && m39View->layer_row_depth(m39RowOf(m39NestedChild)) == 2
            && m39View->layer_row_expandable(m39RowOf(QStringLiteral("1")))
            && m39View->layer_row_expandable(m39RowOf(m39Nested));
        const int m39RenameBase = m39View->history_count();
        const bool m39TreeRenameOk =
            m39View->set_layer_name_path(QStringLiteral("1/1"), QStringLiteral("Nested"))
            && m39View->layer_row_name(m39RowOf(QStringLiteral("1/1")))
                   == QStringLiteral("Nested")
            && m39View->history_count() == m39RenameBase + 1
            && !m39View->set_layer_name_path(QStringLiteral("1/9"), QStringLiteral("Bogus"))
            && m39View->history_count() == m39RenameBase + 1;
        std::fprintf(stderr, "pictura self-test: m39_tree order=%d add=%d rename=%d\n",
                     m39OrderOk ? 1 : 0, m39AddOk ? 1 : 0, m39TreeRenameOk ? 1 : 0);
        std::fflush(stderr);
        if (!m39OrderOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M39 projection order\n");
            return 102;
        }
        if (!m39AddOk || !m39TreeRenameOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M39 tree add/nested rename\n");
            return 103;
        }

        // 8.2 m39_multi (104-105): a batch is one undo step, the Background and
        // a group are skipped per node, and multi group/ungroup is one step.
        const QString m39Bg = QStringLiteral("0");
        const QString m39PixA = QStringLiteral("1/1");
        const QString m39PixB = QStringLiteral("1/0");
        const int m39BlendBase = m39View->history_count();
        const int m39BlendChanged =
            m39View->set_layers_blend(QStringList{m39PixA, m39PixB}, QStringLiteral("mul "));
        const bool m39BatchOk =
            m39BlendChanged == 2 && m39View->history_count() == m39BlendBase + 1
            && m39View->layer_row_blend(m39RowOf(m39PixA)) == QStringLiteral("mul ")
            && m39View->layer_row_blend(m39RowOf(m39PixB)) == QStringLiteral("mul ");
        const bool m39BatchUndoOk =
            m39View->undo()
            && m39View->layer_row_blend(m39RowOf(m39PixA)) == QStringLiteral("norm")
            && m39View->layer_row_blend(m39RowOf(m39PixB)) == QStringLiteral("norm")
            && m39View->redo()
            && m39View->layer_row_blend(m39RowOf(m39PixA)) == QStringLiteral("mul ");

        const int m39SkipBase = m39View->history_count();
        const int m39BgChanged =
            m39View->set_layers_blend(QStringList{m39Bg, m39PixA}, QStringLiteral("scrn"));
        const int m39FillBase = m39View->history_count();
        const int m39GroupFill = m39View->set_layers_fill(QStringList{m39Group, m39PixB}, 128);
        const bool m39SkipOk =
            m39BgChanged == 1 && m39FillBase == m39SkipBase + 1
            && m39View->layer_row_blend(m39RowOf(m39Bg)) == QStringLiteral("norm")
            && m39View->layer_row_blend(m39RowOf(m39PixA)) == QStringLiteral("scrn")
            && m39GroupFill == 1 && m39View->history_count() == m39FillBase + 1
            && m39View->layer_row_fill(m39RowOf(m39Group)) == 255
            && m39View->layer_row_fill(m39RowOf(m39PixB)) == 128;

        const int m39GroupBase = m39View->history_count();
        const QString m39Wrapped = m39View->group_layers(QStringList{m39PixA, m39PixB});
        const bool m39GroupStepOk =
            !m39Wrapped.isEmpty() && m39View->history_count() == m39GroupBase + 1;
        const int m39UngroupBase = m39View->history_count();
        const int m39Ungrouped = m39View->ungroup_layers(QStringList{m39Wrapped});
        const bool m39UngroupOk =
            m39Ungrouped == 1 && m39View->history_count() == m39UngroupBase + 1;

        std::fprintf(stderr,
                     "pictura self-test: m39_multi batch=%d skip=%d group=%d undo=%d\n",
                     m39BatchOk ? 1 : 0, m39SkipOk ? 1 : 0,
                     (m39GroupStepOk && m39UngroupOk) ? 1 : 0, m39BatchUndoOk ? 1 : 0);
        std::fflush(stderr);
        if (!m39BatchOk || !m39SkipOk || !m39BatchUndoOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M39 multi-selection batch\n");
            return 104;
        }
        if (!m39GroupStepOk || !m39UngroupOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M39 multi group/ungroup\n");
            return 105;
        }

        // 8.3 m39_solo (106): Alt-solo hides every other row, a second Alt
        // restores the exact prior per-row visibility, and undo restores.
        const QString m39SoloHidden = QStringLiteral("1/1");
        m39View->set_layers_visible(QStringList{m39SoloHidden}, false);
        m39Panel->refresh();
        QStringList m39PriorHidden;
        for (int i = 0; i < m39View->layer_row_count(); ++i) {
            if (!m39View->layer_row_visible(i)) {
                m39PriorHidden.push_back(m39View->layer_row_path(i));
            }
        }
        const QString m39SoloPath = QStringLiteral("1/0");
        m39Panel->toggleSoloForTest(m39SoloPath);
        m39Panel->refresh();
        bool m39SoloOk = true;
        for (int i = 0; i < m39View->layer_row_count(); ++i) {
            const QString path = m39View->layer_row_path(i);
            const bool expected = path == m39SoloPath || path == m39Group;
            m39SoloOk = m39SoloOk && m39View->layer_row_visible(i) == expected;
        }
        const bool m39SoloUndo = m39View->undo();
        m39Panel->refresh();
        bool m39SoloUndoOk = m39SoloUndo;
        for (int i = 0; i < m39View->layer_row_count(); ++i) {
            m39SoloUndoOk = m39SoloUndoOk
                            && m39View->layer_row_visible(i)
                                   == !m39PriorHidden.contains(m39View->layer_row_path(i));
        }
        const bool m39SoloRedo = m39View->redo();
        m39Panel->refresh();
        m39Panel->toggleSoloForTest(m39Group);
        m39Panel->refresh();
        bool m39SoloRestoreOk = m39SoloRedo;
        for (int i = 0; i < m39View->layer_row_count(); ++i) {
            m39SoloRestoreOk =
                m39SoloRestoreOk
                && m39View->layer_row_visible(i)
                       == !m39PriorHidden.contains(m39View->layer_row_path(i));
        }
        std::fprintf(stderr, "pictura self-test: m39_solo solo=%d restore=%d undo=%d\n",
                     m39SoloOk ? 1 : 0, m39SoloRestoreOk ? 1 : 0, m39SoloUndoOk ? 1 : 0);
        std::fflush(stderr);
        if (!m39SoloOk || !m39SoloRestoreOk || !m39SoloUndoOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M39 solo visibility\n");
            return 106;
        }

        // 8.4 m39_rename (107): Tab commits and moves down, Shift+Tab up, and
        // the ends do not wrap. The key is delivered to the delegate's own
        // event filter (its role on the inline editor); headless focus routing
        // makes QApplication::sendEvent to the editor unreliable, so the
        // delegate is invoked directly and the view still performs the move.
        QCoreApplication::processEvents();
        m39Panel->refresh();
        QCoreApplication::processEvents();
        auto m39Edit = [m39Panel](const QString& path, const QString& text, int key) {
            QCoreApplication::processEvents();
            if (!m39Panel->beginRenameForTest(path)) {
                return false;
            }
            QCoreApplication::processEvents();
            // Drop editors from the previous step that are pending deleteLater,
            // so findChild returns the editor the view currently has registered.
            QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
            QCoreApplication::processEvents();
            auto* tree = m39Panel->findChild<QTreeView*>();
            QLineEdit* editor = tree ? tree->viewport()->findChild<QLineEdit*>() : nullptr;
            QObject* delegate = m39Panel->itemDelegateForTest();
            if (!editor || !delegate) {
                return false;
            }
            editor->setText(text);
            QKeyEvent event(QEvent::KeyPress, key, Qt::NoModifier);
            delegate->eventFilter(editor, &event);
            QCoreApplication::processEvents();
            m39Panel->refresh();
            return true;
        };
        const QString m39FirstRow = QStringLiteral("2");
        const QString m39LastRow = QStringLiteral("0");
        const bool m39DownSent = m39Edit(m39FirstRow, QStringLiteral("TopAdj"), Qt::Key_Tab);
        const bool m39DownMoved =
            m39DownSent
            && m39View->layer_row_name(m39RowOf(m39FirstRow)) == QStringLiteral("TopAdj")
            && m39Panel->currentPath() == QStringLiteral("1");
        // The commit is exactly one undoable step: one undo restores the prior
        // name and one redo reapplies it.
        const bool m39DownUndoOk =
            m39View->undo()
            && m39View->layer_row_name(m39RowOf(m39FirstRow)) == QStringLiteral("Invert")
            && m39View->redo()
            && m39View->layer_row_name(m39RowOf(m39FirstRow)) == QStringLiteral("TopAdj");
        const bool m39DownOk = m39DownMoved && m39DownUndoOk;
        const bool m39UpOk =
            m39Edit(m39LastRow, QStringLiteral("Bottom"), Qt::Key_Backtab)
            && m39View->layer_row_name(m39RowOf(m39LastRow)) == QStringLiteral("Bottom")
            && m39Panel->currentPath() == QStringLiteral("1");
        const bool m39NoWrapFirst =
            m39Edit(m39FirstRow, QStringLiteral("First"), Qt::Key_Backtab)
            && m39View->layer_row_name(m39RowOf(m39FirstRow)) == QStringLiteral("First")
            && m39Panel->currentPath() == m39FirstRow;
        const bool m39NoWrapLast =
            m39Edit(m39LastRow, QStringLiteral("Last"), Qt::Key_Tab)
            && m39View->layer_row_name(m39RowOf(m39LastRow)) == QStringLiteral("Last")
            && m39Panel->currentPath() == m39LastRow;
        const bool m39NowrapOk = m39NoWrapFirst && m39NoWrapLast;
        std::fprintf(stderr, "pictura self-test: m39_rename down=%d up=%d nowrap=%d\n",
                     m39DownOk ? 1 : 0, m39UpOk ? 1 : 0, m39NowrapOk ? 1 : 0);
        std::fflush(stderr);
        if (!m39DownOk || !m39UpOk || !m39NowrapOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M39 Tab rename\n");
            return 107;
        }

        // 8.5 m39_options (108): the defaults are Medium / Entire Document /
        // on, and a session save/load round-trip preserves a changed triple.
        const pictura::SessionState m39Fresh = pictura::loadSession();
        const bool m39DefaultsOk =
            m39Fresh.layersThumbSize == 2 && m39Fresh.layersThumbContents == 0
            && m39Fresh.layersExpandNewEffects
            && m39Panel->thumbSizeIndexForTest() == 2
            && m39Panel->thumbContentsForTest() == 0
            && m39Panel->expandNewEffectsForTest();
        m39Panel->setOptionsForTest(3, 1, false);
        const pictura::SessionState m39Saved = pictura::loadSession();
        auto* m39Reloaded = new pictura::LayersPanel();
        const bool m39RoundtripOk =
            m39Saved.layersThumbSize == 3 && m39Saved.layersThumbContents == 1
            && !m39Saved.layersExpandNewEffects
            && m39Reloaded->thumbSizeIndexForTest() == 3
            && m39Reloaded->thumbContentsForTest() == 1
            && !m39Reloaded->expandNewEffectsForTest();
        delete m39Reloaded;
        std::fprintf(stderr, "pictura self-test: m39_options defaults=%d roundtrip=%d\n",
                     m39DefaultsOk ? 1 : 0, m39RoundtripOk ? 1 : 0);
        std::fflush(stderr);
        if (!m39DefaultsOk || !m39RoundtripOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M39 panel options\n");
            return 108;
        }

        // 8.6 m39_badges (109): the mask and adjustment badge data reaches the
        // model. No bridge op creates a clipped layer, so `clip` verifies the
        // clip role is plumbed consistently (and the group expand indicator).
        const bool m39BadgeMask =
            m39View->layer_row_has_mask(m39RowOf(QStringLiteral("2")))
            && m39Panel->rowHasMaskForTest(QStringLiteral("2"));
        const bool m39BadgeFx =
            m39View->layer_row_has_adjustment(m39RowOf(QStringLiteral("2")))
            && m39Panel->rowHasAdjustmentForTest(QStringLiteral("2"));
        bool m39BadgeClip = m39Panel->rowExpandableForTest(QStringLiteral("1"))
                            && m39Panel->rowExpandableForTest(QStringLiteral("1/2"));
        for (int i = 0; i < m39View->layer_row_count(); ++i) {
            const QString path = m39View->layer_row_path(i);
            m39BadgeClip = m39BadgeClip
                           && m39Panel->rowClippingForTest(path)
                                  == m39View->layer_row_clipping(i)
                           && !m39Panel->rowClipBaseForTest(path);
        }
        std::fprintf(stderr, "pictura self-test: m39_badges mask=%d fx=%d clip=%d\n",
                     m39BadgeMask ? 1 : 0, m39BadgeFx ? 1 : 0, m39BadgeClip ? 1 : 0);
        std::fflush(stderr);
        if (!m39BadgeMask || !m39BadgeFx || !m39BadgeClip) {
            std::fprintf(stderr, "pictura self-test: FAIL: M39 row badges\n");
            return 109;
        }

        // m39_menus (110): the panel and row menus carry exactly the wired
        // commands (no unimplemented entry) and the eight color labels.
        const QStringList m39ExpectedPanel = {
            QStringLiteral("Panel Options…"), QStringLiteral("New Layer"),
            QStringLiteral("New Group"), QStringLiteral("Duplicate Layer(s)"),
            QStringLiteral("Delete Layer(s)"), QStringLiteral("Group Layers"),
            QStringLiteral("Ungroup Layers"), QStringLiteral("Move Layer Up"),
            QStringLiteral("Move Layer Down")};
        const QStringList m39ExpectedRow = {
            QStringLiteral("Rename"), QStringLiteral("New Layer"), QStringLiteral("New Group"),
            QStringLiteral("Duplicate Layer(s)"), QStringLiteral("Delete Layer(s)"),
            QStringLiteral("Group Layers"), QStringLiteral("Ungroup Layers"),
            QStringLiteral("Move Layer Up"), QStringLiteral("Move Layer Down"),
            QStringLiteral("Color Label")};
        const QStringList m39ExpectedColor = {
            QStringLiteral("None"), QStringLiteral("Red"), QStringLiteral("Orange"),
            QStringLiteral("Yellow"), QStringLiteral("Green"), QStringLiteral("Blue"),
            QStringLiteral("Violet"), QStringLiteral("Gray")};
        const bool m39PanelMenuOk = m39Panel->panelMenuTextsForTest() == m39ExpectedPanel;
        const bool m39RowMenuOk = m39Panel->rowMenuTextsForTest() == m39ExpectedRow;
        const bool m39ColorMenuOk = m39Panel->colorLabelTextsForTest() == m39ExpectedColor;
        std::fprintf(stderr, "pictura self-test: m39_menus panel=%d row=%d color=%d\n",
                     m39PanelMenuOk ? 1 : 0, m39RowMenuOk ? 1 : 0, m39ColorMenuOk ? 1 : 0);
        std::fflush(stderr);
        if (!m39PanelMenuOk || !m39RowMenuOk || !m39ColorMenuOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M39 panel/row menus\n");
            return 110;
        }

        // m39_tooltip (111): every row's tooltip is "<name> (<kind>)".
        const QString m39GroupTip = m39Panel->rowToolTipForTest(QStringLiteral("1"));
        const QString m39PixelTip = m39Panel->rowToolTipForTest(QStringLiteral("1/1"));
        const QString m39AdjTip = m39Panel->rowToolTipForTest(QStringLiteral("2"));
        const bool m39TipOk =
            m39GroupTip
                == QStringLiteral("%1 (group)")
                       .arg(m39View->layer_row_name(m39RowOf(QStringLiteral("1"))))
            && m39PixelTip
                   == QStringLiteral("%1 (pixel)")
                          .arg(m39View->layer_row_name(m39RowOf(QStringLiteral("1/1"))))
            && m39AdjTip
                   == QStringLiteral("%1 (adjustment)")
                          .arg(m39View->layer_row_name(m39RowOf(QStringLiteral("2"))));
        std::fprintf(stderr, "pictura self-test: m39_tooltip ok=%d\n", m39TipOk ? 1 : 0);
        std::fflush(stderr);
        if (!m39TipOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M39 row tooltip\n");
            return 111;
        }

        // m39_strip (112): exactly the seven CS6 strip buttons, no Move text
        // buttons (reordering lives in the menus).
        const QStringList m39StripNames = {
            QStringLiteral("layersStripLink"), QStringLiteral("layersStripFx"),
            QStringLiteral("layersStripMask"), QStringLiteral("layersStripFillAdjustment"),
            QStringLiteral("layersStripGroup"), QStringLiteral("layersStripNewLayer"),
            QStringLiteral("layersStripDelete")};
        QWidget* m39Dock = frame.findChild<QWidget*>(QStringLiteral("layersPanel"));
        int m39StripFound = 0;
        for (const QString& name : m39StripNames) {
            if (m39Dock && m39Dock->findChild<QToolButton*>(name)) {
                ++m39StripFound;
            }
        }
        bool m39NoMoveButtons = true;
        if (m39Dock) {
            const QList<QAbstractButton*> m39Buttons =
                m39Dock->findChildren<QAbstractButton*>();
            for (QAbstractButton* button : m39Buttons) {
                if (button->text().contains(QStringLiteral("Move"))) {
                    m39NoMoveButtons = false;
                }
            }
        }
        const bool m39StripOk =
            m39StripFound == m39StripNames.size() && m39NoMoveButtons;
        std::fprintf(stderr, "pictura self-test: m39_strip seven=%d\n", m39StripOk ? 1 : 0);
        std::fflush(stderr);
        if (!m39StripOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M39 action strip\n");
            return 112;
        }
        // M40: the tools panel — one/two columns, the flyout indicator, opening,
        // and keys, Shift-key cycling, the standalone dock, and session v4.
        // Exit codes 113–119.
        auto* m40Toolbox = frame.findChild<pictura::Toolbox*>(QStringLiteral("toolsPanel"));
        const QList<QToolButton*> m40Slots =
            m40Toolbox ? m40Toolbox->slotButtons() : QList<QToolButton*>();

        // m40_columns (113/114): default one column; two columns reflow the 23
        // slots row-major and widen the dock; toggling back restores one column;
        // the colour control and the Screen Mode button stay below the slots in
        // both layouts.
        QCoreApplication::processEvents();
        auto m40PinnedBelow = [m40Toolbox, &m40Slots]() {
            if (!m40Toolbox || m40Slots.isEmpty()) {
                return false;
            }
            int slotBottom = 0;
            for (QToolButton* button : m40Slots) {
                slotBottom = qMax(
                    slotBottom, button->mapTo(m40Toolbox, QPoint(0, button->height())).y());
            }
            auto* fgbg = m40Toolbox->findChild<pictura::ForegroundBackgroundWidget*>();
            auto* screenMode =
                m40Toolbox->findChild<QToolButton*>(QStringLiteral("screenModeButton"));
            return fgbg != nullptr && screenMode != nullptr
                   && fgbg->mapTo(m40Toolbox, QPoint(0, 0)).y() > slotBottom
                   && screenMode->mapTo(m40Toolbox, QPoint(0, 0)).y() > slotBottom;
        };
        const bool m40Default =
            m40Toolbox != nullptr && m40Toolbox->columns() == 1 && m40Slots.size() == 23;
        const int m40Min1 = m40Toolbox ? m40Toolbox->minimumWidth() : 0;
        const int m40Width1 = m40Toolbox ? m40Toolbox->width() : 0;
        const bool m40Pinned1 = m40PinnedBelow();
        if (m40Toolbox) {
            m40Toolbox->setColumns(2);
        }
        QCoreApplication::processEvents();
        const int m40Min2 = m40Toolbox ? m40Toolbox->minimumWidth() : 0;
        const int m40Width2 = m40Toolbox ? m40Toolbox->width() : 0;
        bool m40RowMajor = m40Slots.size() == 23;
        for (int i = 0; i + 2 < m40Slots.size() && m40RowMajor; ++i) {
            const QPoint a = m40Slots.at(i)->mapTo(m40Toolbox, QPoint(0, 0));
            const QPoint b = m40Slots.at(i + 2)->mapTo(m40Toolbox, QPoint(0, 0));
            if (a.x() != b.x() || a.y() >= b.y()) {
                m40RowMajor = false;
            }
        }
        for (int i = 0; i + 1 < m40Slots.size() && m40RowMajor; i += 2) {
            const QPoint a = m40Slots.at(i)->mapTo(m40Toolbox, QPoint(0, 0));
            const QPoint b = m40Slots.at(i + 1)->mapTo(m40Toolbox, QPoint(0, 0));
            if (a.y() != b.y() || a.x() >= b.x()) {
                m40RowMajor = false;
            }
        }
        const bool m40Pinned2 = m40PinnedBelow();
        if (m40Toolbox) {
            m40Toolbox->setColumns(1);
        }
        QCoreApplication::processEvents();
        const bool m40Restored = m40Toolbox != nullptr && m40Toolbox->columns() == 1;
        const bool m40Widened = m40Min2 > m40Min1 && m40Width2 >= m40Width1;
        const bool m40ColumnsOk = m40Default && m40Restored && m40RowMajor && m40Widened;
        const bool m40PinnedOk = m40Pinned1 && m40Pinned2;
        std::fprintf(stderr, "pictura self-test: m40_columns default=%d two=%d pinned=%d\n",
                     m40Default ? 1 : 0,
                     (m40Restored && m40RowMajor && m40Widened) ? 1 : 0,
                     m40PinnedOk ? 1 : 0);
        std::fflush(stderr);
        if (!m40ColumnsOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M40 column layout\n");
            return 113;
        }
        if (!m40PinnedOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M40 pinned controls\n");
            return 114;
        }

        // m40_flyout (115): a multi-member group shows the triangle and opens a
        // menu of its members at the button's bottom edge; a single-member group
        // shows no triangle.
        const bool m40TriMulti = m40Toolbox && m40Toolbox->hasFlyoutTriangleForTest(2);
        const bool m40SingleTri = m40Toolbox && m40Toolbox->hasFlyoutTriangleForTest(1);
        const bool m40TriSingle = m40Toolbox && !m40SingleTri;
        bool m40MenuActions = false;
        if (m40Toolbox) {
            const QList<QAction*> actions = m40Toolbox->slotMenuActionsForTest(2);
            QList<pictura::ToolId> members;
            for (pictura::ToolId id : pictura::allToolIds()) {
                if (pictura::toolInfo(id).group == 2) {
                    members << id;
                }
            }
            m40MenuActions = actions.size() == members.size() && !members.isEmpty();
            for (int i = 0; m40MenuActions && i < members.size(); ++i) {
                const pictura::ToolInfo& info = pictura::toolInfo(members.at(i));
                if (actions.at(i)->text() != QString::fromLatin1(info.label)
                    || actions.at(i)->isEnabled() != info.implemented) {
                    m40MenuActions = false;
                }
            }
        }
        bool m40Below = false;
        if (m40Toolbox) {
            m40Toolbox->openSlotFlyoutForTest(2);
            QCoreApplication::processEvents();
            QMenu* menu = m40Toolbox->slotMenuForTest(2);
            QToolButton* button = m40Slots.value(1);
            if (menu && button && menu->isVisible()) {
                m40Below = menu->pos().y()
                           == button->mapToGlobal(QPoint(0, button->height())).y();
            }
            if (menu) {
                menu->close();
                QCoreApplication::processEvents();
            }
        }
        const bool m40FlyoutOk = m40TriMulti && m40TriSingle && m40MenuActions && m40Below;
        std::fprintf(stderr, "pictura self-test: m40_flyout tri=%d/%d menu=%d below=%d\n",
                     m40TriMulti ? 1 : 0,
                     m40SingleTri ? 1 : 0,
                     m40MenuActions ? 1 : 0,
                     m40Below ? 1 : 0);
        std::fflush(stderr);
        if (!m40FlyoutOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M40 flyout\n");
            return 115;
        }

        // m40_keys (116): every flyout action carries the group's letter, visible
        // in the context menu; a disabled member stays disabled, keeps its tooltip
        // and still shows the key.
        bool m40KeysShown = m40Toolbox != nullptr;
        bool m40KeysDisabled = false;
        if (m40Toolbox) {
            const QList<QAction*> actions = m40Toolbox->slotMenuActionsForTest(8);
            for (QAction* action : actions) {
                const bool keyShown =
                    action->shortcut() == QKeySequence(QStringLiteral("B"))
                    && action->isShortcutVisibleInContextMenu();
                if (!keyShown) {
                    m40KeysShown = false;
                }
                if (!action->isEnabled()) {
                    const bool disabledOk =
                        action->shortcut() == QKeySequence(QStringLiteral("B"))
                        && action->toolTip().contains(QStringLiteral("not implemented yet"));
                    m40KeysDisabled = m40KeysDisabled || disabledOk;
                    if (!disabledOk) {
                        m40KeysShown = false;
                    }
                }
            }
        }
        const bool m40KeysOk = m40KeysShown && m40KeysDisabled;
        std::fprintf(stderr, "pictura self-test: m40_keys shown=%d disabled=%d\n",
                     m40KeysShown ? 1 : 0,
                     m40KeysDisabled ? 1 : 0);
        std::fflush(stderr);
        if (!m40KeysOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M40 flyout keys\n");
            return 116;
        }

        // m40_shift (117): drive the real QShortcut path by synthesizing a key
        // press on the window. Preference on: the plain letter activates the
        // slot's current member and Shift+letter cycles the implemented members;
        // an all-unimplemented group is a no-op; preference off: the plain letter
        // cycles.
        frame.activateWindow();
        QCoreApplication::processEvents();
        auto m40SendKey = [&frame](int key, Qt::KeyboardModifiers mods, const QString& text) {
            QKeyEvent event(QEvent::KeyPress, key, mods, text);
            QApplication::sendEvent(&frame, &event);
        };
        frame.setActiveTool(pictura::ToolId::Move);
        QCoreApplication::processEvents();
        m40SendKey(Qt::Key_B, Qt::NoModifier, QStringLiteral("b"));
        const bool m40Plain = frame.activeTool() == pictura::ToolId::Brush;
        m40SendKey(Qt::Key_B, Qt::ShiftModifier, QStringLiteral("B"));
        const bool m40Shift = frame.activeTool() == pictura::ToolId::Pencil;
        m40SendKey(Qt::Key_B, Qt::ShiftModifier, QStringLiteral("B"));
        const bool m40Wrap = frame.activeTool() == pictura::ToolId::Brush;
        m40SendKey(Qt::Key_J, Qt::NoModifier, QStringLiteral("j"));
        const bool m40NoImpl = frame.activeTool() == pictura::ToolId::Brush;
        if (m40Toolbox) {
            m40Toolbox->setShiftKeyForToolSwitch(false);
        }
        m40SendKey(Qt::Key_B, Qt::NoModifier, QStringLiteral("b"));
        const bool m40Off = frame.activeTool() == pictura::ToolId::Pencil;
        if (m40Toolbox) {
            m40Toolbox->setShiftKeyForToolSwitch(true);
        }
        const bool m40ShiftOk = m40Plain && m40Shift && m40Wrap && m40NoImpl && m40Off;
        std::fprintf(stderr,
                     "pictura self-test: m40_shift plain=%d shift=%d noimpl=%d off=%d\n",
                     m40Plain ? 1 : 0,
                     (m40Shift && m40Wrap) ? 1 : 0,
                     m40NoImpl ? 1 : 0,
                     m40Off ? 1 : 0);
        std::fflush(stderr);
        if (!m40ShiftOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M40 shift cycling\n");
            return 117;
        }

        // m40_dock (118): M45 T2 restricts the allowed sides to left/right again
        // (M44 had widened them to all four); it stays movable/floatable/closable
        // and carries no tab group. The right-hand panels are no longer docks, so
        // this asserts the Tools dock carries no tab group after the frame's
        // re-dock fallback runs (`tabifiedDockWidgets` is empty without a
        // companion).
        QDockWidget* m40Dock = m40Toolbox;
        const bool m40Areas =
            m40Dock
            && m40Dock->allowedAreas()
                   == (Qt::LeftDockWidgetArea | Qt::RightDockWidgetArea);
        const bool m40Features =
            m40Dock
            && m40Dock->features()
                   == (QDockWidget::DockWidgetMovable | QDockWidget::DockWidgetFloatable
                       | QDockWidget::DockWidgetClosable);
        frame.ensureToolsNotTabified();
        QCoreApplication::processEvents();
        const bool m40NoTab = m40Dock && frame.tabifiedDockWidgets(m40Dock).isEmpty();
        const bool m40DockOk = m40Areas && m40Features && m40NoTab;
        std::fprintf(stderr, "pictura self-test: m40_dock areas=%d feat=%d no_tab=%d\n",
                     m40Areas ? 1 : 0,
                     m40Features ? 1 : 0,
                     m40NoTab ? 1 : 0);
        std::fflush(stderr);
        if (!m40DockOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M40 standalone dock\n");
            return 118;
        }

        // m40_session (119): the two v4 fields round-trip, and a store lacking
        // them loads the defaults (one column, Shift required).
        pictura::SessionState m40State = pictura::loadSession();
        m40State.toolsColumns = 2;
        m40State.useShiftKeyForToolSwitch = false;
        const bool m40Saved = pictura::saveSession(m40State);
        const pictura::SessionState m40Reloaded = pictura::loadSession();
        const bool m40Roundtrip = m40Saved && m40Reloaded.toolsColumns == 2
                                  && !m40Reloaded.useShiftKeyForToolSwitch;
        bool m40Defaults = false;
        {
            QFile store(pictura::sessionFilePath());
            if (store.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
                store.write("{\"schemaVersion\":3,\"brightnessLevel\":2}");
                store.close();
            }
            const pictura::SessionState m40Missing = pictura::loadSession();
            m40Defaults =
                m40Missing.toolsColumns == 1 && m40Missing.useShiftKeyForToolSwitch;
        }
        std::fprintf(stderr, "pictura self-test: m40_session roundtrip=%d defaults=%d\n",
                     m40Roundtrip ? 1 : 0,
                     m40Defaults ? 1 : 0);
        std::fflush(stderr);
        if (!m40Roundtrip || !m40Defaults) {
            std::fprintf(stderr, "pictura self-test: FAIL: M40 session v4\n");
            return 119;
        }
        // M41: the panel column — top tabs, the width toggle, the iconic strip
        // with its Qt::Popup flyout, the seven-item tab menu, minimize vs
        // collapse-to-icons, and the content-fit Tools panel. Exit codes
        // 120–124 and 130.
        auto* m41Column = frame.panelColumn();

        // m41_tabs (120): every group's tabs are North, a single-panel group
        // still has its one tab, and no group-title label widget exists.
        bool m41TabsNorth = m41Column != nullptr;
        bool m41SingleTab = false;
        bool m41NoLabel = true;
        if (m41Column) {
            for (pictura::PanelGroup* group : m41Column->groups()) {
                if (group->tabPositionForTest() != static_cast<int>(QTabWidget::North)) {
                    m41TabsNorth = false;
                }
                if (group->titleCountForTest() == 1
                    && !group->titleTextsForTest().value(0).isEmpty()) {
                    m41SingleTab = true;
                }
                if (group->groupLabelForTest()) {
                    m41NoLabel = false;
                }
            }
        }
        const bool m41TabsOk = m41Column && m41TabsNorth && m41SingleTab && m41NoLabel;
        std::fprintf(stderr, "pictura self-test: m41_tabs tabs=north single=%d nolabel=%d\n",
                     m41SingleTab ? 1 : 0,
                     m41NoLabel ? 1 : 0);
        std::fflush(stderr);
        if (!m41TabsOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M41 top tabs\n");
            return 120;
        }

        // m41_width (121): the `panelColumnToggle` flips normal/iconic, the
        // column has no hard minimum, and it is scrollable.
        QToolButton* m41Toggle = m41Column ? m41Column->panelColumnToggleForTest() : nullptr;
        const bool m41ToggleExists =
            m41Toggle && m41Toggle->objectName() == QStringLiteral("panelColumnToggle");
        const bool m41ModeBefore = m41Column && m41Column->railMode();
        if (m41Toggle) {
            m41Toggle->click();
            QCoreApplication::processEvents();
        }
        const bool m41Flipped = m41Column && m41Column->railMode() != m41ModeBefore;
        if (m41Toggle) {
            m41Toggle->click();
            QCoreApplication::processEvents();
        }
        const bool m41ModeRestored = m41Column && m41Column->railMode() == m41ModeBefore;
        // M42 amends this check to the new normal-mode width floor: the column
        // now enforces a bounded content-derived minimum width (so it cannot be
        // squeezed to nothing) while keeping no height minimum and staying
        // scrollable.
        const int m41MinFloor = m41Column ? m41Column->minimumWidthForTest() : 0;
        const bool m41NoMin = m41Column && m41MinFloor >= 160 && m41MinFloor <= 400
                              && m41Column->minimumHeight() == 0;
        const bool m41Scroll = m41Column && m41Column->scrollableForTest();
        const bool m41WidthOk =
            m41ToggleExists && m41Flipped && m41ModeRestored && m41NoMin && m41Scroll;
        std::fprintf(stderr,
                     "pictura self-test: m41_width toggle=%d modes=%d min=%d scroll=%d\n",
                     m41ToggleExists ? 1 : 0,
                     (m41Flipped && m41ModeRestored) ? 1 : 0,
                     m41NoMin ? 1 : 0,
                     m41Scroll ? 1 : 0);
        std::fflush(stderr);
        if (!m41WidthOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M41 column width toggle\n");
            return 121;
        }

        // m41_iconic (122): iconic mode shows the icon strip with a divider,
        // widening shows the labels, an icon opens a visible Qt::Popup flyout,
        // and closing it restores the panel to its group.
        if (m41Column) {
            m41Column->setRailMode(true);
            QCoreApplication::processEvents();
        }
        const bool m41Strip = m41Column && m41Column->iconStripVisibleForTest();
        const bool m41Dividers = m41Column && m41Column->dividerCountForTest() >= 1;
        // Widening the column past the label threshold shows the panel labels.
        // A temporary minimum forces the splitter to give the column the width.
        if (m41Column) {
            m41Column->setMinimumWidth(320);
            QCoreApplication::processEvents();
        }
        const bool m41Labels = m41Column && m41Column->iconLabelsShownForTest();
        if (m41Column) {
            m41Column->setMinimumWidth(0);
            QCoreApplication::processEvents();
        }
        const bool m41Opened =
            m41Column && m41Column->openIconFlyoutForTest(QStringLiteral("layersPanel"));
        for (int i = 0; i < 20 && !(m41Column && m41Column->iconFlyoutVisibleForTest()); ++i) {
            QCoreApplication::processEvents();
        }
        const bool m41Popup = m41Opened && m41Column && m41Column->iconFlyoutVisibleForTest();
        auto* m41Flyout = m41Column
            ? m41Column->findChild<QWidget*>(QStringLiteral("panelIconFlyout"))
            : nullptr;
        if (m41Flyout) {
            m41Flyout->close();
            QCoreApplication::processEvents();
        }
        pictura::PanelGroup* m41LayerGroup =
            m41Column ? m41Column->groupForPanel(QStringLiteral("layersPanel")) : nullptr;
        const bool m41Restore =
            m41LayerGroup && m41LayerGroup->containsPanel(QStringLiteral("layersPanel"));
        if (m41Column) {
            m41Column->setRailMode(false);
            QCoreApplication::processEvents();
        }
        const bool m41IconicOk = m41Strip && m41Dividers && m41Labels && m41Popup && m41Restore;
        std::fprintf(stderr,
                     "pictura self-test: m41_iconic strip=%d dividers=%d labels=%d popup=%d "
                     "restore=%d\n",
                     m41Strip ? 1 : 0,
                     m41Dividers ? 1 : 0,
                     m41Labels ? 1 : 0,
                     m41Popup ? 1 : 0,
                     m41Restore ? 1 : 0);
        std::fflush(stderr);
        if (!m41IconicOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M41 iconic strip\n");
            return 122;
        }

        // m41_menu (123): the seven tab-menu items in exact order, both
        // checkables toggle, and `Interface Options…` fires the signal.
        const QStringList m41ExpectedMenu = {
            QStringLiteral("Close"),
            QStringLiteral("Close Panel Group"),
            QStringLiteral("Minimize"),
            QStringLiteral("Collapse to Icons"),
            QStringLiteral("Auto-Collapse Iconic Panels"),
            QStringLiteral("Auto-Show Hidden Panels"),
            QStringLiteral("Interface Options\u2026"),
        };
        const QStringList m41Menu =
            m41Column ? m41Column->tabMenuActionsForTest() : QStringList();
        const bool m41MenuOrder = m41Menu == m41ExpectedMenu;
        bool m41Check = m41Column != nullptr;
        if (m41Column) {
            const bool a0 = m41Column->autoCollapseIconicForTest();
            m41Column->triggerTabMenuForTest(QStringLiteral("Auto-Collapse Iconic Panels"));
            const bool a1 = m41Column->autoCollapseIconicForTest();
            m41Column->triggerTabMenuForTest(QStringLiteral("Auto-Collapse Iconic Panels"));
            const bool a2 = m41Column->autoCollapseIconicForTest();
            const bool b0 = m41Column->autoShowHiddenForTest();
            m41Column->triggerTabMenuForTest(QStringLiteral("Auto-Show Hidden Panels"));
            const bool b1 = m41Column->autoShowHiddenForTest();
            m41Column->triggerTabMenuForTest(QStringLiteral("Auto-Show Hidden Panels"));
            const bool b2 = m41Column->autoShowHiddenForTest();
            m41Check = a1 != a0 && a2 == a0 && b1 != b0 && b2 == b0;
        }
        bool m41OptionsFired = false;
        if (m41Column) {
            QObject::connect(m41Column, &pictura::PanelColumn::interfaceOptionsRequested,
                             m41Column, [&m41OptionsFired]() { m41OptionsFired = true; });
            m41Column->triggerTabMenuForTest(QStringLiteral("Interface Options\u2026"));
        }
        const bool m41MenuOk =
            m41Menu.size() == 7 && m41MenuOrder && m41Check && m41OptionsFired;
        std::fprintf(stderr, "pictura self-test: m41_menu count=%d order=%d check=%d\n",
                     m41Menu.size(),
                     m41MenuOrder ? 1 : 0,
                     m41Check ? 1 : 0);
        std::fflush(stderr);
        if (!m41MenuOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M41 tab menu\n");
            return 123;
        }

        // m41_minimize (124): minimize hides the content but keeps the tab bar,
        // restores, and collapse-to-icons is a distinct state.
        pictura::PanelGroup* m41MinGroup =
            m41Column ? m41Column->groupForPanel(QStringLiteral("layersPanel")) : nullptr;
        bool m41Min = false;
        bool m41Content = false;
        bool m41MinRestore = false;
        bool m41Icons = false;
        if (m41MinGroup) {
            m41MinGroup->setMinimizedForTest(true);
            QCoreApplication::processEvents();
            m41Min = m41MinGroup->isMinimizedForTest();
            m41Content = m41MinGroup->contentHiddenForTest()
                         && m41MinGroup->tabBarVisibleForTest();
            m41MinGroup->setMinimizedForTest(false);
            QCoreApplication::processEvents();
            m41MinRestore = !m41MinGroup->isMinimizedForTest()
                            && !m41MinGroup->contentHiddenForTest();
            m41MinGroup->setCollapsedToIconsForTest(true);
            QCoreApplication::processEvents();
            m41Icons = m41MinGroup->isCollapsedToIconsForTest()
                       && !m41MinGroup->isMinimizedForTest();
            m41MinGroup->setCollapsedToIconsForTest(false);
            QCoreApplication::processEvents();
        }
        const bool m41MinimizeOk = m41Min && m41Content && m41MinRestore && m41Icons;
        std::fprintf(stderr,
                     "pictura self-test: m41_minimize min=%d content=%d restore=%d icons=%d\n",
                     m41Min ? 1 : 0,
                     m41Content ? 1 : 0,
                     m41MinRestore ? 1 : 0,
                     m41Icons ? 1 : 0);
        std::fflush(stderr);
        if (!m41MinimizeOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M41 minimize\n");
            return 124;
        }

        // m41_tools (130): no `Tools` title, content-fit widths, and the
        // foreground/background widget fits within the current column width in
        // both column counts.
        auto* m41Toolbox = frame.findChild<pictura::Toolbox*>(QStringLiteral("toolsPanel"));
        const bool m41Title = m41Toolbox && m41Toolbox->titleTextForTest().isEmpty();
        bool m41Min1 = false;
        bool m41Min2 = false;
        bool m41Fit = false;
        if (m41Toolbox) {
            m41Toolbox->setColumns(1);
            QCoreApplication::processEvents();
            const int min1 = m41Toolbox->minimumWidth();
            const int content1 = m41Toolbox->contentWidthForTest();
            const int fg1 = m41Toolbox->foregroundBackgroundWidthForTest();
            const int width1 = m41Toolbox->width();
            m41Toolbox->setColumns(2);
            QCoreApplication::processEvents();
            const int min2 = m41Toolbox->minimumWidth();
            const int content2 = m41Toolbox->contentWidthForTest();
            const int fg2 = m41Toolbox->foregroundBackgroundWidthForTest();
            const int width2 = m41Toolbox->width();
            m41Toolbox->setColumns(1);
            QCoreApplication::processEvents();
            m41Min1 = min1 > 0 && min1 >= content1;
            m41Min2 = min2 > min1 && min2 >= content2;
            m41Fit = fg1 <= width1 && fg2 <= width2 && fg1 <= min1 && fg2 <= min2;
        }
        const bool m41ToolsOk = m41Title && m41Min1 && m41Min2 && m41Fit;
        std::fprintf(stderr,
                     "pictura self-test: m41_tools title=%d min1=%d min2=%d fit=%d\n",
                     m41Title ? 1 : 0,
                     m41Min1 ? 1 : 0,
                     m41Min2 ? 1 : 0,
                     m41Fit ? 1 : 0);
        std::fflush(stderr);
        if (!m41ToolsOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M41 tools panel\n");
            return 130;
        }

        // m41_drag (126): a tab reorders within its group, a tab regroups into
        // another group at the drop index, a drop between groups inserts a new
        // group, and the thick blue indicator is drawn at the candidate
        // position and cleared afterwards. Every step drives the column's real
        // begin/update/commit drag path.
        bool m41DragReorder = false;
        bool m41DragRegroup = false;
        bool m41DragInsert = false;
        bool m41DragIndicator = false;
        bool m41DragClear = false;
        if (m41Column) {
            // Give the column a normal width so the group tab bars are inside
            // the scroll viewport; the default splitter share is icon-strip
            // narrow under xvfb.
            m41Column->setMinimumWidth(360);
            QCoreApplication::processEvents();
            // Reorder: move the second Layers tab to the front of its own bar.
            pictura::PanelGroup* reorderGroup =
                m41Column->groupForPanel(QStringLiteral("layersPanel"));
            if (reorderGroup && reorderGroup->titleCountForTest() >= 2) {
                const QStringList before = reorderGroup->titleTextsForTest();
                const QString moving = reorderGroup->panels().at(1)->objectName();
                m41Column->ensureGroupVisibleForTest(moving);
                QCoreApplication::processEvents();
                const QPoint target = reorderGroup->tabInsertionGlobalPointForTest(0);
                m41Column->beginTabDragForTest(moving);
                m41Column->dragToForTest(target);
                const bool visible = m41Column->dropIndicatorVisibleForTest();
                const QRect geometry = m41Column->dropIndicatorGeometryForTest();
                const bool vertical = geometry.height() > geometry.width() && geometry.height() > 0
                                      && geometry.width() == 3;
                m41Column->dropForTest(target);
                const QStringList after = reorderGroup->titleTextsForTest();
                m41DragReorder = !before.isEmpty() && after != before
                                 && after.value(0) == before.value(1);
                m41DragIndicator = visible && vertical;
                m41DragClear = !m41Column->dropIndicatorVisibleForTest();
            }

            // Regroup: append the Styles tab to the Layers group.
            pictura::PanelGroup* colorGroup =
                m41Column->groupForPanel(QStringLiteral("stylesPanel"));
            pictura::PanelGroup* layerGroup =
                m41Column->groupForPanel(QStringLiteral("layersPanel"));
            if (colorGroup && layerGroup && colorGroup != layerGroup) {
                const int targetIndex = layerGroup->titleCountForTest();
                m41Column->ensureGroupVisibleForTest(QStringLiteral("layersPanel"));
                QCoreApplication::processEvents();
                const QPoint target = layerGroup->tabInsertionGlobalPointForTest(targetIndex);
                m41Column->beginTabDragForTest(QStringLiteral("stylesPanel"));
                m41Column->dragToForTest(target);
                m41Column->dropForTest(target);
                m41DragRegroup =
                    layerGroup->containsPanel(QStringLiteral("stylesPanel"))
                    && !colorGroup->containsPanel(QStringLiteral("stylesPanel"))
                    && m41Column->groupForPanel(QStringLiteral("stylesPanel")) == layerGroup;
            }

            // Insert: drop Histogram into the gap between two groups, which
            // must create a new one-panel group at that boundary.
            pictura::PanelGroup* navGroup =
                m41Column->groupForPanel(QStringLiteral("histogramPanel"));
            if (navGroup && m41Column->groups().size() >= 2) {
                const int groupCount = m41Column->groups().size();
                const QPoint target = m41Column->boundaryPointForTest(1);
                m41Column->beginTabDragForTest(QStringLiteral("histogramPanel"));
                m41Column->dragToForTest(target);
                const bool visible = m41Column->dropIndicatorVisibleForTest();
                const QRect geometry = m41Column->dropIndicatorGeometryForTest();
                const bool horizontal = geometry.width() > geometry.height()
                                        && geometry.height() == 3;
                m41Column->dropForTest(target);
                pictura::PanelGroup* inserted =
                    m41Column->groupForPanel(QStringLiteral("histogramPanel"));
                m41DragInsert = m41Column->groups().size() == groupCount + 1 && inserted
                                && inserted->titleCountForTest() == 1
                                && inserted->containsPanel(QStringLiteral("histogramPanel"));
                m41DragClear = m41DragClear && visible && horizontal
                               && !m41Column->dropIndicatorVisibleForTest();
            }
        }
        const bool m41DragOk = m41DragReorder && m41DragRegroup && m41DragInsert
                               && m41DragIndicator && m41DragClear;
        std::fprintf(stderr,
                     "pictura self-test: m41_drag reorder=%d regroup=%d insert=%d indicator=%d "
                     "clear=%d\n",
                     m41DragReorder ? 1 : 0,
                     m41DragRegroup ? 1 : 0,
                     m41DragInsert ? 1 : 0,
                     m41DragIndicator ? 1 : 0,
                     m41DragClear ? 1 : 0);
        std::fflush(stderr);
        if (!m41DragOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M41 drag and drop\n");
            return 126;
        }

        // m41_tearoff (127): tearing a group off the column creates a visible
        // `panelFloat` holding the group's panels, and re-docking returns them
        // to the column at the drop index and leaves no float behind.
        bool m41TearFloat = false;
        bool m41TearPanels = false;
        bool m41Redock = false;
        bool m41TearEmpty = false;
        if (m41Column) {
            pictura::PanelGroup* source =
                m41Column->groupForPanel(QStringLiteral("histogramPanel"));
            QStringList expected;
            if (source) {
                for (QWidget* panel : source->panels()) {
                    if (panel) {
                        expected << panel->objectName();
                    }
                }
            }
            const int groupCount = m41Column->groups().size();
            m41TearFloat = source
                           && m41Column->tearOffForTest(source->objectName())
                           && m41Column->floatCountForTest() == 1;
            if (m41TearFloat) {
                m41TearPanels = m41Column->floatPanelNamesForTest(0) == expected
                                && !expected.isEmpty()
                                && !m41Column->groupForPanel(QStringLiteral("histogramPanel"));
                const bool redocked = m41Column->redockForTest(0, 1);
                pictura::PanelGroup* back =
                    m41Column->groupForPanel(QStringLiteral("histogramPanel"));
                m41Redock = redocked && back
                            && back->containsPanel(QStringLiteral("histogramPanel"))
                            && m41Column->groups().size() == groupCount;
                m41TearEmpty = m41Column->floatCountForTest() == 0;
            }
        }
        const bool m41TearoffOk = m41TearFloat && m41TearPanels && m41Redock && m41TearEmpty;
        std::fprintf(stderr,
                     "pictura self-test: m41_tearoff float=%d panels=%d redock=%d empty=%d\n",
                     m41TearFloat ? 1 : 0,
                     m41TearPanels ? 1 : 0,
                     m41Redock ? 1 : 0,
                     m41TearEmpty ? 1 : 0);
        std::fflush(stderr);
        if (!m41TearoffOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M41 tear-off and re-dock\n");
            return 127;
        }
        if (m41Column) {
            m41Column->setMinimumWidth(0);
            QCoreApplication::processEvents();
        }

        // m41_prefs (125): the Preferences dialog has exactly General and
        // Interface, opens from the command path and from Interface Options…,
        // and its checkboxes drive the toolbox and the column and persist.
        pictura::PreferencesDialog* m41Prefs = nullptr;
        bool m41PrefsPages = false;
        bool m41PrefsOpen = false;
        bool m41PrefsRoundtrip = false;
        bool m41PrefsShift = false;
        bool m41PrefsIconic = false;
        if (QAction* general = frame.registry()->action(
                QString::fromLatin1(pictura::command_ids::EditPreferencesGeneral))) {
            general->trigger();
            QCoreApplication::processEvents();
        }
        m41Prefs = frame.preferencesDialog();
        const bool m41OpenedGeneral =
            m41Prefs && m41Prefs->isVisible()
            && m41Prefs->currentPageForTest() == QStringLiteral("General");
        if (m41Column) {
            m41Column->triggerTabMenuForTest(QStringLiteral("Interface Options\u2026"));
            QCoreApplication::processEvents();
        }
        const bool m41OpenedInterface =
            m41Prefs && m41Prefs->isVisible()
            && m41Prefs->currentPageForTest() == QStringLiteral("Interface");
        m41PrefsPages =
            m41Prefs
            && m41Prefs->pagesForTest()
                   == QStringList({QStringLiteral("General"), QStringLiteral("Interface")});
        m41PrefsOpen = m41OpenedGeneral && m41OpenedInterface;
        if (m41Prefs) {
            auto* prefsToolbox = frame.findChild<pictura::Toolbox*>(QStringLiteral("toolsPanel"));
            const bool shiftToggled = m41Prefs->setCheckboxForTest(
                QStringLiteral("useShiftKeyForToolSwitch"), false);
            QCoreApplication::processEvents();
            m41PrefsShift = shiftToggled && prefsToolbox
                            && !prefsToolbox->shiftKeyForToolSwitch();
            frame.saveSession();
            m41PrefsRoundtrip = !pictura::loadSession().useShiftKeyForToolSwitch;
            m41Prefs->setCheckboxForTest(QStringLiteral("useShiftKeyForToolSwitch"), true);
            QCoreApplication::processEvents();
            frame.saveSession();
            m41Prefs->setCheckboxForTest(QStringLiteral("autoCollapseIconic"), true);
            QCoreApplication::processEvents();
            m41PrefsIconic = m41Column && m41Column->autoCollapseIconicForTest();
            m41Prefs->setCheckboxForTest(QStringLiteral("autoCollapseIconic"), false);
            QCoreApplication::processEvents();
            m41Prefs->close();
        }
        const bool m41PrefsOk = m41PrefsPages && m41PrefsOpen && m41PrefsRoundtrip
                                && m41PrefsShift && m41PrefsIconic;
        std::fprintf(stderr,
                     "pictura self-test: m41_prefs pages=%d open=%d roundtrip=%d shift=%d "
                     "iconic=%d\n",
                     m41Prefs ? m41Prefs->pagesForTest().size() : 0,
                     m41PrefsOpen ? 1 : 0,
                     m41PrefsRoundtrip ? 1 : 0,
                     m41PrefsShift ? 1 : 0,
                     m41PrefsIconic ? 1 : 0);
        std::fflush(stderr);
        if (!m41PrefsOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M41 preferences dialog\n");
            return 125;
        }

        // m41_session (128): the v5 fields round-trip through the real
        // save/load path, a schema-4 store loads the v5 defaults and keeps its
        // own keys, an unknown key survives a rewrite, and per-group panel
        // visibility/order/minimized/collapsed are saved and restored.
        if (m41Column) {
            m41Column->setAutoCollapseIconic(true);
            m41Column->setAutoShowHidden(true);
            m41Column->setRailMode(true);
        }
        frame.saveSession();
        pictura::SessionState m41Reloaded = pictura::loadSession();
        pictura::SessionState m41WidthState = pictura::loadSession();
        m41WidthState.panelRailMode = QStringLiteral("normal");
        m41WidthState.railWidth = 260;
        pictura::saveSession(m41WidthState);
        const bool m41WidthRound = pictura::loadSession().railWidth == 260;
        const bool m41V5 = !m41Reloaded.panelGroups.isEmpty()
                           && m41Reloaded.schemaVersion >= 5
                           && m41Reloaded.panelRailMode == QStringLiteral("iconic")
                           && m41Reloaded.autoCollapseIconic && m41Reloaded.autoShowHidden
                           && m41WidthRound;

        bool m41Defaults = false;
        bool m41V4 = false;
        {
            QFile store(pictura::sessionFilePath());
            if (store.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
                store.write("{\"schemaVersion\":4,\"toolsColumns\":2,"
                            "\"useShiftKeyForToolSwitch\":false}");
                store.close();
            }
            const pictura::SessionState v4 = pictura::loadSession();
            m41Defaults = v4.panelRailMode == QStringLiteral("normal") && v4.railWidth == 0
                          && !v4.autoCollapseIconic && !v4.autoShowHidden;
            m41V4 = v4.toolsColumns == 2 && !v4.useShiftKeyForToolSwitch;
        }

        bool m41Unknown = false;
        {
            QFile store(pictura::sessionFilePath());
            if (store.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
                store.write("{\"schemaVersion\":4,\"toolsColumns\":1,"
                            "\"unknownKey\":\"keep-me\"}");
                store.close();
            }
            frame.saveSession();
            QFile read(pictura::sessionFilePath());
            if (read.open(QIODevice::ReadOnly)) {
                const QByteArray rewritten = read.readAll();
                m41Unknown = rewritten.contains("unknownKey")
                             && rewritten.contains("keep-me");
            }
        }

        bool m41Panels = false;
        if (m41Column) {
            pictura::PanelGroup* m41PanelGroup =
                m41Column->groupForPanel(QStringLiteral("layersPanel"));
            if (m41PanelGroup) {
                auto panelNames = [](pictura::PanelGroup* group) {
                    QStringList names;
                    for (QWidget* panel : group->panels()) {
                        if (panel) {
                            names << panel->objectName();
                        }
                    }
                    return names;
                };
                const QJsonArray m41Baseline = m41Column->savePanelState();
                QStringList m41Order = panelNames(m41PanelGroup);
                if (m41Order.size() >= 2) {
                    m41Order.move(0, m41Order.size() - 1);
                    m41PanelGroup->setPanelOrder(m41Order);
                }
                m41PanelGroup->setMinimizedForTest(true);
                m41Column->showPanel(QStringLiteral("channelsPanel"), false);
                QCoreApplication::processEvents();
                const QJsonArray m41Mutated = m41Column->savePanelState();
                m41Column->restorePanelState(m41Baseline);
                const bool m41BackToBaseline =
                    !m41Column->groupForPanel(QStringLiteral("layersPanel"))->isMinimizedForTest()
                    && m41Column->isPanelVisible(QStringLiteral("channelsPanel"));
                m41Column->restorePanelState(m41Mutated);
                pictura::PanelGroup* m41Restored =
                    m41Column->groupForPanel(QStringLiteral("layersPanel"));
                m41Panels = m41BackToBaseline && m41Restored
                            && m41Restored->isMinimizedForTest()
                            && !m41Column->isPanelVisible(QStringLiteral("channelsPanel"))
                            && panelNames(m41Restored) == m41Order;
            }
            m41Column->setAutoCollapseIconic(false);
            m41Column->setAutoShowHidden(false);
            m41Column->setRailMode(false);
            QCoreApplication::processEvents();
        }
        const bool m41SessionOk =
            m41V5 && m41Defaults && m41V4 && m41Unknown && m41Panels;
        std::fprintf(stderr,
                     "pictura self-test: m41_session v5=%d defaults=%d v4=%d unknown=%d "
                     "panels=%d\n",
                     m41V5 ? 1 : 0,
                     m41Defaults ? 1 : 0,
                     m41V4 ? 1 : 0,
                     m41Unknown ? 1 : 0,
                     m41Panels ? 1 : 0);
        std::fflush(stderr);
        if (!m41SessionOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M41 session v5\n");
            return 128;
        }
        // M42 Phase A: the column's normal-mode minimum width and smallest-width
        // compact transition, the bigger strip and tool icons, the fg/bg swap
        // control and `X` key, the menu-bar clearance + stale-layout guard, and
        // the floated Tools dock geometry. Exit codes 131, 132, 137, 138, 139.
        auto* m42Toolbox = frame.findChild<pictura::Toolbox*>(QStringLiteral("toolsPanel"));

        // m42_minwidth (131): normal mode enforces a bounded content-derived
        // minimum width (no height minimum, still scrollable); entering iconic
        // lands at the smallest strip width and leaving restores the prior width.
        int m42NormalMin = 0;
        int m42IconicMin = 0;
        bool m42NormalWidth = false;
        bool m42IconicSmall = false;
        bool m42RestoredWidth = false;
        if (m41Column) {
            m41Column->setRailMode(false);
            QCoreApplication::processEvents();
            m42NormalMin = m41Column->minimumWidthForTest();
            m42NormalWidth = m42NormalMin >= 180 && m42NormalMin <= 400
                             && m41Column->minimumHeight() == 0
                             && m41Column->scrollableForTest();
            const int before = m41Column->width();
            m41Column->setRailMode(true);
            QCoreApplication::processEvents();
            m42IconicMin = m41Column->minimumWidthForTest();
            m42IconicSmall = m42IconicMin > 0 && m42IconicMin < m42NormalMin
                             && m41Column->width() <= m42IconicMin + 8;
            m41Column->setRailMode(false);
            QCoreApplication::processEvents();
            m42RestoredWidth = m41Column->minimumWidthForTest() == m42NormalMin
                               && m41Column->width() >= m42NormalMin
                               && (before <= 0 || m41Column->width() >= before - 4);
        }
        const bool m42MinWidthOk =
            m42NormalWidth && m42IconicSmall && m42RestoredWidth;
        std::fprintf(stderr,
                     "pictura self-test: m42_minwidth normal=%d iconic=%d restored=%d "
                     "min=%d strip=%d\n",
                     m42NormalWidth ? 1 : 0,
                     m42IconicSmall ? 1 : 0,
                     m42RestoredWidth ? 1 : 0,
                     m42NormalMin,
                     m42IconicMin);
        std::fflush(stderr);
        if (!m42MinWidthOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M42 column minimum width\n");
            return 131;
        }

        // m42_iconic (132): the iconic-strip buttons and their pixmap are larger
        // than M41 (24 button / 16 pixmap), and iconic mode reports a strip
        // narrower than the normal-mode floor. Active/pressed look and flyout are
        // Phase C and are not asserted here.
        bool m42IconBig = false;
        bool m42IconSmallest = false;
        if (m41Column) {
            m41Column->setRailMode(true);
            QCoreApplication::processEvents();
            QToolButton* stripButton =
                m41Column->findChild<QToolButton*>(QStringLiteral("panelIcon_layersPanel"));
            if (!stripButton) {
                for (QToolButton* candidate : m41Column->findChildren<QToolButton*>()) {
                    if (candidate->objectName().startsWith(QStringLiteral("panelIcon_"))) {
                        stripButton = candidate;
                        break;
                    }
                }
            }
            m42IconBig = stripButton && stripButton->minimumWidth() >= 28
                         && stripButton->iconSize().width() >= 20;
            m42IconSmallest = m41Column->minimumWidthForTest() < 180
                              && m41Column->iconStripVisibleForTest();
            m41Column->setRailMode(false);
            QCoreApplication::processEvents();
        }
        const bool m42IconicOk = m42IconBig && m42IconSmallest;
        std::fprintf(stderr, "pictura self-test: m42_iconic bigger=%d smallest=%d\n",
                     m42IconBig ? 1 : 0,
                     m42IconSmallest ? 1 : 0);
        std::fflush(stderr);
        if (!m42IconicOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M42 iconic strip\n");
            return 132;
        }

        // m42_fgbg (137): the swap control exists as a real hit target, clicking
        // it exchanges fg/bg, and the `X` key swaps too (`X` is unassigned in the
        // tool letter catalogue).
        bool m42SwapControl = false;
        bool m42SwapClick = false;
        bool m42SwapKey = false;
        if (m42Toolbox) {
            auto* fgbg = m42Toolbox->foregroundBackgroundForTest();
            if (fgbg) {
                const QColor fg0 = fgbg->foregroundForTest();
                const QColor bg0 = fgbg->backgroundForTest();
                const int count0 = fgbg->swapCountForTest();
                const QRect swap = fgbg->swapRectForTest();
                m42SwapControl = swap.isValid() && swap.width() > 0 && swap.height() > 0
                                 && fgbg->rect().contains(swap);
                const QPoint local = swap.center();
                QMouseEvent press(QEvent::MouseButtonPress, QPointF(local),
                                  QPointF(fgbg->mapToGlobal(local)), Qt::LeftButton,
                                  Qt::LeftButton, Qt::NoModifier);
                QApplication::sendEvent(fgbg, &press);
                QCoreApplication::processEvents();
                m42SwapClick = fgbg->swapCountForTest() == count0 + 1
                               && fgbg->foregroundForTest() == bg0
                               && fgbg->backgroundForTest() == fg0;
                const int count1 = fgbg->swapCountForTest();
                const QColor fg1 = fgbg->foregroundForTest();
                const QColor bg1 = fgbg->backgroundForTest();
                frame.activateWindow();
                QCoreApplication::processEvents();
                QKeyEvent xEvent(QEvent::KeyPress, Qt::Key_X, Qt::NoModifier,
                                 QStringLiteral("x"));
                QApplication::sendEvent(&frame, &xEvent);
                QCoreApplication::processEvents();
                m42SwapKey = fgbg->swapCountForTest() == count1 + 1
                             && fgbg->foregroundForTest() == bg1
                             && fgbg->backgroundForTest() == fg1;
            }
        }
        const bool m42FgbgOk = m42SwapControl && m42SwapClick && m42SwapKey;
        std::fprintf(stderr,
                     "pictura self-test: m42_fgbg control=%d click=%d key=%d\n",
                     m42SwapControl ? 1 : 0,
                     m42SwapClick ? 1 : 0,
                     m42SwapKey ? 1 : 0);
        std::fflush(stderr);
        if (!m42FgbgOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M42 fg/bg swap\n");
            return 137;
        }

        // m42_menubar (138): no visible child widget's global geometry overlaps
        // the menu-bar rect, and a persisted layout from another chrome revision
        // is discarded instead of restored.
        bool m42MenuClear = frame.menuBar() != nullptr;
        QString m42MenuOffender;
        if (QMenuBar* bar = frame.menuBar()) {
            QCoreApplication::processEvents();
            const QRect barRect(bar->mapToGlobal(QPoint(0, 0)), bar->size());
            for (QWidget* child : frame.findChildren<QWidget*>()) {
                if (!child || child == bar || !child->isVisible()
                    || bar->isAncestorOf(child) || child->window() != &frame) {
                    continue;
                }
                const QRect childRect(child->mapToGlobal(QPoint(0, 0)), child->size());
                if (childRect.intersects(barRect)) {
                    m42MenuClear = false;
                    if (m42MenuOffender.isEmpty()) {
                        m42MenuOffender = child->objectName().isEmpty()
                                              ? QString::fromLatin1(
                                                    child->metaObject()->className())
                                              : child->objectName();
                    }
                }
            }
        }
        const bool m42StaleDiscarded =
            !frame.restoreStoredLayout(QByteArrayLiteral("stale-layout"),
                                       frame.layoutRevisionForTest() - 1);
        frame.saveSession();
        const bool m42RevisionSaved =
            pictura::loadSession().layoutRevision == frame.layoutRevisionForTest();
        const bool m42MenubarOk = m42MenuClear && m42StaleDiscarded && m42RevisionSaved;
        std::fprintf(stderr, "pictura self-test: m42_menubar clear=%d stale=%d rev=%d\n",
                     m42MenuClear ? 1 : 0,
                     m42StaleDiscarded ? 1 : 0,
                     m42RevisionSaved ? 1 : 0);
        std::fflush(stderr);
        if (!m42MenubarOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M42 menu-bar overlay=%s\n",
                         m42MenuOffender.toLocal8Bit().constData());
            return 138;
        }

        // m42_tools (139): the slot/screen-mode icons are larger than M40 (30
        // button / 20 pixmap), the one- and two-column minimum widths equal the
        // content width exactly, and a floated dock's body hugs its content
        // height (stretch 0, no leftover vertical space).
        bool m42ToolIcons = false;
        bool m42ToolTight = false;
        bool m42ToolFloat = false;
        int m42ToolMin1 = 0;
        int m42ToolContent1 = 0;
        int m42ToolMin2 = 0;
        int m42ToolContent2 = 0;
        if (m42Toolbox) {
            m42Toolbox->setColumns(1);
            QCoreApplication::processEvents();
            const QList<QToolButton*> m42Slots = m42Toolbox->slotButtons();
            m42ToolIcons = !m42Slots.isEmpty();
            for (QToolButton* button : m42Slots) {
                if (button->minimumWidth() < 32 || button->iconSize().width() < 22) {
                    m42ToolIcons = false;
                }
            }
            auto* screenMode =
                m42Toolbox->findChild<QToolButton*>(QStringLiteral("screenModeButton"));
            if (screenMode && !screenMode->icon().isNull()
                && (screenMode->minimumWidth() < 32
                    || screenMode->iconSize().width() < 22)) {
                m42ToolIcons = false;
            }

            m42ToolMin1 = m42Toolbox->minimumWidth();
            m42ToolContent1 = m42Toolbox->contentWidthForTest();
            m42Toolbox->setColumns(2);
            QCoreApplication::processEvents();
            m42ToolMin2 = m42Toolbox->minimumWidth();
            m42ToolContent2 = m42Toolbox->contentWidthForTest();
            m42ToolTight = m42ToolMin1 > 0 && m42ToolMin1 == m42ToolContent1
                           && m42ToolMin2 == m42ToolContent2 && m42ToolMin2 > m42ToolMin1;

            if (!m42Toolbox->isFloating()) {
                m42Toolbox->setFloating(true);
                QCoreApplication::processEvents();
                QCoreApplication::processEvents();
                const int bodyHeight = m42Toolbox->bodyHeightForTest();
                const int hintHeight = m42Toolbox->bodySizeHintHeightForTest();
                m42ToolFloat = m42Toolbox->bodyStretchForTest() == 0
                               && hintHeight > 0 && bodyHeight <= hintHeight + 8;
                m42Toolbox->setFloating(false);
                QCoreApplication::processEvents();
            }
            m42Toolbox->setColumns(1);
            QCoreApplication::processEvents();
        }
        const bool m42ToolsOk = m42ToolIcons && m42ToolTight && m42ToolFloat;
        std::fprintf(stderr,
                     "pictura self-test: m42_tools icons=%d tight=%d float=%d "
                     "min1=%d content1=%d min2=%d content2=%d\n",
                     m42ToolIcons ? 1 : 0,
                     m42ToolTight ? 1 : 0,
                     m42ToolFloat ? 1 : 0,
                     m42ToolMin1,
                     m42ToolContent1,
                     m42ToolMin2,
                     m42ToolContent2);
        std::fflush(stderr);
        if (!m42ToolsOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M42 tools geometry\n");
            return 139;
        }

        // m42_dragstrip (133): a strip icon reorders within the strip and the
        // order persists, and dropping a strip icon on the normal-mode group
        // stack moves the panel into that group. The strip drop indicator is
        // drawn and cleared. Both paths drive the real begin/update/commit drag.
        bool m42StripReorder = false;
        bool m42StripMove = false;
        bool m42StripIndicator = false;
        bool m42StripClear = false;
        if (m41Column) {
            m41Column->setMinimumWidth(360);
            m41Column->setRailMode(true);
            QCoreApplication::processEvents();
            const QStringList m42StripBefore = m41Column->stripOrderForTest();
            int m42StripMoveIndex = -1;
            for (int i = 1; i < m42StripBefore.size(); ++i) {
                const QString sameGroup = m41Column->groupOfForTest(m42StripBefore.at(i));
                if (!sameGroup.isEmpty()
                    && sameGroup == m41Column->groupOfForTest(m42StripBefore.at(i - 1))) {
                    m42StripMoveIndex = i;
                    break;
                }
            }
            if (m42StripMoveIndex > 0) {
                const QString moving = m42StripBefore.at(m42StripMoveIndex);
                const QPoint target =
                    m41Column->stripInsertionPointForTest(m42StripMoveIndex - 1);
                m41Column->beginStripDragForTest(moving);
                QCoreApplication::processEvents();
                m41Column->dragToForTest(target);
                m42StripIndicator =
                    m41Column->dropIndicatorVisibleForTest()
                    && m41Column->stripDropIndexForTest() == m42StripMoveIndex - 1;
                m41Column->dropForTest(target);
                const QStringList after = m41Column->stripOrderForTest();
                m42StripReorder = after != m42StripBefore
                                 && after.value(m42StripMoveIndex - 1) == moving;
                m42StripClear = !m41Column->dropIndicatorVisibleForTest();
            }

            pictura::PanelGroup* m42DestGroup =
                m41Column->groupForPanel(QStringLiteral("layersPanel"));
            if (!m42DestGroup && !m41Column->groups().isEmpty()) {
                m42DestGroup = m41Column->groups().first();
            }
            QString m42DestPanel;
            if (m42DestGroup) {
                for (QWidget* panel : m42DestGroup->visiblePanels()) {
                    if (panel) {
                        m42DestPanel = panel->objectName();
                        break;
                    }
                }
            }
            QString m42SourcePanel;
            for (pictura::PanelGroup* group : m41Column->groups()) {
                if (!group || group == m42DestGroup) {
                    continue;
                }
                for (QWidget* panel : group->visiblePanels()) {
                    if (panel) {
                        m42SourcePanel = panel->objectName();
                        break;
                    }
                }
                if (!m42SourcePanel.isEmpty()) {
                    break;
                }
            }
            if (!m42SourcePanel.isEmpty() && !m42DestPanel.isEmpty()) {
                const bool moved =
                    m41Column->dropStripOnGroupForTest(m42SourcePanel, m42DestPanel);
                m42StripMove = moved
                               && m41Column->groupForPanel(m42SourcePanel) == m42DestGroup
                               && !m41Column->railMode();
            }
            m41Column->setRailMode(false);
            m41Column->setMinimumWidth(0);
            QCoreApplication::processEvents();
        }
        const bool m42DragStripOk =
            m42StripReorder && m42StripMove && m42StripIndicator && m42StripClear;
        std::fprintf(stderr,
                     "pictura self-test: m42_dragstrip reorder=%d move=%d indicator=%d "
                     "clear=%d\n",
                     m42StripReorder ? 1 : 0,
                     m42StripMove ? 1 : 0,
                     m42StripIndicator ? 1 : 0,
                     m42StripClear ? 1 : 0);
        std::fflush(stderr);
        if (!m42DragStripOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M42 strip drag\n");
            return 133;
        }

        // m42_flyout (134): the flyout opens on the inner side of the right-hand
        // column, its header names the panel and carries the close chevron, the
        // open icon renders active, and closing clears the active state.
        bool m42FlyoutSide = false;
        bool m42FlyoutActive = false;
        bool m42FlyoutGroup = false;
        bool m42FlyoutClose = false;
        if (m41Column) {
            m41Column->setRailMode(true);
            QCoreApplication::processEvents();
            m42FlyoutSide = m41Column->flyoutSideForTest() == QStringLiteral("left");
            const bool opened =
                m41Column->openIconFlyoutForTest(QStringLiteral("layersPanel"));
            for (int i = 0; i < 20 && !m41Column->iconFlyoutVisibleForTest(); ++i) {
                QCoreApplication::processEvents();
            }
            m42FlyoutActive = opened && m41Column->iconFlyoutVisibleForTest()
                              && m41Column->activeIconNameForTest()
                                     == QStringLiteral("layersPanel");
            m42FlyoutGroup =
                m41Column->flyoutHeaderTitleForTest() == QStringLiteral("Layers")
                && m41Column->flyoutHeaderCloseForTest();
            const bool closed = m41Column->triggerFlyoutCloseForTest();
            m42FlyoutClose = closed && !m41Column->iconFlyoutVisibleForTest()
                             && m41Column->activeIconNameForTest().isEmpty();
            m41Column->setRailMode(false);
            QCoreApplication::processEvents();
        }
        const bool m42FlyoutOk =
            m42FlyoutSide && m42FlyoutActive && m42FlyoutGroup && m42FlyoutClose;
        std::fprintf(stderr,
                     "pictura self-test: m42_flyout side=%d active=%d group=%d close=%d\n",
                     m42FlyoutSide ? 1 : 0,
                     m42FlyoutActive ? 1 : 0,
                     m42FlyoutGroup ? 1 : 0,
                     m42FlyoutClose ? 1 : 0);
        std::fflush(stderr);
        if (!m42FlyoutOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M42 compact flyout\n");
            return 134;
        }

        // m42_widgetmenu (135): each group's tab header carries a per-widget
        // action button at its right; the menu follows the current tab and is
        // per-panel, not per-group; unimplemented entries are disabled with the
        // "<label> — not implemented yet" tooltip; `Close`/`Close Panel Group`
        // stay off it; the Layers `Panel Options…` entry is enabled and wired.
        bool m42WmButton = false;
        bool m42WmPerPanel = false;
        bool m42WmDisabled = false;
        bool m42WmNoClose = false;
        if (m41Column) {
            m41Column->setRailMode(false);
            m41Column->ensureGroupVisibleForTest(QStringLiteral("layersPanel"));
            QCoreApplication::processEvents();
            pictura::PanelGroup* m42WmLayerGroup =
                m41Column->groupForPanel(QStringLiteral("layersPanel"));
            if (m42WmLayerGroup) {
                m41Column->showPanel(QStringLiteral("layersPanel"), true);
                QCoreApplication::processEvents();
                QToolButton* m42WmBtnPtr = m42WmLayerGroup->headerMenuButtonForTest();
                QToolButton* m42WmHooked = m41Column->widgetMenuButtonForTest(
                    m42WmLayerGroup->objectName());
                m42WmButton = m42WmBtnPtr && m42WmHooked == m42WmBtnPtr
                              && m42WmBtnPtr->isVisible()
                              && m42WmLayerGroup->headerMenuAtRightForTest();

                const QStringList m42WmLayerTexts =
                    m42WmLayerGroup->panelMenuTextsForTest();
                m41Column->showPanel(QStringLiteral("channelsPanel"), true);
                QCoreApplication::processEvents();
                const QStringList m42WmChannelTexts =
                    m42WmLayerGroup->panelMenuTextsForTest();
                const bool m42WmFollows =
                    m42WmLayerGroup->headerMenuButtonForTest()
                    && m42WmLayerGroup->headerMenuButtonForTest()->objectName()
                           == QStringLiteral("panelWidgetMenu_channelsPanel");
                const QStringList m42WmColorTexts =
                    m41Column->widgetMenuTextsForTest(QStringLiteral("colorPanel"));
                const bool m42WmFirstsDiffer = !m42WmLayerTexts.isEmpty()
                                               && !m42WmChannelTexts.isEmpty()
                                               && !m42WmColorTexts.isEmpty()
                                               && m42WmLayerTexts.first()
                                                      != m42WmChannelTexts.first()
                                               && m42WmLayerTexts.first()
                                                      != m42WmColorTexts.first();
                m41Column->showPanel(QStringLiteral("layersPanel"), true);
                QCoreApplication::processEvents();
                const bool m42WmOptions =
                    m42WmLayerGroup->panelMenuEnabledForTest(QStringLiteral("Panel Options…"));
                m42WmPerPanel = m42WmFollows && m42WmFirstsDiffer && m42WmOptions;

                const bool m42WmCopyDisabled =
                    !m42WmLayerGroup->panelMenuEnabledForTest(QStringLiteral("Copy CSS"));
                const bool m42WmBlendDisabled = !m42WmLayerGroup->panelMenuEnabledForTest(
                    QStringLiteral("Blending Options…"));
                const bool m42WmTriggerBlocked = !m41Column->triggerWidgetMenuForTest(
                    QStringLiteral("layersPanel"), QStringLiteral("Copy CSS"));
                m42WmDisabled =
                    m42WmCopyDisabled && m42WmBlendDisabled && m42WmTriggerBlocked
                    && m42WmLayerGroup->panelMenuToolTipForTest(QStringLiteral("Copy CSS"))
                           == QStringLiteral("Copy CSS — not implemented yet");
            }
            m42WmNoClose = m41Column->widgetMenuHasCloseForTest(QStringLiteral("layersPanel"))
                           && m41Column->widgetMenuHasCloseForTest(
                               QStringLiteral("channelsPanel"))
                           && m41Column->widgetMenuHasCloseForTest(
                               QStringLiteral("colorPanel"))
                           && m41Column->widgetMenuHasCloseForTest(
                               QStringLiteral("historyPanel"));
        }
        const bool m42WmOk = m42WmButton && m42WmPerPanel && m42WmDisabled && m42WmNoClose;
        std::fprintf(stderr,
                     "pictura self-test: m42_widgetmenu button=%d perpanel=%d disabled=%d "
                     "noclose=%d\n",
                     m42WmButton ? 1 : 0,
                     m42WmPerPanel ? 1 : 0,
                     m42WmDisabled ? 1 : 0,
                     m42WmNoClose ? 1 : 0);
        std::fflush(stderr);
        if (!m42WmOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M42 per-widget menu\n");
            return 135;
        }

        // m42_float_overlay (136): a torn-off group is an in-window child
        // overlay (never a top-level window), a move toward or past the main
        // window edge is clamped inside the central area, and the group
        // re-docks and the overlay disappears.
        bool m42FloatChild = false;
        bool m42FloatClamped = false;
        bool m42FloatMove = false;
        bool m42FloatRedock = false;
        if (m41Column) {
            m41Column->setRailMode(false);
            QCoreApplication::processEvents();
            pictura::PanelGroup* m42FloatGroup =
                m41Column->groupForPanel(QStringLiteral("colorPanel"));
            if (!m42FloatGroup) {
                m42FloatGroup = m41Column->groupForPanel(QStringLiteral("layersPanel"));
            }
            if (!m42FloatGroup && !m41Column->groups().isEmpty()) {
                m42FloatGroup = m41Column->groups().first();
            }
            QString m42FloatPanel;
            if (m42FloatGroup) {
                for (QWidget* panel : m42FloatGroup->panels()) {
                    if (panel) {
                        m42FloatPanel = panel->objectName();
                        break;
                    }
                }
            }
            if (!m42FloatPanel.isEmpty()) {
                const bool m42FloatTore = m41Column->tearOffForTest(m42FloatPanel);
                for (int i = 0; i < 20; ++i) {
                    QCoreApplication::processEvents();
                }
                m42FloatChild = m42FloatTore && m41Column->floatCountForTest() == 1
                                && !m41Column->floatIsWindowForTest(0)
                                && !m41Column->groupForPanel(m42FloatPanel);
                if (m42FloatChild) {
                    const QRect m42FloatHost = m41Column->floatHostRectForTest();
                    const QRect m42FloatBefore = m41Column->floatGeometryForTest(0);
                    const bool m42FloatPastBr = m41Column->floatClampedForTest(
                        0, m42FloatHost.bottomRight() + QPoint(400, 400));
                    const QRect m42FloatAfterBr = m41Column->floatGeometryForTest(0);
                    const bool m42FloatPastTl = m41Column->floatClampedForTest(
                        0, m42FloatHost.topLeft() - QPoint(400, 400));
                    const QRect m42FloatAfterTl = m41Column->floatGeometryForTest(0);
                    m42FloatClamped = m42FloatPastBr && m42FloatPastTl;
                    m42FloatMove = m42FloatAfterBr.topLeft() != m42FloatBefore.topLeft()
                                   && m42FloatAfterTl.topLeft() != m42FloatAfterBr.topLeft();
                    if (m42FloatClamped) {
                        const bool m42FloatRedocked = m41Column->redockForTest(0, 1);
                        for (int i = 0; i < 20; ++i) {
                            QCoreApplication::processEvents();
                        }
                        m42FloatRedock = m42FloatRedocked
                                         && m41Column->floatCountForTest() == 0
                                         && m41Column->groupForPanel(m42FloatPanel);
                    }
                }
            }
        }
        const bool m42FloatOk =
            m42FloatChild && m42FloatClamped && m42FloatMove && m42FloatRedock;
        std::fprintf(stderr,
                     "pictura self-test: m42_float_overlay child=%d clipped=%d move=%d "
                     "redock=%d\n",
                     m42FloatChild ? 1 : 0,
                     m42FloatClamped ? 1 : 0,
                     m42FloatMove ? 1 : 0,
                     m42FloatRedock ? 1 : 0);
        std::fflush(stderr);
        if (!m42FloatOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M42 in-window float overlay\n");
            return 136;
        }
        // M43 Phase A: tab-vs-group drag + one-panel float (140), panel tab
        // colours (141), corner button at minimum width (142), one-panel float
        // re-dock (146), Tools fixed width (147), compact icon size (148),
        // strict inner-side flyout (149), and the `D` default-colours reset
        // (150). Exit codes 140-150.
        auto m43Pump = [](int n) {
            for (int i = 0; i < n; ++i) {
                QCoreApplication::processEvents();
            }
        };
        auto m43DropPanelOnGroup = [&](const QString& panel, const QString& targetPanel) {
            if (!m41Column) {
                return false;
            }
            pictura::PanelGroup* dest = m41Column->groupForPanel(targetPanel);
            if (!dest) {
                return false;
            }
            m41Column->ensureGroupVisibleForTest(targetPanel);
            m43Pump(4);
            if (!m41Column->beginTabDragForTest(panel)) {
                return false;
            }
            const QPoint target = dest->tabInsertionGlobalPointForTest(0);
            m41Column->dragToForTest(target);
            const bool dropped = m41Column->dropForTest(target);
            m43Pump(4);
            return dropped;
        };

        // m43_tabdrag (140): a tab drag that leaves the column floats only that
        // panel; an empty-header drag floats the whole group; and a tab drag out
        // of a float moves only that panel, leaving the rest in the old float.
        bool m43TabPanel = false;
        bool m43GroupPanel = false;
        bool m43FloatPanel = false;
        if (m41Column) {
            m41Column->setRailMode(false);
            m41Column->setMinimumWidth(360);
            m41Column->setPreferredWidth(360);
            m43Pump(6);

            pictura::PanelGroup* layersGroup =
                m41Column->groupForPanel(QStringLiteral("layersPanel"));
            if (layersGroup && layersGroup->titleCountForTest() >= 3) {
                const int layersBefore = layersGroup->titleCountForTest();
                const int before = m41Column->floatCountForTest();
                const bool tore =
                    m41Column->tearOffPanelForTest(QStringLiteral("channelsPanel"));
                m43Pump(4);
                const QStringList names =
                    m41Column->floatPanelNamesForTest(m41Column->floatCountForTest() - 1);
                m43TabPanel = tore && m41Column->floatCountForTest() == before + 1
                              && names.size() == 1
                              && names.first() == QStringLiteral("channelsPanel")
                              && layersGroup->titleCountForTest() == layersBefore - 1
                              && layersGroup->containsPanel(QStringLiteral("layersPanel"))
                              && !layersGroup->containsPanel(QStringLiteral("channelsPanel"));
                if (m43TabPanel) {
                    m43DropPanelOnGroup(QStringLiteral("channelsPanel"),
                                        QStringLiteral("layersPanel"));
                }
            }

            QStringList layersPanels;
            pictura::PanelGroup* groupFloatGroup =
                m41Column->groupForPanel(QStringLiteral("layersPanel"));
            if (groupFloatGroup) {
                for (QWidget* panel : groupFloatGroup->panels()) {
                    if (panel) {
                        layersPanels << panel->objectName();
                    }
                }
            }
            const int beforeGroup = m41Column->floatCountForTest();
            const bool groupTore = !layersPanels.isEmpty() && layersPanels.size() >= 2
                                   && m41Column->tearOffForTest(QStringLiteral("layersPanel"));
            m43Pump(4);
            const QStringList groupNames =
                m41Column->floatPanelNamesForTest(m41Column->floatCountForTest() - 1);
            m43GroupPanel = groupTore && m41Column->floatCountForTest() == beforeGroup + 1
                            && groupNames == layersPanels
                            && !m41Column->groupForPanel(QStringLiteral("layersPanel"));

            if (m43GroupPanel) {
                const int beforeFloat = m41Column->floatCountForTest();
                const bool tore = m41Column->beginTabDragForTest(QStringLiteral("channelsPanel"));
                const QPoint outside =
                    m41Column->mapToGlobal(QPoint(-40, m41Column->height() / 2));
                m41Column->dragToForTest(outside);
                m41Column->dropForTest(outside);
                m43Pump(4);
                const int after = m41Column->floatCountForTest();
                const QStringList newest = m41Column->floatPanelNamesForTest(after - 1);
                const QStringList original = m41Column->floatPanelNamesForTest(beforeFloat - 1);
                m43FloatPanel = tore && after == beforeFloat + 1 && newest.size() == 1
                                && newest.first() == QStringLiteral("channelsPanel")
                                && original.size() == layersPanels.size() - 1
                                && !original.contains(QStringLiteral("channelsPanel"));
            }

            // Clean up so later checks see a docked column.
            for (int i = 0; i < 8 && m41Column->floatCountForTest() > 0; ++i) {
                if (!m41Column->redockForTest(0, 0)) {
                    break;
                }
                m43Pump(4);
            }
        }
        const bool m43TabDragOk = m43TabPanel && m43GroupPanel && m43FloatPanel;
        std::fprintf(stderr, "pictura self-test: m43_tabdrag tab=%d group=%d floatpanel=%d\n",
                     m43TabPanel ? 1 : 0,
                     m43GroupPanel ? 1 : 0,
                     m43FloatPanel ? 1 : 0);
        std::fflush(stderr);
        if (!m43TabDragOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M43 tab vs group drag\n");
            return 140;
        }

        // m43_tabcolors (141): the panel tab bar is named `panelTabBar`; its
        // selected tab uses the widget surface `${window}` colour and its
        // inactive tab uses the darker `${base}` (which differs); the document
        // tab bar is not the scoped one, so it keeps the unscoped rules.
        bool m43ColorsActive = false;
        bool m43ColorsInactive = false;
        bool m43ColorsDiffer = false;
        {
            const QString ss = qApp->styleSheet();
            const QColor base = qApp->palette().color(QPalette::Base);
            const QColor windowColor = qApp->palette().color(QPalette::Window);
            const QString baseHex = base.name(QColor::HexRgb);
            const QString windowHex = windowColor.name(QColor::HexRgb);
            const QString activeRule =
                QStringLiteral("QTabBar#panelTabBar::tab:selected { background: ") + windowHex;
            const QString inactiveRule =
                QStringLiteral("QTabBar#panelTabBar::tab { background: ") + baseHex;
            QTabBar* docBar = frame.findChild<QTabBar*>(QStringLiteral("documentTabBar"));
            const bool docUnscoped =
                docBar && docBar->objectName() != QStringLiteral("panelTabBar");
            pictura::PanelGroup* anyGroup =
                m41Column && !m41Column->groups().isEmpty() ? m41Column->groups().first() : nullptr;
            const bool panelNamed =
                anyGroup && anyGroup->tabBar()
                && anyGroup->tabBar()->objectName() == QStringLiteral("panelTabBar");
            m43ColorsActive = ss.contains(activeRule);
            m43ColorsInactive = ss.contains(inactiveRule);
            m43ColorsDiffer = m43ColorsActive && m43ColorsInactive && baseHex != windowHex
                              && docUnscoped && panelNamed;
        }
        const bool m43TabColorsOk = m43ColorsActive && m43ColorsInactive && m43ColorsDiffer;
        std::fprintf(stderr, "pictura self-test: m43_tabcolors active=%d inactive=%d differ=%d\n",
                     m43ColorsActive ? 1 : 0,
                     m43ColorsInactive ? 1 : 0,
                     m43ColorsDiffer ? 1 : 0);
        std::fflush(stderr);
        if (!m43TabColorsOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M43 panel tab colours\n");
            return 141;
        }

        // m43_corner (142): at the column minimum width the `▾` corner button is
        // fully inside the header row and the tab bar elides instead of forcing
        // width.
        bool m43CornerVisible = false;
        bool m43CornerElide = false;
        if (m41Column) {
            m41Column->setRailMode(false);
            m41Column->showPanel(QStringLiteral("layersPanel"), true);
            // Toggle through iconic mode so `updateMinimumWidth` recomputes the
            // content-derived normal minimum (clearing any test-set width).
            m41Column->setRailMode(true);
            m43Pump(2);
            m41Column->setRailMode(false);
            m43Pump(4);
            pictura::PanelGroup* lg = m41Column->groupForPanel(QStringLiteral("layersPanel"));
            if (lg) {
                m41Column->setPreferredWidth(m41Column->minimumWidthForTest());
                m43Pump(6);
                QToolButton* corner = lg->headerMenuButtonForTest();
                QTabBar* bar = lg->tabBar();
                m43CornerElide = bar && bar->elideMode() == Qt::ElideRight && !bar->expanding();
                m43CornerVisible = corner && corner->isVisible()
                                   && lg->headerMenuAtRightForTest()
                                   && corner->mapToGlobal(QPoint(0, 0)).x()
                                          >= lg->mapToGlobal(QPoint(0, 0)).x();
            }
        }
        const bool m43CornerOk = m43CornerVisible && m43CornerElide;
        std::fprintf(stderr, "pictura self-test: m43_corner visible=%d elide=%d\n",
                     m43CornerVisible ? 1 : 0,
                     m43CornerElide ? 1 : 0);
        std::fflush(stderr);
        if (!m43CornerOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M43 corner button\n");
            return 142;
        }

        // m43_newcolumn (143): a panel dropped at the left or right workspace
        // edge, or beside the Tools dock, allocates a new PanelColumn on that
        // side; moving the panel back into the primary column empties and
        // removes the new column. All drops run the real resolve/commit path.
        bool m43NewLeft = false;
        bool m43NewRight = false;
        bool m43NewTools = false;
        bool m43NewRemoved = false;
        {
            const QString m43NewPanel = QStringLiteral("stylesPanel");
            auto m43TrySide = [&](const QString& side, bool& flag) {
                const int before = frame.panelColumnCountForTest();
                const bool created = frame.newColumnDropForTest(m43NewPanel, side);
                pictura::PanelColumn* destination = frame.columnForPanel(m43NewPanel);
                const bool placed = created && destination && destination != frame.panelColumn()
                                    && frame.panelColumnCountForTest() == before + 1;
                const bool back = frame.dropIntoGroupForTest(m43NewPanel,
                                                             QStringLiteral("colorPanel"), -1);
                const bool removed = back && frame.panelColumnCountForTest() == before;
                flag = placed && removed;
                m43NewRemoved = m43NewRemoved || removed;
            };
            m43TrySide(QStringLiteral("left"), m43NewLeft);
            m43TrySide(QStringLiteral("right"), m43NewRight);
            m43TrySide(QStringLiteral("tools"), m43NewTools);
        }
        const bool m43NewColumnOk = m43NewLeft && m43NewRight && m43NewTools && m43NewRemoved;
        std::fprintf(stderr,
                     "pictura self-test: m43_newcolumn left=%d right=%d tools=%d removed=%d\n",
                     m43NewLeft ? 1 : 0,
                     m43NewRight ? 1 : 0,
                     m43NewTools ? 1 : 0,
                     m43NewRemoved ? 1 : 0);
        std::fflush(stderr);
        if (!m43NewColumnOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M43 new column drop\n");
            return 143;
        }

        // m43_intogroup (144): dropping a panel inside another group inserts it
        // as a tab at the requested index, in normal mode and in compact mode.
        bool m43IntoNormal = false;
        bool m43IntoCompact = false;
        bool m43IntoIndex = false;
        if (m41Column) {
            m41Column->setRailMode(false);
            m43Pump(4);
            // A target group whose second panel is stable at runtime (earlier
            // checks reorder the groups), never the panel's own group.
            QString m43IntoTarget;
            for (pictura::PanelGroup* group : m41Column->groups()) {
                if (group && group->panels().size() >= 2
                    && !group->containsPanel(QStringLiteral("swatchesPanel"))) {
                    m43IntoTarget = group->panels().at(1)->objectName();
                    break;
                }
            }
            if (!m43IntoTarget.isEmpty()) {
                const bool moved = frame.dropIntoGroupForTest(
                    QStringLiteral("swatchesPanel"), m43IntoTarget, 1);
                pictura::PanelGroup* dest = m41Column->groupForPanel(m43IntoTarget);
                const bool atIndex =
                    dest && dest->indexOfPanel(QStringLiteral("swatchesPanel")) == 1;
                m43IntoNormal = moved && atIndex;
                m43IntoIndex = atIndex;
                frame.dropIntoGroupForTest(QStringLiteral("swatchesPanel"),
                                           QStringLiteral("colorPanel"), 1);

                m41Column->setRailMode(true);
                m43Pump(6);
                const bool movedCompact = frame.dropIntoGroupForTest(
                    QStringLiteral("swatchesPanel"), m43IntoTarget);
                pictura::PanelGroup* destCompact =
                    m41Column->groupForPanel(m43IntoTarget);
                const bool atIndexCompact =
                    destCompact
                    && destCompact->indexOfPanel(QStringLiteral("swatchesPanel")) == 1;
                m43IntoCompact = movedCompact && atIndexCompact;
                m43IntoIndex = m43IntoIndex && atIndexCompact;
                m41Column->setRailMode(false);
                m43Pump(4);
                frame.dropIntoGroupForTest(QStringLiteral("swatchesPanel"),
                                           QStringLiteral("colorPanel"), 1);
                m43Pump(4);
            }
        }
        const bool m43IntoGroupOk = m43IntoNormal && m43IntoCompact && m43IntoIndex;
        std::fprintf(stderr, "pictura self-test: m43_intogroup normal=%d compact=%d index=%d\n",
                     m43IntoNormal ? 1 : 0,
                     m43IntoCompact ? 1 : 0,
                     m43IntoIndex ? 1 : 0);
        std::fflush(stderr);
        if (!m43IntoGroupOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M43 into-group drop\n");
            return 144;
        }

        // m43_boundary (145): dropping a panel above or below a group inserts a
        // fresh one-panel group at that boundary, in normal mode and compact.
        bool m43Above = false;
        bool m43Below = false;
        bool m43BoundaryCompact = false;
        if (m41Column) {
            m41Column->setRailMode(false);
            m43Pump(4);
            {
                m43Above = frame.dropBoundaryForTest(
                    QStringLiteral("channelsPanel"), QStringLiteral("navigatorPanel"), true);
                frame.dropIntoGroupForTest(QStringLiteral("channelsPanel"),
                                           QStringLiteral("layersPanel"), -1);
            }
            {
                m43Below = frame.dropBoundaryForTest(
                    QStringLiteral("pathsPanel"), QStringLiteral("layersPanel"), false);
                frame.dropIntoGroupForTest(QStringLiteral("pathsPanel"),
                                           QStringLiteral("layersPanel"), -1);
            }
            m41Column->setRailMode(true);
            m43Pump(6);
            m43BoundaryCompact = frame.dropBoundaryForTest(
                QStringLiteral("channelsPanel"), QStringLiteral("adjustmentsPanel"), true);
            m41Column->setRailMode(false);
            m43Pump(4);
            frame.dropIntoGroupForTest(QStringLiteral("channelsPanel"),
                                       QStringLiteral("layersPanel"), -1);
            m43Pump(4);
        }
        const bool m43BoundaryOk = m43Above && m43Below && m43BoundaryCompact;
        std::fprintf(stderr, "pictura self-test: m43_boundary above=%d below=%d compact=%d\n",
                     m43Above ? 1 : 0,
                     m43Below ? 1 : 0,
                     m43BoundaryCompact ? 1 : 0);
        std::fflush(stderr);
        if (!m43BoundaryOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M43 boundary drop\n");
            return 145;
        }

        // m43_singlefloat (146): a one-panel float carries only its panel, and
        // re-docking it restores the full group with the other panels.
        bool m43SinglePanel = false;
        bool m43SingleRedock = false;
        if (m41Column) {
            m41Column->setRailMode(false);
            m41Column->setMinimumWidth(360);
            m41Column->setPreferredWidth(360);
            m43Pump(6);
            pictura::PanelGroup* navGroup =
                m41Column->groupForPanel(QStringLiteral("navigatorPanel"));
            const int navCount = navGroup ? navGroup->titleCountForTest() : 0;
            const int before = m41Column->floatCountForTest();
            const bool tore =
                m41Column->tearOffPanelForTest(QStringLiteral("infoPanel"));
            m43Pump(4);
            const QStringList names =
                m41Column->floatPanelNamesForTest(m41Column->floatCountForTest() - 1);
            m43SinglePanel = tore && navCount >= 2 && navGroup
                             && names.size() == 1
                             && names.first() == QStringLiteral("infoPanel")
                             && navGroup->containsPanel(QStringLiteral("navigatorPanel"))
                             && !navGroup->containsPanel(QStringLiteral("infoPanel"));
            if (m43SinglePanel) {
                m43DropPanelOnGroup(QStringLiteral("infoPanel"),
                                    QStringLiteral("navigatorPanel"));
                pictura::PanelGroup* restored =
                    m41Column->groupForPanel(QStringLiteral("infoPanel"));
                m43SingleRedock = restored == navGroup
                                  && m41Column->floatCountForTest() == before
                                  && navGroup->titleCountForTest() == navCount;
            }
            // Safety cleanup.
            for (int i = 0; i < 8 && m41Column->floatCountForTest() > 0; ++i) {
                if (!m41Column->redockForTest(0, 0)) {
                    break;
                }
                m43Pump(4);
            }
        }
        const bool m43SingleFloatOk = m43SinglePanel && m43SingleRedock;
        std::fprintf(stderr, "pictura self-test: m43_singlefloat panel=%d redock=%d\n",
                     m43SinglePanel ? 1 : 0,
                     m43SingleRedock ? 1 : 0);
        std::fflush(stderr);
        if (!m43SingleFloatOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M43 single-panel float\n");
            return 146;
        }

        // m43_tools (147): the Tools dock width is fixed to its content width in
        // one and two columns and while floating, and a resize/separator attempt
        // cannot change it.
        bool m43ToolMin1 = false;
        bool m43ToolMin2 = false;
        bool m43ToolFloat = false;
        bool m43ToolLocked = false;
        int m43ToolsContent1 = 0;
        int m43ToolsContent2 = 0;
        if (m42Toolbox) {
            m42Toolbox->setColumns(1);
            m43Pump(4);
            m43ToolsContent1 = m42Toolbox->contentWidthForTest();
            m43ToolMin1 = m42Toolbox->minimumWidth() == m43ToolsContent1
                          && m42Toolbox->maximumWidth() == m43ToolsContent1
                          && m42Toolbox->width() == m43ToolsContent1;
            m42Toolbox->setColumns(2);
            m43Pump(4);
            m43ToolsContent2 = m42Toolbox->contentWidthForTest();
            m43ToolMin2 = m42Toolbox->minimumWidth() == m43ToolsContent2
                          && m42Toolbox->maximumWidth() == m43ToolsContent2
                          && m42Toolbox->width() == m43ToolsContent2
                          && m43ToolsContent2 > m43ToolsContent1;
            m42Toolbox->resize(m43ToolsContent2 + 60, m42Toolbox->height());
            m43Pump(4);
            m43ToolLocked = m42Toolbox->width() == m43ToolsContent2;
            if (!m42Toolbox->isFloating()) {
                m42Toolbox->setFloating(true);
                m43Pump(4);
                m43ToolFloat = m42Toolbox->minimumWidth() == m43ToolsContent2
                               && m42Toolbox->maximumWidth() == m43ToolsContent2
                               && m42Toolbox->width() == m43ToolsContent2
                               && m42Toolbox->bodyStretchForTest() == 0;
                m42Toolbox->setFloating(false);
                m43Pump(4);
            } else {
                m43ToolFloat = m42Toolbox->minimumWidth() == m43ToolsContent2
                               && m42Toolbox->maximumWidth() == m43ToolsContent2;
            }
            m42Toolbox->setColumns(1);
            m43Pump(4);
        }
        const bool m43ToolsOk = m43ToolMin1 && m43ToolMin2 && m43ToolFloat && m43ToolLocked;
        std::fprintf(stderr,
                     "pictura self-test: m43_tools min1=%d min2=%d float=%d locked=%d "
                     "content1=%d content2=%d\n",
                     m43ToolMin1 ? 1 : 0,
                     m43ToolMin2 ? 1 : 0,
                     m43ToolFloat ? 1 : 0,
                     m43ToolLocked ? 1 : 0,
                     m43ToolsContent1,
                     m43ToolsContent2);
        std::fflush(stderr);
        if (!m43ToolsOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M43 tools fixed width\n");
            return 147;
        }

        // m43_icon (148): the compact strip button and pixmap are larger than
        // M42 (30/20).
        bool m43IconBigger = false;
        if (m41Column) {
            m41Column->setRailMode(true);
            m43Pump(4);
            QToolButton* strip = m41Column->findChild<QToolButton*>(
                QStringLiteral("panelIcon_layersPanel"));
            if (!strip) {
                for (QToolButton* candidate : m41Column->findChildren<QToolButton*>()) {
                    if (candidate->objectName().startsWith(QStringLiteral("panelIcon_"))) {
                        strip = candidate;
                        break;
                    }
                }
            }
            m43IconBigger = strip && strip->width() >= 34 && strip->iconSize().width() >= 24;
            m41Column->setRailMode(false);
            m43Pump(4);
        }
        std::fprintf(stderr, "pictura self-test: m43_icon bigger=%d\n", m43IconBigger ? 1 : 0);
        std::fflush(stderr);
        if (!m43IconBigger) {
            std::fprintf(stderr, "pictura self-test: FAIL: M43 compact icon size\n");
            return 148;
        }

        // m43_flyout (149): the flyout meets the clicked button's actual edge on
        // the inner side and never overlaps or crosses to the outer side.
        bool m43FlyoutInner = false;
        bool m43FlyoutNoOverlap = false;
        if (m41Column) {
            m41Column->setRailMode(true);
            m43Pump(4);
            QToolButton* button = m41Column->findChild<QToolButton*>(
                QStringLiteral("panelIcon_layersPanel"));
            const QRect buttonRect =
                button ? QRect(button->mapToGlobal(QPoint(0, 0)), button->size()) : QRect();
            const bool opened =
                m41Column->openIconFlyoutForTest(QStringLiteral("layersPanel"));
            m43Pump(8);
            const QRect fly = m41Column->iconFlyoutGeometryForTest();
            const QString side = m41Column->flyoutSideForTest();
            m43FlyoutInner = opened && button && fly.isValid()
                             && (side == QStringLiteral("left")
                                     ? fly.right() < buttonRect.left()
                                     : fly.left() > buttonRect.right());
            m43FlyoutNoOverlap = button && fly.isValid() && !fly.intersects(buttonRect);
            m41Column->triggerFlyoutCloseForTest();
            m43Pump(4);
            m41Column->setRailMode(false);
            m43Pump(4);
        }
        const bool m43FlyoutOk = m43FlyoutInner && m43FlyoutNoOverlap;
        std::fprintf(stderr, "pictura self-test: m43_flyout inner=%d nooverlap=%d\n",
                     m43FlyoutInner ? 1 : 0,
                     m43FlyoutNoOverlap ? 1 : 0);
        std::fflush(stderr);
        if (!m43FlyoutOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M43 inner-side flyout\n");
            return 149;
        }

        // m43_dreset (150): `D` resets the foreground to black and the
        // background to white (`X` stays the swap).
        bool m43DResetFg = false;
        bool m43DResetBg = false;
        if (m42Toolbox) {
            auto* fgbg = m42Toolbox->foregroundBackgroundForTest();
            if (fgbg) {
                m42Toolbox->swapForegroundBackground();
                if (fgbg->foregroundForTest() == QColor(Qt::black)
                    && fgbg->backgroundForTest() == QColor(Qt::white)) {
                    m42Toolbox->swapForegroundBackground();
                }
                const bool dirty = fgbg->foregroundForTest() != QColor(Qt::black)
                                   || fgbg->backgroundForTest() != QColor(Qt::white);
                frame.activateWindow();
                m43Pump(2);
                QKeyEvent dEvent(QEvent::KeyPress, Qt::Key_D, Qt::NoModifier,
                                 QStringLiteral("d"));
                QApplication::sendEvent(&frame, &dEvent);
                m43Pump(4);
                m43DResetFg = fgbg->foregroundForTest() == QColor(Qt::black);
                m43DResetBg = fgbg->backgroundForTest() == QColor(Qt::white) && dirty;
            }
        }
        const bool m43DResetOk = m43DResetFg && m43DResetBg;
        std::fprintf(stderr, "pictura self-test: m43_dreset fg=%d bg=%d\n",
                     m43DResetFg ? 1 : 0,
                     m43DResetBg ? 1 : 0);
        std::fflush(stderr);
        if (!m43DResetOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M43 default-colours reset\n");
            return 150;
        }

        // m43_session (151): the v6 store carries the per-column layout. A
        // two-column layout built through the real drop path round-trips
        // (sides, order, nested groups) and re-applies to the same column
        // count; a v5 flat store loads as one right-hand column; an unknown key
        // survives a rewrite.
        bool m43SessionV6 = false;
        bool m43SessionColumns = false;
        bool m43SessionV5 = false;
        bool m43SessionRound = false;
        bool m43SessionUnknown = false;
        {
            const bool m43Made = frame.newColumnDropForTest(QStringLiteral("stylesPanel"),
                                                            QStringLiteral("left"));
            frame.saveSession();
            const pictura::SessionState m43Loaded = pictura::loadSession();
            m43SessionV6 = m43Loaded.schemaVersion == 6;

            bool m43HasLeft = false;
            bool m43HasRight = false;
            bool m43LeftStyles = false;
            for (const QJsonValue& value : m43Loaded.panelColumns) {
                const QJsonObject column = value.toObject();
                const QString side = column.value(QStringLiteral("side")).toString();
                if (side == QStringLiteral("left")) {
                    m43HasLeft = true;
                    const QJsonArray groups =
                        column.value(QStringLiteral("groups")).toArray();
                    for (const QJsonValue& group : groups) {
                        const QJsonArray order =
                            group.toObject().value(QStringLiteral("order")).toArray();
                        if (order.contains(QStringLiteral("stylesPanel"))) {
                            m43LeftStyles = true;
                        }
                    }
                } else if (side == QStringLiteral("right")) {
                    m43HasRight = true;
                }
            }
            m43SessionColumns = m43Made && m43Loaded.panelColumns.size() == 2 && m43HasLeft
                                && m43HasRight && m43LeftStyles;

            // Re-run the real startup restore path and check the rebuilt columns.
            frame.applyPanelSessionForTest(m43Loaded);
            m43Pump(8);
            pictura::PanelColumn* m43Restored =
                frame.columnForPanel(QStringLiteral("stylesPanel"));
            m43SessionRound = frame.panelColumnCountForTest() == 2 && m43Restored
                              && frame.sideOf(m43Restored) == pictura::PanelSide::Left
                              && frame.panelColumnSideForTest(0) == QStringLiteral("left");

            // A v5 store (no `panelColumns`) migrates to one right-hand column.
            {
                QFile store(pictura::sessionFilePath());
                if (store.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
                    store.write("{\"schemaVersion\":5,\"panelGroups\":[{\"name\":"
                                "\"panelGroup_layersPanel\",\"order\":[\"layersPanel\"],"
                                "\"visible\":[\"layersPanel\"],\"minimized\":false,"
                                "\"collapsed\":false}]}");
                    store.close();
                }
                const pictura::SessionState m43V5 = pictura::loadSession();
                m43SessionV5 =
                    m43V5.panelColumns.size() == 1
                    && m43V5.panelColumns.first().toObject().value(
                           QStringLiteral("side")).toString() == QStringLiteral("right")
                    && !m43V5.panelGroups.isEmpty();
            }

            // An unknown key survives a load-then-write rewrite.
            {
                QFile store(pictura::sessionFilePath());
                if (store.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
                    store.write("{\"schemaVersion\":6,\"unknownM43Key\":\"keep-me\"}");
                    store.close();
                }
                frame.saveSession();
                QFile read(pictura::sessionFilePath());
                if (read.open(QIODevice::ReadOnly)) {
                    const QByteArray rewritten = read.readAll();
                    m43SessionUnknown = rewritten.contains("unknownM43Key")
                                        && rewritten.contains("keep-me");
                }
            }

            // Collapse back to the default single right-hand column.
            frame.applyPanelSessionForTest(pictura::SessionState{});
            m43Pump(6);
        }
        const bool m43SessionOk = m43SessionV6 && m43SessionColumns && m43SessionV5
                                  && m43SessionRound && m43SessionUnknown;
        std::fprintf(stderr,
                     "pictura self-test: m43_session v6=%d v5=%d columns=%d roundtrip=%d "
                     "unknown=%d\n",
                     m43SessionV6 ? 1 : 0,
                     m43SessionV5 ? 1 : 0,
                     m43SessionColumns ? 1 : 0,
                     m43SessionRound ? 1 : 0,
                     m43SessionUnknown ? 1 : 0);
        std::fflush(stderr);
        if (!m43SessionOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M43 session v6\n");
            return 151;
        }

        // m44_newdoc (153): E1. A newly created white document must present
        // pure white immediately: the first `image()` read, before any move or
        // recomposite, is uniformly opaque white and the canvas already holds
        // it. Pins the regression where the M0.5 GPU smoke probe wrote its demo
        // gradient into the document's display image.
        const int m44DocsBefore = frame.documentCount();
        const bool m44Made = frame.newDocument(QStringLiteral("M44 White"), 32, 24,
                                               QStringLiteral("rgb"), 8,
                                               QStringLiteral("white"));
        pictura::PictureView* m44View = frame.activeView();
        const QImage m44Image = m44View ? m44View->image() : QImage();
        pictura::ImageView* m44Canvas = frame.imageView();
        const QImage m44CanvasImage = m44Canvas ? m44Canvas->image() : QImage();
        bool m44Uniform = m44Made && m44View && !m44Image.isNull();
        if (m44Uniform) {
            const QRgb m44First = m44Image.pixel(0, 0);
            for (int y = 0; y < m44Image.height() && m44Uniform; ++y) {
                for (int x = 0; x < m44Image.width(); ++x) {
                    if (m44Image.pixel(x, y) != m44First) {
                        m44Uniform = false;
                        break;
                    }
                }
            }
        }
        const bool m44White = m44Uniform && m44Image.pixel(0, 0) == 0xFFFFFFFFu;
        const bool m44Immediate = m44White && m44CanvasImage == m44Image;
        const bool m44NewdocOk = m44Made && m44Image.width() == 32 && m44Image.height() == 24
                                  && m44White && m44Uniform && m44Immediate;
        std::fprintf(stderr,
                     "pictura self-test: m44_newdoc white=%d uniform=%d immediate=%d\n",
                     m44White ? 1 : 0, m44Uniform ? 1 : 0, m44Immediate ? 1 : 0);
        std::fflush(stderr);
        if (!m44NewdocOk) {
            std::fprintf(stderr, "pictura self-test: FAIL: M44 new-document render\n");
            return 153;
        }
        frame.closeDocument(frame.documentCount() - 1, false);
        // M44 Phase B (154, 155, 156, 159, 161, 164, 165, 166): panel chrome,
        // borders, and the shared widget styling. Every check reads a real style
        // property (icon pixmap, tab index, QSS/palette colour, elided text,
        // splitter handle width) rather than only that a stylesheet exists.
        {
            const QString m44Sheet = qApp->styleSheet();
            const QColor m44Window = qApp->palette().color(QPalette::Window);
            const QString m44WindowHex = m44Window.name(QColor::HexRgb);
            const QString m44BaseHex = qApp->palette().color(QPalette::Base).name(QColor::HexRgb);
            const QString m44BorderHex = m44Window.darker(135).name(QColor::HexRgb);
            const QString m44BorderWidth = QString::number(pictura::Theme::kPanelBorderWidth);

            // 154: the collapse toggle shows the action's target icon, not the
            // inverted one (collapse-to-icons in normal, expand in iconic).
            bool m44ChevronsMode = false;
            bool m44ChevronsTarget = false;
            if (m41Column) {
                const QImage m44CollapseIcon =
                    pictura::icon(QStringLiteral("panel.columnsTwo")).pixmap(16, 16).toImage();
                const QImage m44ExpandIcon =
                    pictura::icon(QStringLiteral("panel.columnsOne")).pixmap(16, 16).toImage();
                m41Column->setRailMode(false);
                m43Pump(2);
                QToolButton* m44Toggle = m41Column->panelColumnToggleForTest();
                m44ChevronsMode =
                    m44Toggle && m44Toggle->icon().pixmap(16, 16).toImage() == m44CollapseIcon;
                m41Column->setRailMode(true);
                m43Pump(2);
                m44ChevronsTarget =
                    m44Toggle && m44Toggle->icon().pixmap(16, 16).toImage() == m44ExpandIcon;
                m41Column->setRailMode(false);
                m43Pump(2);
            }
            std::fprintf(stderr, "pictura self-test: m44_chevrons mode=%d target=%d\n",
                         m44ChevronsMode ? 1 : 0, m44ChevronsTarget ? 1 : 0);
            std::fflush(stderr);
            if (!(m44ChevronsMode && m44ChevronsTarget)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M44 collapse chevrons\n");
                return 154;
            }

            // 155: a restored layout leaves the first visible panel active, not
            // the last (the old path made each shown panel current in turn).
            bool m44DefaultActive = m41Column != nullptr;
            int m44ActiveGroups = 0;
            if (m41Column) {
                m41Column->restorePanelState(m41Column->savePanelState());
                m43Pump(2);
                for (pictura::PanelGroup* group : m41Column->groups()) {
                    if (!group) {
                        continue;
                    }
                    const int first = group->firstVisibleTabIndexForTest();
                    if (first < 0) {
                        continue;
                    }
                    ++m44ActiveGroups;
                    if (group->currentTabIndexForTest() != first) {
                        m44DefaultActive = false;
                    }
                }
                m44DefaultActive = m44DefaultActive && m44ActiveGroups > 0;
            }
            std::fprintf(stderr, "pictura self-test: m44_defaultactive first=%d\n",
                         m44DefaultActive ? 1 : 0);
            std::fflush(stderr);
            if (!m44DefaultActive) {
                std::fprintf(stderr, "pictura self-test: FAIL: M44 default active panel\n");
                return 155;
            }

            // 156: the active panel tab matches the widget surface (`window`),
            // the inactive tab is the darker `base`, and neither uses the other.
            const bool m44TabSelected =
                m44Sheet.contains(QStringLiteral("QTabBar#panelTabBar::tab:selected { background: ")
                                  + m44WindowHex)
                && m44Sheet.contains(
                    QStringLiteral("QTabWidget#panelGroupTabs::pane { border: 0; background: ")
                    + m44WindowHex);
            const bool m44TabInactive =
                m44Sheet.contains(QStringLiteral("QTabBar#panelTabBar::tab { background: ")
                                  + m44BaseHex);
            const bool m44TabDiffer =
                m44TabSelected && m44TabInactive && m44WindowHex != m44BaseHex;
            std::fprintf(stderr,
                         "pictura self-test: m44_tabswap selected=%d unselected=%d differ=%d\n",
                         m44TabSelected ? 1 : 0,
                         m44TabInactive ? 1 : 0,
                         m44TabDiffer ? 1 : 0);
            std::fflush(stderr);
            if (!m44TabDiffer) {
                std::fprintf(stderr, "pictura self-test: FAIL: M44 panel tab colours\n");
                return 156;
            }

            // 159: the inter-group splitter handle is thicker and darker grey.
            const bool m44DividerThick =
                m41Column && m41Column->groupDividerWidthForTest()
                                 >= pictura::Theme::kGroupDividerWidth;
            const bool m44DividerDark = m44Sheet.contains(
                QStringLiteral("QSplitter#panelColumnSplitter::handle { background: ")
                + m44BorderHex);
            std::fprintf(stderr, "pictura self-test: m44_divider thick=%d dark=%d\n",
                         m44DividerThick ? 1 : 0, m44DividerDark ? 1 : 0);
            std::fflush(stderr);
            if (!(m44DividerThick && m44DividerDark)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M44 group divider\n");
                return 159;
            }

            // 161: the compact-strip group divider is dark grey, not white.
            bool m44CompactDividerDark = false;
            if (m41Column) {
                m41Column->setRailMode(true);
                m43Pump(4);
                m44CompactDividerDark =
                    m41Column->dividerCountForTest() > 0
                    && m44Sheet.contains(QStringLiteral("QFrame#panelIconDivider { background: ")
                                         + m44BorderHex)
                    && m44BorderHex != QStringLiteral("#ffffff");
                m41Column->setRailMode(false);
                m43Pump(2);
            }
            std::fprintf(stderr, "pictura self-test: m44_compactdivider dark=%d\n",
                         m44CompactDividerDark ? 1 : 0);
            std::fflush(stderr);
            if (!m44CompactDividerDark) {
                std::fprintf(stderr, "pictura self-test: FAIL: M44 compact divider\n");
                return 161;
            }

            // 164: at a strip width with room only beyond the icon button, the
            // label appears and its text is elided to that width.
            bool m44ElidePartial = false;
            if (m41Column) {
                m41Column->setRailMode(true);
                m41Column->showPanel(QStringLiteral("layersPanel"), true);
                m43Pump(4);
                pictura::PanelGroup* m44Layers =
                    m41Column->groupForPanel(QStringLiteral("layersPanel"));
                const QString m44FullTitle =
                    m44Layers ? m44Layers->titleForPanel(QStringLiteral("layersPanel"))
                              : QString();
                m41Column->setIconStripWidthForTest(64);
                const QString m44Shown =
                    m41Column->stripLabelTextForTest(QStringLiteral("layersPanel"));
                const bool m44Visible =
                    m41Column->stripLabelVisibleForTest(QStringLiteral("layersPanel"));
                m44ElidePartial = m44Visible && !m44FullTitle.isEmpty() && !m44Shown.isEmpty()
                                  && m44Shown.size() < m44FullTitle.size()
                                  && m44Shown.contains(QChar(0x2026));
                m41Column->setRailMode(false);
                m43Pump(2);
            }
            std::fprintf(stderr, "pictura self-test: m44_elide partial=%d\n",
                         m44ElidePartial ? 1 : 0);
            std::fflush(stderr);
            if (!m44ElidePartial) {
                std::fprintf(stderr, "pictura self-test: FAIL: M44 strip label elide\n");
                return 164;
            }

            // 165: the document tab strip has a dark grey right border and no
            // extra top border (the options bar above already draws one).
            QTabBar* m44DocBar = frame.findChild<QTabBar*>(QStringLiteral("documentTabBar"));
            const bool m44FileRight =
                m44DocBar
                && m44Sheet.contains(QStringLiteral("QTabBar#documentTabBar { border-right: ")
                                     + m44BorderWidth + QStringLiteral("px solid ") + m44BorderHex);
            const bool m44FileNoTop =
                m44Sheet.contains(
                    QStringLiteral("QTabWidget#documentTabs::pane { border-top: 0; }"))
                && m44Sheet.contains(
                    QStringLiteral("QTabBar#documentTabBar { border-right: ") + m44BorderWidth
                    + QStringLiteral("px solid ") + m44BorderHex
                    + QStringLiteral("; border-top: 0; }"));
            std::fprintf(stderr, "pictura self-test: m44_filebar right=%d notop=%d\n",
                         m44FileRight ? 1 : 0, m44FileNoTop ? 1 : 0);
            std::fflush(stderr);
            if (!(m44FileRight && m44FileNoTop)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M44 document tab strip borders\n");
                return 165;
            }

            // 166: Tools, normal widget panels, and the compact strip share the
            // darker grey border; flyout and float use the same palette.
            const QString m44BorderRule =
                m44BorderWidth + QStringLiteral("px solid ") + m44BorderHex;
            const bool m44PanelTools =
                m44Sheet.contains(QStringLiteral("QDockWidget#toolsPanel { border: ")
                                  + m44BorderRule + QStringLiteral("; }"));
            const bool m44PanelNormal =
                m44Sheet.contains(QStringLiteral("QTabWidget#panelGroupTabs { border: ")
                                  + m44BorderRule + QStringLiteral("; }"));
            const bool m44PanelCompact =
                m44Sheet.contains(QStringLiteral("QWidget#panelColumnIconStrip { border: ")
                                  + m44BorderRule + QStringLiteral("; }"));
            const bool m44PanelParity =
                m44Sheet.contains(QStringLiteral("QWidget#panelFloat { background: ")
                                  + m44WindowHex)
                && m44Sheet.contains(QStringLiteral("QWidget#panelIconFlyout { background: ")
                                     + m44WindowHex);
            std::fprintf(stderr,
                         "pictura self-test: m44_panelborder tools=%d panel=%d compact=%d\n",
                         m44PanelTools ? 1 : 0,
                         m44PanelNormal ? 1 : 0,
                         m44PanelCompact ? 1 : 0);
            std::fflush(stderr);
            if (!(m44PanelTools && m44PanelNormal && m44PanelCompact && m44PanelParity)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M44 panel borders\n");
                return 166;
            }
        }
        // M44 Phase C (157, 158, 160, 162, 163): float-drag continuation,
        // any-side column/dock docking, popup/dock style parity, and the compact
        // group-relative drop with its drag handle.
        {
            // 157: a widget-tab float drag stays active after the float is
            // created, tracks the cursor, and only the release commits it. The
            // source group must survive so its tab bar keeps the mouse grab.
            bool m44FdActive = false;
            bool m44FdFollows = false;
            bool m44FdRelease = false;
            if (m41Column) {
                m41Column->setRailMode(false);
                m41Column->showPanel(QStringLiteral("historyPanel"), true);
                m43Pump(8);
                pictura::PanelGroup* m44FdGroup =
                    m41Column->groupForPanel(QStringLiteral("historyPanel"));
                if (m44FdGroup && m44FdGroup->titleCountForTest() == 1) {
                    const int before = m41Column->floatCountForTest();
                    const QPoint outsideA =
                        m41Column->mapToGlobal(QPoint(-60, m41Column->height() / 3));
                    const QPoint outsideB =
                        m41Column->mapToGlobal(QPoint(-60, (m41Column->height() * 2) / 3));
                    const bool began =
                        m41Column->beginTabDragForTest(QStringLiteral("historyPanel"));
                    m41Column->dragToForTest(outsideA);
                    m43Pump(8);
                    const bool floated = m41Column->floatCountForTest() == before + 1;
                    const QRect g1 = m41Column->floatGeometryForTest(before);
                    m44FdActive = began && floated && m41Column->dragActiveForTest()
                                  && m41Column->dragSourceGroupAliveForTest();
                    m41Column->dragToForTest(outsideB);
                    m43Pump(8);
                    const QRect g2 = m41Column->floatGeometryForTest(before);
                    m44FdFollows = m41Column->dragActiveForTest() && g1.isValid()
                                   && g2.isValid() && g2.topLeft() != g1.topLeft()
                                   && m41Column->floatCountForTest() == before + 1;
                    const bool dropped = m41Column->dropForTest(outsideB);
                    m43Pump(8);
                    m44FdRelease = dropped && !m41Column->dragActiveForTest();
                    for (int i = 0; i < 8 && m41Column->floatCountForTest() > 0; ++i) {
                        if (!m41Column->redockForTest(0, 0)) {
                            break;
                        }
                        m43Pump(4);
                    }
                }
            }
            std::fprintf(stderr,
                         "pictura self-test: m44_floatdrag active=%d follows=%d release=%d\n",
                         m44FdActive ? 1 : 0,
                         m44FdFollows ? 1 : 0,
                         m44FdRelease ? 1 : 0);
            std::fflush(stderr);
            if (!(m44FdActive && m44FdFollows && m44FdRelease)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M44 float drag continuation\n");
                return 157;
            }

            // 160: the compact flyout shares the docked widget's scoped
            // style/container (no per-host inline stylesheet).
            bool m44PsParity = false;
            if (m41Column) {
                m41Column->setRailMode(true);
                m43Pump(6);
                const bool opened =
                    m41Column->openIconFlyoutForTest(QStringLiteral("layersPanel"));
                m43Pump(10);
                m44PsParity = opened && m41Column->iconFlyoutVisibleForTest()
                              && m41Column->popupStyleParityForTest(
                                     QStringLiteral("layersPanel"));
                m41Column->triggerFlyoutCloseForTest();
                m43Pump(6);
                m41Column->setRailMode(false);
                m43Pump(4);
            }
            std::fprintf(stderr, "pictura self-test: m44_popupstyle parity=%d\n",
                         m44PsParity ? 1 : 0);
            std::fflush(stderr);
            if (!m44PsParity) {
                std::fprintf(stderr, "pictura self-test: FAIL: M44 popup style parity\n");
                return 160;
            }

            // 162: compact drop placement. Onto a group inserts there; on the
            // inter-group divider, above the top, and below the bottom create a
            // fresh one-panel group at that boundary.
            bool m44CdInto = false;
            bool m44CdBetween = false;
            bool m44CdAbove = false;
            bool m44CdBelow = false;
            if (m41Column) {
                m41Column->setRailMode(true);
                m43Pump(8);

                const QString m44IntoSource = QStringLiteral("swatchesPanel");
                const QString m44IntoTarget = QStringLiteral("layersPanel");
                pictura::PanelGroup* m44IntoTargetGroup =
                    m41Column->groupForPanel(m44IntoTarget);
                if (m44IntoTargetGroup
                    && m41Column->groupForPanel(m44IntoSource) != m44IntoTargetGroup) {
                    const QPoint intoPoint =
                        m41Column->stripEntryPointForTest(m44IntoTarget, 0);
                    const bool began = m41Column->beginStripDragForTest(m44IntoSource);
                    m41Column->dragToForTest(intoPoint);
                    const bool dropped = m41Column->dropForTest(intoPoint);
                    m43Pump(8);
                    m44CdInto = began && dropped
                                && m41Column->groupForPanel(m44IntoSource)
                                       == m44IntoTargetGroup;
                }

                if (m41Column->compactStripGroupOrderForTest().size() >= 2) {
                    const QString m44Between = QStringLiteral("channelsPanel");
                    pictura::PanelGroup* m44Before =
                        m41Column->groupForPanel(m44Between);
                    const QPoint betweenPoint =
                        m41Column->compactStripBoundaryPointForTest(1);
                    const bool began = m41Column->beginStripDragForTest(m44Between);
                    m41Column->dragToForTest(betweenPoint);
                    const bool dropped = m41Column->dropForTest(betweenPoint);
                    m43Pump(8);
                    pictura::PanelGroup* landed =
                        m41Column->groupForPanel(m44Between);
                    m44CdBetween = began && dropped && landed && landed != m44Before
                                   && landed->titleCountForTest() == 1;
                }

                {
                    const QString m44Above = QStringLiteral("pathsPanel");
                    const QPoint abovePoint =
                        m41Column->compactStripBoundaryPointForTest(0);
                    const bool began = m41Column->beginStripDragForTest(m44Above);
                    m41Column->dragToForTest(abovePoint);
                    const bool dropped = m41Column->dropForTest(abovePoint);
                    m43Pump(8);
                    const QStringList order =
                        m41Column->compactStripGroupOrderForTest();
                    pictura::PanelGroup* landed =
                        m41Column->groupForPanel(m44Above);
                    m44CdAbove = began && dropped && landed
                                 && landed->titleCountForTest() == 1 && !order.isEmpty()
                                 && order.first() == landed->objectName();
                }

                {
                    const QString m44Below = QStringLiteral("histogramPanel");
                    const int count =
                        m41Column->compactStripGroupOrderForTest().size();
                    const QPoint belowPoint =
                        m41Column->compactStripBoundaryPointForTest(count);
                    const bool began = m41Column->beginStripDragForTest(m44Below);
                    m41Column->dragToForTest(belowPoint);
                    const bool dropped = m41Column->dropForTest(belowPoint);
                    m43Pump(8);
                    const QStringList order =
                        m41Column->compactStripGroupOrderForTest();
                    pictura::PanelGroup* landed =
                        m41Column->groupForPanel(m44Below);
                    m44CdBelow = began && dropped && landed
                                 && landed->titleCountForTest() == 1 && !order.isEmpty()
                                 && order.last() == landed->objectName();
                }

                m41Column->setRailMode(false);
                m43Pump(4);
            }
            std::fprintf(stderr,
                         "pictura self-test: m44_compactdrop into=%d between=%d above=%d "
                         "below=%d\n",
                         m44CdInto ? 1 : 0,
                         m44CdBetween ? 1 : 0,
                         m44CdAbove ? 1 : 0,
                         m44CdBelow ? 1 : 0);
            std::fflush(stderr);
            if (!(m44CdInto && m44CdBetween && m44CdAbove && m44CdBelow)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M44 compact drop\n");
                return 162;
            }

            // 163: every compact group carries a dots grip and dragging it moves
            // the whole group to a new boundary.
            bool m44DhDots = false;
            bool m44DhGroup = false;
            if (m41Column) {
                m41Column->setRailMode(true);
                m43Pump(8);
                const QStringList order = m41Column->compactStripGroupOrderForTest();
                if (order.size() >= 2) {
                    const QString first = order.first();
                    m44DhDots = m41Column->dragHandleForTest(first) != nullptr;
                    if (m44DhDots) {
                        const int count =
                            m41Column->compactStripGroupOrderForTest().size();
                        const QPoint to =
                            m41Column->compactStripBoundaryPointForTest(count);
                        const bool began = m41Column->beginGroupGripDragForTest(first);
                        m41Column->dragToForTest(to);
                        const bool dropped = m41Column->dropForTest(to);
                        m43Pump(8);
                        const QStringList after =
                            m41Column->compactStripGroupOrderForTest();
                        m44DhGroup = began && dropped && after.size() == order.size()
                                     && after != order && after.last() == first;
                    }
                }
                m41Column->setRailMode(false);
                m43Pump(4);
            }
            std::fprintf(stderr, "pictura self-test: m44_draghandle dots=%d groupdrag=%d\n",
                         m44DhDots ? 1 : 0, m44DhGroup ? 1 : 0);
            std::fflush(stderr);
            if (!(m44DhDots && m44DhGroup)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M44 compact drag handle\n");
                return 163;
            }

            // 158: a widget column docks at the toolbar side, beside another
            // column, and at the workspace edge; the Tools dock now allows only
            // the left/right sides (M45 T2) and its floating height is pinned
            // (T1/T2/W5).
            bool m44DsToolbar = false;
            bool m44DsColumn = false;
            bool m44DsWorkspace = false;
            bool m44DsFloat = false;
            if (m40Toolbox && m41Column) {
                m41Column->setRailMode(false);
                m43Pump(8);
                const int baseCount = frame.panelColumnCountForTest();

                const bool ws =
                    frame.newColumnDropForTest(QStringLiteral("stylesPanel"),
                                               QStringLiteral("right"));
                const bool wsBack =
                    ws && frame.dropIntoGroupForTest(QStringLiteral("stylesPanel"),
                                                     QStringLiteral("colorPanel"), -1);
                m44DsWorkspace =
                    ws && wsBack && frame.panelColumnCountForTest() == baseCount;

                const bool tb =
                    frame.newColumnDropForTest(QStringLiteral("gradientsPanel"),
                                               QStringLiteral("tools"));
                const bool tbBack =
                    tb && frame.dropIntoGroupForTest(QStringLiteral("gradientsPanel"),
                                                     QStringLiteral("colorPanel"), -1);
                m44DsToolbar =
                    tb && tbBack && frame.panelColumnCountForTest() == baseCount;

                const bool madeAnchor =
                    frame.newColumnDropForTest(QStringLiteral("patternsPanel"),
                                               QStringLiteral("right"));
                const bool beside =
                    madeAnchor
                    && frame.newColumnBesideForTest(QStringLiteral("librariesPanel"),
                                                    QStringLiteral("patternsPanel"));
                const bool besideBack =
                    beside
                    && frame.dropIntoGroupForTest(QStringLiteral("librariesPanel"),
                                                  QStringLiteral("colorPanel"), -1)
                    && frame.dropIntoGroupForTest(QStringLiteral("patternsPanel"),
                                                  QStringLiteral("colorPanel"), -1);
                m44DsColumn = madeAnchor && beside && besideBack
                              && frame.panelColumnCountForTest() == baseCount;

                const bool areas =
                    m40Toolbox->allowedAreas()
                    == (Qt::LeftDockWidgetArea | Qt::RightDockWidgetArea);
                const bool wasFloating = m40Toolbox->isFloating();
                if (!wasFloating) {
                    m40Toolbox->setFloating(true);
                    m43Pump(8);
                }
                const int floatH = m40Toolbox->floatHeightForTest();
                m40Toolbox->resize(m40Toolbox->width(), floatH + 80);
                m43Pump(8);
                const bool locked = m40Toolbox->floatHeightLockedForTest()
                                    && m40Toolbox->height() == floatH;
                if (!wasFloating) {
                    m40Toolbox->setFloating(false);
                    m43Pump(8);
                }
                m44DsFloat = areas && locked;
            }
            std::fprintf(stderr,
                         "pictura self-test: m44_docksides toolbar=%d column=%d workspace=%d "
                         "float=%d\n",
                         m44DsToolbar ? 1 : 0,
                         m44DsColumn ? 1 : 0,
                         m44DsWorkspace ? 1 : 0,
                         m44DsFloat ? 1 : 0);
            std::fflush(stderr);
            if (!(m44DsToolbar && m44DsColumn && m44DsWorkspace && m44DsFloat)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M44 any-side docking\n");
                return 158;
            }
        }
        // M45 Phase A (167-169): the Tools toolbar content sizing, left/right-only
        // docking, and placement beside a widget column through the column
        // grammar. Geometry is read after a bounded pump so it is real offscreen.
        {
            // 167: both axes are content-derived in one and two columns and
            // stable across a 1->2->1 toggle; neither axis is cut or over-tall.
            bool m45One = false;
            bool m45Two = false;
            bool m45Stable = false;
            int m45W1 = 0;
            int m45H1 = 0;
            int m45W2 = 0;
            int m45H2 = 0;
            if (m40Toolbox) {
                m40Toolbox->setColumns(1);
                m43Pump(8);
                m45W1 = m40Toolbox->width();
                m45H1 = m40Toolbox->height();
                const int cw1 = m40Toolbox->contentWidthForTest();
                const int ch1 = m40Toolbox->contentHeightForTest();
                m45One = cw1 > 0 && ch1 > 0 && m45W1 == cw1 && m45H1 == ch1
                         && m40Toolbox->minimumWidth() == cw1
                         && m40Toolbox->maximumWidth() == cw1
                         && m40Toolbox->minimumHeight() == ch1
                         && m40Toolbox->maximumHeight() == ch1;
                m40Toolbox->setColumns(2);
                m43Pump(8);
                m45W2 = m40Toolbox->width();
                m45H2 = m40Toolbox->height();
                const int cw2 = m40Toolbox->contentWidthForTest();
                const int ch2 = m40Toolbox->contentHeightForTest();
                m45Two = cw2 > cw1 && ch2 > 0 && ch2 < ch1 && m45W2 == cw2 && m45H2 == ch2;
                m40Toolbox->setColumns(1);
                m43Pump(8);
                m45Stable = m40Toolbox->width() == cw1 && m40Toolbox->height() == ch1;
            }
            std::fprintf(stderr,
                         "pictura self-test: m45_tools_sizing one=%d two=%d stable=%d "
                         "w1=%d h1=%d w2=%d h2=%d\n",
                         m45One ? 1 : 0, m45Two ? 1 : 0, m45Stable ? 1 : 0, m45W1, m45H1,
                         m45W2, m45H2);
            std::fflush(stderr);
            if (!(m45One && m45Two && m45Stable)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M45 tools sizing\n");
                return 167;
            }
        }

        {
            // 168: the Tools dock allows left/right only; top and bottom refused.
            const Qt::DockWidgetAreas m45Areas =
                m40Toolbox ? m40Toolbox->allowedAreas() : Qt::NoDockWidgetArea;
            const bool m45Left = m45Areas.testFlag(Qt::LeftDockWidgetArea);
            const bool m45Right = m45Areas.testFlag(Qt::RightDockWidgetArea);
            const bool m45NoTop = !m45Areas.testFlag(Qt::TopDockWidgetArea)
                                  && !m45Areas.testFlag(Qt::BottomDockWidgetArea);
            std::fprintf(stderr,
                         "pictura self-test: m45_tools_sides left=%d right=%d notop=%d\n",
                         m45Left ? 1 : 0, m45Right ? 1 : 0, m45NoTop ? 1 : 0);
            std::fflush(stderr);
            if (!(m45Left && m45Right && m45NoTop)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M45 tools sides\n");
                return 168;
            }
        }

        {
            // 169: the floating Tools panel docks to the left of the primary
            // widget column and to the right of a dynamically created one, with
            // the single indicator shown, through the column grammar.
            const bool m45BesideLeft =
                frame.toolboxBesideColumnForTest(QStringLiteral("left"), false);
            const bool m45BesideRight =
                frame.toolboxBesideColumnForTest(QStringLiteral("right"), true);
            std::fprintf(stderr,
                         "pictura self-test: m45_tools_beside_column left=%d right=%d\n",
                         m45BesideLeft ? 1 : 0, m45BesideRight ? 1 : 0);
            std::fflush(stderr);
            if (!(m45BesideLeft && m45BesideRight)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M45 tools beside column\n");
                return 169;
            }
        }
        // M45 Phase B (170-177): the widget-panel drop indicator is rendered
        // from the resolved DropTarget in the owning column (W1-W3, W6); a
        // dynamic column emptied by any path is removed (W4); minimize collapses
        // the group to its tab bar with a state-derived menu label (W5); a
        // column never clips and shares one minimum-width floor (W7/W8). Every
        // check pumps the event loop bounded and forces layout so geometry is
        // real offscreen.
        {
            auto m45Pump = [](int n) {
                for (int i = 0; i < n; ++i) {
                    QCoreApplication::processEvents();
                }
            };
            // A visible panel in the primary column that is not in `exclude`.
            auto m45OtherPanel = [&](pictura::PanelGroup* exclude,
                                     const QString& alsoExclude) -> QString {
                if (!m41Column) {
                    return QString();
                }
                for (pictura::PanelGroup* group : m41Column->groups()) {
                    if (!group || group == exclude || !group->isVisible()
                        || group->titleCountForTest() == 0) {
                        continue;
                    }
                    for (QWidget* panel : group->panels()) {
                        if (panel && panel->objectName() != alsoExclude) {
                            return panel->objectName();
                        }
                    }
                }
                return QString();
            };
            auto m45CollapseDynamics = [&]() {
                // Geometry-independent teardown: move every dynamic column's
                // groups (and their panels) back into the primary column, then
                // drop the empty columns. Keeps the tested resolve/commit path
                // untouched without depending on a drop's pixel geometry.
                for (pictura::PanelColumn* column : frame.panelColumns()) {
                    if (!column || column == m41Column) {
                        continue;
                    }
                    const QList<pictura::PanelGroup*> groups = column->groups();
                    for (pictura::PanelGroup* group : groups) {
                        if (!group) {
                            continue;
                        }
                        if (pictura::PanelGroup* taken =
                                column->takeGroup(group->objectName())) {
                            m41Column->adoptGroup(taken);
                        }
                    }
                    frame.removeColumnIfEmpty(column);
                }
                m45Pump(6);
            };

            // 170: the line follows the resolved target's side in the owning
            // (destination) column, never the drag's source column.
            bool m45SideLeft = false;
            bool m45SideRight = false;
            if (m41Column) {
                m41Column->setRailMode(false);
                m45Pump(4);
                pictura::PanelColumn* dest =
                    frame.createPanelColumn(pictura::PanelSide::Right, m41Column);
                pictura::PanelGroup* dg = nullptr;
                for (pictura::PanelGroup* group : m41Column->groups()) {
                    if (group && group->isVisible() && group->titleCountForTest() == 1) {
                        dg = group;
                        break;
                    }
                }
                if (dest && dg) {
                    const QString seedName = dg->objectName();
                    if (pictura::PanelGroup* taken = m41Column->takeGroup(seedName)) {
                        dest->adoptGroup(taken);
                    }
                }
                if (dest) {
                    m41Column->setPreferredWidth(180);
                    dest->setPreferredWidth(420);
                    m45Pump(6);
                }
                const QString srcA = m45OtherPanel(dg, QString());
                const QString srcB = m45OtherPanel(dg, srcA);
                if (dest && dest != m41Column && dg && !srcA.isEmpty() && !srcB.isEmpty()) {
                    // Append one panel (right side), then insert the other
                    // between the first two tabs (left side); both points are
                    // interior to the target bar, so they exercise the
                    // cross-column target rather than a column-edge anchor.
                    const int endR = dg->titleCountForTest();
                    const QPoint rightPoint = dg->tabInsertionGlobalPointForTest(endR);
                    m41Column->beginTabDragForTest(srcA);
                    m41Column->dragToForTest(rightPoint);
                    const QRect rg = dest->dropIndicatorGlobalGeometryForTest();
                    m45SideRight = dest->dropIndicatorVisibleForTest()
                                   && !m41Column->dropIndicatorVisibleForTest() && rg.isValid()
                                   && qAbs(rg.center().x() - rightPoint.x()) <= 8;
                    m41Column->dropForTest(rightPoint);
                    m45Pump(4);
                    m45SideRight = m45SideRight && dest->groupForPanel(srcA) == dg
                                   && dg->indexOfPanel(srcA) == endR;

                    const QPoint leftPoint = dg->tabInsertionGlobalPointForTest(1);
                    m41Column->beginTabDragForTest(srcB);
                    m41Column->dragToForTest(leftPoint);
                    const QRect lg = dest->dropIndicatorGlobalGeometryForTest();
                    m45SideLeft = dest->dropIndicatorVisibleForTest()
                                  && !m41Column->dropIndicatorVisibleForTest() && lg.isValid()
                                  && qAbs(lg.center().x() - leftPoint.x()) <= 8;
                    m41Column->dropForTest(leftPoint);
                    m45Pump(4);
                    m45SideLeft = m45SideLeft && dest->groupForPanel(srcB) == dg
                                  && dg->indexOfPanel(srcB) == 1;
                }
                m45CollapseDynamics();
            }
            std::fprintf(stderr, "pictura self-test: m45_indicator_side left=%d right=%d\n",
                         m45SideLeft ? 1 : 0, m45SideRight ? 1 : 0);
            std::fflush(stderr);
            if (!(m45SideLeft && m45SideRight)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M45 indicator side\n");
                return 170;
            }

            // 171: a cross-column IntoGroup drop shows the line in the target
            // column (shown) and lands the panel there (placed).
            bool m45CrossShown = false;
            bool m45CrossPlaced = false;
            if (m41Column) {
                m41Column->setRailMode(false);
                m45Pump(4);
                pictura::PanelColumn* dest =
                    frame.createPanelColumn(pictura::PanelSide::Right, m41Column);
                pictura::PanelGroup* dg = nullptr;
                for (pictura::PanelGroup* group : m41Column->groups()) {
                    if (group && group->isVisible() && group->titleCountForTest() == 1) {
                        dg = group;
                        break;
                    }
                }
                if (dest && dg) {
                    const QString seedName = dg->objectName();
                    if (pictura::PanelGroup* taken = m41Column->takeGroup(seedName)) {
                        dest->adoptGroup(taken);
                    }
                }
                if (dest) {
                    m41Column->setPreferredWidth(180);
                    dest->setPreferredWidth(420);
                    m45Pump(6);
                }
                const QString src = m45OtherPanel(dg, QString());
                if (dest && dest != m41Column && dg && !src.isEmpty()) {
                    const QPoint p = dg->tabInsertionGlobalPointForTest(
                        dg->titleCountForTest());
                    m41Column->beginTabDragForTest(src);
                    m41Column->dragToForTest(p);
                    m45CrossShown = dest->dropIndicatorVisibleForTest()
                                    && !m41Column->dropIndicatorVisibleForTest();
                    m41Column->dropForTest(p);
                    m45Pump(4);
                    m45CrossPlaced = dest->groupForPanel(src) == dg;
                }
                m45CollapseDynamics();
            }
            std::fprintf(stderr,
                         "pictura self-test: m45_indicator_cross_column shown=%d placed=%d\n",
                         m45CrossShown ? 1 : 0, m45CrossPlaced ? 1 : 0);
            std::fflush(stderr);
            if (!(m45CrossShown && m45CrossPlaced)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M45 cross-column indicator\n");
                return 171;
            }

            // 172: the rightmost tab insertion draws at that tab's index, not
            // the leftmost edge, and the panel lands last.
            bool m45Rightmost = false;
            if (m41Column) {
                m41Column->setRailMode(false);
                m41Column->setPreferredWidth(420);
                m45Pump(6);
                pictura::PanelGroup* target =
                    m41Column->groupForPanel(QStringLiteral("layersPanel"));
                const QString src = m45OtherPanel(target, QString());
                if (target && target->titleCountForTest() >= 1 && !src.isEmpty()) {
                    const int end = target->titleCountForTest();
                    const QPoint p = target->tabInsertionGlobalPointForTest(end);
                    QTabBar* bar = target->tabBar();
                    const int firstLeft =
                        bar ? bar->mapToGlobal(QPoint(target->tabInsertionX(0), 0)).x() : 0;
                    const int lastLeft = bar ? bar->mapToGlobal(
                                                   QPoint(target->tabInsertionX(end - 1), 0))
                                                   .x()
                                             : 0;
                    m41Column->beginTabDragForTest(src);
                    m41Column->dragToForTest(p);
                    const QRect g = m41Column->dropIndicatorGlobalGeometryForTest();
                    m45Rightmost = m41Column->dropIndicatorVisibleForTest() && g.isValid()
                                   && qAbs(g.center().x() - p.x()) <= 8
                                   && g.center().x() >= lastLeft
                                   && g.center().x() > firstLeft;
                    m41Column->dropForTest(p);
                    m45Pump(4);
                    m45Rightmost = m45Rightmost && target->indexOfPanel(src) == end
                                   && m41Column->groupForPanel(src) == target;
                }
            }
            std::fprintf(stderr, "pictura self-test: m45_indicator_rightmost_tab rightmost=%d\n",
                         m45Rightmost ? 1 : 0);
            std::fflush(stderr);
            if (!m45Rightmost) {
                std::fprintf(stderr, "pictura self-test: FAIL: M45 rightmost-tab indicator\n");
                return 172;
            }

            // 173: a dynamic column emptied by a close path and by a move path
            // is removed; a closed panel is rehomed so it can be shown again.
            bool m45Closed = false;
            bool m45Moved = false;
            if (m41Column) {
                m41Column->setRailMode(false);
                m45Pump(4);
                // closed: a dynamic column whose group is closed is removed;
                // the hidden group is rehomed so the panel can be shown again.
                const int beforeClose = frame.panelColumnCountForTest();
                pictura::PanelColumn* closeDest =
                    frame.createPanelColumn(pictura::PanelSide::Right, m41Column);
                pictura::PanelGroup* closeGroup = nullptr;
                for (pictura::PanelGroup* group : m41Column->groups()) {
                    if (group && group->isVisible() && group->titleCountForTest() == 1
                        && group->objectName() != QStringLiteral("panelGroup_layersPanel")) {
                        closeGroup = group;
                        break;
                    }
                }
                if (closeDest && closeGroup) {
                    const QString closeName = closeGroup->objectName();
                    if (pictura::PanelGroup* taken = m41Column->takeGroup(closeName)) {
                        closeDest->adoptGroup(taken);
                    }
                    const QString closePanel = closeGroup->panels().isEmpty()
                                                   ? QString()
                                                   : closeGroup->panels().first()->objectName();
                    m45Pump(6);
                    closeDest->closeGroup(closeGroup);
                    m45Pump(6);
                    m45Closed = frame.panelColumnCountForTest() == beforeClose
                                && !closePanel.isEmpty()
                                && m41Column->groupForPanel(closePanel) != nullptr;
                }
                // moved: a dynamic column whose panel is moved out through the
                // real drag path empties and is removed.
                const int beforeMove = frame.panelColumnCountForTest();
                pictura::PanelColumn* moveDest =
                    frame.createPanelColumn(pictura::PanelSide::Right, m41Column);
                pictura::PanelGroup* moveGroup = nullptr;
                for (pictura::PanelGroup* group : m41Column->groups()) {
                    if (group && group->isVisible() && group->titleCountForTest() == 1
                        && group->objectName() != QStringLiteral("panelGroup_layersPanel")) {
                        moveGroup = group;
                        break;
                    }
                }
                if (moveDest && moveGroup) {
                    const QString moveName = moveGroup->objectName();
                    if (pictura::PanelGroup* taken = m41Column->takeGroup(moveName)) {
                        moveDest->adoptGroup(taken);
                    }
                    const QString movePanel = moveGroup->panels().isEmpty()
                                                  ? QString()
                                                  : moveGroup->panels().first()->objectName();
                    m41Column->setPreferredWidth(420);
                    m45Pump(6);
                    const bool back =
                        frame.dropIntoGroupForTest(movePanel, QStringLiteral("colorPanel"), -1);
                    m45Pump(6);
                    m45Moved = back && frame.panelColumnCountForTest() == beforeMove;
                }
                m45CollapseDynamics();
            }
            std::fprintf(stderr, "pictura self-test: m45_empty_column_removed closed=%d moved=%d\n",
                         m45Closed ? 1 : 0, m45Moved ? 1 : 0);
            std::fflush(stderr);
            if (!(m45Closed && m45Moved)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M45 empty column removed\n");
                return 173;
            }

            // 174: minimize collapses the group to its tab-bar height with the
            // content hidden and the tab menu reading `Expand Panel`.
            bool m45MinHeight = false;
            bool m45MinContent = false;
            bool m45MinLabel = false;
            if (m41Column) {
                m41Column->setRailMode(false);
                m41Column->showPanel(QStringLiteral("layersPanel"), true);
                m41Column->setPreferredWidth(360);
                m45Pump(8);
                pictura::PanelGroup* g =
                    m41Column->groupForPanel(QStringLiteral("layersPanel"));
                if (g) {
                    g->setMinimizedForTest(false);
                    m45Pump(8);
                    const int expanded = g->height();
                    const int barH = g->tabBar() ? g->tabBar()->sizeHint().height() : 24;
                    g->setMinimizedForTest(true);
                    g->updateGeometry();
                    m41Column->updateGeometry();
                    m45Pump(16);
                    const int collapsed = g->height();
                    m45MinHeight = expanded > barH + 4 && collapsed > 0
                                   && collapsed <= barH + 6 && collapsed < expanded
                                   && g->maximumHeight() <= barH + 4;
                    m45MinContent = g->contentHiddenForTest();
                    const QStringList texts = m41Column->tabMenuActionsForTest(g->objectName());
                    m45MinLabel = texts.size() == 7
                                  && texts.value(2) == QStringLiteral("Expand Panel");
                    g->setMinimizedForTest(false);
                    g->updateGeometry();
                    m45Pump(8);
                }
            }
            std::fprintf(stderr,
                         "pictura self-test: m45_minimize_collapse height=%d content=%d "
                         "label=%d\n",
                         m45MinHeight ? 1 : 0, m45MinContent ? 1 : 0, m45MinLabel ? 1 : 0);
            std::fflush(stderr);
            if (!(m45MinHeight && m45MinContent && m45MinLabel)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M45 minimize collapse\n");
                return 174;
            }

            // 175: a bottom-boundary insert draws the horizontal line at the
            // last group's bottom edge and lands a new last group.
            bool m45Bottom = false;
            if (m41Column) {
                m41Column->setRailMode(false);
                m45Pump(4);
                const QString src = m45OtherPanel(nullptr, QString());
                const QPoint p = m41Column->boundaryPointForTest(1000);
                if (!src.isEmpty() && !p.isNull()) {
                    m41Column->beginTabDragForTest(src);
                    m41Column->dragToForTest(p);
                    const QRect g = m41Column->dropIndicatorGeometryForTest();
                    const bool horiz =
                        g.isValid() && g.width() > g.height() && g.height() == 3;
                    m41Column->dropForTest(p);
                    m45Pump(6);
                    pictura::PanelGroup* landed = m41Column->groupForPanel(src);
                    m45Bottom = horiz && g.center().y() > m41Column->height() / 2 && landed
                                && landed->titleCountForTest() == 1;
                }
            }
            std::fprintf(stderr, "pictura self-test: m45_indicator_bottom bottom=%d\n",
                         m45Bottom ? 1 : 0);
            std::fflush(stderr);
            if (!m45Bottom) {
                std::fprintf(stderr, "pictura self-test: FAIL: M45 bottom indicator\n");
                return 175;
            }

            // 176: the column never clips — the tab text elides, the corner
            // button keeps its width, and the column is wide enough that no
            // horizontal scrollbar is ever needed (policy off).
            bool m45NoClipElide = false;
            bool m45NoClipScroll = false;
            if (m41Column) {
                m41Column->setRailMode(false);
                m45Pump(4);
                bool any = false;
                bool elide = true;
                bool corner = true;
                for (pictura::PanelGroup* g : m41Column->groups()) {
                    if (!g || !g->isVisible()) {
                        continue;
                    }
                    any = true;
                    QTabBar* bar = g->tabBar();
                    elide = elide && bar && bar->elideMode() == Qt::ElideRight
                            && !bar->expanding();
                    corner = corner && g->headerCornerWidthForTest() > 0;
                }
                m45NoClipElide = any && elide && corner;
                m45NoClipScroll = m41Column->horizontalScrollPolicyForTest()
                                  == static_cast<int>(Qt::ScrollBarAlwaysOff);
            }
            std::fprintf(stderr, "pictura self-test: m45_no_clip elide=%d scroll=%d\n",
                         m45NoClipElide ? 1 : 0, m45NoClipScroll ? 1 : 0);
            std::fflush(stderr);
            if (!(m45NoClipElide && m45NoClipScroll)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M45 no clip\n");
                return 176;
            }

            // 177: every normal-mode widget column shares one minimum-width
            // floor and cannot vanish when resized to it.
            bool m45FloorShared = false;
            bool m45FloorNotGone = false;
            if (m41Column) {
                m41Column->setRailMode(true);
                m45Pump(2);
                m41Column->setRailMode(false);
                m45Pump(4);
                pictura::PanelColumn* dest =
                    frame.createPanelColumn(pictura::PanelSide::Right, m41Column);
                pictura::PanelGroup* seed = nullptr;
                for (pictura::PanelGroup* group : m41Column->groups()) {
                    if (group && group->isVisible() && group->titleCountForTest() == 1) {
                        seed = group;
                        break;
                    }
                }
                if (dest && seed) {
                    const QString seedName = seed->objectName();
                    if (pictura::PanelGroup* taken = m41Column->takeGroup(seedName)) {
                        dest->adoptGroup(taken);
                    }
                }
                if (dest && dest != m41Column) {
                    const int floor = m41Column->minimumWidthFloorForTest();
                    const int minA = m41Column->minimumWidthForTest();
                    const int minB = dest->minimumWidthForTest();
                    m45FloorShared = floor > 0 && minA == floor && minB == floor;
                    dest->setPreferredWidth(floor);
                    m41Column->setPreferredWidth(floor);
                    m45Pump(8);
                    m45FloorNotGone = minA > 0 && !m41Column->isHidden()
                                      && !dest->isHidden() && m41Column->width() >= floor;
                    m45CollapseDynamics();
                }
            }
            std::fprintf(stderr, "pictura self-test: m45_min_width_floor shared=%d notgone=%d\n",
                         m45FloorShared ? 1 : 0, m45FloorNotGone ? 1 : 0);
            std::fflush(stderr);
            if (!(m45FloorShared && m45FloorNotGone)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M45 min width floor\n");
                return 177;
            }
        }
        // M45 Phase C (178-179): the compact popup hosts the whole PanelGroup
        // (all tabs, clicked panel active) and restores it exactly once on close
        // (C1); a whole-group compact drag draws above the drag-handle dots (C2).
        {
            auto m45Pump = [](int n) {
                for (int i = 0; i < n; ++i) {
                    QCoreApplication::processEvents();
                }
            };

            // 178: the popup contains the full group's tabs, the clicked panel
            // is current, it is the very same widget as the docked group (shared
            // style/container), and closing restores it exactly once.
            bool m45PopupTabs = false;
            bool m45PopupActive = false;
            bool m45PopupParity = false;
            bool m45PopupRestore = false;
            if (m41Column) {
                m41Column->setRailMode(true);
                m45Pump(6);
                // The Layers group carries three tabs (Layers/Channels/Paths),
                // so clicking Channels proves the whole group is hosted.
                const QString clicked = QStringLiteral("channelsPanel");
                pictura::PanelGroup* group = m41Column->groupForPanel(clicked);
                if (group) {
                    group->setCurrentPanel(QStringLiteral("layersPanel"));
                    m45Pump(2);
                    const QStringList expectedTitles = group->titles();
                    const int expectedCount = group->titleCountForTest();
                    const bool opened = m41Column->openIconFlyoutForTest(clicked);
                    m45Pump(10);
                    pictura::PanelGroup* hosted = m41Column->iconFlyoutGroupForTest();
                    m45PopupTabs = opened && hosted == group && group->isVisible()
                                   && hosted->titleCountForTest() == expectedCount
                                   && hosted->titles() == expectedTitles;
                    m45PopupActive = hosted && hosted->currentPanelName() == clicked;
                    m45PopupParity = m41Column->iconFlyoutVisibleForTest()
                                     && m41Column->popupStyleParityForTest(clicked);
                    const bool closed = m41Column->triggerFlyoutCloseForTest();
                    m45Pump(10);
                    int occurrences = 0;
                    for (pictura::PanelGroup* cand : m41Column->groups()) {
                        if (cand == group) {
                            ++occurrences;
                        }
                    }
                    QWidget* parent = group->parentWidget();
                    m45PopupRestore =
                        closed && !m41Column->iconFlyoutVisibleForTest()
                        && m41Column->iconFlyoutGroupForTest() == nullptr
                        && occurrences == 1 && parent
                        && parent->objectName() == QStringLiteral("panelColumnSplitter")
                        && group->titleCountForTest() == expectedCount
                        && group->currentPanelName() == clicked
                        && m41Column->groupForPanel(clicked) == group;
                }
                m41Column->setRailMode(false);
                m45Pump(4);
            }
            std::fprintf(stderr,
                         "pictura self-test: m45_popup_group tabs=%d active=%d parity=%d "
                         "restore=%d\n",
                         m45PopupTabs ? 1 : 0, m45PopupActive ? 1 : 0, m45PopupParity ? 1 : 0,
                         m45PopupRestore ? 1 : 0);
            std::fflush(stderr);
            if (!(m45PopupTabs && m45PopupActive && m45PopupParity && m45PopupRestore)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M45 popup group\n");
                return 178;
            }

            // 179: dragging a whole group in compact mode shows the placement
            // line above the group's drag-handle grip (at its insertion
            // boundary), never inside the group below the dots.
            bool m45CompactAbove = false;
            if (m41Column) {
                m41Column->setRailMode(true);
                m45Pump(8);
                const QStringList order = m41Column->compactStripGroupOrderForTest();
                const QString first = order.isEmpty() ? QString() : order.first();
                QWidget* grip = first.isEmpty() ? nullptr : m41Column->dragHandleForTest(first);
                const QPoint boundary = m41Column->compactStripBoundaryPointForTest(0);
                const bool began =
                    !first.isEmpty() && m41Column->beginGroupGripDragForTest(first);
                m41Column->dragToForTest(boundary);
                const QRect indicator = m41Column->dropIndicatorGeometryForTest();
                const QRect indicatorGlobal = m41Column->dropIndicatorGlobalGeometryForTest();
                const QRect gripGlobal =
                    grip ? QRect(grip->mapToGlobal(QPoint(0, 0)), grip->size()) : QRect();
                m45CompactAbove =
                    began && grip && boundary != QPoint() && indicator.isValid()
                    && indicator.width() > indicator.height()
                    && indicatorGlobal.isValid() && gripGlobal.isValid()
                    && indicatorGlobal.center().y() <= gripGlobal.top()
                    && qAbs(indicatorGlobal.center().y() - boundary.y()) <= 3;
                m41Column->cancelDragForTest();
                m45Pump(6);
                m41Column->setRailMode(false);
                m45Pump(4);
            }
            std::fprintf(stderr, "pictura self-test: m45_compact_group_line above=%d\n",
                         m45CompactAbove ? 1 : 0);
            std::fflush(stderr);
            if (!m45CompactAbove) {
                std::fprintf(stderr, "pictura self-test: FAIL: M45 compact group line\n");
                return 179;
            }
        }

        // M46 (180-186): regression checks for the panel drop indicator,
        // floating-Tools gesture, splitter floor, minimize height, and primary
        // empty-column lifecycle. Each drives the real resolved DropTarget
        // geometry (not just the helper) and restores the layout afterwards.
        {
            auto m46Pump = [](int n) {
                for (int i = 0; i < n; ++i) {
                    QCoreApplication::processEvents();
                }
            };
            auto m46OtherPanel = [&](pictura::PanelGroup* exclude,
                                     const QString& alsoExclude) -> QString {
                if (!m41Column) {
                    return QString();
                }
                for (pictura::PanelGroup* group : m41Column->groups()) {
                    if (!group || group == exclude || !group->isVisible()
                        || group->titleCountForTest() == 0) {
                        continue;
                    }
                    for (QWidget* panel : group->panels()) {
                        if (panel && panel->objectName() != alsoExclude) {
                            return panel->objectName();
                        }
                    }
                }
                return QString();
            };
            auto m46CollapseDynamics = [&]() {
                for (pictura::PanelColumn* column : frame.panelColumns()) {
                    if (!column || column == m41Column) {
                        continue;
                    }
                    const QList<pictura::PanelGroup*> groups = column->groups();
                    for (pictura::PanelGroup* group : groups) {
                        if (!group) {
                            continue;
                        }
                        if (pictura::PanelGroup* taken =
                                column->takeGroup(group->objectName())) {
                            m41Column->adoptGroup(taken);
                        }
                    }
                    frame.removeColumnIfEmpty(column);
                }
                if (m41Column) {
                    m41Column->show();
                }
                m46Pump(6);
            };

            // 180: a group with a hidden tab maps the rightmost insertion over
            // the visible tabs, so the line draws at the right visible tab (not
            // the far left) and the dropped panel lands at that index.
            bool m46HiddenShown = false;
            bool m46HiddenPlaced = false;
            if (m41Column) {
                m41Column->setRailMode(false);
                m41Column->setPreferredWidth(180);
                m46Pump(6);
                pictura::PanelColumn* dest =
                    frame.createPanelColumn(pictura::PanelSide::Right, m41Column);
                pictura::PanelGroup* dg = nullptr;
                for (pictura::PanelGroup* group : m41Column->groups()) {
                    if (group && group->isVisible() && group->titleCountForTest() >= 3) {
                        dg = group;
                        break;
                    }
                }
                if (!dg) {
                    for (pictura::PanelGroup* group : m41Column->groups()) {
                        if (group && group->isVisible()
                            && group->titleCountForTest() >= 2) {
                            dg = group;
                            break;
                        }
                    }
                }
                if (dest && dg) {
                    const QString seedName = dg->objectName();
                    if (pictura::PanelGroup* taken = m41Column->takeGroup(seedName)) {
                        dest->adoptGroup(taken);
                    }
                }
                if (dest) {
                    m41Column->setPreferredWidth(180);
                    dest->setPreferredWidth(300);
                    m46Pump(6);
                }
                const QString src = m46OtherPanel(nullptr, QString());
                if (dest && dest != m41Column && dg && !src.isEmpty()) {
                    QString hiddenName;
                    const QList<QWidget*> panels = dg->panels();
                    if (!panels.isEmpty()) {
                        hiddenName = panels.last()->objectName();
                    }
                    const bool hid =
                        !hiddenName.isEmpty() && dg->setPanelVisible(hiddenName, false);
                    m46Pump(4);
                    const bool invariant =
                        dg->titleCountForTest() > dg->visibleTitles().size();
                    const int end = dg->titleCountForTest();
                    const QPoint p = dg->tabInsertionGlobalPointForTest(end);
                    QTabBar* bar = dg->tabBar();
                    const int barCenter =
                        bar ? bar->mapToGlobal(QPoint(0, 0)).x() + bar->width() / 2 : -1;
                    m41Column->beginTabDragForTest(src);
                    m41Column->dragToForTest(p);
                    const QRect g = dest->dropIndicatorGlobalGeometryForTest();
                    m46HiddenShown = hid && invariant && dest->dropIndicatorVisibleForTest()
                                     && !m41Column->dropIndicatorVisibleForTest()
                                     && g.isValid() && g.center().x() >= barCenter
                                     && qAbs(g.center().x() - p.x()) <= 8;
                    m41Column->dropForTest(p);
                    m46Pump(4);
                    m46HiddenPlaced =
                        dg->indexOfPanel(src) == end && dest->groupForPanel(src) == dg;
                    if (!hiddenName.isEmpty()) {
                        dg->setPanelVisible(hiddenName, true);
                    }
                }
                m46CollapseDynamics();
            }
            std::fprintf(stderr,
                         "pictura self-test: m46_indicator_hidden_tab shown=%d placed=%d\n",
                         m46HiddenShown ? 1 : 0, m46HiddenPlaced ? 1 : 0);
            std::fflush(stderr);
            if (!(m46HiddenShown && m46HiddenPlaced)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M46 hidden-tab indicator\n");
                return 180;
            }

            // 181: a cross-column drop onto a group body (and between two groups)
            // draws the line in the target column and lands the panel there.
            bool m46CrossShown = false;
            bool m46CrossPlaced = false;
            bool m46BoundaryShown = false;
            bool m46BoundaryPlaced = false;
            if (m41Column) {
                m41Column->setRailMode(false);
                m41Column->setPreferredWidth(180);
                m46Pump(6);
                pictura::PanelColumn* dest =
                    frame.createPanelColumn(pictura::PanelSide::Right, m41Column);
                QList<pictura::PanelGroup*> destGroups;
                for (pictura::PanelGroup* group : m41Column->groups()) {
                    if (destGroups.size() >= 2) {
                        break;
                    }
                    if (group && group->isVisible()) {
                        const QString nm = group->objectName();
                        if (dest) {
                            if (pictura::PanelGroup* taken = m41Column->takeGroup(nm)) {
                                dest->adoptGroup(taken);
                                destGroups << taken;
                            }
                        }
                    }
                }
                if (dest) {
                    m41Column->setPreferredWidth(180);
                    dest->setPreferredWidth(180);
                    m46Pump(6);
                }
                pictura::PanelGroup* bg =
                    destGroups.isEmpty() ? nullptr : destGroups.first();
                const QString src = m46OtherPanel(nullptr, QString());
                if (dest && dest != m41Column && bg && !src.isEmpty()) {
                    const QRect barRect = bg->tabBarGlobalRect();
                    const QRect groupRect(bg->mapToGlobal(QPoint(0, 0)), bg->size());
                    const QPoint body(groupRect.center().x(),
                                      (barRect.bottom() + groupRect.bottom()) / 2);
                    m41Column->beginTabDragForTest(src);
                    m41Column->dragToForTest(body);
                    m46CrossShown = dest->dropIndicatorVisibleForTest()
                                    && !m41Column->dropIndicatorVisibleForTest();
                    m41Column->dropForTest(body);
                    m46Pump(6);
                    m46CrossPlaced = dest->groupForPanel(src) != nullptr;
                }
                if (dest && destGroups.size() >= 2) {
                    pictura::PanelGroup* g0 = destGroups.at(0);
                    pictura::PanelGroup* g1 = destGroups.at(1);
                    const QString src2 = m46OtherPanel(nullptr, QString());
                    if (g0 && g1 && !src2.isEmpty()) {
                        const QRect r0(g0->mapToGlobal(QPoint(0, 0)), g0->size());
                        const QRect r1(g1->mapToGlobal(QPoint(0, 0)), g1->size());
                        const QPoint boundary(r0.center().x(),
                                             (r0.bottom() + r1.top()) / 2);
                        m41Column->beginTabDragForTest(src2);
                        m41Column->dragToForTest(boundary);
                        m46BoundaryShown = dest->dropIndicatorVisibleForTest()
                                           && !m41Column->dropIndicatorVisibleForTest();
                        m41Column->dropForTest(boundary);
                        m46Pump(6);
                        m46BoundaryPlaced = dest->groupForPanel(src2) != nullptr;
                    }
                }
                m46CollapseDynamics();
            }
            std::fprintf(stderr,
                         "pictura self-test: m46_indicator_cross_body shown=%d placed=%d "
                         "boundary_shown=%d boundary_placed=%d\n",
                         m46CrossShown ? 1 : 0, m46CrossPlaced ? 1 : 0,
                         m46BoundaryShown ? 1 : 0, m46BoundaryPlaced ? 1 : 0);
            std::fflush(stderr);
            if (!(m46CrossShown && m46CrossPlaced && m46BoundaryShown
                  && m46BoundaryPlaced)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M46 cross-column indicator\n");
                return 181;
            }

            // 182: the bottom-boundary line is clamped inside the scroll
            // viewport so none of it is drawn past the visible area. Use a
            // fresh column with one group so the group fills the viewport and
            // its bottom is a reachable BelowGroup boundary.
            bool m46BottomInside = false;
            if (m41Column) {
                m41Column->setRailMode(false);
                m46Pump(4);
                pictura::PanelColumn* dest =
                    frame.createPanelColumn(pictura::PanelSide::Right, m41Column);
                if (dest) {
                    for (pictura::PanelGroup* group : m41Column->groups()) {
                        if (group && group->isVisible() && !group->visibleTitles().isEmpty()) {
                            if (pictura::PanelGroup* taken = m41Column->takeGroup(group->objectName())) {
                                dest->adoptGroup(taken);
                            }
                            break;
                        }
                    }
                    m46Pump(6);
                    QString src;
                    for (pictura::PanelGroup* group : dest->groups()) {
                        if (group && group->isVisible() && !group->visibleTitles().isEmpty()) {
                            src = group->visibleTitles().first();
                            break;
                        }
                    }
                    const QPoint p = dest->boundaryPointForTest(1000);
                    if (!src.isEmpty() && !p.isNull()) {
                        dest->beginTabDragForTest(src);
                        dest->dragToForTest(p);
                        const QRect g = dest->dropIndicatorGeometryForTest();
                        const int vh = dest->scrollViewportHeightForTest();
                        m46BottomInside =
                            g.isValid() && vh > 0 && g.top() >= 0 && g.bottom() <= vh;
                        dest->dropForTest(p);
                        m46Pump(6);
                    }
                }
                m46CollapseDynamics();
            }
            std::fprintf(stderr,
                         "pictura self-test: m46_indicator_bottom_inside inside=%d\n",
                         m46BottomInside ? 1 : 0);
            std::fflush(stderr);
            if (!m46BottomInside) {
                std::fprintf(stderr, "pictura self-test: FAIL: M46 bottom inside viewport\n");
                return 182;
            }

            // 183: a real title-bar press still drives the floating Tools drag
            // after Qt's dock mouse grab reroutes move/release to the dock.
            bool m46ToolsMoved = false;
            bool m46ToolsFinished = false;
            if (auto* tb =
                    frame.findChild<pictura::Toolbox*>(QStringLiteral("toolsPanel"))) {
                auto* cs = frame.findChild<QSplitter*>(QStringLiteral("centerSplitter"));
                if (cs && cs->indexOf(tb) >= 0) {
                    frame.addDockWidget(Qt::LeftDockWidgetArea, tb);
                    m46Pump(6);
                }
                if (!tb->isFloating()) {
                    tb->setFloating(true);
                    m46Pump(6);
                }
                int moved = 0;
                int finished = 0;
                const QMetaObject::Connection c1 = QObject::connect(
                    tb, &pictura::Toolbox::toolbarDragMoved, tb,
                    [&moved](const QPoint&) { ++moved; });
                const QMetaObject::Connection c2 = QObject::connect(
                    tb, &pictura::Toolbox::toolbarDragFinished, tb,
                    [&finished](const QPoint&) { ++finished; });
                if (QWidget* title = tb->titleBarWidget()) {
                    const QPoint titleCenter = title->rect().center();
                    const QPointF titleLocal(titleCenter);
                    const QPointF titleGlobal(title->mapToGlobal(titleCenter));
                    QMouseEvent press(QEvent::MouseButtonPress, titleLocal, titleGlobal,
                                      Qt::LeftButton, Qt::LeftButton, Qt::NoModifier);
                    QCoreApplication::sendEvent(title, &press);
                    const QPoint dockCenter = tb->rect().center();
                    const QPointF dockLocal(dockCenter);
                    const QPointF dockGlobal(tb->mapToGlobal(dockCenter));
                    QMouseEvent move(QEvent::MouseMove, dockLocal, dockGlobal, Qt::NoButton,
                                     Qt::LeftButton, Qt::NoModifier);
                    QCoreApplication::sendEvent(tb, &move);
                    m46ToolsMoved = moved > 0;
                    QMouseEvent release(QEvent::MouseButtonRelease, dockLocal, dockGlobal,
                                        Qt::LeftButton, Qt::NoButton, Qt::NoModifier);
                    QCoreApplication::sendEvent(tb, &release);
                    m46ToolsFinished = finished > 0;
                }
                QObject::disconnect(c1);
                QObject::disconnect(c2);
                tb->setFloating(false);
                m46Pump(6);
            }
            std::fprintf(stderr,
                         "pictura self-test: m46_tools_gesture moved=%d finished=%d\n",
                         m46ToolsMoved ? 1 : 0, m46ToolsFinished ? 1 : 0);
            std::fflush(stderr);
            if (!(m46ToolsMoved && m46ToolsFinished)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M46 tools gesture\n");
                return 183;
            }

            // 184: neither splitter collapses a pane, and the column cannot be
            // squeezed away below its shared floor.
            bool m46NoCollapse = false;
            bool m46ColumnKept = false;
            if (m41Column) {
                auto* cs = frame.findChild<QSplitter*>(QStringLiteral("centerSplitter"));
                auto* gs = m41Column->findChild<QSplitter*>(
                    QStringLiteral("panelColumnSplitter"));
                m46NoCollapse = cs && gs && !cs->childrenCollapsible()
                                && !gs->childrenCollapsible();
                m41Column->show();
                m41Column->setPreferredWidth(1);
                m46Pump(6);
                m46ColumnKept = m41Column->isVisible()
                                && m41Column->width() >= m41Column->minimumWidthForTest();
            }
            std::fprintf(stderr,
                         "pictura self-test: m46_splitter_nocollapse nocollapse=%d kept=%d\n",
                         m46NoCollapse ? 1 : 0, m46ColumnKept ? 1 : 0);
            std::fflush(stderr);
            if (!(m46NoCollapse && m46ColumnKept)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M46 splitter no collapse\n");
                return 184;
            }

            // 185: minimizing a group whose content minimum exceeds its tab bar
            // clamps the group to the tab-bar height and hides the content.
            bool m46MinTall = false;
            if (m41Column) {
                m41Column->setRailMode(false);
                m41Column->setPreferredWidth(360);
                m41Column->showPanel(QStringLiteral("colorPanel"), true);
                m46Pump(8);
                pictura::PanelGroup* g =
                    m41Column->groupForPanel(QStringLiteral("colorPanel"));
                if (!g) {
                    g = m41Column->groupForPanel(QStringLiteral("swatchesPanel"));
                }
                if (g) {
                    g->setMinimizedForTest(false);
                    m46Pump(8);
                    g->setMinimizedForTest(true);
                    g->updateGeometry();
                    m41Column->updateGeometry();
                    m46Pump(16);
                    const int barH = g->tabBar() ? g->tabBar()->sizeHint().height() : 24;
                    m46MinTall = g->height() <= barH + 6 && g->contentHiddenForTest();
                    g->setMinimizedForTest(false);
                    g->updateGeometry();
                    m46Pump(8);
                }
            }
            std::fprintf(stderr, "pictura self-test: m46_minimize_tall height=%d\n",
                         m46MinTall ? 1 : 0);
            std::fflush(stderr);
            if (!m46MinTall) {
                std::fprintf(stderr, "pictura self-test: FAIL: M46 minimize tall group\n");
                return 185;
            }

            // 186: the primary column hides when its last visible panel closes
            // and re-shows on the next Window-menu show.
            bool m46PrimaryHide = false;
            bool m46PrimaryShow = false;
            if (m41Column) {
                const bool wasVisible = m41Column->isVisible();
                const QList<pictura::PanelGroup*> groups = m41Column->groups();
                for (pictura::PanelGroup* g : groups) {
                    if (g) {
                        m41Column->closeGroup(g);
                    }
                }
                m46Pump(8);
                m46PrimaryHide = !m41Column->isVisible();
                m41Column->showPanel(QStringLiteral("layersPanel"), true);
                m46Pump(8);
                m46PrimaryShow = m41Column->isVisible()
                                 && m41Column->groupForPanel(QStringLiteral("layersPanel"))
                                        != nullptr;
                if (!wasVisible || !m41Column->isVisible()) {
                    m41Column->show();
                }
            }
            std::fprintf(stderr,
                         "pictura self-test: m46_primary_empty hide=%d show=%d\n",
                         m46PrimaryHide ? 1 : 0, m46PrimaryShow ? 1 : 0);
            std::fflush(stderr);
            if (!(m46PrimaryHide && m46PrimaryShow)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M46 primary empty\n");
                return 186;
            }

            // M47 (187-195): empty-column-after-float, ghost-group hiding,
            // compact icon float, compact grip group creation, no horizontal
            // scroll, fixed iconic width, Tools pane hosting, float close, and
            // compact chrome shading. Exit codes 187-195.
            auto m47FirstVisiblePanel = [](pictura::PanelGroup* group) -> QString {
                if (!group) {
                    return QString();
                }
                const QList<QWidget*> visible = group->visiblePanels();
                return visible.isEmpty() ? QString() : visible.first()->objectName();
            };
            auto m47ShowPrimary = [&]() {
                m41Column->setRailMode(false);
                for (pictura::PanelGroup* group : m41Column->groups()) {
                    if (!group) {
                        continue;
                    }
                    for (QWidget* panel : group->panels()) {
                        if (panel) {
                            m41Column->showPanel(panel->objectName(), true);
                        }
                    }
                }
                m46Pump(8);
            };
            auto m47AdoptOneInto = [&](pictura::PanelColumn* dest) -> bool {
                if (!dest) {
                    return false;
                }
                for (pictura::PanelGroup* group : m41Column->groups()) {
                    if (group && !group->visiblePanels().isEmpty()) {
                        if (pictura::PanelGroup* taken =
                                m41Column->takeGroup(group->objectName())) {
                            dest->adoptGroup(taken);
                            return true;
                        }
                    }
                }
                return false;
            };
            auto m47CloseAllFloats = [&]() {
                for (int i = 0; i < 24 && m41Column->floatCountForTest() > 0; ++i) {
                    const int count = m41Column->floatCountForTest();
                    // M47: a re-homed float's close callback is repointed at the
                    // primary, so closing tears the overlay down; fall back to a
                    // re-dock only if a float refuses to close.
                    if (!m41Column->closeFloatForTest(0)
                        && !m41Column->redockForTest(0, 0)) {
                        m41Column->cancelDragForTest();
                    }
                    m46Pump(4);
                    if (m41Column->floatCountForTest() >= count) {
                        break;
                    }
                }
            };

            // 187: the last group of a dynamic column is torn off and left
            // floating; the now-empty column is removed and its float is
            // re-homed to the primary (still live and re-dockable).
            bool m47EmptyGone = false;
            bool m47EmptyFloat = false;
            bool m47EmptyPrimary = false;
            if (m41Column) {
                m47ShowPrimary();
                m47CloseAllFloats();
                const int floatsBefore = m41Column->floatCountForTest();
                const int before = frame.panelColumnCountForTest();
                pictura::PanelColumn* dest =
                    frame.createPanelColumn(pictura::PanelSide::Right, m41Column);
                pictura::PanelGroup* destGroup = nullptr;
                if (dest) {
                    for (pictura::PanelGroup* group : m41Column->groups()) {
                        if (group && !group->visiblePanels().isEmpty()) {
                            if (pictura::PanelGroup* taken =
                                    m41Column->takeGroup(group->objectName())) {
                                dest->adoptGroup(taken);
                                destGroup = taken;
                            }
                            break;
                        }
                    }
                    m46Pump(6);
                }
                const QString adoptPanel = m47FirstVisiblePanel(destGroup);
                if (dest && !adoptPanel.isEmpty()) {
                    // A point over the source column's own header is outside its
                    // scroll viewport (and resolves to no sibling), so the drag
                    // leaves a live float without re-docking it.
                    const QPoint outside =
                        dest->mapToGlobal(QPoint(dest->width() / 2, 3));
                    dest->beginGroupDragForTest(adoptPanel);
                    dest->dragToForTest(outside);
                    dest->dropForTest(outside);
                    m46Pump(8);
                }
                m47EmptyGone = dest && frame.panelColumnCountForTest() == before;
                m47EmptyFloat = m41Column->floatCountForTest() == floatsBefore + 1;
                m47EmptyPrimary = frame.panelColumn() == m41Column && m41Column->isVisible();
                // A float re-homed by the removal must close through the primary
                // (its close callback is repointed), not strand the overlay.
                const bool m47RehomedCloses =
                    m41Column->closeFloatForTest(0)
                    && m41Column->floatCountForTest() == floatsBefore;
                m47EmptyFloat = m47EmptyFloat && m47RehomedCloses;
                m47CloseAllFloats();
                m46CollapseDynamics();
            }
            std::fprintf(stderr,
                         "pictura self-test: m47_empty_after_float gone=%d float=%d "
                         "primary=%d\n",
                         m47EmptyGone ? 1 : 0, m47EmptyFloat ? 1 : 0,
                         m47EmptyPrimary ? 1 : 0);
            std::fflush(stderr);
            if (!(m47EmptyGone && m47EmptyFloat && m47EmptyPrimary)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M47 empty after float\n");
                return 187;
            }

            // 188: moving the last visible tab out of a group hides the emptied
            // group (no ghost shell) while its hidden panel stays reachable and
            // re-showable.
            bool m47GhostHidden = false;
            bool m47GhostFound = false;
            bool m47GhostShown = false;
            if (m41Column) {
                m47ShowPrimary();
                pictura::PanelGroup* ghost =
                    m41Column->groupForPanel(QStringLiteral("adjustmentsPanel"));
                pictura::PanelGroup* groupTarget =
                    m41Column->groupForPanel(QStringLiteral("colorPanel"));
                if (ghost && groupTarget && ghost != groupTarget
                    && ghost->titleCountForTest() >= 2) {
                    ghost->setPanelVisible(QStringLiteral("propertiesPanel"), false);
                    m46Pump(4);
                    const QString last = m47FirstVisiblePanel(ghost);
                    const QPoint p = groupTarget->tabInsertionGlobalPointForTest(0);
                    const bool began =
                        !last.isEmpty() && m41Column->beginTabDragForTest(last);
                    m41Column->dragToForTest(p);
                    const bool dropped = m41Column->dropForTest(p);
                    m46Pump(8);
                    m47GhostHidden = began && dropped && !ghost->isVisible();
                    m47GhostFound =
                        m41Column->groupForPanel(QStringLiteral("propertiesPanel")) == ghost;
                    m41Column->showPanel(QStringLiteral("propertiesPanel"), true);
                    m46Pump(8);
                    m47GhostShown =
                        ghost->isVisible()
                        && m41Column->groupForPanel(QStringLiteral("propertiesPanel")) == ghost;
                }
                m46CollapseDynamics();
            }
            std::fprintf(stderr,
                         "pictura self-test: m47_ghost_group hidden=%d found=%d shown=%d\n",
                         m47GhostHidden ? 1 : 0, m47GhostFound ? 1 : 0,
                         m47GhostShown ? 1 : 0);
            std::fflush(stderr);
            if (!(m47GhostHidden && m47GhostFound && m47GhostShown)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M47 ghost group\n");
                return 188;
            }
            // 189: a real press+move on a compact strip icon tears the panel
            // into a float that survives the (now non-rebuilding) strip and
            // commits on release.
            bool m47IconDrag = false;
            bool m47IconFloat = false;
            bool m47IconReleased = false;
            if (m41Column) {
                m47ShowPrimary();
                m47CloseAllFloats();
                const int floatsBefore = m41Column->floatCountForTest();
                m41Column->setRailMode(true);
                m46Pump(8);
                QToolButton* button = nullptr;
                for (QToolButton* candidate : m41Column->findChildren<QToolButton*>()) {
                    if (candidate
                        && candidate->objectName().startsWith(QStringLiteral("panelIcon_"))
                        && candidate->isVisible()) {
                        button = candidate;
                        break;
                    }
                }
                const QPoint outside =
                    m41Column->mapToGlobal(QPoint(-40, m41Column->height() / 2));
                if (button) {
                    const QPoint local = button->rect().center();
                    const QPointF localF(local);
                    const QPointF globalF(button->mapToGlobal(local));
                    QMouseEvent press(QEvent::MouseButtonPress, localF, globalF,
                                      Qt::LeftButton, Qt::LeftButton, Qt::NoModifier);
                    QCoreApplication::sendEvent(button, &press);
                    const QPointF outsideF(outside);
                    QMouseEvent move(QEvent::MouseMove, outsideF, outsideF, Qt::NoButton,
                                     Qt::LeftButton, Qt::NoModifier);
                    QCoreApplication::sendEvent(button, &move);
                    m46Pump(4);
                    m47IconDrag = m41Column->dragActiveForTest();
                    m47IconFloat = m41Column->floatCountForTest() == floatsBefore + 1;
                    QMouseEvent release(QEvent::MouseButtonRelease, outsideF, outsideF,
                                        Qt::LeftButton, Qt::NoButton, Qt::NoModifier);
                    QCoreApplication::sendEvent(button, &release);
                    m46Pump(8);
                    m47IconReleased = !m41Column->dragActiveForTest();
                }
                m47CloseAllFloats();
                m41Column->setRailMode(false);
                m46Pump(6);
                m46CollapseDynamics();
            }
            std::fprintf(stderr,
                         "pictura self-test: m47_compact_icon_float drag=%d float=%d "
                         "released=%d\n",
                         m47IconDrag ? 1 : 0, m47IconFloat ? 1 : 0,
                         m47IconReleased ? 1 : 0);
            std::fflush(stderr);
            if (!(m47IconDrag && m47IconFloat && m47IconReleased)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M47 compact icon float\n");
                return 189;
            }
            // 190: a single compact panel dropped on a group's grip creates a
            // new group immediately above (before) that group.
            bool m47GripGroup = false;
            if (m41Column) {
                m47ShowPrimary();
                m41Column->setRailMode(true);
                m46Pump(8);
                pictura::PanelGroup* groupTarget = nullptr;
                QString src;
                for (pictura::PanelGroup* group : m41Column->groups()) {
                    if (!group || group->visiblePanels().isEmpty()) {
                        continue;
                    }
                    if (!groupTarget) {
                        groupTarget = group;
                    } else if (src.isEmpty()) {
                        src = group->visiblePanels().first()->objectName();
                    }
                }
                if (groupTarget && !src.isEmpty()) {
                    QWidget* grip = m41Column->dragHandleForTest(groupTarget->objectName());
                    const int targetIndex = m41Column->groups().indexOf(groupTarget);
                    if (grip && targetIndex >= 0) {
                        const QPoint gripCenter = grip->mapToGlobal(grip->rect().center());
                        const bool began = m41Column->beginTabDragForTest(src);
                        m41Column->dragToForTest(gripCenter);
                        const bool dropped = m41Column->dropForTest(gripCenter);
                        m46Pump(8);
                        pictura::PanelGroup* landed = m41Column->groupForPanel(src);
                        const int landedIndex =
                            landed ? m41Column->groups().indexOf(landed) : -1;
                        const int targetNow = m41Column->groups().indexOf(groupTarget);
                        m47GripGroup = began && dropped && landed && landed != groupTarget
                                       && landedIndex == targetIndex
                                       && targetNow == targetIndex + 1;
                    }
                }
                m41Column->setRailMode(false);
                m46Pump(6);
                m46CollapseDynamics();
            }
            std::fprintf(stderr, "pictura self-test: m47_compact_grip_group above=%d\n",
                         m47GripGroup ? 1 : 0);
            std::fflush(stderr);
            if (!m47GripGroup) {
                std::fprintf(stderr, "pictura self-test: FAIL: M47 compact grip group\n");
                return 190;
            }
            // 191: the column never scrolls horizontally, even at its minimum
            // width, because the shared floor keeps the content visible.
            bool m47HScrollOff = false;
            bool m47HScrollZero = false;
            if (m41Column) {
                m41Column->setRailMode(false);
                m46Pump(4);
                m47HScrollOff = m41Column->horizontalScrollPolicyForTest()
                                == static_cast<int>(Qt::ScrollBarAlwaysOff);
                m41Column->setPreferredWidth(m41Column->minimumWidthForTest());
                m46Pump(8);
                m47HScrollZero = m41Column->horizontalScrollRangeForTest() == 0;
                m46CollapseDynamics();
            }
            std::fprintf(stderr,
                         "pictura self-test: m47_no_hscroll off=%d range=%d\n",
                         m47HScrollOff ? 1 : 0, m47HScrollZero ? 1 : 0);
            std::fflush(stderr);
            if (!(m47HScrollOff && m47HScrollZero)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M47 no hscroll\n");
                return 191;
            }
            // 192: an iconic column is fixed to its strip width and a preferred
            // width change cannot grow it; leaving iconic clears the maximum.
            bool m47IconicWidth = false;
            bool m47IconicMax = false;
            bool m47IconicNoGrow = false;
            bool m47IconicExit = false;
            if (m41Column) {
                m47ShowPrimary();
                m41Column->setRailMode(true);
                m46Pump(8);
                const int stripMin = m41Column->minimumWidthForTest();
                const int iconicWidth = m41Column->width();
                m47IconicWidth = stripMin > 0 && stripMin <= 60 && iconicWidth == stripMin;
                m47IconicMax = m41Column->maximumWidth() == m41Column->minimumWidth();
                m41Column->setPreferredWidth(400);
                m46Pump(8);
                m47IconicNoGrow = m41Column->width() <= stripMin;
                m41Column->setRailMode(false);
                m46Pump(8);
                m47IconicExit = m41Column->maximumWidth() > stripMin;
                m46CollapseDynamics();
            }
            std::fprintf(stderr,
                         "pictura self-test: m47_iconic_fixed_width width=%d max=%d "
                         "nogrow=%d exit=%d\n",
                         m47IconicWidth ? 1 : 0, m47IconicMax ? 1 : 0,
                         m47IconicNoGrow ? 1 : 0, m47IconicExit ? 1 : 0);
            std::fflush(stderr);
            if (!(m47IconicWidth && m47IconicMax && m47IconicNoGrow && m47IconicExit)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M47 iconic fixed width\n");
                return 192;
            }
            // 193: the Tools panel is dropped between two widget columns and is
            // hosted as a fixed-width central-splitter pane at that index.
            bool m47ToolsResolved = false;
            bool m47ToolsPane = false;
            bool m47ToolsIndex = false;
            if (auto* toolbox =
                    frame.findChild<pictura::Toolbox*>(QStringLiteral("toolsPanel"))) {
                auto* cs = frame.findChild<QSplitter*>(QStringLiteral("centerSplitter"));
                if (cs) {
                    if (cs->indexOf(toolbox) >= 0) {
                        frame.addDockWidget(Qt::LeftDockWidgetArea, toolbox);
                        m46Pump(6);
                    }
                    m47ShowPrimary();
                    pictura::PanelColumn* c1 =
                        frame.createPanelColumn(pictura::PanelSide::Right, m41Column);
                    m46Pump(6);
                    m47AdoptOneInto(c1);
                    m46Pump(6);
                    pictura::PanelColumn* c2 =
                        c1 ? frame.createPanelColumn(pictura::PanelSide::Right, c1) : nullptr;
                    m46Pump(6);
                    m47AdoptOneInto(c2);
                    m46Pump(8);
                    if (c1 && c2) {
                        const QRect r1(c1->mapToGlobal(QPoint(0, 0)), c1->size());
                        const QRect r2(c2->mapToGlobal(QPoint(0, 0)), c2->size());
                        const QPoint between((r1.right() + r2.left()) / 2,
                                             (r1.center().y() + r2.center().y()) / 2);
                        pictura::PanelColumn* anchor = nullptr;
                        int side = -1;
                        const bool resolved =
                            frame.resolveToolboxDrop(between, &anchor, &side);
                        const int anchorIndex =
                            anchor ? cs->indexOf(anchor) : -1;
                        const int expected =
                            anchorIndex < 0 ? -1
                                            : (side == 0 ? anchorIndex : anchorIndex + 1);
                        const bool committed = frame.commitToolboxDrop(between);
                        m47ToolsResolved = resolved && anchor != nullptr;
                        m47ToolsPane = committed && toolbox->isSplitterPane()
                                       && cs->indexOf(toolbox) >= 0;
                        m47ToolsIndex = expected >= 0 && cs->indexOf(toolbox) == expected;
                    }
                    frame.addDockWidget(Qt::LeftDockWidgetArea, toolbox);
                    toolbox->setSplitterPane(false);
                    m46Pump(6);
                }
                m46CollapseDynamics();
            }
            std::fprintf(stderr,
                         "pictura self-test: m47_tools_pane resolved=%d pane=%d index=%d\n",
                         m47ToolsResolved ? 1 : 0, m47ToolsPane ? 1 : 0,
                         m47ToolsIndex ? 1 : 0);
            std::fflush(stderr);
            if (!(m47ToolsResolved && m47ToolsPane && m47ToolsIndex)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M47 tools pane\n");
                return 193;
            }
            // 194: a floating group shows a visible close control; closing it
            // removes the overlay, hides the group's panels, and keeps the group
            // restorable in a column.
            bool m47FloatCloseBtn = false;
            bool m47FloatClosed = false;
            bool m47FloatRestore = false;
            if (m41Column) {
                m47ShowPrimary();
                m47CloseAllFloats();
                pictura::PanelGroup* floater = nullptr;
                for (pictura::PanelGroup* group : m41Column->groups()) {
                    if (group && !group->visiblePanels().isEmpty()) {
                        floater = group;
                        break;
                    }
                }
                const QString hiddenName = m47FirstVisiblePanel(floater);
                if (floater && !hiddenName.isEmpty()) {
                    const bool torn = m41Column->tearOffForTest(floater->objectName());
                    m46Pump(8);
                    QToolButton* close = m41Column->floatCountForTest() == 1
                                             ? m41Column->floatCloseButtonForTest(0)
                                             : nullptr;
                    m47FloatCloseBtn = torn && close && close->isVisible();
                    const bool clicked = m41Column->closeFloatForTest(0);
                    m46Pump(8);
                    pictura::PanelGroup* back = m41Column->groupForPanel(hiddenName);
                    m47FloatClosed = clicked && m41Column->floatCountForTest() == 0
                                     && back != nullptr && !back->isVisible();
                    if (back) {
                        m41Column->showPanel(hiddenName, true);
                        m46Pump(6);
                        m47FloatRestore = back->isVisible();
                    }
                }
                m46CollapseDynamics();
            }
            std::fprintf(stderr,
                         "pictura self-test: m47_float_close button=%d closed=%d "
                         "restore=%d\n",
                         m47FloatCloseBtn ? 1 : 0, m47FloatClosed ? 1 : 0,
                         m47FloatRestore ? 1 : 0);
            std::fflush(stderr);
            if (!(m47FloatCloseBtn && m47FloatClosed && m47FloatRestore)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M47 float close\n");
                return 194;
            }
            // 195: the compact group container uses the panel surface shade and
            // its drag dots are dark gray, not the near-white window text.
            bool m47Shade = false;
            bool m47Dots = false;
            {
                const QString ss = qApp->styleSheet();
                const QColor windowColor = qApp->palette().color(QPalette::Window);
                const QColor disabledColor =
                    qApp->palette().color(QPalette::Disabled, QPalette::WindowText);
                const QString windowHex = windowColor.name(QColor::HexRgb);
                const QString disabledHex = disabledColor.name(QColor::HexRgb);
                m47Shade = ss.contains(
                    QStringLiteral("QWidget#panelIconGroup { background: ") + windowHex);
                const int grip = ss.indexOf(QStringLiteral("QWidget#panelIconGroupGrip {"));
                if (grip >= 0) {
                    const int end = ss.indexOf(QLatin1Char('}'), grip);
                    const QString rule = ss.mid(grip, end - grip);
                    m47Dots = rule.contains(QStringLiteral("color: ") + disabledHex);
                }
            }
            std::fprintf(stderr,
                         "pictura self-test: m47_compact_shade shade=%d dots=%d\n",
                         m47Shade ? 1 : 0, m47Dots ? 1 : 0);
            std::fflush(stderr);
            if (!(m47Shade && m47Dots)) {
                std::fprintf(stderr, "pictura self-test: FAIL: M47 compact shade\n");
                return 195;
            }
        }

        frame.closeDocument(m39DocIndex, false);
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

    return 0;
}
