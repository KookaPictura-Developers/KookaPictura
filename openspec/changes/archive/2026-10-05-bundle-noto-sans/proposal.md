# Proposal: bundle-noto-sans

## Why

The UI asks for a heavier-than-regular weight on the file-bar and panel-group
tab labels (`QTabBar#documentTabBar::tab` / `QTabBar#panelTabBar::tab`,
`font-weight: 700`) and the programmatic `QFont::Bold` on the panel tab bar —
but no font family is ever pinned. Qt resolves the application default to a
platform family that has no intermediate weights on this system (DejaVu Sans:
Book/Bold only), so a weight request snaps to an unintended face and the
requested bold never renders reliably. The family also differs across OSes, so
the chrome is not consistent. Bundling Noto Sans — which ships real
400/500/600/700 upright and italic faces — and pinning it as the application UI
font makes the weighted chrome resolve to the requested face everywhere.

## What Changes

- Vendor the Noto Sans static faces (`Regular`, `Medium`, `SemiBold`, `Bold` and
  their italics) into `assets/fonts/` and embed them through the existing
  `assets/pictura.qrc` resource.
- Register the faces with the Qt font database after the `QApplication` is
  constructed and install Noto Sans at a pinned 12 px as the application UI
  font, on every OS. The chrome derivations that run two pixels under the app
  font (tab labels, status footer) keep working from the pinned size.
- Move the tab-label chrome to the bundled **Bold (700)** face. It is applied to
  the bar itself for both the document (`frame.cpp`) and panel
  (`panels/panel_group.cpp`) tab bars through the shared `applyTabBarFont()`
  helper, so the weight and QTabBar's elision metrics are explicit and identical;
  the QSS `font-weight: 700` rules mirror it.
- Ship the SIL Open Font License 1.1 text and record the vendored version in the
  asset provenance and the generated third-party notices.

## Capabilities

### New Capabilities

- `ui/bundled-fonts`: the embedded application font, its registration, the
  pinned application-font contract, and the requirement that weighted chrome
  resolves to a bundled face rather than a system fallback.

### Modified Capabilities

(none — the weighted-chrome request itself is unchanged; only the font that
answers it becomes a bundled, guaranteed face and the tab weight is raised to
SemiBold)
