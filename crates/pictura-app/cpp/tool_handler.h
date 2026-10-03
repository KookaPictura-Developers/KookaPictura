#pragma once

#include "tool_context.h"

#include <QtCore/QPointF>
#include <QtCore/QRect>
#include <QtCore/QString>
#include <QtCore/Qt>

namespace pictura {

// One tool's pointer behavior. Returning true from `onPress` consumes the
// event; the other hooks are fire-and-forget. The default no-ops let a handler
// override only what it needs.
class ToolHandler {
public:
    virtual ~ToolHandler() = default;
    virtual bool onPress(ToolContext& ctx, const QPointF& imagePos,
                         Qt::KeyboardModifiers mods)
    {
        return false;
    }
    virtual void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) {}
    virtual void onRelease(ToolContext& ctx, const QPointF& imagePos,
                           Qt::KeyboardModifiers mods)
    {
    }

    // Lifecycle: the controller activates the new handler and deactivates the
    // old one on tool switch and canvas unbind, so handlers hold no stale state.
    virtual void onActivate(ToolContext& ctx) {}
    virtual void onDeactivate(ToolContext& ctx) {}
    // The frame refreshed (a document edit, undo, or tab switch) with this tool
    // active; re-read any document state the tool mirrors on the canvas.
    virtual void onDocumentRefreshed(ToolContext& ctx) {}
    // A tool option changed (e.g. the crop aspect ratio); re-apply it.
    virtual void onOptionsChanged(ToolContext& ctx) {}

    // Tool-specific commands the controller forwards by active tool.
    virtual bool commitPolygonLasso() { return false; }
    virtual bool cancelPolygonLasso() { return false; }
    // Delete: drop the last point of a click-driven lasso outline.
    virtual bool removeLassoPoint() { return false; }
    // True while a click-driven outline is open, so presses inside a live
    // selection extend it instead of starting a selection move.
    virtual bool lassoInProgress() const { return false; }
    virtual bool commitCrop() { return false; }
    // The Type tools' Commit (Ctrl+Enter) and Cancel (Esc), and whether text is
    // being typed.
    virtual bool commitText() { return false; }
    virtual bool cancelText() { return false; }
    virtual bool textActive() const { return false; }
    // Insert `text` at the open type edit's caret (the Glyphs panel); false
    // when no edit is open.
    virtual bool insertText(const QString& text) { return false; }
    // The options bar's Clear: drop the tool's color samplers, notes, or ruler.
    virtual bool clearAnnotations() { return false; }

    // A suffix for the tool's cursor id while the pointer hovers something the
    // next press acts on specially (the Pen's ".close" over the first anchor);
    // the controller re-applies the cursor when it changes.
    virtual QString cursorVariant() const { return QString(); }

    // The modifiers captured at press for a selection drag; only the marquee
    // handlers keep them (the self-test reads them through the controller).
    virtual Qt::KeyboardModifiers dragMods() const { return Qt::NoModifier; }
    // The Crop handler's staged rectangle; the controller exposes it.
    virtual bool hasPendingCrop() const { return false; }
    virtual QRect pendingCropRect() const { return QRect(); }
};

} // namespace pictura
