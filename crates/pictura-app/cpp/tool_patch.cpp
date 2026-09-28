// Patch tool: outline a region, then drag the outline (see
// RegionDragToolHandler). Source repairs the selection from the dragged-to
// area, Destination applies the selection there, Content-Aware rebuilds it in
// place (`patch_selection` in `cxxqt_object/healing.rs`).

#include "tool_region_drag.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/healing.cxxqt.h"

#include <memory>

namespace pictura {

namespace {

class PatchToolHandler : public RegionDragToolHandler {
protected:
    // ponytail: no Adaptation, Sample All Layers, or Use Pattern yet.
    bool apply(ToolContext& ctx, PictureView& view, int dx, int dy) override
    {
        return patch_selection(view, dx, dy, ctx.patchContentAware(), ctx.patchDestination(),
                               ctx.patchTransparent());
    }

    QString verb() const override { return QStringLiteral("patch"); }
};

} // namespace

std::unique_ptr<ToolHandler> makePatchToolHandler()
{
    return std::make_unique<PatchToolHandler>();
}

} // namespace pictura
