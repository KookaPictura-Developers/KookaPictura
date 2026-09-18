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
                           command_ids::FileCloseAll}) {
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

    // M37: layer creation and grouping. The target is the active document's
    // current layer — the Layers panel's selected row, -1 when none.
    // ponytail: one current layer; M38's multi-selection upgrades these to
    // per-selection operations.
    registry_->setHandler(command_ids::LayerNewLayer, [this]() {
        if (PictureView* view = activeView()) {
            view->add_layer(layersPanel_ ? layersPanel_->currentLayer() : -1);
            refresh();
        }
    });
    registry_->setHandler(command_ids::LayerNewGroup, [this]() {
        if (PictureView* view = activeView()) {
            view->add_group(layersPanel_ ? layersPanel_->currentLayer() : -1);
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
        if (PictureView* view = activeView()) {
            view->group_layer(layersPanel_ ? layersPanel_->currentLayer() : -1);
            refresh();
        }
    });
    registry_->setHandler(command_ids::LayerUngroupLayers, [this]() {
        if (PictureView* view = activeView()) {
            view->ungroup_layer(layersPanel_ ? layersPanel_->currentLayer() : -1);
            refresh();
        }
    });
    for (const char* id : {command_ids::LayerNewLayer, command_ids::LayerNewGroup,
                           command_ids::LayerDuplicateLayer, command_ids::LayerGroupLayers,
                           command_ids::LayerUngroupLayers}) {
        registry_->setEnabledProvider(id,
                                      [this]() { return activeView() && activeView()->has_document(); });
    }

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

    registry_->setHandler(command_ids::WindowPanelsTools, [this]() {
        QAction* action = registry_->action(command_ids::WindowPanelsTools);
        if (toolsDock_ && action) {
            toolsDock_->setVisible(action->isChecked());
        }
    });
    registry_->setCheckedProvider(command_ids::WindowPanelsTools,
                                  [this]() { return toolsDock_ && toolsDock_->isVisible(); });

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

} // namespace pictura
