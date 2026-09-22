#include "frame_includes.h"

namespace pictura {

void PicturaMainWindow::buildMenus()
{
    registry_->buildMenuBar(menuBar());

    // Icon for every implemented command; ids without an asset are skipped.
    static const char* const kIconCommands[] = {
        command_ids::FileNew,
        command_ids::FileOpen,
        command_ids::FileSave,
        command_ids::FileSaveAs,
        command_ids::FileRevert,
        command_ids::FileClose,
        command_ids::FileCloseAll,
        command_ids::FileExit,
        command_ids::EditUndo,
        command_ids::EditRedo,
        command_ids::EditStepForward,
        command_ids::EditStepBackward,
        command_ids::ImageRotate90Cw,
        command_ids::ImageRotate90Ccw,
        command_ids::ImageRotate180,
        command_ids::ImageFlipHorizontal,
        command_ids::ImageFlipVertical,
        command_ids::ImageCrop,
        command_ids::SelectAll,
        command_ids::SelectDeselect,
        command_ids::ViewZoomIn,
        command_ids::ViewZoomOut,
        command_ids::ViewFitOnScreen,
        command_ids::ViewActualPixels,
        command_ids::ViewScreenModeStandard,
        command_ids::ViewScreenModeFullWithMenuBar,
        command_ids::ViewScreenModeFull,
        command_ids::ViewOptions,
        command_ids::WindowPanelsLayers,
        command_ids::WindowPanelsTools,
        command_ids::HelpAbout,
    };
    for (const char* id : kIconCommands) {
        const QString commandId = QString::fromLatin1(id);
        if (QAction* action = registry_->action(commandId)) {
            action->setIcon(icon(commandId));
        }
    }
}

void PicturaMainWindow::registerHandlers()
{
    registry_->setHandler(command_ids::FileNew, [this]() { showNewDocumentDialog(); });
    registry_->setHandler(command_ids::FileOpen, [this]() { showOpenDialog(); });
    registry_->setHandler(command_ids::FileOpenAsSmartObject, [this]() {
        const QString path = QFileDialog::getOpenFileName(
            this, tr("Open As Smart Object"), QString(),
            QStringLiteral("Photoshop files (*.psd *.psb)"));
        if (!path.isEmpty()) {
            openAsSmartObjectPath(path);
        }
    });
    registry_->setHandler(command_ids::FilePlace, [this]() {
        PictureView* view = activeView();
        if (!view) {
            return;
        }
        const QString filter = QStringLiteral(
            "Images (*.png *.jpg *.jpeg *.gif *.bmp *.tif *.tiff *.webp);;"
            "Photoshop files (*.psd *.psb);;All files (*)");
        const QString path = QFileDialog::getOpenFileName(this, tr("Place"), QString(), filter);
        if (path.isEmpty()) {
            return;
        }
        const bool psd = PicturaMainWindow::isNativeDocumentPath(path);
        const QString created =
            psd ? view->place_smart_object(path) : view->place_image(path);
        if (!created.isEmpty()) {
            beginFreeTransform(created);
            selectLayerPath(created);
            refresh();
        }
    });

    registry_->setHandler(command_ids::FileInfo, [this]() { showFileInfo(); });

    registry_->setHandler(command_ids::FileSave, [this]() { saveActive(); });
    registry_->setHandler(command_ids::FileSaveAs, [this]() {
        QString path = QFileDialog::getSaveFileName(this, tr("Save As"), activeFilePath(),
                                                    QStringLiteral("Photoshop files (*.psd *.psb)"));
        if (path.isEmpty()) {
            return;
        }
        if (QFileInfo(path).suffix().isEmpty()) {
            path += QStringLiteral(".psd");
        }
        saveActiveAs(path);
    });
    registry_->setHandler(command_ids::FileRevert, [this]() {
        const int index = activeDocumentIndex();
        if (index < 0 || documentPath(index).isEmpty()) {
            return;
        }
        if (isDocumentDirty(index)) {
            switch (askUnsaved(this, documentName(index))) {
            case UnsavedChoice::Cancel:
                return;
            case UnsavedChoice::Save:
                setActiveDocumentIndex(index);
                if (!saveActive()) {
                    return;
                }
                break;
            case UnsavedChoice::Discard:
                break;
            }
        }
        revertActive();
    });
    registry_->setHandler(command_ids::FileClose, [this]() { closeActiveDocument(true); });
    registry_->setHandler(command_ids::FileCloseAll, [this]() {
        for (int i = docs_.size() - 1; i >= 0; --i) {
            if (!closeDocument(i, true)) {
                break;
            }
        }
    });
    registry_->setHandler(command_ids::FileExit, [this]() {
        for (int i = docs_.size() - 1; i >= 0; --i) {
            if (!closeDocument(i, true)) {
                break;
            }
        }
        if (documentCount() == 0) {
            qApp->quit();
        }
    });

    auto hasDocument = [this]() { return documentCount() > 0; };
    for (const char* id : {command_ids::FileSave, command_ids::FileSaveAs, command_ids::FileClose,
                           command_ids::FileCloseAll, command_ids::FilePlace,
                           command_ids::FileInfo}) {
        registry_->setEnabledProvider(id, hasDocument);
    }
    registry_->setEnabledProvider(command_ids::FileRevert,
                                  [this]() { return !activeFilePath().isEmpty(); });

    registry_->setHandler(command_ids::EditUndo, [this]() {
        PictureView* view = activeView();
        if (!view) {
            return;
        }
        const bool ok = view->can_undo() ? view->undo() : view->redo();
        if (ok) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::EditUndo, [this]() {
        PictureView* view = activeView();
        return view && (view->can_undo() || view->can_redo());
    });
    registry_->setLabelProvider(command_ids::EditUndo, [this]() {
        PictureView* view = activeView();
        return view && !view->can_undo() ? QStringLiteral("Redo") : QStringLiteral("Undo");
    });

    registry_->setHandler(command_ids::EditRedo, [this]() {
        if (PictureView* view = activeView(); view && view->redo()) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::EditRedo,
                                  [this]() { return activeView() && activeView()->can_redo(); });

    registry_->setHandler(command_ids::EditStepBackward, [this]() {
        if (PictureView* view = activeView(); view && view->undo()) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::EditStepBackward,
                                  [this]() { return activeView() && activeView()->can_undo(); });

    registry_->setHandler(command_ids::EditStepForward, [this]() {
        if (PictureView* view = activeView(); view && view->redo()) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::EditStepForward,
                                  [this]() { return activeView() && activeView()->can_redo(); });

    registry_->setHandler(command_ids::EditAssignProfile,
                          [this]() { showProfileCommand(false); });
    registry_->setHandler(command_ids::EditConvertProfile,
                          [this]() { showProfileCommand(true); });
    for (const char* id : {command_ids::EditAssignProfile, command_ids::EditConvertProfile}) {
        registry_->setEnabledProvider(id, [this]() {
            PictureView* view = activeView();
            return view && view->has_document()
                && view->document_mode() == QStringLiteral("rgb");
        });
    }

    registry_->setHandler(command_ids::EditFreeTransform, [this]() {
        PictureView* view = activeView();
        const QString path = (layersPanel_ && layersPanel_->selectedPaths().size() == 1)
                                 ? layersPanel_->currentPath()
                                 : QString();
        if (!view) {
            return;
        }
        if (beginFreeTransform(path)) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::EditFreeTransform, [this]() {
        PictureView* view = activeView();
        const QString path = (layersPanel_ && layersPanel_->selectedPaths().size() == 1)
                                 ? layersPanel_->currentPath()
                                 : QString();
        return view && view->has_document() && !path.isEmpty()
            && view->layer_can_free_transform(path);
    });

    registry_->setHandler(command_ids::ImageRotate90Cw, [this]() {
        if (PictureView* view = activeView(); view && view->rotate_doc(1)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::ImageRotate90Ccw, [this]() {
        if (PictureView* view = activeView(); view && view->rotate_doc(3)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::ImageRotate180, [this]() {
        if (PictureView* view = activeView(); view && view->rotate_doc(2)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::ImageFlipHorizontal, [this]() {
        if (PictureView* view = activeView(); view && view->flip_doc(true)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::ImageFlipVertical, [this]() {
        if (PictureView* view = activeView(); view && view->flip_doc(false)) {
            refresh();
        }
    });
    for (const char* id : {command_ids::ImageRotate90Cw, command_ids::ImageRotate90Ccw,
                           command_ids::ImageRotate180, command_ids::ImageFlipHorizontal,
                           command_ids::ImageFlipVertical}) {
        registry_->setEnabledProvider(id,
                                      [this]() { return activeView() && activeView()->has_document(); });
    }

    registry_->setHandler(command_ids::ImageCrop, [this]() {
        if (tools_ && tools_->hasPendingCrop()) {
            commitCrop();
            return;
        }
        PictureView* view = activeView();
        if (!view) {
            return;
        }
        const QString bounds = view->selection_bounds();
        if (bounds.isEmpty()) {
            return;
        }
        const QStringList parts = bounds.split(QLatin1Char(' '), Qt::SkipEmptyParts);
        if (parts.size() != 4) {
            return;
        }
        if (view->crop(parts.at(0).toInt(), parts.at(1).toInt(), parts.at(2).toInt(),
                       parts.at(3).toInt())) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::ImageCrop,
                                  [this]() { return activeView() && activeView()->has_document(); });

    registry_->setHandler(command_ids::SelectAll, [this]() {
        if (PictureView* view = activeView()) {
            view->select_all();
        }
    });
    registry_->setEnabledProvider(command_ids::SelectAll,
                                  [this]() { return activeView() && activeView()->has_document(); });
    registry_->setHandler(command_ids::SelectDeselect, [this]() {
        if (PictureView* view = activeView()) {
            view->deselect();
        }
    });
    registry_->setEnabledProvider(command_ids::SelectDeselect,
                                  [this]() { return activeView() && activeView()->has_document(); });

    registerSelectHandlers();

    // M37: layer creation and grouping. Layer…/Group… open the modal dialog,
    // whose accept step places the node above the selection; Group from Layers…
    // wraps the panel selection. The remaining commands target the active
    // document's current layer — the Layers panel's selected row, -1 when none.
    // ponytail: one current layer; M38's multi-selection upgrades these to
    // per-selection operations.
    registry_->setHandler(command_ids::LayerNewLayer, [this]() {
        if (layersPanel_) {
            layersPanel_->openNewLayerDialog();
            refresh();
        }
    });
    registry_->setHandler(command_ids::LayerNewGroup, [this]() {
        if (layersPanel_) {
            layersPanel_->openNewGroupDialog();
            refresh();
        }
    });
    registry_->setHandler(command_ids::LayerNewGroupFromLayers, [this]() {
        if (layersPanel_) {
            layersPanel_->openGroupFromLayersDialog();
            refresh();
        }
    });
    registry_->setHandler(command_ids::LayerDuplicateLayer, [this]() {
        if (PictureView* view = activeView()) {
            view->duplicate_layer(layersPanel_ ? layersPanel_->currentLayer() : -1);
            refresh();
        }
    });
    registry_->setHandler(command_ids::LayerGroupLayers, [this]() {
        if (layersPanel_) {
            layersPanel_->groupSelection();
            refresh();
        }
    });
    registry_->setHandler(command_ids::LayerUngroupLayers, [this]() {
        if (layersPanel_) {
            layersPanel_->ungroupSelection();
            refresh();
        }
    });
    for (const char* id : {command_ids::LayerNewLayer, command_ids::LayerNewGroup,
                           command_ids::LayerDuplicateLayer, command_ids::LayerGroupLayers,
                           command_ids::LayerUngroupLayers}) {
        registry_->setEnabledProvider(id,
                                      [this]() { return activeView() && activeView()->has_document(); });
    }
    registry_->setEnabledProvider(command_ids::LayerNewGroupFromLayers, [this]() {
        return activeView() && activeView()->has_document() && layersPanel_
            && !layersPanel_->selectedPaths().isEmpty();
    });

    // Merge and flatten. Merge Down and Merge Layers share one id (design D4):
    // one selected path merges down, several merge the selection.
    registry_->setHandler(command_ids::LayerMergeLayers, [this]() {
        PictureView* view = activeView();
        const QStringList paths = layersPanel_ ? layersPanel_->selectedPaths() : QStringList{};
        if (view && !paths.isEmpty() && view->merge_layers(paths) > 0) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::LayerMergeLayers, [this]() {
        PictureView* view = activeView();
        return view && view->has_document() && layersPanel_
            && view->can_merge_layers(layersPanel_->selectedPaths());
    });

    registry_->setHandler(command_ids::LayerMergeVisible, [this]() {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (view && !path.isEmpty() && view->merge_visible(path) > 0) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::LayerMergeVisible, [this]() {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || !view->has_document() || path.isEmpty()) {
            return false;
        }
        for (int i = 0; i < view->layer_row_count(); ++i) {
            if (view->layer_row_path(i) == path) {
                return view->layer_row_visible(i);
            }
        }
        return false;
    });

    registry_->setHandler(command_ids::LayerMergeClippingMask, [this]() {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (view && !path.isEmpty() && view->merge_clipping_mask(path) > 0) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::LayerMergeClippingMask, [this]() {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        return view && view->has_document() && !path.isEmpty()
            && view->can_merge_clipping_mask(path);
    });

    registry_->setHandler(command_ids::LayerFlattenImage, [this]() {
        PictureView* view = activeView();
        if (view && view->flatten_image() > 0) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::LayerFlattenImage,
                                  [this]() { return activeView() && activeView()->has_document(); });

    // Background conversion. The applicable kind gates each command: "Layer
    // from Background…" needs the Background; "Background From Layer" needs a
    // raster pixel layer (groups and adjustment/fill layers are refused).
    auto currentLayerKind = [this]() -> QString {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || path.isEmpty()) {
            return QString();
        }
        for (int i = 0; i < view->layer_row_count(); ++i) {
            if (view->layer_row_path(i) == path) {
                return view->layer_row_kind(i);
            }
        }
        return QString();
    };
    registry_->setHandler(command_ids::LayerNewLayerFromBackground, [this]() {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (view && !path.isEmpty() && view->layer_from_background(path)) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::LayerNewLayerFromBackground,
                                  [currentLayerKind]() {
                                      return currentLayerKind() == QStringLiteral("background");
                                  });
    registry_->setHandler(command_ids::LayerNewBackgroundFromLayer, [this]() {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (view && !path.isEmpty() && view->background_from_layer(path)) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::LayerNewBackgroundFromLayer, [currentLayerKind]() {
        return currentLayerKind() == QStringLiteral("pixel");
    });

    // Layer via Copy / Cut. Both need an active selection and an active layer
    // (the panel's current row, which the bridge resolves).
    const auto layerVia = [this](bool cut) {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || path.isEmpty()) {
            return;
        }
        const QString created =
            cut ? view->layer_via_cut(path) : view->layer_via_copy(path);
        if (!created.isEmpty()) {
            refresh();
        }
    };
    registry_->setHandler(command_ids::LayerNewLayerViaCopy, [layerVia]() { layerVia(false); });
    registry_->setHandler(command_ids::LayerNewLayerViaCut, [layerVia]() { layerVia(true); });
    for (const char* id : {command_ids::LayerNewLayerViaCopy, command_ids::LayerNewLayerViaCut}) {
        registry_->setEnabledProvider(id, [this]() {
            return activeView() && activeView()->has_document() && activeView()->has_selection()
                && layersPanel_ && !layersPanel_->currentPath().isEmpty();
        });
    }

    // Select Similar/Linked and Link/Unlink. A Select command pushes the matched
    // paths back into the panel so the result becomes the panel selection; Link
    // and Unlink only touch the transient link sets. None records history.
    registry_->setHandler(command_ids::LayerSelectSimilar, [this]() {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || path.isEmpty()) {
            return;
        }
        const QStringList matched = view->select_similar(path);
        if (layersPanel_) {
            layersPanel_->selectPaths(matched, path);
        }
        refresh();
    });
    registry_->setEnabledProvider(command_ids::LayerSelectSimilar, [this]() {
        return activeView() && activeView()->has_document() && layersPanel_
            && !layersPanel_->currentPath().isEmpty();
    });

    registry_->setHandler(command_ids::LayerSelectLinked, [this]() {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || path.isEmpty()) {
            return;
        }
        const QStringList linked = view->select_linked(path);
        if (layersPanel_) {
            layersPanel_->selectPaths(linked, path);
        }
        refresh();
    });
    registry_->setEnabledProvider(command_ids::LayerSelectLinked, [this]() {
        return activeView() && activeView()->has_document() && layersPanel_
            && !layersPanel_->currentPath().isEmpty();
    });

    registry_->setHandler(command_ids::LayerLinkLayers, [this]() {
        PictureView* view = activeView();
        const QStringList paths = layersPanel_ ? layersPanel_->selectedPaths() : QStringList{};
        if (view && !paths.isEmpty()) {
            view->link_layers(paths, true);
        }
    });
    registry_->setHandler(command_ids::LayerUnlinkLayers, [this]() {
        PictureView* view = activeView();
        const QStringList paths = layersPanel_ ? layersPanel_->selectedPaths() : QStringList{};
        if (view && !paths.isEmpty()) {
            view->link_layers(paths, false);
        }
    });
    for (const char* id : {command_ids::LayerLinkLayers, command_ids::LayerUnlinkLayers}) {
        registry_->setEnabledProvider(id, [this]() {
            return activeView() && activeView()->has_document() && layersPanel_
                && !layersPanel_->selectedPaths().isEmpty();
        });
    }

    // Delete Hidden Layers needs only a document (it records nothing when
    // nothing is hidden); Hide Layers needs a selection.
    registry_->setHandler(command_ids::LayerDeleteHiddenLayers, [this]() {
        PictureView* view = activeView();
        if (view && view->delete_hidden_layers() > 0) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::LayerDeleteHiddenLayers,
                                  [this]() { return activeView() && activeView()->has_document(); });

    registry_->setHandler(command_ids::LayerHideLayers, [this]() {
        PictureView* view = activeView();
        const QStringList paths = layersPanel_ ? layersPanel_->selectedPaths() : QStringList{};
        if (view && !paths.isEmpty() && view->hide_layers(paths) > 0) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::LayerHideLayers, [this]() {
        return activeView() && activeView()->has_document() && layersPanel_
            && !layersPanel_->selectedPaths().isEmpty();
    });

    // Solid-color fill creation. The color is the frame's foreground, which is
    // not yet plumbed from the toolbox swatch, so it resolves to opaque black.
    registry_->setHandler(command_ids::LayerNewFillSolidColor, [this]() {
        PictureView* view = activeView();
        if (!view) {
            return;
        }
        const QColor fg = foregroundColor();
        const unsigned int argb = fg.isValid() ? fg.rgba() : 0xff000000u;
        if (!view->add_solid_fill(argb).isEmpty()) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::LayerNewFillSolidColor,
                                  [this]() { return activeView() && activeView()->has_document(); });

    // Gradient fill creation: a fixed black-to-white Linear fill at angle 0.
    registry_->setHandler(command_ids::LayerNewFillGradient, [this]() {
        PictureView* view = activeView();
        if (view && !view->add_gradient_fill().isEmpty()) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::LayerNewFillGradient,
                                  [this]() { return activeView() && activeView()->has_document(); });

    // Rasterize. Fill Content and Layer both need a current fill-content layer;
    // All Layers needs a document with at least one. Type/Shape/Layer Style/
    // Video/3D have no handler and stay disabled (their kinds do not exist).
    const auto currentFillPath = [this]() -> QString {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || path.isEmpty() || !view->layer_is_fill_content(path)) {
            return QString();
        }
        return path;
    };
    registry_->setHandler(command_ids::LayerRasterizeFillContent, [this, currentFillPath]() {
        if (PictureView* view = activeView()) {
            const QString path = currentFillPath();
            if (!path.isEmpty() && view->rasterize_fill_content(path)) {
                refresh();
            }
        }
    });
    registry_->setEnabledProvider(command_ids::LayerRasterizeFillContent,
                                  [currentFillPath]() { return !currentFillPath().isEmpty(); });
    registry_->setHandler(command_ids::LayerRasterizeLayer, [this, currentFillPath]() {
        if (PictureView* view = activeView()) {
            const QString path = currentFillPath();
            if (!path.isEmpty() && view->rasterize_layer(path)) {
                refresh();
            }
        }
    });
    registry_->setEnabledProvider(command_ids::LayerRasterizeLayer,
                                  [currentFillPath]() { return !currentFillPath().isEmpty(); });
    registry_->setHandler(command_ids::LayerRasterizeAllLayers, [this]() {
        PictureView* view = activeView();
        if (view && view->rasterize_all_layers() > 0) {
            refresh();
        }
    });
    registry_->setEnabledProvider(command_ids::LayerRasterizeAllLayers, [this]() {
        PictureView* view = activeView();
        if (!view || !view->has_document()) {
            return false;
        }
        for (int i = 0; i < view->layer_row_count(); ++i) {
            if (view->layer_is_fill_content(view->layer_row_path(i))) {
                return true;
            }
        }
        return false;
    });

    // Smart Objects. Convert to Smart Object needs the current layer to be a
    // raster pixel layer (not a group, adjustment, Background, or already a
    // smart object).
    const auto currentSmartPath = [this]() -> QString {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || path.isEmpty() || !view->layer_can_convert_to_smart_object(path)) {
            return QString();
        }
        return path;
    };
    registry_->setHandler(command_ids::LayerSmartObjectConvertTo, [this, currentSmartPath]() {
        if (PictureView* view = activeView()) {
            const QString path = currentSmartPath();
            if (!path.isEmpty() && view->convert_to_smart_object(path)) {
                refresh();
            }
        }
    });
    registry_->setEnabledProvider(command_ids::LayerSmartObjectConvertTo,
                                  [currentSmartPath]() { return !currentSmartPath().isEmpty(); });

    const auto currentRasterizableSmartPath = [this]() -> QString {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || path.isEmpty() || !view->layer_can_rasterize_smart_object(path)) {
            return QString();
        }
        return path;
    };
    registry_->setHandler(command_ids::LayerRasterizeSmartObject,
                          [this, currentRasterizableSmartPath]() {
                              if (PictureView* view = activeView()) {
                                  const QString path = currentRasterizableSmartPath();
                                  if (!path.isEmpty() && view->rasterize_smart_object(path)) {
                                      refresh();
                                  }
                              }
                          });
    registry_->setEnabledProvider(command_ids::LayerRasterizeSmartObject,
                                  [currentRasterizableSmartPath]() {
                                      return !currentRasterizableSmartPath().isEmpty();
                                  });

    // Replace Contents needs the current layer to be a replaceable smart object.
    const auto currentReplaceableSmartPath = [this]() -> QString {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || path.isEmpty() || !view->layer_can_replace_smart_object_contents(path)) {
            return QString();
        }
        return path;
    };
    registry_->setHandler(command_ids::LayerSmartObjectReplaceContents,
                          [this, currentReplaceableSmartPath]() {
                              PictureView* view = activeView();
                              const QString path = currentReplaceableSmartPath();
                              if (!view || path.isEmpty()) {
                                  return;
                              }
                              const QString file = QFileDialog::getOpenFileName(
                                  this, tr("Replace Contents"), QString(),
                                  QStringLiteral("Photoshop files (*.psd *.psb)"));
                              if (!file.isEmpty()
                                  && view->replace_smart_object_contents(path, file)) {
                                  refresh();
                              }
                          });
    registry_->setEnabledProvider(command_ids::LayerSmartObjectReplaceContents,
                                  [currentReplaceableSmartPath]() {
                                      return !currentReplaceableSmartPath().isEmpty();
                                  });

    // Export Contents needs the current layer to carry a non-empty embedded
    // payload. Export only reads the document, so nothing is refreshed.
    const auto currentExportableSmartPath = [this]() -> QString {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || path.isEmpty() || !view->layer_can_export_smart_object_contents(path)) {
            return QString();
        }
        return path;
    };
    registry_->setHandler(command_ids::LayerSmartObjectExportContents,
                          [this, currentExportableSmartPath]() {
                              PictureView* view = activeView();
                              const QString path = currentExportableSmartPath();
                              if (!view || path.isEmpty()) {
                                  return;
                              }
                              QString dest = QFileDialog::getSaveFileName(
                                  this, tr("Export Contents"), activeFilePath(),
                                  QStringLiteral("Photoshop files (*.psd)"));
                              if (dest.isEmpty()) {
                                  return;
                              }
                              if (QFileInfo(dest).suffix().isEmpty()) {
                                  dest += QStringLiteral(".psd");
                              }
                              if (!view->export_smart_object_contents(path, dest)) {
                                  QMessageBox::warning(
                                      this, tr("Export Contents"),
                                      tr("Could not export the smart object contents."));
                              }
                          });
    registry_->setEnabledProvider(command_ids::LayerSmartObjectExportContents,
                                  [currentExportableSmartPath]() {
                                      return !currentExportableSmartPath().isEmpty();
                                  });

    // Edit Contents needs the current layer to be an embedded PSD/PSB source,
    // so it can open as an ordinary document in an untitled editor tab.
    const auto currentEditableSmartPath = [this]() -> QString {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || path.isEmpty() || !view->layer_can_edit_smart_object_contents(path)) {
            return QString();
        }
        return path;
    };
    registry_->setHandler(command_ids::LayerSmartObjectEditContents,
                          [this, currentEditableSmartPath]() {
                              const QString path = currentEditableSmartPath();
                              if (!path.isEmpty() && editSmartObjectContents(path)) {
                                  refresh();
                              }
                          });
    registry_->setEnabledProvider(command_ids::LayerSmartObjectEditContents,
                                  [currentEditableSmartPath]() {
                                      return !currentEditableSmartPath().isEmpty();
                                  });

    registry_->setHandler(command_ids::ViewZoomIn, [this]() {
        if (ImageView* canvas = imageView()) {
            canvas->zoomIn();
        }
    });
    registry_->setHandler(command_ids::ViewZoomOut, [this]() {
        if (ImageView* canvas = imageView()) {
            canvas->zoomOut();
        }
    });
    registry_->setHandler(command_ids::ViewFitOnScreen, [this]() {
        if (ImageView* canvas = imageView()) {
            canvas->fitOnScreen();
        }
    });
    registry_->setHandler(command_ids::ViewActualPixels, [this]() {
        if (ImageView* canvas = imageView()) {
            canvas->actualPixels();
        }
    });

    registry_->setHandler(command_ids::ViewScreenModeStandard,
                          [this]() { setScreenMode(ScreenMode::Standard); });
    registry_->setHandler(command_ids::ViewScreenModeFullWithMenuBar,
                          [this]() { setScreenMode(ScreenMode::FullWithMenuBar); });
    registry_->setHandler(command_ids::ViewScreenModeFull,
                          [this]() { setScreenMode(ScreenMode::Full); });
    registry_->setCheckedProvider(command_ids::ViewScreenModeStandard,
                                  [this]() { return screenMode_ == ScreenMode::Standard; });
    registry_->setCheckedProvider(command_ids::ViewScreenModeFullWithMenuBar, [this]() {
        return screenMode_ == ScreenMode::FullWithMenuBar;
    });
    registry_->setCheckedProvider(command_ids::ViewScreenModeFull,
                                  [this]() { return screenMode_ == ScreenMode::Full; });

    registry_->setHandler(command_ids::ViewOptions, [this]() {
        QAction* action = registry_->action(command_ids::ViewOptions);
        if (optionsBar_ && action) {
            optionsBar_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::ViewOptions,
                                  [this]() { return optionsBar_ && optionsBar_->isVisible(); });

    registry_->setHandler(command_ids::ViewGpuCompute, [this]() {
        const QAction* action = registry_->action(command_ids::ViewGpuCompute);
        gpuCompute_ = action && action->isChecked();
        for (const DocEntry& entry : docs_) {
            if (entry.view) {
                entry.view->set_gpu_compute(gpuCompute_);
            }
        }
        saveSession();
        registry_->refresh();
        refresh();
    });
    registry_->setEnabledProvider(command_ids::ViewGpuCompute, [this]() { return gpuAvailable_; });
    registry_->setCheckedProvider(command_ids::ViewGpuCompute, [this]() { return gpuCompute_; });

    registry_->setHandler(command_ids::ViewShowSelectionEdges, [this]() {
        const QAction* action = registry_->action(command_ids::ViewShowSelectionEdges);
        if (ImageView* canvas = imageView()) {
            canvas->setSelectionEdgesVisible(action && action->isChecked());
        }
    });
    registry_->setEnabledProvider(command_ids::ViewShowSelectionEdges,
                                  [this]() { return documentCount() > 0; });
    registry_->setCheckedProvider(command_ids::ViewShowSelectionEdges, [this]() {
        ImageView* canvas = imageView();
        return canvas ? canvas->selectionEdgesVisible() : true;
    });

    registry_->setHandler(command_ids::WindowPanelsTools, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsTools);
        if (toolsColumn_ && action) {
            toolsColumn_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsTools,
                                  [this]() { return toolsColumn_ && toolsColumn_->isVisible(); });

    // The two implemented Preferences leaves; the other CS6 panes stay
    // disabled and are no-ops (their command-tree enablement is unchanged).
    registry_->setHandler(command_ids::EditPreferencesGeneral,
                          [this]() { showPreferences(PreferencesDialog::kGeneral); });
    registry_->setHandler(command_ids::EditPreferencesInterface,
                          [this]() { showPreferences(PreferencesDialog::kInterface); });

    // Every other panel is a content widget hosted by the PanelColumn; the
    // Window > Panels toggles drive and reflect the column's registry rather
    // than a dock's visibility.
    struct PanelToggle {
        const char* command;
        const char* panel;
    };
    static const PanelToggle kPanelToggles[] = {
        {command_ids::WindowPanelsLayers, "layersPanel"},
        {command_ids::WindowPanelsNavigator, "navigatorPanel"},
        {command_ids::WindowPanelsHistory, "historyPanel"},
        {command_ids::WindowPanelsColor, "colorPanel"},
        {command_ids::WindowPanelsSwatches, "swatchesPanel"},
        {command_ids::WindowPanelsInfo, "infoPanel"},
        {command_ids::WindowPanelsHistogram, "histogramPanel"},
        {command_ids::WindowPanelsGradients, "gradientsPanel"},
        {command_ids::WindowPanelsPatterns, "patternsPanel"},
        {command_ids::WindowPanelsProperties, "propertiesPanel"},
        {command_ids::WindowPanelsAdjustments, "adjustmentsPanel"},
        {command_ids::WindowPanelsLibraries, "librariesPanel"},
        {command_ids::WindowPanelsChannels, "channelsPanel"},
        {command_ids::WindowPanelsPaths, "pathsPanel"},
        {command_ids::WindowPanelsActions, "actionsPanel"},
    };
    for (const PanelToggle& toggle : kPanelToggles) {
        const QString command = QString::fromLatin1(toggle.command);
        const QString panel = QString::fromLatin1(toggle.panel);
        registry_->setHandler(command, [this, command, panel]() {
            QAction* action = registry_->action(command);
            PanelColumn* owner = columnForPanel(panel);
            if (!owner) {
                owner = panelColumn_;
            }
            if (owner && action) {
                owner->showPanel(panel, action->isChecked());
            }
        });
        registry_->setCheckedProvider(command, [this, panel]() {
            PanelColumn* owner = columnForPanel(panel);
            if (!owner) {
                owner = panelColumn_;
            }
            return owner && owner->isPanelVisible(panel);
        });
    }

    registry_->setHandler(command_ids::HelpAbout, [this]() {
        QMessageBox::about(this, tr("About Kooka Pictura"),
                           tr("Kooka Pictura — a Photoshop CS6 reimplementation in Rust and Qt."));
    });
}

bool PicturaMainWindow::beginFreeTransform(const QString& path)
{
    return tools_ && tools_->beginFreeTransform(path);
}

void PicturaMainWindow::selectLayerPath(const QString& path)
{
    if (!layersPanel_ || path.isEmpty()) {
        return;
    }
    layersPanel_->setView(activeView());
    layersPanel_->refresh();
    layersPanel_->selectPaths(QStringList{path}, path);
}

} // namespace pictura
