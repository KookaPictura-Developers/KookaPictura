#include "control_server.h"

#include <QtCore/QDir>
#include <QtCore/QJsonArray>
#include <QtCore/QJsonDocument>
#include <QtCore/QJsonParseError>
#include <QtCore/QProcessEnvironment>
#include <QtCore/QSet>
#include <QtGui/QAction>
#include <QtNetwork/QLocalServer>
#include <QtNetwork/QLocalSocket>

#include <cmath>
#include <sys/stat.h>
#include <unistd.h>

#include "commands.h"
#include "dialogs.h"
#include "frame.h"
#include "image_view.h"
#include "panels/panel_column.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

namespace pictura {

namespace {

// Commands whose real menu handler can open a modal dialog (`QFileDialog`,
// `QInputDialog`, `QMessageBox`, the New Document / Preferences / File Info
// dialogs) or reach the unsaved prompt (`close`/`revert`/`exit` call
// `askUnsaved`). `dispatch_command` refuses these because dispatch is
// synchronous on the GUI thread, so a modal would hang the socket forever.
//
// ponytail: hand-maintained denylist — a new modal-opening command must be
// added here or `dispatch_command` will hang the GUI thread on it. When
// uncertain, add it.
bool isModalCommand(const QString& id)
{
    static const QSet<QString> kModal = {
        QString::fromLatin1(command_ids::FileNew),
        QString::fromLatin1(command_ids::FileOpen),
        QString::fromLatin1(command_ids::FileOpenAsSmartObject),
        QString::fromLatin1(command_ids::FilePlace),
        QString::fromLatin1(command_ids::FileInfo),
        QString::fromLatin1(command_ids::FileSave),
        QString::fromLatin1(command_ids::FileSaveAs),
        QString::fromLatin1(command_ids::FileRevert),
        QString::fromLatin1(command_ids::FileClose),
        QString::fromLatin1(command_ids::FileCloseAll),
        QString::fromLatin1(command_ids::FileExit),
        QString::fromLatin1(command_ids::EditAssignProfile),
        QString::fromLatin1(command_ids::EditConvertProfile),
        QString::fromLatin1(command_ids::EditColorSettings),
        QString::fromLatin1(command_ids::EditPreferencesGeneral),
        QString::fromLatin1(command_ids::EditPreferencesInterface),
        QString::fromLatin1(command_ids::LayerNewLayer),
        QString::fromLatin1(command_ids::LayerNewGroup),
        QString::fromLatin1(command_ids::LayerNewGroupFromLayers),
        QString::fromLatin1(command_ids::LayerSmartObjectReplaceContents),
        QString::fromLatin1(command_ids::LayerSmartObjectExportContents),
        QString::fromLatin1(command_ids::SelectModifyBorder),
        QString::fromLatin1(command_ids::SelectModifySmooth),
        QString::fromLatin1(command_ids::SelectModifyExpand),
        QString::fromLatin1(command_ids::SelectModifyContract),
        QString::fromLatin1(command_ids::SelectModifyFeather),
        QString::fromLatin1(command_ids::SelectSave),
        QString::fromLatin1(command_ids::SelectLoad),
        QString::fromLatin1(command_ids::HelpAbout),
    };
    return kModal.contains(id);
}

QString screenModeName(PicturaMainWindow::ScreenMode mode)
{
    switch (mode) {
    case PicturaMainWindow::ScreenMode::FullWithMenuBar:
        return QStringLiteral("fullWithMenuBar");
    case PicturaMainWindow::ScreenMode::Full:
        return QStringLiteral("full");
    case PicturaMainWindow::ScreenMode::Standard:
    default:
        return QStringLiteral("standard");
    }
}

QJsonObject documentInfo(PicturaMainWindow* frame, int index)
{
    PictureView* view = frame->viewAt(index);
    ImageView* canvas = frame->canvasAt(index);
    QJsonObject info;
    info.insert(QStringLiteral("index"), index);
    info.insert(QStringLiteral("name"), frame->documentName(index));
    info.insert(QStringLiteral("path"), frame->documentPath(index));
    info.insert(QStringLiteral("width"), view ? view->document_width() : 0);
    info.insert(QStringLiteral("height"), view ? view->document_height() : 0);
    info.insert(QStringLiteral("mode"), view ? view->document_mode() : QString());
    info.insert(QStringLiteral("depth"), view ? view->document_depth_bits() : 0);
    info.insert(QStringLiteral("dirty"), frame->isDocumentDirty(index));
    info.insert(QStringLiteral("layers"), view ? view->layer_count() : 0);
    info.insert(QStringLiteral("selection_px"), view ? view->selection_count() : 0);
    info.insert(QStringLiteral("zoom"), canvas ? canvas->zoom() : 0.0);
    return info;
}

} // namespace

ControlServer::ControlServer(PicturaMainWindow* frame, const QString& socketPath,
                             QObject* parent)
    : QObject(parent)
    , frame_(frame)
    , socketPath_(socketPath.isEmpty() ? defaultSocketPath() : socketPath)
{
}

ControlServer::~ControlServer()
{
    shutdown();
}

QString ControlServer::defaultSocketPath()
{
    const QString runtime = qEnvironmentVariable("XDG_RUNTIME_DIR");
    if (!runtime.isEmpty()) {
        return runtime + QStringLiteral("/pictura-control.sock");
    }
    return QDir::tempPath() + QStringLiteral("/pictura-control-%1.sock").arg(::getuid());
}

bool ControlServer::listen()
{
    // Never clobber a non-socket path: only remove a stale file that stat
    // confirms is a Unix-domain socket.
    struct stat existing{};
    if (::stat(socketPath_.toLocal8Bit().constData(), &existing) == 0
        && !S_ISSOCK(existing.st_mode)) {
        return false;
    }
    QLocalServer::removeServer(socketPath_);
    server_ = new QLocalServer(this);
    server_->setSocketOptions(QLocalServer::UserAccessOption);
    if (!server_->listen(socketPath_)) {
        delete server_;
        server_ = nullptr;
        return false;
    }
    connect(server_, &QLocalServer::newConnection, this, &ControlServer::onNewConnection);
    return true;
}

void ControlServer::shutdown()
{
    if (server_) {
        server_->close();
        delete server_;
        server_ = nullptr;
        // Only unlink the socket this server created. `QLocalServer::removeServer`
        // blindly QFile::removes the path, so calling it after a failed listen()
        // (e.g. `--control-socket` pointed at a regular file) would delete the
        // user's file. Re-stat and require a socket; a repeated shutdown is a
        // no-op because `server_` is already null.
        struct stat current{};
        if (::stat(socketPath_.toLocal8Bit().constData(), &current) == 0
            && S_ISSOCK(current.st_mode)) {
            QLocalServer::removeServer(socketPath_);
        }
    }
}

void ControlServer::onNewConnection()
{
    if (!server_) {
        return;
    }
    while (QLocalSocket* socket = server_->nextPendingConnection()) {
        buffers_.insert(socket, QByteArray());
        connect(socket, &QLocalSocket::readyRead, this, &ControlServer::onReadyRead);
        connect(socket, &QLocalSocket::disconnected, this, &ControlServer::onDisconnected);
    }
}

void ControlServer::onReadyRead()
{
    QLocalSocket* socket = qobject_cast<QLocalSocket*>(sender());
    if (!socket || !buffers_.contains(socket)) {
        return;
    }
    buffers_[socket].append(socket->readAll());
    // ponytail: 1 MiB per-socket ceiling; reject an unterminated request before
    // it can grow without bound rather than buffering forever.
    {
        const auto it = buffers_.find(socket);
        if (it != buffers_.end() && it->size() > kMaxRequestBytes && it->indexOf('\n') < 0) {
            writeLine(socket, error(QStringLiteral("bad_request"),
                                    QStringLiteral("request exceeds maximum size")));
            buffers_.remove(socket);
            socket->disconnectFromServer();
            return;
        }
    }
    for (;;) {
        const auto it = buffers_.find(socket);
        if (it == buffers_.end()) {
            return;
        }
        const int newline = it->indexOf('\n');
        if (newline < 0) {
            return;
        }
        const QByteArray line = it->left(newline);
        it->remove(0, newline + 1);
        handleLine(socket, line);
    }
}

void ControlServer::onDisconnected()
{
    QLocalSocket* socket = qobject_cast<QLocalSocket*>(sender());
    if (!socket) {
        return;
    }
    buffers_.remove(socket);
    socket->deleteLater();
}

void ControlServer::handleLine(QLocalSocket* socket, const QByteArray& line)
{
    if (line.size() > kMaxRequestBytes) {
        writeLine(socket, error(QStringLiteral("bad_request"),
                                QStringLiteral("request exceeds maximum size")));
        return;
    }
    QJsonParseError parseError{};
    const QJsonDocument document = QJsonDocument::fromJson(line, &parseError);
    if (parseError.error != QJsonParseError::NoError || !document.isObject()) {
        writeLine(socket, error(QStringLiteral("bad_request"),
                                QStringLiteral("request is not a JSON object")));
        return;
    }
    const QJsonObject request = document.object();

    // `id`, when present, must be an integer; echo it on early errors.
    const bool hasId = request.contains(QStringLiteral("id"));
    const QJsonValue idValue = request.value(QStringLiteral("id"));
    bool idValid = true;
    int id = 0;
    if (hasId) {
        if (idValue.isDouble()) {
            const double asDouble = idValue.toDouble();
            if (asDouble != std::floor(asDouble) || asDouble < -2147483648.0
                || asDouble > 2147483647.0) {
                idValid = false;
            } else {
                id = static_cast<int>(asDouble);
            }
        } else {
            idValid = false;
        }
    }
    const bool methodValid = request.value(QStringLiteral("method")).isString();
    const bool paramsValid =
        !request.contains(QStringLiteral("params")) || request.value(QStringLiteral("params")).isObject();
    if (!idValid || !methodValid || !paramsValid) {
        QJsonObject response = error(QStringLiteral("bad_request"),
                                     QStringLiteral("id, method, and params have invalid types"));
        if (hasId && idValid) {
            response.insert(QStringLiteral("id"), id);
        }
        writeLine(socket, response);
        return;
    }
    const QString method = request.value(QStringLiteral("method")).toString();
    const QJsonObject params = request.value(QStringLiteral("params")).toObject();
    QJsonObject response = dispatch(method, params);
    response.insert(QStringLiteral("id"), id);
    writeLine(socket, response);
}

void ControlServer::writeLine(QLocalSocket* socket, const QJsonObject& response)
{
    socket->write(QJsonDocument(response).toJson(QJsonDocument::Compact));
    socket->write("\n");
    socket->flush();
}

QJsonObject ControlServer::ok(const QJsonObject& result) const
{
    return QJsonObject{{QStringLiteral("ok"), true}, {QStringLiteral("result"), result}};
}

QJsonObject ControlServer::error(const QString& code, const QString& message) const
{
    return QJsonObject{
        {QStringLiteral("ok"), false},
        {QStringLiteral("error"),
         QJsonObject{{QStringLiteral("code"), code}, {QStringLiteral("message"), message}}}};
}

QJsonObject ControlServer::dispatch(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("status")) {
        return methodStatus(params);
    }
    if (method == QStringLiteral("get_pixel")) {
        return methodGetPixel(params);
    }
    if (method == QStringLiteral("list_layers")) {
        return methodListLayers(params);
    }
    if (method == QStringLiteral("list_commands")) {
        return methodListCommands(params);
    }
    if (method == QStringLiteral("dispatch_command")) {
        return methodDispatchCommand(params);
    }
    if (method == QStringLiteral("document")) {
        return methodDocument(params);
    }
    if (method == QStringLiteral("edit")) {
        return methodEdit(params);
    }
    if (method == QStringLiteral("set_unsaved_policy")) {
        return methodSetUnsavedPolicy(params);
    }
    return error(QStringLiteral("unknown_method"),
                 QStringLiteral("unknown method: %1").arg(method));
}

QJsonObject ControlServer::methodStatus(const QJsonObject&)
{
    QJsonObject result;
    const int count = frame_->documentCount();
    result.insert(QStringLiteral("documents"), count);
    result.insert(QStringLiteral("active"), frame_->activeDocumentIndex());
    QJsonArray documents;
    for (int i = 0; i < count; ++i) {
        documents.append(documentInfo(frame_, i));
    }
    result.insert(QStringLiteral("documents_info"), documents);
    result.insert(QStringLiteral("active_tool"), toolIdName(frame_->activeTool()));
    PictureView* active = frame_->activeView();
    result.insert(QStringLiteral("backend"),
                  active ? active->active_backend() : QString());
    result.insert(QStringLiteral("gpu_available"), active ? active->gpu_available() : false);
    result.insert(QStringLiteral("brightness"), frame_->brightnessLevel());
    result.insert(QStringLiteral("screen_mode"), screenModeName(frame_->screenMode()));
    QJsonArray panels;
    for (const QString& name : frame_->panelObjectNames()) {
        PanelColumn* column = frame_->columnForPanel(name);
        panels.append(QJsonObject{
            {QStringLiteral("name"), name},
            {QStringLiteral("visible"), column && column->isPanelVisible(name)}});
    }
    result.insert(QStringLiteral("panels"), panels);
    return ok(result);
}

QJsonObject ControlServer::methodGetPixel(const QJsonObject& params)
{
    PictureView* view = frame_->activeView();
    if (!view || !view->has_document()) {
        return error(QStringLiteral("no_document"), QStringLiteral("no active document"));
    }
    if (!params.contains(QStringLiteral("x")) || !params.contains(QStringLiteral("y"))) {
        return error(QStringLiteral("invalid_param"),
                     QStringLiteral("x and y are required"));
    }
    const int x = params.value(QStringLiteral("x")).toInt(-1);
    const int y = params.value(QStringLiteral("y")).toInt(-1);
    if (x < 0 || y < 0 || x >= view->document_width() || y >= view->document_height()) {
        return error(QStringLiteral("invalid_param"),
                     QStringLiteral("coordinate out of range"));
    }
    const quint32 argb = view->sample_argb(x, y);
    QJsonObject result;
    result.insert(QStringLiteral("x"), x);
    result.insert(QStringLiteral("y"), y);
    result.insert(QStringLiteral("argb"),
                  QStringLiteral("0x%1").arg(argb, 8, 16, QLatin1Char('0')));
    result.insert(QStringLiteral("a"), static_cast<int>((argb >> 24) & 0xff));
    result.insert(QStringLiteral("r"), static_cast<int>((argb >> 16) & 0xff));
    result.insert(QStringLiteral("g"), static_cast<int>((argb >> 8) & 0xff));
    result.insert(QStringLiteral("b"), static_cast<int>(argb & 0xff));
    return ok(result);
}

QJsonObject ControlServer::methodListLayers(const QJsonObject&)
{
    PictureView* view = frame_->activeView();
    if (!view || !view->has_document()) {
        return error(QStringLiteral("no_document"), QStringLiteral("no active document"));
    }
    const int count = view->layer_count();
    QJsonArray layers;
    for (int i = 0; i < count; ++i) {
        layers.append(QJsonObject{
            {QStringLiteral("index"), i},
            {QStringLiteral("name"), view->layer_name(i)},
            {QStringLiteral("kind"), view->layer_kind(i)},
            {QStringLiteral("visible"), view->layer_visible(i)},
            {QStringLiteral("opacity"), view->layer_opacity(i)},
            {QStringLiteral("blend"), view->layer_blend(i)},
            {QStringLiteral("fill"), view->layer_fill(i)},
            {QStringLiteral("lock"), view->layer_lock(i)},
            {QStringLiteral("color"), view->layer_color(i)}});
    }
    QJsonObject result;
    result.insert(QStringLiteral("layers"), layers);
    result.insert(QStringLiteral("count"), count);
    return ok(result);
}

QJsonObject ControlServer::methodListCommands(const QJsonObject& params)
{
    const bool implementedOnly = params.value(QStringLiteral("implemented_only")).toBool(false);
    QJsonArray commands;
    for (const CommandInfo& info : frame_->registry()->describe()) {
        if (info.id.isEmpty() || (implementedOnly && !info.implemented)) {
            continue;
        }
        commands.append(QJsonObject{
            {QStringLiteral("id"), info.id},
            {QStringLiteral("path"), QJsonArray::fromStringList(info.path)},
            {QStringLiteral("label"), info.label},
            {QStringLiteral("implemented"), info.implemented},
            {QStringLiteral("enabled"), info.enabled},
            {QStringLiteral("checked"), info.checked}});
    }
    QJsonObject result;
    result.insert(QStringLiteral("commands"), commands);
    result.insert(QStringLiteral("count"), commands.size());
    return ok(result);
}

QJsonObject ControlServer::methodDispatchCommand(const QJsonObject& params)
{
    if (!params.value(QStringLiteral("id")).isString()) {
        return error(QStringLiteral("invalid_param"), QStringLiteral("id must be a string"));
    }
    const QString id = params.value(QStringLiteral("id")).toString();
    CommandRegistry* registry = frame_->registry();

    const QList<CommandInfo> infos = registry->describe();
    const CommandInfo* info = nullptr;
    for (const CommandInfo& candidate : infos) {
        if (candidate.id == id) {
            info = &candidate;
            break;
        }
    }
    if (!info) {
        return error(QStringLiteral("invalid_param"),
                     QStringLiteral("unknown command id: %1").arg(id));
    }
    if (isModalCommand(id)) {
        return error(QStringLiteral("refused"),
                     QStringLiteral("command opens a modal dialog: %1").arg(id));
    }
    if (!info->implemented) {
        return error(QStringLiteral("refused"),
                     QStringLiteral("command is not implemented: %1").arg(id));
    }
    if (!info->enabled) {
        return error(QStringLiteral("refused"),
                     QStringLiteral("command is not enabled: %1").arg(id));
    }
    QAction* action = registry->action(id);
    const bool dispatched = registry->dispatch(id);
    registry->refresh();
    QJsonObject result;
    result.insert(QStringLiteral("dispatched"), dispatched);
    result.insert(QStringLiteral("enabled"), action ? action->isEnabled() : false);
    return ok(result);
}

QJsonObject ControlServer::methodDocument(const QJsonObject& params)
{
    const QString op = params.value(QStringLiteral("op")).toString();
    if (op.isEmpty()) {
        return error(QStringLiteral("invalid_param"), QStringLiteral("op is required"));
    }

    if (op == QStringLiteral("list")) {
        QJsonArray documents;
        for (int i = 0; i < frame_->documentCount(); ++i) {
            documents.append(documentInfo(frame_, i));
        }
        QJsonObject result;
        result.insert(QStringLiteral("documents"), documents);
        result.insert(QStringLiteral("active"), frame_->activeDocumentIndex());
        result.insert(QStringLiteral("count"), frame_->documentCount());
        return ok(result);
    }

    bool applied = false;
    if (op == QStringLiteral("open")) {
        const QString path = params.value(QStringLiteral("path")).toString();
        if (path.isEmpty()) {
            return error(QStringLiteral("invalid_param"), QStringLiteral("path is required"));
        }
        applied = PicturaMainWindow::isNativeDocumentPath(path) ? frame_->openPath(path)
                                                               : frame_->openImagePath(path);
        if (!applied) {
            return error(QStringLiteral("io_error"),
                         QStringLiteral("could not open %1").arg(path));
        }
    } else if (op == QStringLiteral("new")) {
        const int width = params.value(QStringLiteral("width")).toInt(0);
        const int height = params.value(QStringLiteral("height")).toInt(0);
        if (width <= 0 || height <= 0) {
            return error(QStringLiteral("invalid_param"),
                         QStringLiteral("width and height must be positive"));
        }
        const QString name =
            params.value(QStringLiteral("name")).toString(QStringLiteral("Untitled"));
        const QString mode =
            params.value(QStringLiteral("mode")).toString(QStringLiteral("rgb"));
        const int depth = params.value(QStringLiteral("depth")).toInt(8);
        const QString background =
            params.value(QStringLiteral("background")).toString(QStringLiteral("white"));
        applied = frame_->newDocument(name, width, height, mode, depth, background);
        if (!applied) {
            return error(QStringLiteral("internal"), QStringLiteral("could not create document"));
        }
    } else if (op == QStringLiteral("save") || op == QStringLiteral("save_as")) {
        if (!frame_->activeView()) {
            return error(QStringLiteral("no_document"), QStringLiteral("no active document"));
        }
        const QString path = params.value(QStringLiteral("path")).toString();
        if (op == QStringLiteral("save_as")) {
            if (path.isEmpty()) {
                return error(QStringLiteral("invalid_param"), QStringLiteral("path is required"));
            }
            applied = frame_->saveActiveAs(path);
        } else if (path.isEmpty()) {
            // `saveActive()` opens Save-As for an untitled document; refuse
            // instead of reaching a modal file dialog.
            if (frame_->activeFilePath().isEmpty()) {
                return error(QStringLiteral("invalid_param"),
                             QStringLiteral("save requires a path for an untitled document"));
            }
            applied = frame_->saveActive();
        } else {
            applied = frame_->saveActiveAs(path);
        }
        if (!applied) {
            return error(QStringLiteral("io_error"), QStringLiteral("save failed"));
        }
    } else if (op == QStringLiteral("revert")) {
        if (!frame_->activeView()) {
            return error(QStringLiteral("no_document"), QStringLiteral("no active document"));
        }
        applied = frame_->revertActive();
        if (!applied) {
            return error(QStringLiteral("io_error"), QStringLiteral("revert failed"));
        }
    } else if (op == QStringLiteral("close")) {
        const int index = params.contains(QStringLiteral("index"))
                              ? params.value(QStringLiteral("index")).toInt(-1)
                              : frame_->activeDocumentIndex();
        if (index < 0 || index >= frame_->documentCount()) {
            return error(QStringLiteral("invalid_param"), QStringLiteral("invalid document index"));
        }
        applied = frame_->closeDocument(index, false);
        if (!applied) {
            return error(QStringLiteral("internal"), QStringLiteral("close failed"));
        }
    } else if (op == QStringLiteral("activate")) {
        const int index = params.value(QStringLiteral("index")).toInt(-1);
        if (index < 0 || index >= frame_->documentCount()) {
            return error(QStringLiteral("invalid_param"), QStringLiteral("invalid document index"));
        }
        frame_->setActiveDocumentIndex(index);
        applied = frame_->activeDocumentIndex() == index;
    } else {
        return error(QStringLiteral("invalid_param"),
                     QStringLiteral("unknown op: %1").arg(op));
    }

    QJsonObject result;
    result.insert(QStringLiteral("applied"), applied);
    result.insert(QStringLiteral("active"), frame_->activeDocumentIndex());
    result.insert(QStringLiteral("count"), frame_->documentCount());
    result.insert(QStringLiteral("path"), frame_->activeFilePath());
    result.insert(QStringLiteral("name"), frame_->activeDocumentName());
    return ok(result);
}

QJsonObject ControlServer::methodEdit(const QJsonObject& params)
{
    PictureView* view = frame_->activeView();
    if (!view || !view->has_document()) {
        return error(QStringLiteral("no_document"), QStringLiteral("no active document"));
    }
    const QString op = params.value(QStringLiteral("op")).toString();
    bool applied = false;
    if (op == QStringLiteral("undo")) {
        applied = view->undo();
    } else if (op == QStringLiteral("redo")) {
        applied = view->redo();
    } else {
        return error(QStringLiteral("invalid_param"),
                     QStringLiteral("op must be undo or redo"));
    }
    QJsonObject result;
    result.insert(QStringLiteral("success"), applied);
    result.insert(QStringLiteral("can_undo"), view->can_undo());
    result.insert(QStringLiteral("can_redo"), view->can_redo());
    result.insert(QStringLiteral("depth"), view->history_depth());
    return ok(result);
}

QJsonObject ControlServer::methodSetUnsavedPolicy(const QJsonObject& params)
{
    bool interactive = params.contains(QStringLiteral("interactive"))
                           ? params.value(QStringLiteral("interactive")).toBool(unsavedInteractive_)
                           : unsavedInteractive_;
    QString choice = params.contains(QStringLiteral("choice"))
                         ? params.value(QStringLiteral("choice")).toString(unsavedChoice_)
                         : unsavedChoice_;
    if (choice != QStringLiteral("cancel") && choice != QStringLiteral("discard")) {
        return error(QStringLiteral("invalid_param"),
                     QStringLiteral("choice must be cancel or discard"));
    }
    setUnsavedPromptInteractive(interactive);
    if (params.contains(QStringLiteral("choice"))) {
        setNonInteractiveUnsavedChoice(choice == QStringLiteral("cancel")
                                           ? UnsavedChoice::Cancel
                                           : UnsavedChoice::Discard);
    }
    unsavedInteractive_ = interactive;
    unsavedChoice_ = choice;
    QJsonObject result;
    result.insert(QStringLiteral("interactive"), interactive);
    result.insert(QStringLiteral("choice"), choice);
    return ok(result);
}

} // namespace pictura
