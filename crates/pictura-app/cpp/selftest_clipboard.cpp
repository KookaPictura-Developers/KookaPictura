#include "selftest_clipboard.h"
#include "selftest_report.h"

#include "commands.h"
#include "frame.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/clipboard.cxxqt.h"

#include <QtGui/QAction>

int pictura::runClipboardChecks(pictura::PicturaMainWindow& frame)
{
    const bool created = frame.newDocument(QStringLiteral("ClipboardCtl"), 8, 8,
                                           QStringLiteral("rgb"), 8, QStringLiteral("white"));
    pictura::PictureView* view = frame.activeView();
    if (!created || !view) {
        return pictura::selfTest().fail(529, "clipboard fixture");
    }
    const int doc = frame.activeDocumentIndex();
    pictura::CommandRegistry* registry = frame.registry();
    const auto enabled = [registry](const char* id) {
        registry->refresh();
        QAction* action = registry->action(QString::fromLatin1(id));
        return action && action->isEnabled();
    };
    // Each step dispatches `id` and reports whether it added exactly one state
    // labelled `label` (or none when `label` is null).
    const auto step = [view, registry](const char* id, const char* label) {
        const int base = view->history_count();
        registry->dispatch(QString::fromLatin1(id));
        if (!label) {
            return view->history_count() == base;
        }
        return view->history_count() == base + 1
            && view->history_label(base) == QString::fromLatin1(label);
    };

    pictura::clipboard_purge();
    const bool pasteOffEmpty = !enabled(command_ids::EditPaste);
    const quint32 white = view->sample_argb(0, 0);
    const QString path = view->add_solid_fill(0xff2244aau);
    const bool raster = view->rasterize_fill_content(path);
    frame.selectLayerPath(path);
    const quint32 fill = view->sample_argb(3, 3);
    view->select_rect(2, 2, 3, 3, QStringLiteral("new"), 0.0);

    const bool copied = step(command_ids::EditCopy, nullptr) && pictura::clipboard_has_contents();
    const bool cut = step(command_ids::EditCut, "Cut") && view->sample_argb(3, 3) == white
        && view->sample_argb(1, 1) == fill;
    const int layers = view->layer_count();
    const auto hasMask = [view]() {
        for (int row = 0; row < view->layer_row_count(); ++row) {
            if (view->layer_row_has_mask(row)) {
                return true;
            }
        }
        return false;
    };
    const bool into = step(command_ids::EditPasteInto, "Paste Into")
        && view->layer_count() == layers + 1 && view->sample_argb(3, 3) == fill
        && hasMask() && !view->has_selection();

    frame.selectLayerPath(path);
    view->select_rect(0, 0, 2, 2, QStringLiteral("new"), 0.0);
    const bool clear = step(command_ids::EditClear, "Clear") && view->sample_argb(0, 1) == white
        && view->sample_argb(6, 0) == fill;
    // Last: a plain paste lands wherever the (headless) canvas centre maps.
    const bool paste = step(command_ids::EditPaste, "Paste") && view->layer_count() == layers + 2;
    const bool purged = step(command_ids::EditPurgeClipboard, nullptr)
        && !pictura::clipboard_has_contents() && !enabled(command_ids::EditPaste);

    ST_BEGIN("edit_clipboard");
    ST_PASS("edit_clipboard raster=%d emptyOff=%d copy=%d cut=%d into=%d paste=%d clear=%d purge=%d",
            raster ? 1 : 0, pasteOffEmpty ? 1 : 0, copied ? 1 : 0, cut ? 1 : 0, into ? 1 : 0,
            paste ? 1 : 0, clear ? 1 : 0, purged ? 1 : 0);
    if (!raster || !pasteOffEmpty || !copied || !cut || !into || !paste || !clear || !purged) {
        return pictura::selfTest().fail(529, "edit clipboard");
    }
    frame.closeDocument(doc, false);
    return 0;
}
