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

int main(int argc, char* argv[])
{
    // xvfb-run sets DISPLAY but a live Wayland session's WAYLAND_DISPLAY leaks
    // through, so Qt prefers the Wayland plugin. There a popup opened without a
    // preceding input event cannot grab and the compositor dismisses it, which
    // makes the UI self-tests flaky. Route the self-test back to the X display.
    bool selfTestArg = false;
    bool headlessArg = false;
    for (int i = 1; i < argc; ++i) {
        selfTestArg = selfTestArg || std::strcmp(argv[i], "--self-test") == 0;
        headlessArg = headlessArg || std::strcmp(argv[i], "--headless") == 0;
    }
    // A headless run must not need X or Wayland; --headless wins over the
    // self-test xcb routing below because it sets the variable that guard checks.
    if (headlessArg && !qEnvironmentVariableIsSet("QT_QPA_PLATFORM")) {
        qputenv("QT_QPA_PLATFORM", "offscreen");
    }
    if (selfTestArg && qEnvironmentVariableIsSet("DISPLAY")
        && qEnvironmentVariableIsSet("WAYLAND_DISPLAY")
        && !qEnvironmentVariableIsSet("QT_QPA_PLATFORM")) {
        qputenv("QT_QPA_PLATFORM", "xcb");
    }

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
    bool headless = false;
    QString psdPath;
    for (int i = 1; i < args.size(); ++i) {
        if (args.at(i) == QStringLiteral("--self-test")) {
            selfTest = true;
        } else if (args.at(i) == QStringLiteral("--interop-probe")) {
            interopProbe = true;
        } else if (args.at(i) == QStringLiteral("--headless")) {
            headless = true;
        } else if (!args.at(i).startsWith(QLatin1Char('-'))) {
            psdPath = args.at(i);
        }
    }

    // A bare --headless run has nothing to show and no way to quit; run the
    // bridge self-test instead of blocking in the event loop.
    if (headless && !selfTest && psdPath.isEmpty()) {
        selfTest = true;
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
    // A normal launch opens no document; only the self-test keeps the scratch
    // white document its null_image/fresh_white/gpu_blank checks need. The GPU
    // smoke probe reports 0 = no GPU, 1 = rendered non-blank, 2 = blank. The
    // probe never replaces the document image (E1), so the scratch document
    // presents pure white rather than the probe's gradient.
    int gpu = 0;
    if (pictura::launchCreatesScratchDocument(selfTest, codecLoaded)) {
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
        const int rc = runSelfTest(app, headless, psdPath, frame, view, image,
                                   codecLoaded, gpu);
        if (rc != 0) {
            return rc;
        }
    }

    return app.exec();
}
