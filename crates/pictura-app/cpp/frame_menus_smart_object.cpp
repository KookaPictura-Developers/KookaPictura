// Layer > Smart Objects: Reset Transform, Convert to Layers, and New Smart
// Object via Copy, over the active smart-object layer. Each edit is one undo
// state; the bridge refuses a layer whose embedded source does not parse.

#include "frame_includes.h"

#include "pictura_app/src/cxxqt_object/impl_layers/smart_object_actions.cxxqt.h"
#include "pictura_app/src/cxxqt_object/layers_smart_filters.cxxqt.h"

namespace pictura {

void PicturaMainWindow::wireAdvancedSmartObjectActions()
{
    // Reset Transform / Convert to Layers need an embedded PSD/PSB source.
    const auto currentResettableSmartPath = [this]() -> QString {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || path.isEmpty() || !layer_can_reset_smart_object_transform(*view, path)) {
            return QString();
        }
        return path;
    };
    registry_->setHandler(command_ids::LayerSmartObjectResetTransform,
                          [this, currentResettableSmartPath]() {
                              if (PictureView* view = activeView()) {
                                  const QString path = currentResettableSmartPath();
                                  if (!path.isEmpty() && smart_object_reset_transform(*view, path)) {
                                      refresh();
                                  }
                              }
                          });
    registry_->setEnabledProvider(command_ids::LayerSmartObjectResetTransform,
                                  [currentResettableSmartPath]() {
                                      return !currentResettableSmartPath().isEmpty();
                                  });

    const auto currentConvertibleToLayersPath = [this]() -> QString {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || path.isEmpty() || !layer_can_convert_smart_object_to_layers(*view, path)) {
            return QString();
        }
        return path;
    };
    registry_->setHandler(command_ids::LayerSmartObjectConvertToLayers,
                          [this, currentConvertibleToLayersPath]() {
                              if (PictureView* view = activeView()) {
                                  const QString path = currentConvertibleToLayersPath();
                                  if (!path.isEmpty() && smart_object_convert_to_layers(*view, path)) {
                                      refresh();
                                  }
                              }
                          });
    registry_->setEnabledProvider(command_ids::LayerSmartObjectConvertToLayers,
                                  [currentConvertibleToLayersPath]() {
                                      return !currentConvertibleToLayersPath().isEmpty();
                                  });

    // New Smart Object via Copy needs a copyable embedded smart object; the
    // copy is selected after it is inserted.
    const auto currentCopyableSmartPath = [this]() -> QString {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || path.isEmpty() || !layer_can_new_smart_object_via_copy(*view, path)) {
            return QString();
        }
        return path;
    };
    registry_->setHandler(command_ids::LayerSmartObjectNewViaCopy,
                          [this, currentCopyableSmartPath]() {
                              PictureView* view = activeView();
                              const QString path = currentCopyableSmartPath();
                              if (!view || path.isEmpty()) {
                                  return;
                              }
                              const QString copy = smart_object_new_via_copy(*view, path);
                              if (copy.isEmpty()) {
                                  return;
                              }
                              refresh();
                              if (layersPanel_) {
                                  layersPanel_->selectPaths({copy}, copy);
                              }
                          });
    registry_->setEnabledProvider(command_ids::LayerSmartObjectNewViaCopy,
                                  [currentCopyableSmartPath]() {
                                      return !currentCopyableSmartPath().isEmpty();
                                  });
}

void PicturaMainWindow::wireSmartFilterActions()
{
    // Clear Smart Filters removes every filter on the current smart-object
    // layer, recording one state. Enabled only when the current layer has one.
    const auto currentSmartFilterPath = [this]() -> QString {
        PictureView* view = activeView();
        const QString path = layersPanel_ ? layersPanel_->currentPath() : QString();
        if (!view || path.isEmpty() || layer_smart_filter_count(*view, path) <= 0) {
            return QString();
        }
        return path;
    };
    registry_->setHandler(command_ids::LayerSmartFiltersClear, [this, currentSmartFilterPath]() {
        if (PictureView* view = activeView()) {
            const QString path = currentSmartFilterPath();
            if (!path.isEmpty() && clear_layer_smart_filters(*view, path)) {
                refresh();
            }
        }
    });
    registry_->setEnabledProvider(command_ids::LayerSmartFiltersClear, [currentSmartFilterPath]() {
        return !currentSmartFilterPath().isEmpty();
    });
}

} // namespace pictura
