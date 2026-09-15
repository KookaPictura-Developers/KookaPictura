#pragma once

#include <QtCore/QString>

class QWidget;

namespace pictura {

enum class UnsavedChoice { Save, Discard, Cancel };

// Ask what to do with a modified document before closing/reverting/replacing it.
// A frame-to-front save prompt is expected when the choice is Save.
UnsavedChoice askUnsaved(QWidget* parent, const QString& documentName);

// Test hook: when interactive is false, askUnsaved does not show a dialog and
// returns the configured choice. Defaults: interactive true, Discard.
void setUnsavedPromptInteractive(bool interactive);
void setNonInteractiveUnsavedChoice(UnsavedChoice choice);

} // namespace pictura
