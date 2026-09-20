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
            const bool appliedOk = frame.panelColumnCountForTest() == 2 && restoredLeft
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
