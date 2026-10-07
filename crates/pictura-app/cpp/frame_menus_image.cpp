#include "frame_includes.h"

#include "pictura_app/src/cxxqt_object/image_adjust/image_ops.cxxqt.h"

namespace pictura {

// Image > Crop, Trim, and Duplicate (Image > Mode is frame_menus_image_mode.cpp).
// Lifted out of registerHandlers() to keep frame_menus.cpp within its size
// budget; the handler bodies are unchanged.
void PicturaMainWindow::wireImageMenu()
{
    wireImageModeMenu();

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

    registry_->setHandler(command_ids::ImageTrim, [this]() {
        PictureView* view = activeView();
        if (view && trim_image(*view)) {
            refresh();
        }
    });
    registry_->setHandler(command_ids::ImageDuplicate, [this]() {
        PictureView* source = activeView();
        if (!source || !source->has_document()) {
            return;
        }
        const QString suggested = duplicate_name(*source);
        DuplicateImageDialog dialog(activeDocumentName(), suggested, this);
        if (runDialog(dialog, this) != QDialog::Accepted) {
            return;
        }
        auto* target = new PictureView(this);
        if (!duplicate_into(*source, *target, dialog.copyName(), dialog.mergedOnly())) {
            delete target;
            return;
        }
        const int index = addDocument(target, QString());
        if (index >= 0) {
            docs_[index].displayName = dialog.copyName();
            updateTabTitle(index);
        }
    });
    const auto imageEnabled = [this]() {
        return activeView() && activeView()->has_document();
    };
    for (const char* id : {command_ids::ImageTrim, command_ids::ImageDuplicate}) {
        registry_->setEnabledProvider(id, imageEnabled);
    }
}

} // namespace pictura
