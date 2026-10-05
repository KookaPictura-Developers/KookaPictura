# Design

## Context

See proposal.md — Why. The app is a Rust `staticlib` linked into the C++/Qt6
`pictura` binary. Assets are compiled once into an object library from
`assets/pictura.qrc` (`CMakeLists.txt`) and linked by the executable and every
Qt Test binary, so a `:/fonts/…` resource is available to both. The application
font is currently inherited from the platform theme; `main.cpp` never sets it,
and `Theme` only sets the Fusion style, palette, and QSS. Two chrome
derivations read `QApplication::font().pixelSize()` and run two pixels under it:
the panel tab bar (`panels/panel_group.cpp`) and the status footer
(`frame_build.cpp`), with the theme QSS scaling document tabs the same way.

The Rust text engine (`pictura-render`) already bundles a font
(`LiberationSans-Regular.ttf`, `include_bytes!`), and `type_fonts.cpp` notes
that a variable font's named instance arrives as the variable font in that
pipeline — so extra weights must be static instances.

## Goals / Non-Goals

**Goals:**

- The application UI font is a bundled family on every OS, so a weight request
  resolves to a face the app controls.
- The tab chrome's requested weight (Bold, 700) actually renders.
- The pinned font size keeps the existing chrome derivations (app size − 2 px)
  correct.
- License and provenance are auditable like every other bundled asset.

**Non-Goals:**

- No change to the Rust text engine or its Liberation Sans fallback.
- No variable font and no runtime font download.
- No other chrome weight changes: the body text stays Regular and only the tab
  labels move to Bold (700).

## Decisions

### Static instances, not the variable font

Google Fonts now distributes Noto Sans only as `NotoSans[wdth,wght].ttf`; the
static per-weight instances live in `notofonts/noto-fonts`
(`hinted/ttf/NotoSans/`). Static instances are chosen because (a) `type_fonts.cpp`
documents that the Rust engine renders a variable font as its default instance,
so a variable bundle would make the type tools ignore weight, and (b) plain
`font-weight` resolution against static faces needs no Qt variable-axis handling.
Four weights (400/500/600/700) upright and italic cover the UI and a useful
subset of the type-tool style list.

### Bold (700) for the tab labels

Noto Sans Medium (500) adds only ~1.6 % ink over Regular at the 10 px tab size
(measured), which read as regular. The tab labels move to Bold (700) (~5.5 %
ink) so the chrome weight is clearly distinct from the body text. The Medium and
SemiBold faces stay bundled for the type tools and future use.

### Tab chrome font set on the bar, not only the QSS

Qt honours a `QTabBar::tab` font in the stylesheet, but relying on it alone left
the file bar depending on paint-time resolution while the panel bar also set the
font on the bar. The file bar therefore rendered regular while the panel bar
rendered bold. Both bars now apply the same font on the bar itself through
`applyTabBarFont()` (bundled family, app px − 2, Bold), which also keeps
QTabBar's elision metrics in step with the painted label. The QSS rule stays as
a mirror.

### Qt resource + `addApplicationFont`, not fontconfig install

The `.ttf` files are embedded in `assets/pictura.qrc` and registered with
`QFontDatabase::addApplicationFont(":/fonts/…")`. This needs no system install,
no `fontconfig` write, and no per-OS packaging step, and it makes the fonts
available to the Qt Test binaries that already link `pictura_assets`. Installs
of Noto Sans on the host are ignored for the application font because the
application font database is consulted first.

### Pin a pixel size, not a point size

The application font is set with `setPixelSize(12)` rather than a point size.
The two chrome derivations read `QApplication::font().pixelSize()`; a
point-sized default returns `-1` and falls back to a hard-coded 12 px, which
would make the tabs drift from the app font on non-96-DPI screens. A pinned
pixel size keeps `pixelSize() - 2` meaning exactly "two pixels under the app
font" and is identical on every OS that renders at the same scale factor. 12 px
is the 9 pt default the current builds already fall back to, so metrics do not
move on the reference platform.

### One registration choke point

`registerBundledFonts()` is idempotent (static flag) and is called from
`applyBundledUiFont()`, which `main.cpp` invokes once after the `QApplication`
and before any widget is built. Tests call the same functions, so the bundled
family is exercised by the new `tst_fonts` suite.

## Risks / Trade-offs

- **Binary size**: ~4.5 MB of fonts, zlib-compressed in the resource. Acceptable
  for a desktop image editor; the Qt Test binaries grow too because they link
  the shared resources object library.
- **Host duplication**: on a machine that already ships Noto Sans, two copies are
  registered. Both provide the same weights, so rendering is consistent; the
  application copy takes precedence for the app font.
- **Italic metrics**: bundling italics adds ~2 MB for a face set the UI barely
  uses, but the type tools and the style list benefit.

## Migration

None. No persisted state changes; the font is resolved at startup.

## Open Questions

None.
