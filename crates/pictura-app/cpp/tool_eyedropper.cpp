#include "tool_handler.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QtGlobal>
#include <QtGui/QColor>

#include <memory>

namespace pictura {

namespace {

class EyedropperToolHandler : public ToolHandler {
public:
    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        PictureView* v = ctx.view();
        if (!v) {
            return true;
        }
        const quint32 argb = v->sample_argb(qRound(imagePos.x()), qRound(imagePos.y()));
        if (argb != 0) {
            ctx.sampledForeground(QColor::fromRgb(argb));
        }
        return true;
    }
};

} // namespace

std::unique_ptr<ToolHandler> makeEyedropperToolHandler()
{
    return std::make_unique<EyedropperToolHandler>();
}

} // namespace pictura
