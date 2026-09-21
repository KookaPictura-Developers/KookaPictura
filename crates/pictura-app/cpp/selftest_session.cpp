#include "selftest_session.h"
#include "selftest_report.h"

#include "frame.h"
#include "panels/panel_column.h"
#include "session.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QCoreApplication>
#include <QtCore/QDir>
#include <QtCore/QFile>
#include <QtCore/QJsonArray>
#include <QtCore/QJsonObject>
#include <QtCore/QJsonValue>
#include <QtCore/QMetaObject>
#include <QtGui/QColor>
#include <QtGui/QImage>
#include <QtWidgets/QApplication>
#include <QtWidgets/QSplitter>
#include <QtWidgets/QTabBar>
#include <QtWidgets/QTabWidget>

namespace pictura {

namespace {

void pump(int count)
{
    for (int i = 0; i < count; ++i) {
        QCoreApplication::processEvents();
    }
}

QString writeSolidImage(const QString& path, const QColor& color)
{
    QImage image(4, 4, QImage::Format_RGBA8888);
    image.fill(color);
    return image.save(path, "PNG") ? path : QString();
}

} // namespace

int runSessionChecks(pictura::PicturaMainWindow& frame)
{
        // lpr_quit_save (317): the application's quit signal (what File > Exit
        // and Ctrl+Q reach) runs the session save even though `closeEvent` never
        // fires. A sentinel `layoutRevision` is overwritten by the live save.
        {
            pictura::SessionState sentinel = pictura::loadSession();
            sentinel.layoutRevision = 12345;
            pictura::saveSession(sentinel);
            if (qApp) {
                // `aboutToQuit` carries a `QPrivateSignal`, so emit it through
                // the meta-object exactly as `QCoreApplication::quit()` does.
                QMetaObject::invokeMethod(qApp, "aboutToQuit", Qt::DirectConnection);
            }
            const pictura::SessionState after = pictura::loadSession();
            const bool quitSaved =
                after.layoutRevision == PicturaMainWindow::kLayoutRevision
                && after.schemaVersion >= 7;
            ST_BEGIN("lpr_quit_save");
            ST_PASS("lpr_quit_save revision=%d schema=%d", after.layoutRevision,
                    after.schemaVersion);
            if (!quitSaved) {
                return pictura::selfTest().fail(317, "quit-path session save");
            }
        }

        // lpr_width_restart (318): resizing the columns and quitting saves each
        // column's width; re-running the startup restore applies it and each
        // width is measurable after the first layout.
        {
            frame.applyPanelSessionForTest(pictura::SessionState{});
            pump(6);
            pictura::PanelColumn* primary = frame.panelColumn();
            const bool madeLeft =
                frame.newColumnDropForTest(QStringLiteral("stylesPanel"), QStringLiteral("left"));
            pump(4);
            pictura::PanelColumn* left = frame.columnForPanel(QStringLiteral("stylesPanel"));
            if (left) {
                left->setPreferredWidth(205);
            }
            if (primary) {
                primary->setPreferredWidth(245);
            }
            pump(6);
            const int leftWidth = left ? left->width() : -1;
            const int primaryWidth = primary ? primary->width() : -1;
            frame.saveSession();
            const pictura::SessionState loaded = pictura::loadSession();

            int storedLeft = -1;
            int storedRight = -1;
            for (const QJsonValue& value : loaded.panelColumns) {
                const QJsonObject entry = value.toObject();
                if (entry.value(QStringLiteral("tools")).toBool()) {
                    continue;
                }
                const int width = entry.value(QStringLiteral("width")).toInt(-1);
                if (entry.value(QStringLiteral("side")).toString() == QStringLiteral("left")
                    && storedLeft < 0) {
                    storedLeft = width;
                } else if (entry.value(QStringLiteral("side")).toString()
                               == QStringLiteral("right")
                           && storedRight < 0) {
                    storedRight = width;
                }
            }
            const bool storedOk =
                madeLeft && leftWidth > 0 && primaryWidth > 0 && storedLeft == leftWidth
                && storedRight == primaryWidth;

            frame.applyPanelSessionForTest(loaded);
            pump(8);
            pictura::PanelColumn* restoredLeft =
                frame.columnForPanel(QStringLiteral("stylesPanel"));
            const bool appliedOk = frame.panelColumnCountForTest() == 3 && restoredLeft
                && restoredLeft->width() == leftWidth
                && frame.panelColumn()->width() == primaryWidth;
            ST_BEGIN("lpr_width_restart");
            ST_PASS("lpr_width_restart left=%d/%d right=%d/%d columns=%d", leftWidth,
                    storedLeft, primaryWidth, storedRight, frame.panelColumnCountForTest());
            if (!storedOk || !appliedOk) {
                return pictura::selfTest().fail(318, "column width restart");
            }
        }

        // lpr_v7_width (319): a v6 per-column entry with no width loads the
        // default, except the primary column, which is seeded from the legacy
        // top-level `railWidth`. An iconic store exposes the loaded normal width
        // through `persistedWidth()` without the normal-mode minimum clamping it.
        {
            frame.applyPanelSessionForTest(pictura::SessionState{});
            pump(6);
            auto writeV6 = [](int railWidth) {
                QFile store(pictura::sessionFilePath());
                if (!store.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
                    return false;
                }
                const QByteArray json =
                    QStringLiteral("{\"schemaVersion\":6,\"panelRailMode\":\"iconic\","
                                   "\"railWidth\":%1,\"panelColumns\":[{\"side\":\"right\","
                                   "\"order\":0,\"groups\":[]}]}")
                        .arg(railWidth)
                        .toUtf8();
                const bool ok = store.write(json) == json.size();
                store.close();
                return ok;
            };

            const bool seededWritten = writeV6(234);
            frame.applyPanelSessionForTest(pictura::loadSession());
            pump(8);
            pictura::PanelColumn* primary = frame.panelColumn();
            const bool iconic = primary && primary->railMode();
            const int seededWidth = primary ? primary->persistedWidth() : -1;

            const bool defaultWritten = writeV6(0);
            frame.applyPanelSessionForTest(pictura::loadSession());
            pump(8);
            primary = frame.panelColumn();
            const int defaultWidth = primary ? primary->persistedWidth() : -1;
            const bool v7Ok = seededWritten && defaultWritten && iconic && primary
                && primary->railMode() && seededWidth == 234
                && defaultWidth == pictura::PanelColumn::kDefaultNormalWidth;
            ST_BEGIN("lpr_v7_width");
            ST_PASS("lpr_v7_width seeded=%d default=%d/%d", seededWidth, defaultWidth,
                    pictura::PanelColumn::kDefaultNormalWidth);
            if (!v7Ok) {
                return pictura::selfTest().fail(319, "v6 default width");
            }
            frame.applyPanelSessionForTest(pictura::SessionState{});
            pump(6);
        }

        // lpr_iconic_flip_width (399): an iconic->normal flip persists a sane
        // normal width, never the icon strip and never an oversized value that
        // would expand the column across the workspace on the next launch.
        {
            frame.applyPanelSessionForTest(pictura::SessionState{});
            pump(6);
            pictura::PanelColumn* primary = frame.panelColumn();
            if (!primary) {
                return pictura::selfTest().fail(399, "iconic flip fixture");
            }
            primary->setPreferredWidth(245);
            pump(6);
            primary->setRailMode(true);
            const int kRemembered = 320;
            primary->setRestoredWidth(kRemembered);
            primary->setRailMode(false);
            frame.saveSession();

            int storedRight = -1;
            const pictura::SessionState loaded = pictura::loadSession();
            for (const QJsonValue& value : loaded.panelColumns) {
                const QJsonObject entry = value.toObject();
                if (entry.value(QStringLiteral("side")).toString() == QStringLiteral("right")
                    && storedRight < 0) {
                    storedRight = entry.value(QStringLiteral("width")).toInt(-1);
                }
            }
            primary->setRailMode(true);
            primary->setRestoredWidth(1000000);
            const int clamped = primary->persistedWidth();
            frame.applyPanelSessionForTest(loaded);
            pump(8);
            primary = frame.panelColumn();
            const bool restoredOk = primary && !primary->railMode();
            const bool storedOk = storedRight >= pictura::PanelColumn::kMinNormalWidth
                && storedRight <= pictura::PanelColumn::kMaxNormalWidth;
            ST_BEGIN("lpr_iconic_flip_width");
            ST_PASS("lpr_iconic_flip_width stored=%d remembered=%d clamped=%d "
                    "restoredMode=%d width=%d",
                    storedRight, kRemembered, clamped, restoredOk ? 1 : 0,
                    primary ? primary->width() : -1);
            if (!storedOk || !restoredOk
                || clamped != pictura::PanelColumn::kMaxNormalWidth) {
                return pictura::selfTest().fail(399, "iconic->normal width persistence");
            }
            // Leave a sane store behind.
            frame.applyPanelSessionForTest(pictura::SessionState{});
            pump(6);
            if (pictura::PanelColumn* reset = frame.panelColumn()) {
                reset->setPreferredWidth(245);
            }
            frame.saveSession();
        }

        // lpr_workspace_space (400): with no document open the workspace pane
        // stays in the splitter at its minimum width and keeps the stretch, so
        // the widget columns can never absorb it and the splitter keeps a
        // grabbable handle on both sides; an iconic column stays at its fixed
        // strip width, and shrinking a normal column gives the slack back to the
        // workspace rather than to a hidden pane.
        {
            frame.applyPanelSessionForTest(pictura::SessionState{});
            pump(6);
            while (frame.documentCount() > 0) {
                frame.closeDocument(0, false);
            }
            pump(6);
            auto* tabs = frame.findChild<QTabWidget*>(QStringLiteral("documentTabs"));
            auto* cs = frame.findChild<QSplitter*>(QStringLiteral("centerSplitter"));
            const int wsIndex = (cs && tabs) ? cs->indexOf(tabs) : -1;
            const int splitterW = cs ? cs->width() : -1;
            const int wsWidth = tabs ? tabs->width() : -1;
            const bool paneShown = tabs && tabs->isVisible();
            const bool stripHidden =
                tabs && tabs->tabBar() && !tabs->tabBar()->isVisible();
            const bool handleLeft =
                cs && wsIndex <= 0 ? true : (cs->handle(wsIndex - 1) != nullptr);
            const bool handleRight = cs && wsIndex >= 0 && wsIndex < cs->count() - 1
                                     && cs->handle(wsIndex) != nullptr;
            int columnsW = 0;
            for (pictura::PanelColumn* column : frame.panelColumns()) {
                columnsW += column->width();
            }
            const bool reserved = splitterW > 0 && wsWidth >= tabs->minimumWidth()
                                  && columnsW + wsWidth <= splitterW;

            pictura::PanelColumn* primary = frame.panelColumn();
            bool iconicFixed = false;
            bool wsAbsorbs = false;
            int beforeWs = -1;
            int beforeCol = -1;
            int afterWs = -1;
            int afterCol = -1;
            if (primary) {
                primary->setRailMode(true);
                pump(6);
                iconicFixed = primary->width() > 0
                              && primary->width() == primary->minimumWidthForTest()
                              && primary->maximumWidth() == primary->minimumWidth();
                primary->setRailMode(false);
                pump(6);
                beforeWs = tabs->width();
                beforeCol = primary->width();
                primary->setPreferredWidth(beforeCol + 60);
                pump(6);
                afterWs = tabs->width();
                afterCol = primary->width();
                // The workspace yields exactly what the column gains: the handle
                // sits between them, not beside a hidden pane.
                wsAbsorbs = afterCol > beforeCol
                            && (beforeWs - afterWs) == (afterCol - beforeCol);
            }
            ST_BEGIN("lpr_workspace_space");
            ST_PASS("lpr_workspace_space shown=%d strip=%d hl=%d hr=%d ws=%d cols=%d "
                    "split=%d iconic=%d absorbs=%d wsBefore=%d wsAfter=%d "
                    "colBefore=%d colAfter=%d",
                    paneShown ? 1 : 0, stripHidden ? 1 : 0, handleLeft ? 1 : 0,
                    handleRight ? 1 : 0, wsWidth, columnsW, splitterW,
                    iconicFixed ? 1 : 0, wsAbsorbs ? 1 : 0, beforeWs, afterWs,
                    beforeCol, afterCol);
            if (!(paneShown && stripHidden && handleLeft && handleRight && reserved
                  && iconicFixed && wsAbsorbs)) {
                return pictura::selfTest().fail(400, "workspace space");
            }
            frame.applyPanelSessionForTest(pictura::SessionState{});
            pump(6);
        }

        // ldt_mode_bits (320): an opened raster's tab reads `base (RGB/8)`
        // rather than a generated `Untitled-N` name.
        const QString modePath = QDir::tempPath() + QStringLiteral("/modebits.png");
        {
            const QString written =
                writeSolidImage(modePath, QColor(255, 0, 0, 255));
            const bool opened = !written.isEmpty() && frame.openImagePath(modePath);
            const int index = frame.activeDocumentIndex();
            const QString tabText = frame.documentTabTextForTest(index);
            const bool titleOk = opened && tabText == QStringLiteral("modebits.png (RGB/8)");
            ST_BEGIN("ldt_mode_bits");
            ST_PASS("ldt_mode_bits opened=%d title=%s", opened ? 1 : 0,
                    qPrintable(tabText));
            if (!titleOk) {
                return pictura::selfTest().fail(320, "tab mode/bits title");
            }
            frame.closeDocument(index, false);
        }
        QFile::remove(modePath);

        // ldt_untitled (321): a document with no name still shows `Untitled-N`.
        {
            const bool made = frame.newDocument(QStringLiteral("Untitled"), 8, 8,
                                                QStringLiteral("rgb"), 8,
                                                QStringLiteral("white"));
            const int index = frame.activeDocumentIndex();
            const QString name = frame.documentName(index);
            const QString tabText = frame.documentTabTextForTest(index);
            const bool untitledOk = made && name.startsWith(QStringLiteral("Untitled-"))
                && tabText.startsWith(name)
                && tabText.contains(QStringLiteral("(RGB/8)"));
            ST_BEGIN("ldt_untitled");
            ST_PASS("ldt_untitled name=%s title=%s", qPrintable(name),
                    qPrintable(tabText));
            if (!untitledOk) {
                return pictura::selfTest().fail(321, "untitled tab name");
            }
            frame.closeDocument(index, false);
        }

        // lim_opaque_background (322): a fully opaque import becomes the locked
        // `Background` with its redundant alpha channel dropped.
        const QString opaquePath =
            QDir::tempPath() + QStringLiteral("/kooka-pictura-opaque.png");
        {
            const QString written =
                writeSolidImage(opaquePath, QColor(20, 120, 220, 255));
            const bool opened = !written.isEmpty() && frame.openImagePath(opaquePath);
            pictura::PictureView* view = frame.activeView();
            const bool opaqueOk = opened && view && view->layer_row_count() == 1
                && view->layer_row_kind(0) == QStringLiteral("background")
                && view->layer_row_lock(0) == 0x05;
            ST_BEGIN("lim_opaque_background");
            ST_PASS("lim_opaque_background opened=%d kind=%s lock=%d", opened ? 1 : 0,
                    view ? qPrintable(view->layer_row_kind(0)) : "-",
                    view ? view->layer_row_lock(0) : -1);
            if (!opaqueOk) {
                return pictura::selfTest().fail(322, "opaque import background");
            }
            frame.closeDocument(frame.activeDocumentIndex(), false);
        }
        QFile::remove(opaquePath);

        // lim_transparent_layer (323): an import with a non-opaque pixel stays a
        // regular alpha layer (not a locked Background).
        const QString alphaPath =
            QDir::tempPath() + QStringLiteral("/kooka-pictura-alpha.png");
        {
            const QString written =
                writeSolidImage(alphaPath, QColor(0, 160, 0, 128));
            const bool opened = !written.isEmpty() && frame.openImagePath(alphaPath);
            pictura::PictureView* view = frame.activeView();
            const unsigned int pixel = view ? view->sample_argb(0, 0) : 0u;
            const bool alphaOk = opened && view && view->layer_row_count() == 1
                && view->layer_row_kind(0) != QStringLiteral("background")
                && view->layer_row_lock(0) != 0x0F
                && ((pixel >> 24) & 0xffu) != 0xffu;
            ST_BEGIN("lim_transparent_layer");
            ST_PASS("lim_transparent_layer opened=%d kind=%s lock=%d pixel=%08x",
                    opened ? 1 : 0,
                    view ? qPrintable(view->layer_row_kind(0)) : "-",
                    view ? view->layer_row_lock(0) : -1, pixel);
            if (!alphaOk) {
                return pictura::selfTest().fail(323, "transparent import layer");
            }
            frame.closeDocument(frame.activeDocumentIndex(), false);
        }
        QFile::remove(alphaPath);

        return 0;
}

} // namespace pictura
