## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m38-icon-cursor-library.md`: the 71-row catalogue
  (id, label, shortcut, group/slot, implemented, `Qt::CursorShape`, hotspot), the
  23-slot order, the four hotspot rules, the icon and cursor style guides, the
  toolbox flyout design, the panel-icon inventory, the verification plan, and the
  source conflicts
- [x] 1.2 Add the numbering note to `docs/dev/layers-panel-program.md` (this
  user-requested icon/cursor milestone takes M38; panel anatomy → M39,
  filtering → M40, management → M41, styles → M42, smart objects → M43) and to
  `docs/dev/STATE.md`
- [x] 1.3 Write `proposal.md`, `design.md`, this `tasks.md`, and the
  `specs/{application-shell,tool-framework,panel-rail,layers-panel,svg-cursors}`
  deltas
- [x] 1.4 Freeze in `design.md`: the catalogue table and `ToolCatalogueEntry`;
  the asset-id rule and the 10 frozen names; the single-column flyout slot model
  and last-used rule; the disabled-unimplemented tooltip string; the
  catalogue-driven hotspot lookup; the icon/cursor style guides; the
  `window.panels.<name>` / `layers.<action>` / `history.snapshot` ids; the
  `gen_qrc.sh` approach; the `m38_icons` self-test contract

## 2. Assets: Move (group 1)

- [x] 2.1 Reuse `assets/icons/tool.move.svg` unchanged; compound
  `assets/cursors/tool.move.svg` from an arrowhead (top-left) and the move cross
  (bottom-right), hotspot at the arrowhead tip (2,2)

## 3. Assets: Marquee (group 2)

- [x] 3.1 Reuse `tool.marquee` icon/cursor (Rectangular); draw
  `tool.ellipticalmarquee` icon and cursor (hotspot (12,12))

## 4. Assets: Lasso (group 3)

- [x] 4.1 Reuse `tool.lasso`; draw `tool.polygonallasso` and
  `tool.magneticlasso` icons and cursors (hotspot (12,12))

## 5. Assets: Selection (group 4)

- [x] 5.1 Reuse `tool.quickselection`; draw `tool.magicwand` icon and cursor
  (hotspot (12,12))

## 6. Assets: Crop / Slice (group 5)

- [x] 6.1 Reuse `tool.crop`; draw `tool.perspectivecrop`, `tool.slice`,
  `tool.sliceselect` icons and cursors (hotspot (12,12))

## 7. Assets: Sampling / Measure (group 6)

- [x] 7.1 Reuse `tool.eyedropper`; draw `tool.colorsampler`, `tool.ruler`,
  `tool.note` icons and cursors at (2,22) and `tool.count` at (12,12)

## 8. Assets: Retouch (group 7)

- [x] 8.1 Draw `tool.spothealingbrush` and `tool.healingbrush` icons/cursors at
  (2,22); `tool.patch`, `tool.contentawaremove`, `tool.redeye` at (12,12)

## 9. Assets: Paint (group 8)

- [x] 9.1 Reuse `tool.brush` and `tool.pencil`; draw `tool.colorreplacement`
  and `tool.mixerbrush` icons/cursors at (2,22)

## 10. Assets: Clone (group 9)

- [x] 10.1 Draw `tool.clonestamp` and `tool.patternstamp` icons/cursors at
  (2,22)

## 11. Assets: History (group 10)

- [x] 11.1 Draw `tool.historybrush` and `tool.arthistorybrush` icons/cursors at
  (2,22)

## 12. Assets: Erase (group 11)

- [x] 12.1 Draw `tool.eraser`, `tool.backgrounderaser`, `tool.magiceraser`
  icons/cursors at (2,22)

## 13. Assets: Fill (group 12)

- [x] 13.1 Draw `tool.gradient` and `tool.paintbucket` icons/cursors at (2,22)

## 14. Assets: Blur (group 13)

- [x] 14.1 Draw `tool.blur`, `tool.sharpen`, `tool.smudge` icons/cursors at
  (2,22) (no shortcut key)

## 15. Assets: Toning (group 14)

- [x] 15.1 Draw `tool.dodge`, `tool.burn`, `tool.sponge` icons/cursors at
  (2,22)

## 16. Assets: Pen (group 15)

- [x] 16.1 Draw `tool.pen`, `tool.freeformpen`, `tool.addanchorpoint`,
  `tool.deleteanchorpoint`, `tool.convertpoint` icons/cursors at (2,2)

## 17. Assets: Type (group 16)

- [x] 17.1 Draw `tool.horizontaltype`, `tool.verticaltype`,
  `tool.horizontaltypemask`, `tool.verticaltypemask` icons/cursors at (12,12)
  (I-beam art)

## 18. Assets: Path Select (group 17)

- [x] 18.1 Draw `tool.pathselection` and `tool.directselection` icons/cursors
  at (12,12)

## 19. Assets: Shape (group 18)

- [x] 19.1 Draw the six shape icons/cursors (rectangle, roundedrectangle,
  ellipse, polygon, line, customshape) at (12,12)

## 20. Assets: 3D Object (group 19, Extended)

- [x] 20.1 Draw the five 3D-object icons/cursors at (12,12)

## 21. Assets: 3D Camera (group 20, Extended)

- [x] 21.1 Draw the five 3D-camera icons/cursors at (12,12)

## 22. Assets: Hand, Rotate View, Zoom (groups 21–23)

- [x] 22.1 Reuse `tool.hand` and `tool.zoom`; draw `tool.rotateview` icon/cursor;
  draw the hand and zoom cursors with the action point at (9,2) (redraw the
  existing two cursors only if they do not already read at that hotspot)

## 23. Assets: panel icons

- [x] 23.1 Reuse `window.panels.layers` and `window.panels.tools`; draw the
  remaining hosted panel icons (`history`, `actions`, `info`, `navigator`,
  `histogram`, `color`, `swatches`, `gradients`, `patterns`, `properties`,
  `adjustments`, `libraries`, `channels`, `paths`)
- [x] 23.2 Draw the reserved panel icons (`brushes`, `toolPresets`,
  `cloneSource`, `measurementLog`, `notes`, `character`, `paragraph`,
  `typeStyles`, `smartObjects`, `masks`, `3d`, `timeline`)

## 24. Assets: Layers strip and History snapshot

- [x] 24.1 Draw `layers.link`, `layers.fx`, `layers.mask`,
  `layers.fillAdjustment`, `layers.group`, `layers.newLayer`, `layers.delete`
- [x] 24.2 Draw `history.snapshot`

## 25. Sequential integration

- [x] 25.1 Publish the frozen catalogue in `tools.{h,cpp}`: expand `ToolInfo`
  to all 71 rows (name, label, shortcut, `Qt::CursorShape`, hint, group 1..23,
  implemented, hotspotX/Y), with `allToolIds()` (71), `implementedToolIds()`
  (10), `toolImplemented()`, and `static_assert(kToolCount == 71)`. The separate
  `ToolCatalogueEntry`/`allCatalogueEntries()` layer from the design was
  collapsed into the existing table — `ToolInfo` already carried every field
- [x] 25.2 Regenerate `assets/pictura.qrc` by hand from the sorted union of
  `assets/icons/*.svg` and `assets/cursors/*.svg` (208 entries; on-disk set ==
  listed set). The `scripts/gen_qrc.sh` helper was not added — the checked-in
  qrc is small enough to keep by hand and the `m38_icons` self-test fails on any
  missing or mismatched entry
- [x] 25.3 Replace the hardcoded hotspot in `icons.cpp` with the catalogue
  lookup (default (12,12))
- [x] 25.4 Rebuild `toolbox.cpp` as the single-column flyout grid: 23 slot
  buttons, corner triangles, `MenuButtonPopup`/`DelayedPopup` flyouts, last-used
  visible member, enabled-member `Alt`/`Shift` cycling, disabled unimplemented
  tools with the exact `<label> — not implemented yet` tooltip, fg/bg widget and
  Screen-Mode button below
- [x] 25.5 Wire panel icons: rail buttons use `icon(commandId)` and
  `Qt::ToolButtonIconOnly`; `layers_panel.cpp` sets the seven CS6-order
  strip-button icons (link/fx/mask disabled, the rest wired);
  `history_panel.cpp` sets `history.snapshot`. Dock-tab window icons were
  **not** wired — the `registerPanel` window-icon step was dropped, so tab
  strips keep their text titles and `m38_panels` covers rail + strip + snapshot
  only
- [x] 25.6 Add the `m38_icons` block to `main.cpp` per §8 of the contract, with
  fresh exit codes and the summary line

## 26. Verification and evidence

- [x] 26.1 `cmake -S . -B build && cmake --build build`
- [x] 26.2 `xvfb-run -a ./build/pictura --self-test` and the `two_layers.psd`
  variant, both exit 0 with no FAILs, printing
  `m38_tools icons=1 cursors=1 slots=1 guard=1` (exit codes 95–98) and
  `m38_panels rail=1 strip=1 history=1` (codes 99–101). The catalogue counts
  (71 tools / 10 implemented / 61 disabled / 23 slots) are enforced by the
  `static_assert` and the loop over `allToolIds()`, so the summary line reports
  booleans rather than raw counts
- [x] 26.3 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --
  -D warnings`; `cargo test --workspace` (unchanged — no Rust change)
- [x] 26.4 `openspec validate m38-icon-cursor-library --strict`;
  `openspec validate --all --strict`; `git status --porcelain` shows only the
  intended files

## 27. Close-out

- [x] 27.1 Record the M38 result in `docs/dev/STATE.md`
- [ ] 27.2 Archive the change (`openspec archive m38-icon-cursor-library`) and
  commit — deferred; this task explicitly does not commit
