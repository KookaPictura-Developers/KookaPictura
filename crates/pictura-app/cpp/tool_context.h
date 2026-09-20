#pragma once

#include <QtGui/QColor>

namespace pictura {

class ImageView;
class PictureView;

// The shared services a tool handler may use, implemented by `ToolController`.
// Kept minimal on purpose: add accessors only as a migrating tool needs them.
struct ToolContext {
    virtual ~ToolContext() = default;
    virtual PictureView* view() const = 0;
    virtual ImageView* canvas() const = 0;
    virtual void sampledForeground(const QColor& color) = 0;
};

} // namespace pictura
