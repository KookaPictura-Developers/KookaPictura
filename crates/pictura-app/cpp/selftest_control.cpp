#include "selftest_control.h"
#include "selftest_report.h"

#include "control_server.h"
#include "frame.h"

#include <QtCore/QEventLoop>
#include <QtCore/QJsonArray>
#include <QtCore/QJsonDocument>
#include <QtCore/QJsonObject>
#include <QtCore/QJsonParseError>
#include <QtCore/QTemporaryDir>
#include <QtCore/QTimer>
#include <QtGui/QImage>
#include <QtNetwork/QLocalSocket>

#include <sys/stat.h>

namespace {

// One blocking send/response round-trip. The server dispatches synchronously on
// the GUI thread, so a nested event loop drives the reply; the self-test never
// re-enters exec().
QByteArray transact(QLocalSocket& socket, const QByteArray& request, bool appendNewline = true,
                    int timeoutMs = 2000)
{
    socket.write(request + (appendNewline ? QByteArrayLiteral("\n") : QByteArray()));
    socket.flush();
    QByteArray buffer;
    QEventLoop loop;
    QTimer timer;
    timer.setSingleShot(true);
    QObject::connect(&timer, &QTimer::timeout, &loop, &QEventLoop::quit);
    QObject::connect(&socket, &QLocalSocket::readyRead, &loop, &QEventLoop::quit);
    timer.start(timeoutMs);
    for (;;) {
        buffer.append(socket.readAll());
        if (buffer.indexOf('\n') >= 0 || !timer.isActive()) {
            break;
        }
        loop.exec();
    }
    const int newline = buffer.indexOf('\n');
    if (newline < 0) {
        return QByteArray();
    }
    socket.readAll();
    return buffer.left(newline);
}

QJsonObject responseObject(const QByteArray& line, bool* parsed)
{
    QJsonParseError error{};
    const QJsonDocument document = QJsonDocument::fromJson(line, &error);
    *parsed = error.error == QJsonParseError::NoError && document.isObject();
    return *parsed ? document.object() : QJsonObject();
}

QJsonObject request(QLocalSocket& socket, int id, const QString& method, const QJsonObject& params,
                    bool* parsed)
{
    const QJsonObject body{{QStringLiteral("id"), id},
                           {QStringLiteral("method"), method},
                           {QStringLiteral("params"), params}};
    const QByteArray line =
        QJsonDocument(body).toJson(QJsonDocument::Compact) + QByteArrayLiteral("\n");
    return responseObject(transact(socket, line, false), parsed);
}

QString errorCode(const QJsonObject& response)
{
    return response.value(QStringLiteral("error"))
        .toObject()
        .value(QStringLiteral("code"))
        .toString();
}

} // namespace

int pictura::runControlChecks(pictura::PicturaMainWindow& frame)
{
    QTemporaryDir dir;
    const QString socketPath = dir.path() + QStringLiteral("/control.sock");
    pictura::ControlServer server(&frame, socketPath);
    ST_BEGIN("mcp_control_server");
    if (!dir.isValid() || !server.listen()) {
        return pictura::selfTest().fail(463, "control server listen");
    }
    ST_PASS("socket=%s", qPrintable(socketPath));

    QLocalSocket socket;
    socket.connectToServer(socketPath);
    if (!socket.waitForConnected(2000)) {
        return pictura::selfTest().fail(464, "control client connect");
    }

    struct stat status{};
    const bool userOnly = ::stat(socketPath.toLocal8Bit().constData(), &status) == 0
                          && (status.st_mode & 0077) == 0;
    ST_BEGIN("mcp_control_socket_mode");
    if (!userOnly) {
        return pictura::selfTest().fail(465, "socket is not user-only");
    }
    ST_PASS("mode=%03o", static_cast<int>(status.st_mode & 0777));

    bool parsed = false;
    QJsonObject response =
        request(socket, 1, QStringLiteral("list_commands"), QJsonObject(), &parsed);
    const QJsonArray commands = response.value(QStringLiteral("result"))
                                    .toObject()
                                    .value(QStringLiteral("commands"))
                                    .toArray();
    ST_BEGIN("mcp_control_commands");
    if (!parsed || !response.value(QStringLiteral("ok")).toBool() || commands.isEmpty()) {
        return pictura::selfTest().fail(466, "list_commands response malformed");
    }
    ST_PASS("commands=%d", static_cast<int>(commands.size()));

    response = request(socket, 2, QStringLiteral("status"), QJsonObject(), &parsed);
    const QJsonObject statusResult = response.value(QStringLiteral("result")).toObject();
    ST_BEGIN("mcp_control_status");
    if (!parsed || !response.value(QStringLiteral("ok")).toBool()
        || !statusResult.contains(QStringLiteral("documents"))
        || !statusResult.contains(QStringLiteral("documents_info"))
        || !statusResult.contains(QStringLiteral("active_tool"))) {
        return pictura::selfTest().fail(467, "status response malformed");
    }
    ST_PASS("documents=%d active_tool=%s", statusResult.value("documents").toInt(),
            qPrintable(statusResult.value("active_tool").toString()));

    const int initialDocuments = frame.documentCount();
    response = request(socket, 3, QStringLiteral("document"),
                       QJsonObject{{QStringLiteral("op"), QStringLiteral("new")},
                                   {QStringLiteral("width"), 8},
                                   {QStringLiteral("height"), 8}},
                       &parsed);
    ST_BEGIN("mcp_control_new");
    if (!parsed || !response.value(QStringLiteral("ok")).toBool()) {
        return pictura::selfTest().fail(468, "document new failed");
    }
    ST_PASS("count=%d", response.value("result").toObject().value("count").toInt());

    response = request(socket, 4, QStringLiteral("dispatch_command"),
                       QJsonObject{{QStringLiteral("id"), QStringLiteral("select.all")}}, &parsed);
    const QJsonObject dispatchResult = response.value(QStringLiteral("result")).toObject();
    ST_BEGIN("mcp_control_dispatch");
    if (!parsed || !response.value(QStringLiteral("ok")).toBool()
        || !dispatchResult.value(QStringLiteral("dispatched")).toBool()) {
        return pictura::selfTest().fail(469, "dispatch_command response malformed");
    }
    ST_PASS("dispatched=%d", dispatchResult.value("dispatched").toBool() ? 1 : 0);

    response = request(socket, 5, QStringLiteral("dispatch_command"),
                       QJsonObject{{QStringLiteral("id"), QStringLiteral("image.rotate90cw")}},
                       &parsed);
    const QJsonObject mutateResult = response.value(QStringLiteral("result")).toObject();
    ST_BEGIN("mcp_control_mutate");
    if (!parsed || !response.value(QStringLiteral("ok")).toBool()
        || !mutateResult.value(QStringLiteral("dispatched")).toBool()) {
        return pictura::selfTest().fail(482, "history mutation not dispatched");
    }
    ST_PASS("dispatched=%d", mutateResult.value("dispatched").toBool() ? 1 : 0);

    response = request(socket, 6, QStringLiteral("edit"),
                       QJsonObject{{QStringLiteral("op"), QStringLiteral("undo")}}, &parsed);
    const QJsonObject undoResult = response.value(QStringLiteral("result")).toObject();
    ST_BEGIN("mcp_control_edit");
    if (!parsed || !response.value(QStringLiteral("ok")).toBool()
        || !undoResult.value(QStringLiteral("success")).toBool()
        || !undoResult.value(QStringLiteral("can_redo")).toBool()
        || !undoResult.contains(QStringLiteral("depth"))) {
        return pictura::selfTest().fail(470, "edit undo did not succeed");
    }
    ST_PASS("undo success=1 can_redo=1 depth=%d", undoResult.value("depth").toInt());

    response = request(socket, 7, QStringLiteral("edit"),
                       QJsonObject{{QStringLiteral("op"), QStringLiteral("redo")}}, &parsed);
    const QJsonObject redoResult = response.value(QStringLiteral("result")).toObject();
    ST_BEGIN("mcp_control_redo");
    if (!parsed || !response.value(QStringLiteral("ok")).toBool()
        || !redoResult.value(QStringLiteral("success")).toBool()
        || !redoResult.value(QStringLiteral("can_undo")).toBool()) {
        return pictura::selfTest().fail(471, "edit redo did not succeed");
    }
    ST_PASS("redo success=1 depth=%d", redoResult.value("depth").toInt());

    response = request(socket, 7, QStringLiteral("dispatch_command"),
                       QJsonObject{{QStringLiteral("id"), QStringLiteral("file.open")}}, &parsed);
    ST_BEGIN("mcp_control_refused");
    if (!parsed || errorCode(response) != QStringLiteral("refused")) {
        return pictura::selfTest().fail(472, "modal command not refused");
    }
    ST_PASS("code=%s", qPrintable(errorCode(response)));

    response = request(socket, 8, QStringLiteral("document"),
                       QJsonObject{{QStringLiteral("op"), QStringLiteral("save")}}, &parsed);
    ST_BEGIN("mcp_control_save_no_path");
    if (!parsed || errorCode(response) != QStringLiteral("invalid_param")) {
        return pictura::selfTest().fail(473, "untitled save did not yield invalid_param");
    }
    ST_PASS("code=%s", qPrintable(errorCode(response)));

    QImage raster(4, 4, QImage::Format_ARGB32);
    raster.fill(Qt::red);
    const QString rasterPath = dir.filePath(QStringLiteral("raster.png"));
    const bool rasterWritten = raster.save(rasterPath, "PNG");
    response = request(socket, 9, QStringLiteral("document"),
                       QJsonObject{{QStringLiteral("op"), QStringLiteral("open")},
                                   {QStringLiteral("path"), rasterPath}},
                       &parsed);
    ST_BEGIN("mcp_control_open_raster");
    if (!rasterWritten || !parsed || !response.value(QStringLiteral("ok")).toBool()) {
        return pictura::selfTest().fail(474, "raster document open failed");
    }
    ST_PASS("count=%d", response.value("result").toObject().value("count").toInt());

    response = request(socket, 10, QStringLiteral("status"), QJsonObject(), &parsed);
    const QJsonObject statusAfter = response.value(QStringLiteral("result")).toObject();
    const QJsonArray infos = statusAfter.value(QStringLiteral("documents_info")).toArray();
    const int activeIndex = statusAfter.value(QStringLiteral("active")).toInt();
    const QJsonObject activeInfo = (activeIndex >= 0 && activeIndex < infos.size())
                                       ? infos.at(activeIndex).toObject()
                                       : QJsonObject();
    ST_BEGIN("mcp_control_status_fields");
    if (!parsed || !activeInfo.contains(QStringLiteral("mode"))
        || !activeInfo.contains(QStringLiteral("depth"))
        || !activeInfo.contains(QStringLiteral("layers"))
        || !activeInfo.contains(QStringLiteral("selection_px"))
        || !activeInfo.contains(QStringLiteral("zoom"))) {
        return pictura::selfTest().fail(475, "status per-document fields missing");
    }
    ST_PASS("mode=%s depth=%d layers=%d", qPrintable(activeInfo.value("mode").toString()),
            activeInfo.value("depth").toInt(), activeInfo.value("layers").toInt());

    response = request(socket, 11, QStringLiteral("list_layers"), QJsonObject(), &parsed);
    const QJsonArray layers =
        response.value(QStringLiteral("result")).toObject().value(QStringLiteral("layers")).toArray();
    const QJsonObject layer0 = layers.isEmpty() ? QJsonObject() : layers.at(0).toObject();
    ST_BEGIN("mcp_control_layers");
    if (!parsed || !response.value(QStringLiteral("ok")).toBool() || layers.isEmpty()
        || !layer0.contains(QStringLiteral("index")) || !layer0.contains(QStringLiteral("name"))
        || !layer0.contains(QStringLiteral("kind")) || !layer0.contains(QStringLiteral("visible"))
        || !layer0.contains(QStringLiteral("opacity")) || !layer0.contains(QStringLiteral("blend"))
        || !layer0.contains(QStringLiteral("fill")) || !layer0.contains(QStringLiteral("lock"))
        || !layer0.contains(QStringLiteral("color"))) {
        return pictura::selfTest().fail(476, "list_layers fields missing");
    }
    ST_PASS("layers=%d name=%s", static_cast<int>(layers.size()),
            qPrintable(layer0.value("name").toString()));

    response = request(socket, 12, QStringLiteral("get_pixel"),
                       QJsonObject{{QStringLiteral("x"), 0}, {QStringLiteral("y"), 0}}, &parsed);
    const QJsonObject pixel = response.value(QStringLiteral("result")).toObject();
    ST_BEGIN("mcp_control_pixel");
    if (!parsed || !response.value(QStringLiteral("ok")).toBool() || !pixel.contains(QStringLiteral("argb"))
        || !pixel.contains(QStringLiteral("r")) || !pixel.contains(QStringLiteral("g"))
        || !pixel.contains(QStringLiteral("b")) || !pixel.contains(QStringLiteral("a"))
        || pixel.value(QStringLiteral("a")).toInt() != 255) {
        return pictura::selfTest().fail(477, "get_pixel response malformed");
    }
    ST_PASS("argb=%s a=%d", qPrintable(pixel.value("argb").toString()),
            pixel.value("a").toInt());

    response = request(socket, 13, QStringLiteral("get_pixel"),
                       QJsonObject{{QStringLiteral("x"), 100000}, {QStringLiteral("y"), 100000}},
                       &parsed);
    ST_BEGIN("mcp_control_pixel_oob");
    if (!parsed || errorCode(response) != QStringLiteral("invalid_param")) {
        return pictura::selfTest().fail(478, "out-of-range get_pixel did not yield invalid_param");
    }
    ST_PASS("code=%s", qPrintable(errorCode(response)));

    response = responseObject(transact(socket, QByteArrayLiteral("this is not json")), &parsed);
    ST_BEGIN("mcp_control_bad_request");
    if (!parsed || response.value(QStringLiteral("ok")).toBool()
        || errorCode(response) != QStringLiteral("bad_request")) {
        return pictura::selfTest().fail(479, "malformed line did not yield bad_request");
    }
    ST_PASS("code=%s", qPrintable(errorCode(response)));

    response = request(socket, 7, QStringLiteral("no_such_method"), QJsonObject(), &parsed);
    ST_BEGIN("mcp_control_unknown_method");
    if (!parsed || response.value(QStringLiteral("id")).toInt() != 7
        || response.value(QStringLiteral("ok")).toBool()
        || errorCode(response) != QStringLiteral("unknown_method")) {
        return pictura::selfTest().fail(480, "unknown method did not yield unknown_method");
    }
    ST_PASS("code=%s", qPrintable(errorCode(response)));

    // Unterminated request larger than the buffer ceiling: rejected, not buffered.
    const QByteArray oversized(pictura::ControlServer::kMaxRequestBytes + 4096, 'x');
    const QByteArray oversizedLine = transact(socket, oversized, false, 5000);
    response = responseObject(oversizedLine, &parsed);
    ST_BEGIN("mcp_control_oversized");
    if (!parsed || errorCode(response) != QStringLiteral("bad_request")) {
        return pictura::selfTest().fail(481, "oversized request not rejected");
    }
    ST_PASS("code=%s", qPrintable(errorCode(response)));

    while (frame.documentCount() > initialDocuments) {
        frame.closeDocument(frame.documentCount() - 1, false);
    }
    return 0;
}
