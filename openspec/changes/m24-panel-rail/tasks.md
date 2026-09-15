## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m24-panel-rail.md` brief
- [x] 1.2 Freeze the `PlaceholderPanel`/`PanelRail` interfaces, the panel objectNames, and the command ids in `design.md`
- [x] 1.3 Commit brief + proposal with a `TASK-ALLOWS-DOCS` message

## 2. Wave 1: placeholder panels and the rail widget (sub-agent, C++)

- [ ] 2.1 `panels/placeholder_panel.{h,cpp}` — `PlaceholderPanel(title, message, parent)` dock with a centred empty-state label
- [ ] 2.2 `panels/panel_rail.{h,cpp}` — `PanelRail : QToolBar` with `addPanel(QString commandId, QIcon, QString tooltip)`; vertical, icon-only, checkable
- [ ] 2.3 `CMakeLists.txt`: add the two new sources
- [ ] 2.4 Syntax-check the new `.cpp` files

## 3. Wave 2: frame grouping, rail, and commands (sub-agent)

- [ ] 3.1 `commands.h`/`command_tree.cpp`: add ids for Gradients, Patterns, Properties, Adjustments, Libraries, Channels, Paths, Actions and mark their `Window > Panels` entries implemented, checkable
- [ ] 3.2 `frame.{h,cpp}`: create the eight placeholder panels with objectNames `gradientsPanel`, `patternsPanel`, `propertiesPanel`, `adjustmentsPanel`, `librariesPanel`, `channelsPanel`, `pathsPanel`, `actionsPanel`; register on the right
- [ ] 3.3 Group: Color+Swatches+Gradients+Patterns; Properties+Adjustments+Libraries; Layers+Channels+Paths (tabify + raise the first)
- [ ] 3.4 Create the `PanelRail` on the right with entries for History, Actions, Info, Navigator, Histogram; each entry dispatches its `Window > Panels` command id; sync checked state from dock visibility
- [ ] 3.5 `registerHandlers()`: add handlers + checked providers for the eight new panel commands, mirroring the existing `WindowPanels*` blocks
- [ ] 3.6 Build green; M16–M23 self-tests still exit 0

## 4. Wave 3: self-test (sub-agent)

- [ ] 4.1 Extend `--self-test` (exit codes from 62): the eight panels exist; the three groups are tabified; the rail has the five buttons; a rail/command toggle shows the panel
- [ ] 4.2 `xvfb-run` self-tests exit 0 (fixture and no-argument)

## 5. Close-out

- [ ] 5.1 `cargo fmt/clippy/test`; `openspec validate --all --strict`; `guard.sh`
- [ ] 5.2 Update `docs/dev/STATE.md` with a `TASK-ALLOWS-DOCS` message
- [ ] 5.3 Archive the change and commit
