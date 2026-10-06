#pragma once

class QFontComboBox;

namespace pictura {

// Keep Qt's per-family previews in `combo`, except for emoji faces: those list
// their name in the UI font with no sample.
// Previewing a color emoji face costs ~400 ms per repaint and draws garbage, so
// a list scrolled near one stays blank for seconds.
void useFastFontPreviews(QFontComboBox* combo);

} // namespace pictura
