#include "tool_handler.h"

#include "image_view.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <memory>

namespace pictura {

namespace {

class MagicWandHandler : public ToolHandler {
public:
    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        PictureView* v = ctx.view();
        if (!v) {
            return true;
        }
        ctx.setDragMode(ctx.resolveSelectionMode(mods, v->has_selection()));
        const bool committed = v->magic_wand(qRound(imagePos.x()), qRound(imagePos.y()),
                                             ctx.tolerance(), ctx.contiguous(),
                                             selectionModeString(ctx.dragMode()));
        if (committed) {
            ctx.emitSelectionCommitted();
        }
        return true;
    }
};

class QuickSelectionHandler : public ToolHandler {
public:
    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        PictureView* v = ctx.view();
        if (!v) {
            return true;
        }
        ctx.setDragMode(ctx.resolveSelectionMode(mods, v->has_selection()));
        ctx.setDragging(true);
        ctx.setDragCommitted(v->quick_select(qRound(imagePos.x()), qRound(imagePos.y()),
                                             ctx.tolerance(),
                                             selectionModeString(ctx.dragMode())));
        return true;
    }

    void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        if (!ctx.dragging()) {
            return;
        }
        PictureView* v = ctx.view();
        if (!v) {
            return;
        }
        if (v->quick_select(qRound(imagePos.x()), qRound(imagePos.y()), ctx.tolerance(),
                            selectionModeString(ctx.dragMode()))) {
            ctx.setDragCommitted(true);
        }
    }

    void onRelease(ToolContext& ctx, const QPointF&, Qt::KeyboardModifiers) override
    {
        if (!ctx.dragging()) {
            return;
        }
        ctx.setDragging(false);
        if (ImageView* canvas = ctx.canvas()) {
            canvas->clearSelectionPreview();
        }
        if (ctx.dragCommitted()) {
            ctx.emitSelectionCommitted();
        }
    }
};

} // namespace

std::unique_ptr<ToolHandler> makeMagicWandToolHandler()
{
    return std::make_unique<MagicWandHandler>();
}

std::unique_ptr<ToolHandler> makeQuickSelectionToolHandler()
{
    return std::make_unique<QuickSelectionHandler>();
}

} // namespace pictura
