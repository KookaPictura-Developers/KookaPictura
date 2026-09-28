// Content-Aware Move tool: outline a region, then drag the outline (see
// RegionDragToolHandler). Move relocates the pixels and rebuilds the hole from
// its surroundings; Extend copies them and keeps the original
// (`content_aware_move` in `cxxqt_object/healing.rs`).

#include "tool_region_drag.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/healing.cxxqt.h"

#include <memory>

namespace pictura {

namespace {

class ContentAwareMoveToolHandler : public RegionDragToolHandler {
protected:
    // ponytail: no Sample All Layers yet; the active layer is both read and
    // written.
    bool apply(ToolContext& ctx, PictureView& view, int dx, int dy) override
    {
        const bool moved = content_aware_move(view, dx, dy, ctx.contentAwareMoveExtend(),
                                              ctx.contentAwareAdaptation());
        if (moved) {
            ctx.emitSelectionCommitted();
        }
        return moved;
    }

    QString verb() const override { return QStringLiteral("move"); }
};

} // namespace

std::unique_ptr<ToolHandler> makeContentAwareMoveToolHandler()
{
    return std::make_unique<ContentAwareMoveToolHandler>();
}

} // namespace pictura
