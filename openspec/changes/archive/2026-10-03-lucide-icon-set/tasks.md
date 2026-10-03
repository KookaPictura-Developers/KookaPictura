# Tasks

## 1. Mapping and provenance foundation

- [x] 1.1 Write `assets/icons/lucide-map.json` with one entry per current icon
  id (`kind: lucide|custom|note|out-of-scope`, plus `slug`/`match`/`note`);
  apply the reviewer's corrections (verified all slugs against `lucide-static`
  1.50.0) and fix the id typos (`layers.clipMask`, `layers.fillAdjustment`,
  `info.bounds`, `info.crosshair`, `window.panels.gradients`). Verify the file
  parses and its key set equals `assets/icons/*.svg` exactly.
- [x] 1.2 Generate `docs/dev/icon-provenance.md` from the map (upstream URL,
  pinned version + commit, license pointer, and an id/slug/match/notes table).
  Verify the doc lists every id and names the pinned release.
- [x] 1.3 Add `scripts/check-icon-provenance.sh` asserting the icon file set and
  the map keys are identical and that `LICENSES/Lucide.txt` exists; wire it into
  `scripts/verify-fast.sh`. Verify it exits 0 on the tree and non-zero when a
  dummy SVG is added.

## 2. Licensing artifacts

- [x] 2.1 Add `LICENSES/Lucide.txt` with the ISC text plus the Feather-derived
  MIT block, verbatim. Verify the file names both licenses.
- [x] 2.2 Add an "Icon assets" section to `NOTICE.md` naming Lucide (ISC),
  the Feather subset (MIT), the pinned release, and the provenance doc.
- [x] 2.3 Extend `scripts/third-party-licenses.py` to append an icon-assets
  section (so the generated file is not Cargo-only); regenerate
  `THIRD-PARTY-LICENSES`. Verify the regenerated file mentions Lucide.

## 3. Tinted icon loader

- [x] 3.1 Add `crates/pictura-app/cpp/svg_icon_engine.{h,cpp}`: a
  `QIconEngine` that renders `:/icons/<id>.svg` at size×DPR and recolours via
  `SourceIn` (explicit colour, else palette; disabled uses the disabled
  colour); `key()` includes the theme generation. Verify it compiles.
- [x] 3.2 Change `icons.{h,cpp}`: `icon(id)` returns an engine-backed `QIcon`
  for SVG ids and the existing `QIcon(path)` for PNG; add
  `icon(id, const QColor&)`; unknown ids stay null.
- [x] 3.3 Add a theme-generation getter/bump in `theme.{h,cpp}` and clear
  `QPixmapCache` in `Theme::apply`.
- [x] 3.4 Register the new files in `CMakeLists.txt` (explicit, no globbing) and
  verify the app target builds.
- [x] 3.5 Extend `tst_icon_assets.cpp`: explicit-colour pixmap sampled equals
  the requested colour; `Disabled` differs from `Normal`; 2× DPR pixmap is
  non-null; the `lia_theme_change` scenario (brightness change updates a
  pixmap). Verify `ctest -R tst_icon_assets` passes.

## 4. Lucide-derived artwork

- [x] 4.1 Add `scripts/sync-lucide-icons.py` (stdlib only): reads the map,
  copies the pinned release's SVG for each `lucide` entry to
  `assets/icons/<id>.svg`, fails on a missing slug. Verify it runs and reports
  the copied/skipped counts.
- [x] 4.2 Run the sync script for the `lucide` entries. Verify every copied
  asset declares `viewBox="0 0 24 24"`, `fill="none"`, `stroke="currentColor"`,
  `stroke-width="2"`, and `assets/pictura.qrc` is unchanged.

## 5. Custom and variant artwork

- [x] 5.1 Author the `custom` SVG bodies in Lucide style, including the
  collision variants (distinguishing marks so shared glyphs stay distinct) and
  the alpha-gradient/shaded icons (`tool.gradient`, `window.panels.gradients`).
  Verify each renders at 16 px and passes the attribute check.
- [x] 5.2 Record `note`/`out-of-scope` entries (3D camera/object tools,
  `layers.kindGroup`, `layers.lockNesting`, the `layers.kindBackground`
  mis-spec) in the provenance doc with a short follow-up note; leave their
  files unchanged.

## 6. Documentation

- [x] 6.1 Update `docs/dev/licensing-compliance-notes.md` section 8 to record
  that the icon assets are Lucide-derived/custom, licensed, and attributed.
- [x] 6.2 Add a third-party icon-set row to the asset policy table in
  `docs/00-overview/licensing-and-provenance.md`.

## 7. Integration verification

- [x] 7.1 `cmake --build build`, `./build/pictura --headless --self-test`, and
  `ctest --test-dir build -R '^tst_'` all pass.
- [x] 7.2 `bash scripts/verify-fast.sh` passes and
  `openspec validate --all --strict` is green.
