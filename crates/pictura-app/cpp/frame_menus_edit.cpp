#include "frame_includes.h"

#include "pictura_app/src/cxxqt_object/clipboard.cxxqt.h"

namespace pictura {

// Edit clipboard. Copy/Cut/Clear act on the Layers panel's current layer; a
// paste lands above it and becomes the current layer. The clipboard is shared by
// every open document; each mutating command records one state in the bridge,
// and a refusal records nothing.
void PicturaMainWindow::registerEditHandlers()
{
    const auto currentPath = [this]() {
        return layersPanel_ ? layersPanel_->currentPath() : QString();
    };
    const auto layerCommand = [this, currentPath](const auto& run) {
        PictureView* view = activeView();
        const QString path = currentPath();
        if (view && !path.isEmpty() && run(*view, path)) {
            refresh();
        }
    };
    registry_->setHandler(command_ids::EditCopy, [layerCommand]() {
        layerCommand([](PictureView& view, const QString& path) {
            return clipboard_copy(view, path, false);
        });
    });
    registry_->setHandler(command_ids::EditCut, [layerCommand]() {
        layerCommand([](PictureView& view, const QString& path) { return clipboard_cut(view, path); });
    });
    registry_->setHandler(command_ids::EditClear, [layerCommand]() {
        layerCommand(
            [](PictureView& view, const QString& path) { return clipboard_clear(view, path); });
    });
    registry_->setHandler(command_ids::EditCopyMerged, [this]() {
        if (PictureView* view = activeView(); view && clipboard_copy(*view, QString(), true)) {
            refresh();
        }
    });

    // A plain paste centres on what the canvas shows; Paste Into centres on the
    // selection in the bridge.
    const auto paste = [this, currentPath](PasteKind kind) {
        PictureView* view = activeView();
        if (!view || !view->has_document()) {
            return;
        }
        QPointF centre(view->document_width() / 2.0, view->document_height() / 2.0);
        if (ImageView* canvas = imageView()) {
            centre = canvas->widgetToImage(QPointF(canvas->width() / 2.0, canvas->height() / 2.0));
        }
        const QString created =
            clipboard_paste(*view, currentPath(), kind, centre.x(), centre.y());
        if (!created.isEmpty()) {
            refresh();
            selectLayerPath(created);
        }
    };
    registry_->setHandler(command_ids::EditPaste, [paste]() { paste(PasteKind::Plain); });
    registry_->setHandler(command_ids::EditPasteInto, [paste]() { paste(PasteKind::Into); });
    registry_->setHandler(command_ids::EditPasteOutside, [paste]() { paste(PasteKind::Outside); });
    registry_->setHandler(command_ids::EditPurgeClipboard, [this]() {
        clipboard_purge();
        refresh();
    });

    const auto hasDocument = [this]() { return activeView() && activeView()->has_document(); };
    for (const char* id : {command_ids::EditCut, command_ids::EditCopy, command_ids::EditClear}) {
        registry_->setEnabledProvider(
            id, [hasDocument, currentPath]() { return hasDocument() && !currentPath().isEmpty(); });
    }
    registry_->setEnabledProvider(command_ids::EditCopyMerged, hasDocument);
    registry_->setEnabledProvider(command_ids::EditPaste,
                                  [hasDocument]() { return hasDocument() && clipboard_has_contents(); });
    for (const char* id : {command_ids::EditPasteInto, command_ids::EditPasteOutside}) {
        registry_->setEnabledProvider(id, [this, hasDocument]() {
            return hasDocument() && clipboard_has_contents() && activeView()->has_selection();
        });
    }
    registry_->setEnabledProvider(command_ids::EditPurgeClipboard,
                                  []() { return clipboard_has_contents(); });
}

} // namespace pictura
