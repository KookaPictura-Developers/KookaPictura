# Proposal

## Why

The app's 176 hand-authored SVG icons hard-code `#c8c8c8` (stroke 1.5) and
ignore the four-level theme, so they neither follow the brightness setting nor
carry any license/provenance record. Adopting Lucide as the base gives a
consistent, permissively licensed (ISC), theme-aware set with an auditable
rename map.

## What Changes

- **Icon bodies**: replace every `assets/icons/<id>.svg` with either a
  Lucide-derived asset (vendored verbatim from a pinned `lucide-static`
  release) or a Lucide-style custom asset (`viewBox 0 0 24 24`, `fill="none"`,
  `stroke="currentColor"`, `stroke-width=2`, round caps/joins; internal shading
  via alpha gradients). Asset ids and `assets/pictura.qrc` are unchanged.
- **Runtime tinting** (**BREAKING** for the icon API): `icon(id)` now renders
  each SVG through a new `QIconEngine` that recolours by alpha with the active
  palette foreground (and the disabled colour for disabled mode), at the
  requested size and device pixel ratio. Add `icon(id, const QColor&)` for an
  explicit colour. The `app` PNG keeps the existing file-icon path.
- **Theme-follow**: a brightness change re-renders icons (cache-bust via the
  engine key plus `QPixmapCache` clear in `Theme::apply`).
- **Licensing**: ship `LICENSES/Lucide.txt` (ISC plus the Feather-derived MIT
  block), add an icon-assets section to `NOTICE.md` and to the generated
  `THIRD-PARTY-LICENSES`, and record the vendored release version/commit.
- **Provenance**: `docs/dev/icon-provenance.md` plus machine-readable
  `assets/icons/lucide-map.json`, a `scripts/sync-lucide-icons.py` vendoring
  tool, and a guard that fails when an icon file is not accounted for.
- **Out of scope**: the 77 cursor SVGs (`assets/cursors/`) move in a follow-up
  issue; the 3D camera/object tool glyphs stay as-is (non-scope, but still
  present so the frozen tool catalogue resolves).

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `ui/icon-assets`: the icon source (original -> Lucide-derived/Lucide-style),
  render-time tinting and theme/disabled behaviour, and new
  provenance/licensing requirements.

## Impact

- C++: `crates/pictura-app/cpp/icons.{h,cpp}`, new
  `crates/pictura-app/cpp/svg_icon_engine.{h,cpp}`, `theme.cpp`,
  `CMakeLists.txt`, `crates/pictura-app/cpp/tests/tst_icon_assets.cpp`.
- Assets: all 176 `assets/icons/*.svg`; `assets/pictura.qrc` unchanged (ids
  stable). Cursors untouched.
- Legal/docs: `LICENSES/Lucide.txt`, `NOTICE.md`,
  `scripts/third-party-licenses.py`, `docs/dev/icon-provenance.md`,
  `docs/dev/licensing-compliance-notes.md`,
  `docs/00-overview/licensing-and-provenance.md`.
- Tooling: `scripts/sync-lucide-icons.py`, `assets/icons/lucide-map.json`,
  provenance guard in `scripts/`.
- No new build dependency. No new Rust crate.
