#pragma once

namespace pictura {

class ImageView;
class PictureView;

// Present `view`'s view pyramid through `canvas`: the canvas crops a pyramid
// level instead of scaling the full-resolution image. Non-owning; the provider
// lambdas hold QPointers so they stay valid if the view is destroyed first.
void wireCanvasLevelProvider(PictureView* view, ImageView* canvas);

} // namespace pictura
