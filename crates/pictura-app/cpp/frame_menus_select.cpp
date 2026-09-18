#include "frame_includes.h"

#include <QtWidgets/QInputDialog>
#include <QtWidgets/QLineEdit>

namespace pictura {

// Select menu operations. Parameters come from QInputDialog; a cancelled dialog
// returns before the bridge, so it records nothing. Each applied command records
// exactly one state in the bridge.
void PicturaMainWindow::registerSelectHandlers()
{
    registry_->setHandler(command_ids::SelectReselect, [this]() {
        if (PictureView* view = activeView(); view && view->reselect()) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::SelectInverse, [this]() {
        if (PictureView* view = activeView(); view && view->invert_selection()) {
            refresh();
        }
    });
    const auto modifyOp = [this](const char* op, const QString& title, double initial, double min,
                                 double max, bool decimal) {
        PictureView* view = activeView();
        if (!view) {
            return;
        }
        bool ok = false;
        const double amount = decimal
            ? QInputDialog::getDouble(this, title, tr("Radius:"), initial, min, max, 1, &ok)
            : QInputDialog::getInt(this, title, tr("Amount:"), int(initial), int(min), int(max), 1, &ok);
        if (!ok) {
            return;
        }
        if (view->modify_selection(QString::fromLatin1(op), amount)) {
            refresh();
        }
    };
    registry_->setHandler(command_ids::SelectModifyBorder, [modifyOp]() {
        modifyOp("border", tr("Border Selection"), 1, 1, 200, false);
    });
    registry_->setHandler(command_ids::SelectModifySmooth, [modifyOp]() {
        modifyOp("smooth", tr("Smooth Selection"), 1, 1, 100, false);
    });
    registry_->setHandler(command_ids::SelectModifyExpand, [modifyOp]() {
        modifyOp("expand", tr("Expand Selection"), 1, 1, 100, false);
    });
    registry_->setHandler(command_ids::SelectModifyContract, [modifyOp]() {
        modifyOp("contract", tr("Contract Selection"), 1, 1, 100, false);
    });
    registry_->setHandler(command_ids::SelectModifyFeather, [modifyOp]() {
        modifyOp("feather", tr("Feather Selection"), 1.0, 0.0, 250.0, true);
    });
    registry_->setHandler(command_ids::SelectGrow, [this]() {
        PictureView* view = activeView();
        const int tolerance = tools_ ? tools_->tolerance() : 0;
        if (view && view->grow_selection(tolerance)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::SelectSimilar, [this]() {
        PictureView* view = activeView();
        const int tolerance = tools_ ? tools_->tolerance() : 0;
        if (view && view->similar_selection(tolerance)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::SelectSave, [this]() {
        PictureView* view = activeView();
        if (!view || !view->has_selection()) {
            return;
        }
        bool ok = false;
        const QString name = QInputDialog::getText(
            this, tr("Save Selection"), tr("Name:"), QLineEdit::Normal,
            QStringLiteral("Alpha %1").arg(view->selection_channel_count() + 1), &ok);
        if (!ok || name.trimmed().isEmpty()) {
            return;
        }
        if (view->save_selection(name)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::SelectLoad, [this]() {
        PictureView* view = activeView();
        const int channels = view ? view->selection_channel_count() : 0;
        if (!view || channels < 1) {
            return;
        }
        QStringList labels;
        for (int i = 1; i <= channels; ++i) {
            labels.push_back(QStringLiteral("Alpha %1").arg(i));
        }
        bool ok = false;
        const QString name = QInputDialog::getItem(this, tr("Load Selection"), tr("Channel:"),
                                                   labels, 0, false, &ok);
        if (!ok || name.isEmpty()) {
            return;
        }
        if (view->load_selection(name)) {
            refresh();
        }
    });

    registry_->setHandler(command_ids::SelectAllLayers, [this]() {
        PictureView* view = activeView();
        if (!view || !layersPanel_) {
            return;
        }
        const QStringList paths = view->select_all_layers();
        if (paths.isEmpty()) {
            return;
        }
        layersPanel_->selectPaths(paths, paths.value(0));
        refresh();
    });
    registry_->setHandler(command_ids::SelectDeselectLayers, [this]() {
        if (layersPanel_) {
            layersPanel_->selectPaths(QStringList{}, QString());
            refresh();
        }
    });
    registry_->setHandler(command_ids::SelectSimilarLayers, [this]() {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || path.isEmpty()) {
            return;
        }
        QStringList matched = view->select_similar(path);
        if (!matched.contains(path)) {
            matched.prepend(path);
        }
        if (layersPanel_) {
            layersPanel_->selectPaths(matched, path);
        }
        refresh();
    });

    const auto selectHasDoc = [this]() {
        return activeView() && activeView()->has_document();
    };
    for (const char* id : {command_ids::SelectReselect, command_ids::SelectInverse,
                           command_ids::SelectModifyBorder, command_ids::SelectModifySmooth,
                           command_ids::SelectModifyExpand, command_ids::SelectModifyContract,
                           command_ids::SelectModifyFeather, command_ids::SelectGrow,
                           command_ids::SelectSimilar, command_ids::SelectSave,
                           command_ids::SelectLoad, command_ids::SelectAllLayers,
                           command_ids::SelectDeselectLayers, command_ids::SelectSimilarLayers}) {
        registry_->setEnabledProvider(id, selectHasDoc);
    }
    for (const char* id : {command_ids::SelectInverse, command_ids::SelectModifyBorder,
                           command_ids::SelectModifySmooth, command_ids::SelectModifyExpand,
                           command_ids::SelectModifyContract, command_ids::SelectModifyFeather,
                           command_ids::SelectGrow, command_ids::SelectSimilar,
                           command_ids::SelectSave}) {
        registry_->setEnabledProvider(id, [this]() {
            return activeView() && activeView()->has_document() && activeView()->has_selection();
        });
    }
    registry_->setEnabledProvider(command_ids::SelectReselect, [this]() {
        return activeView() && activeView()->has_document()
            && activeView()->has_deselected_selection();
    });
    registry_->setEnabledProvider(command_ids::SelectLoad, [this]() {
        return activeView() && activeView()->has_document()
            && activeView()->selection_channel_count() > 0;
    });
    registry_->setEnabledProvider(command_ids::SelectSimilarLayers, [this]() {
        return activeView() && activeView()->has_document() && layersPanel_
            && !layersPanel_->currentPath().isEmpty();
    });
}

} // namespace pictura
