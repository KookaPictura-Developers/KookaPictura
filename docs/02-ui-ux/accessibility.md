# Accessibility

- **Spec ID:** `UI-012`
- **Status:** `Draft`
- **Parity tier:** `Core` (keyboard navigation and scalable UI are Core; screen-reader depth is a
  *superset* of CS6, which Adobe never fully exposed).
- **New in CS6:** `No` / `Changed` — CS6 adds no accessibility-specific feature. It keeps the
  CS5-era model: a very large shortcut set, a few theme/font-size controls, and no documented
  screen-reader (MSAA/UIA/VoiceOver) support. The CS6 Help reference contains **no**
  assistive-technology chapter; its only "accessibility" content concerns *exporting* accessible
  content (accessible PDF/HTML), not operating the application. Kooka Pictura therefore proposes
  to **exceed** CS6 parity here, using Qt6 + AT-SPI2.
- **Depends on:** `02-ui-ux/preferences.md` (`UI-010`), `02-ui-ux/keyboard-shortcuts.md`
  (`UI-011`), `01-architecture/qt6-ui-design.md`, `01-architecture/rust-qt-interop.md`,
  `05-layers/layer-management-ui.md`, `07-color-painting/color-picker.md`,
  `00-overview/feasibility-and-non-goals.md`.

> Qt6/Linux accessibility is a **design proposal**. CS6's own assistive-technology support is
> poorly documented; claims about it are marked *(unverified)* and collected under
> `## Open questions`.

## CS6 behavior

### What CS6 documents

The CS6 Help reference (Feb 2013) has no section on screen readers, MSAA, UI Automation,
VoiceOver, high contrast, or color blindness. Its entire "accessibility" usage is about producing
accessible *output*:

- **Rich Content PDF** export creates accessible PDF files that contain tags, hyperlinks, bookmarks, interactive elements, and layers.
- **Always Add Alt Attribute** adds the ALT attribute to IMG elements so exported pages meet government web accessibility standards.
- **Save for Web / Optimize** produces HTML with alt attributes / accessibility attributes.

This means: for an edition that Adobe marketed to enterprises and government, the app's own
operability by disabled users was effectively undocumented. The de-facto accessibility story is
**keyboard shortcuts** plus OS-level conventions.

### Keyboard-first operation (documented)

- Nearly every command has a shortcut or a menu path; `Edit > Keyboard Shortcuts` lets users
  create/rebind them (`UI-011`).
- Standard menu navigation works: `Alt`-accelerators (Windows) expose menu mnemonics; on macOS
  the user enables "Keyboard navigation" / "Full Keyboard Access" in System Settings to Tab
  through all controls.
- Dialogs are conventionally Tab-navigable with OK/Cancel/Next/Prev.
- `Caps Lock` toggles precise cursors; `Tab`/`Shift+Tab` and arrow keys move around panels and
  numeric fields; `Enter`/`Escape` commit/cancel.
- Context menus (right-click / Control-click) duplicate many menu commands.
- Preferences reset gesture `Alt+Ctrl+Shift` / `Option+Command+Shift` at launch is keyboard-only.

### Screen readers

Community evidence is that Adobe's Creative Suite apps — Photoshop included — did **not** expose
their interface to macOS VoiceOver (a 2019 Adobe Community thread titled "Why does Adobe Creative
Cloud products have zero screen reader support?" documents this), and CS6's Help is silent on
Windows screen readers too. Modern Adobe Accessibility Conformance Reports exist for Photoshop
21.x+, but no CS6-era report is prominent. Whether CS6 exposed *any* controls through MSAA (Win32)
or NSAccessibility (Mac) is *(unverified)*.

**Conclusion for parity:** CS6 parity does not require a bespoke screen-reader contract, because
CS6 did not have a documented one. Kooka Pictura should nonetheless expose a real AT-SPI tree for
Linux, because Qt6 makes it cheap and because the project's stated Linux-native goal makes
keyboard/screen-reader users first-class.

### UI scaling, contrast, and color

- **Themes:** CS6's Interface preference offers 4 gray levels (Color Theme); the lightest is the
  closest thing to a "contrast" option. There is no true high-contrast theme and no
  color-blind mode.
- **Text size:** Interface → **UI Font Size** = Tiny/Small/Medium/Large, application restart
  required. This scales panel/tool-tip text, not the canvas.
- **Canvas zoom/magnification:** Zoom tool, Navigator, and OS magnifiers; CS6 does not integrate
  with OS screen magnifiers beyond the whole-window level.
- **Color-only signalling:** most states are also conveyed by shape/position (lock icons,
  visibility eye, badge glyphs), but several are color-only — **layer color labels**, **guide /
  grid / slice line colors**, **gamut warning overlay**, and **channel/selection overlays**.
  A color-blind user can lose these.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Edit > Preferences > Interface > Color Theme` | Preference | `Ctrl/Cmd+K`+pane | 4 gray themes; lightest ≈ high contrast. |
| `Edit > Preferences > Interface > UI Font Size` | Preference | — | Tiny/Small/Medium/Large; restart. |
| `Edit > Preferences > General` | Preference | — | Zoom/flick-panning options affect motor load. |
| `Edit > Preferences > Cursors` | Preference | — | Precise cursors help low-vision users locate pointers. |
| `Edit > Keyboard Shortcuts` | Dialog | — | Full keyboard operation; no mouse needed. |
| `Window > Workspace` | Menu | — | Keyboard-only workspace switching. |
| All menus / dialogs | UI | `Alt`-mnemonics / `Tab` | Keyboard navigation. |
| Canvas / document window | Custom surface | — | Screen readers need a custom accessible object. |
| Panels (Layers, Channels, Paths, etc.) | Dock widgets | `Tab`/arrows/`F7`… | Must expose names, roles, states. |
| Status bar / progress | UI | — | Progress must be announced (live region). |
| OS settings | External | — | macOS Full Keyboard Access; Windows Magnifier/Narrator; Linux Orca. |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Color Theme | enum | mid-dark | 4 gray levels | Add a true high-contrast theme (proposal). |
| UI Font Size | enum | Small | Tiny/Small/Medium/Large | Scales UI text only. |
| High-DPI scale | float | system | 1.0–n, per-monitor | Qt6 scaling (proposal). |
| Focus indicator | style | OS default | visible focus ring | Must never be removed. |
| Accessible name/role/state | attrs | — | per widget | AT-SPI contract. |
| Minimum contrast | ratio | n/a in CS6 | ≥ 4.5:1 (WCAG AA) proposed | For text/UI; not a CS6 field. |
| Announcement verbosity | enum | standard | standard/verbose | Screen-reader output detail. |
| Color-blind-safe overlays | bool | off | on/off | Use pattern/shape, not hue only. |

## Algorithms & pipeline

Accessibility is an **out-of-band semantic mirror** of the visible UI, not a render pass:

1. **Semantic tree construction.** Every `QWidget` / QML item has a `QAccessibleInterface`
   describing role, name, description, state and relations. Qt ships these for built-in widgets;
   custom widgets (canvas, rulers, histogram, color wheel, curve editor) need hand-written
   interfaces or an accessible factory function.
2. **Event propagation.** When focus/state/text/visibility changes, the app emits
   `QAccessibleEvent` (`Focus`, `StateChanged`, `ValueChanged`, `NameChanged`, `Announcement`)
   via `QAccessible::updateAccessibility`; Qt maps these to the platform API.
3. **Platform bridge.** On Linux, Qt publishes the tree over **AT-SPI2 D-Bus**; **Orca** (and
   other AT clients) consume it. On Windows Qt maps to MSAA/UIA; on macOS to NSAccessibility.
   `QAccessible::isActive()` reports whether an AT client is attached; when false, expensive
   accessibility work can be skipped.
4. **Keyboard routing.** A layered dispatcher (modal → text-edit → transient tool → panel → menu,
   see `UI-011`) guarantees every command is reachable without a pointer; focus order is explicit.
5. **Scaling.** Qt's high-DPI pipeline (`QT_ENABLE_HIGHDPI_SCALING`, per-monitor
   `devicePixelRatio`, `HighDpiScaleFactorRoundingPolicy`) rescales widgets and pixmaps; font
   metrics come from the system.
6. **Color adaptation.** The platform theme/palette is applied by default; on Linux, honor the
   desktop's color scheme (KDE/GTK) and offer a high-contrast palette. Never encode a state by
   color alone.
7. **Canvas semantics.** Pixels cannot be meaningfully narrated. The proposal exposes
   *structured* information instead: active tool, cursor position/color sample, selection bounds,
   active layer, zoom level, and histogram summary — via an accessible document object and an
   accessible "status" live region.

Behavioral parity note: CS6 documents none of steps 1–3 or 7; they are added value, not parity
requirements.

## Rust module mapping

- `crate::access::Node` — a semantic UI node: `{ id, role, name, description, state, children,
  relations, value }`. Built by the UI layer, independent of Qt so it is testable.
- `crate::access::tree::AccessTree` — maintains the node hierarchy and diffs changes into events.
- `crate::access::events::{AccessEvent, Role, StateBit, Relation}` — platform-neutral event model.
- `crate::access::announce` — a queue for polite/assertive announcements (progress, tool change,
  selection change).
- `crate::access::canvas::CanvasSemantics` — derives structured canvas descriptions (tool, zoom,
  selection, active layer, sample) from the document/cursor state.
- `crate::input::focus` — focus ring/graph model shared with `UI-011`'s dispatcher; enforces no
  keyboard traps.
- `crate::prefs::InterfacePrefs` — supplies color theme, UI font size, and a new
  `high_contrast`/`color_blind_safe` flag (`UI-010`).
- Types crossing the Rust↔Qt boundary: `AccessNode snapshot`, `AccessEvent`, `Role`, `StateBit`.

## Qt6 component mapping

- `QAccessible` / `QAccessibleInterface` — the framework; custom widgets implement
  `QAccessibleInterface` (or register a factory via `QAccessible::installFactory`).
- `QAccessibleWidget` — base for `QWidget` subclasses; custom canvas implements its own.
- `QAccessibleEvent` / `QAccessibleValueChangeEvent` / `QAccessibleAnnouncementEvent` —
  state/announcement notifications.
- `QAccessible::updateAccessibility(&event)` — emit from `setFocus` overrides and model changes
  (Qt docs show a `MyWidget::setFocus` example).
- Native widgets (`QPushButton`, `QMenu`, `QLineEdit`, …) provide names/roles/states out of the
  box; use Qt Widgets (not a bespoke-painted QML UI) for panels, dialogs and the shortcut editor
  so this metadata exists. Reserve QML for purely visual canvas overlays that are re-described
  through `CanvasSemantics`.
- `QKeySequence`, `QShortcut`, `QAction` — keyboard operation; `QAction` names are announced.
- High-DPI: `QGuiApplication::setHighDpiScaleFactorRoundingPolicy`, `QT_SCALE_FACTOR`,
  `QScreen::devicePixelRatio`; `QStyleHints::colorScheme` (Qt 6.5+) for light/dark.
- Linux bridge: Qt6 publishes `QAccessible` over **AT-SPI2** (D-Bus) for **Orca**; no external
  `qt-at-spi` plugin is needed for Qt5.4+/Qt6 (the old KDE `qt-at-spi` bridge was for Qt4).
  `AT_SPI_BUS` presence / `QAccessible::isActive()` gates activity.

## Data-model impact

- **No document/PSD/XMP changes.** Accessibility is a presentation-layer concern; it never
  serializes into files or the undo stack.
- **Accessible metadata may be stored per UI element at runtime** (names/descriptions),
  typically generated from existing labels/translations rather than a parallel store.
- **New preference fields** (`high_contrast`, `color_blind_safe`, announcement verbosity) live in
  `InterfacePrefs` and follow the preferences store/migration rules (`UI-010`).
- **Accessible name source of truth:** use the localized command/tool/panel labels so screen
  readers and menus agree; do not maintain a second string table.
- **Canvas descriptions are derived, not stored** — recomputed from document + cursor state.

## Edge cases

- **No AT client attached:** skip expensive semantics; `QAccessible::isActive()` false. Do not
  degrade normal performance.
- **Screen reader + custom canvas:** pixels are not narratable; announce structure (tool, zoom,
  selection, active layer) and never fabricate image descriptions.
- **Keyboard trap in modal/canvas:** every modal and the canvas must release focus on
  `Escape`/`Tab`; the focus graph is validated.
- **Type tool + single-letter shortcuts:** ensure typing text does not trigger tool switches
  (`UI-011`), which is also an accessibility requirement.
- **High-DPI + fractional scaling:** pixmap assets and custom-painted cursors must scale; verify
  at 125 %/150 %/200 %.
- **Color-blind-safe mode:** verify layer labels, guides/grid/slices, gamut warning, selection
  and channel overlays remain distinguishable; add shape/pattern cues.
- **Contrast of the default dark theme:** some CS6 grays fail WCAG AA; the new high-contrast
  theme must pass.
- **Localization:** accessible names/descriptions must be translated; RTL/Arabic/Hebrew type and
  UI mirroring.
- **Reduced motion:** Animated Zoom / flick panning should honor the OS "reduce motion"
  preference rather than only the app toggle.
- **Multiple monitors with different DPI:** focus ring and magnifier regions must follow the
  active screen.
- **Wayland vs. X11:** AT-SPI and key/gesture availability differ; verify Orca on both.

## Parity acceptance criteria

1. Given any command available in the CS6 default set, it is reachable by keyboard only
   (menu path or shortcut) and shown with a visible focus indicator; no command is pointer-only.
2. Given Orca running on Linux with `QAccessible::isActive()` true, focusing panels, menus,
   dialogs and the toolbox announces role + name + state; navigation via Orca works.
3. Given `Tab` pressed through any dialog, focus cycles through every control and returns, with no
   keyboard trap; `Escape` closes/cancels.
4. Given the custom canvas focused, an announcement describes active tool, zoom %, active layer,
   and selection bounds — and does not attempt to describe pixels.
5. Given a color-blind-safe mode enabled, every color-encoded state (layer labels, guides/grid,
   gamut warning, selection/channel overlays) has a non-hue cue and remains distinguishable under
   the three common color-vision deficiencies.
6. Given a high-contrast theme, all text/UI meets ≥ 4.5:1 contrast and the focus ring is visible.
7. Given 200 % high-DPI scaling, the UI, custom cursors and canvases scale without clipping, and
   accessible hit-boxes match visible controls.
8. Given an OS "reduce motion" preference, Animated Zoom and flick panning are disabled unless
   explicitly re-enabled in Preferences.
9. Given no AT client attached, enabling accessibility adds no measurable idle overhead.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` (Adobe Photoshop CS6
  Help and tutorials, Feb 2013) — primary. Establishes: CS6 has **no** assistive-technology
  chapter; the only "accessibility" content is accessible *output* (Rich Content PDF creates
  accessible PDF files containing tags, hyperlinks, bookmarks, interactive elements, and layers;
  Always Add Alt Attribute adds ALT attributes to IMG elements so that pages meet government web
  accessibility standards); keyboard-shortcut-driven operation; Interface Color Theme / UI Font
  Size; precise cursors.
- `https://www.psdvault.com/basics/photoshop-accessibility-features` — secondary, **modern
  Photoshop (2024), not CS6**; used only as a claim list to check against CS6. Its mentions of
  "Accessibility Description panel", "Voice Control", touchscreen support and "Accessibility"
  preferences are **CC-era** and must not be attributed to CS6.
- `https://community.adobe.com/t5/download-install-discussions/why-does-adobe-creative-cloud-products-have-zero-screen-reader-support/td-p/10415225`
  — Adobe Community (2019): user reports no VoiceOver support in Photoshop/InDesign; broad
  evidence of limited Adobe screen-reader support.
- `https://doc.qt.io/qt-6/accessible.html` (Qt 6.11 "Accessibility") — fetched. Establishes Qt's
  scalable UI, keyboard navigation, colors/contrast, sound, and assistive-tool support; Qt uses
  platform-specific APIs and ships metadata for ready-made widgets/controls; custom widgets can
  expose and enhance it.
- Context7 `/websites/doc_qt_io_qt-6_8` (`QAccessible`, `QAccessibleEvent`, `QAccessible::isActive`)
  — `QAccessible::isActive()` semantics; `QAccessibleEvent`/`updateAccessibility`; the
  `setFocus` notification example; Qt supports MSAA, macOS Accessibility, and Unix/X11 AT-SPI
  with custom backends.
- Search results (not directly fetched, treated as leads): Adobe Accessibility Compliance
  (`https://www.adobe.com/accessibility/compliance.html`) and the Photoshop 21.x Accessibility
  Conformance Report (Windows) — establishes that formal Photoshop ACRs exist only for much later
  versions; KDE wiki/GitHub `qt-at-spi` — historical Qt4 AT-SPI bridge.

## Open questions

- **Did CS6 expose any MSAA/NSAccessibility tree at all?** No Adobe documentation found. *Resolve:*
  test CS6 on Windows with Narrator/Inspect.exe and on macOS with VoiceOver/Accessibility
  Inspector; search archived CS6 release notes.
- **Is there a CS6-era VPAT/ACR for Photoshop?** None surfaced. *Resolve:* Adobe accessibility
  compliance archive (`adobe.com/accessibility/compliance.html`) and Section 508 procurement
  records.
- **Exact CS6 Color Theme gray values / contrast ratios.** The 4 swatch values are not published.
  *Resolve:* sample a CS6 install.
- **Which CS6 controls are keyboard-focusable on each OS** (macOS Full Keyboard Access interaction
  with panel docks, numeric fields). *Resolve:* test matrix.
- **AT-SPI behavior under Wayland** for Qt6 apps (bus activation, key forwarding, Orca). *Resolve:*
  Qt6/AT-SPI2 and desktop test matrix; confirm `QT_ACCESSIBILITY` / `AT_SPI_BUS` gating.
- **High-contrast theme source on Linux:** whether to consume the KDE/GTK high-contrast palette,
  read `kdeglobals`/GSettings, or ship our own. *Resolve:* Qt platform-theme plugin capabilities
  (QGtk3Theme / qgnomeplatform / qt6ct) on target distros.
- **Accessible description of generated pixels/AI features** (if any later scope): out of CS6
  parity; defer.
- **Whether to add color-blind simulation/preview** as a feature beyond CS6 parity. *Resolve:*
  product decision; track under non-goals.
