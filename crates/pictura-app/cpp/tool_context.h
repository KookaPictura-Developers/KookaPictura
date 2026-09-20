#pragma once

#include <QtCore/QString>
#include <QtCore/Qt>
#include <QtGui/QColor>

namespace pictura {

class ImageView;
class PictureView;
enum class SelectionMode;

// The shared services a tool handler may use, implemented by `ToolController`.
// Kept minimal on purpose: add accessors only as a migrating tool needs them.
struct ToolContext {
    virtual ~ToolContext() = default;
    virtual PictureView* view() const = 0;
    virtual ImageView* canvas() const = 0;
    virtual void sampledForeground(const QColor& color) = 0;

    virtual QColor foreground() const = 0;
    virtual QColor background() const = 0;
    virtual int brushSize() const = 0;
    virtual int brushHardness() const = 0;
    virtual int brushOpacity() const = 0;
    virtual int brushFlow() const = 0;
    virtual QString brushMode() const = 0;
    virtual bool autoErase() const = 0;
    virtual int tolerance() const = 0;
    virtual bool contiguous() const = 0;

    virtual bool dragging() const = 0;
    virtual void setDragging(bool dragging) = 0;
    virtual bool dragCommitted() const = 0;
    virtual void setDragCommitted(bool committed) = 0;
    virtual SelectionMode dragMode() const = 0;
    virtual void setDragMode(SelectionMode mode) = 0;
    virtual SelectionMode resolveSelectionMode(Qt::KeyboardModifiers mods,
                                               bool hasExistingSelection) const = 0;

    virtual void refused(const QString& message) = 0;
    virtual void emitSelectionCommitted() = 0;
};

} // namespace pictura
