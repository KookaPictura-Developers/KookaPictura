# M19 — SVG icon set and cursors

Goal: give the shell original imagery. An original (independent-creation) SVG icon set for
the app, the tools, and every implemented command; an SVG cursor per tool; a Qt
resource bundle and a small loader; and icons used by the window, Tools panel,
and menu actions, with the active tool's cursor. OpenSpec change `m19-svg-icons`
(new capabilities `icon-assets`, `svg-cursors`).

## Scope

- `assets/icons/<id>.svg` for `app`, `tool.*` (8), and every implemented command
  id (File, Edit, Image, Select, View, Window, Help) — 40 icons.
- `assets/cursors/tool.*.svg` — 8 cursors with defined hotspots.
- `assets/pictura.qrc` bundle; `icons.{h,cpp}` with `icon(id)`/`cursor(id)`.
- Build: `Qt6::Svg`, `CMAKE_AUTORCC`, the `.qrc`.
- Wiring: application icon, Tools-panel action icons, menu-action icons, active
  tool cursor.
- Self-test: every icon and cursor id resolves; the window icon is set.

## Out of scope (later milestones)

- Icons for unimplemented tools or the full menu tree (dead assets).
- Recolourable/themed icons; multi-resolution or animated cursors.

## Process

Orchestrator: brief, OpenSpec artifacts, dispatch, integration, verification,
archive, commit. Sub-agents own all code and assets: three asset agents (by
icon family), then one code agent (resource, loader, CMake, wiring) and one
self-test agent. All artwork is original; no Adobe assets.

## Verification

- `cmake -S . -B build && cmake --build build`
- fixture and no-argument self-tests exit 0 with new icon/cursor checks (exit
  codes from 47)
- `cargo fmt/clippy/test`; `openspec validate --all --strict`; `guard.sh`
