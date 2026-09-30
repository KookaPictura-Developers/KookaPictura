// Mixer Brush: wet paint that mixes the brush's reservoir with the canvas under
// it (Wet / Load / Mix / Flow). The stroke is the Brush's live stroke begun
// with `begin_mixer_brush` (`cxxqt_object/paint_tools.rs`); the reservoir lives
// on the controller so it carries from one stroke to the next unless the bar's
// Load / Clean after-stroke toggles say otherwise. Alt-click loads the brush
// from the image. Ported from photorust's mixer stroke handling.

#include "tool_handler.h"

#include "paint_tip.h"

#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/paint_tools.cxxqt.h"

#include <QtCore/QObject>
#include <QtGui/QColor>

#include <memory>

namespace pictura {

namespace {

class MixerBrushToolHandler : public ToolHandler {
public:
    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        PictureView* v = ctx.view();
        if (!v) {
            return true;
        }
        if (mods.testFlag(Qt::AltModifier)) {
            // ponytail: loads one solid colour; CS6's sampled-variation load
            // (and Load Solid Colors Only) is not modelled.
            ctx.setMixerReservoir(QColor::fromRgba(
                v->sample_argb(qRound(imagePos.x()), qRound(imagePos.y()))));
            return true;
        }
        const MixerOptions o = ctx.mixerOptions();
        // ponytail: Sample All Layers is not wired; the active layer is sampled.
        if (!begin_mixer_brush(*v, ctx.mixerReservoir().rgba(), paintTip(ctx), o.wet, o.load,
                               o.mix, o.flow)) {
            if (activePixelLocked(v)) {
                ctx.refused(QObject::tr("Could not paint: the layer's pixels are locked."));
            } else if (!v->active_layer_visible()) {
                ctx.refused(QObject::tr("Could not paint: the active layer is invisible."));
            } else if (v->active_layer_path().isEmpty()) {
                ctx.refused(QObject::tr("Could not paint: select a single layer first."));
            }
            return true;
        }
        ctx.setDragging(true);
        ctx.setDragCommitted(false);
        v->paint_dab(imagePos.x(), imagePos.y(), 1.0);
        return true;
    }

    void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        if (!ctx.dragging()) {
            return;
        }
        if (PictureView* v = ctx.view()) {
            v->paint_dab(imagePos.x(), imagePos.y(), 1.0);
        }
    }

    void onRelease(ToolContext& ctx, const QPointF&, Qt::KeyboardModifiers) override
    {
        if (!ctx.dragging()) {
            return;
        }
        ctx.setDragging(false);
        PictureView* v = ctx.view();
        if (!v) {
            return;
        }
        const QColor carried = QColor::fromRgba(mixer_reservoir(*v));
        v->end_paint();
        // Clean wins over Load when both are on: a cleaned brush is not then
        // reloaded.
        const MixerOptions o = ctx.mixerOptions();
        if (o.cleanAfterStroke) {
            ctx.setMixerReservoir(QColor(Qt::transparent));
        } else if (o.loadAfterStroke) {
            ctx.setMixerReservoir(ctx.foreground());
        } else {
            ctx.setMixerReservoir(carried);
        }
    }
};

} // namespace

std::unique_ptr<ToolHandler> makeMixerBrushToolHandler()
{
    return std::make_unique<MixerBrushToolHandler>();
}

} // namespace pictura
