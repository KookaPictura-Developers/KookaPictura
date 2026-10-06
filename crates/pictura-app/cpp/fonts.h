#pragma once

#include <QtGui/QFont>

class QTabBar;

namespace pictura {

// Pinned UI font size in device-independent pixels. The theme and the chrome
// derivations (tab labels, status footer) read the application font and run two
// pixels under it, so this is the anchor they expect. 12 px is the 9 pt default
// Qt reports at 96 DPI, made explicit so every OS agrees.
inline constexpr int kBundledUiFontPx = 12;

// The bundled Noto Sans family name.
extern const char* const kBundledUiFamily;

// Register the bundled Noto Sans faces (upright + italic, 400/500/600/700)
// with the Qt font database. Idempotent; call after the QApplication exists.
void registerBundledFonts();

// The application UI font: bundled Noto Sans at kBundledUiFontPx.
QFont bundledUiFont();

// Register the bundled faces and install bundledUiFont() as the application
// font. Call once after the QApplication exists and before the UI is built.
void applyBundledUiFont();

// Apply the shared tab-label chrome font to `bar`: the bundled family at the
// app size minus two pixels, marked Bold. QTabBar computes elision from its own
// font, so the bar font must match the painted label; doing it here keeps the
// document and panel tab bars identical and does not depend on the QSS
// subcontrol rule.
void applyTabBarFont(QTabBar* bar);

} // namespace pictura
