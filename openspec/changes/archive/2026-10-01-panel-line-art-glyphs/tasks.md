# Tasks: panel-line-art-glyphs

## 0. Scope

The glyphs are added ahead of the panels that will use them. There is no Paths
panel (the slot is still a `PlaceholderPanel`) and no search, reset, shape, or
smart-object buttons, so no shipped code requests these ids yet.

## 1. Assets

- [x] 1.1 Extract the four `layers.*` glyphs from photorust `LayerIcons.cpp`.
- [x] 1.2 Extract the seven `path.*` glyphs from photorust `PathIcons.cpp`.
- [x] 1.3 Register all eleven in `assets/pictura.qrc` (explicit list).
- [x] 1.4 Record the upstream origin in `assets/PROVENANCE.md`.

## 2. Verification

- [x] 2.1 Add `tst_icon_assets` to `PICTURA_QT_TESTS`: each new id resolves and renders at 16 px, an unknown id is null.
- [x] 2.2 `openspec validate --all --strict`, `cmake --build build --parallel`, `ctest --test-dir build -R '^tst_'`.
