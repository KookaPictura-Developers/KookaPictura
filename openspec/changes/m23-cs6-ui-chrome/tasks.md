## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m23-cs6-ui-chrome.md` brief referencing the stored screenshot
- [x] 1.2 Freeze `Theme::styleSheet(level)`, the toolbox grid/colour-control interface, and the default dock groups in `design.md`
- [x] 1.3 Commit brief, proposal, and the reference screenshot with a `TASK-ALLOWS-DOCS` message

## 2. Wave 1: CS6 stylesheet (sub-agent, C++)

- [ ] 2.1 `theme.{h,cpp}`: build a CS6-style QSS string per brightness ramp (menu bar, options bar, dock tabs/title bars, tool buttons, status bar, scrollbars, menus, tooltips)
- [ ] 2.2 `Theme::apply` sets the palette then the stylesheet; expose `Theme::styleSheet(level)` for the self-test
- [ ] 2.3 Verified: build green; `Shift+F1`/`Shift+F2` restyle; M16–M22 self-tests still exit 0

## 3. Wave 2: CS6 toolbox (sub-agent, C++)

- [ ] 3.1 `toolbox.{h,cpp}`: two-column `QToolButton` grid, icon-only with tooltips, in CS6 slot order; keep the existing action shortcuts and `activeToolChanged` wiring
- [ ] 3.2 `ForegroundBackgroundWidget`: overlapping fg/bg swatches, active-swatch click, a reset-to-default affordance, and a screen-mode button
- [ ] 3.3 `frame.cpp`: bind the control to `ColorState` (shared with the Color panel) and the screen-mode button to the screen-mode cycle
- [ ] 3.4 Verified: tool switching, cursor, shortcuts, and `Tab` hide-all unchanged; build green

## 4. Wave 3: default dock grouping and canvas (sub-agent, C++)

- [ ] 4.1 `frame.cpp` `buildPanels`: `tabifyDockWidget` Color+Swatches, Layers+History, Navigator+Info+Histogram; raise the first tab of each group; keep stable `objectName`s
- [ ] 4.2 Canvas/tab-strip styling: dark canvas colour from the ramp, styled `QTabBar` for document tabs
- [ ] 4.3 Verified: grouped docks exist, `Tab`/`Shift+Tab` still work, session layout round-trips

## 5. Wave 4: self-test (sub-agent)

- [ ] 5.1 Extend `--self-test` (exit codes from 59): stylesheet non-empty and level-dependent; toolbox grid button count and fg/bg widget present; default dock groups share a tabbed dock
- [ ] 5.2 `xvfb-run` self-tests exit 0 (fixture and no-argument)

## 6. Close-out

- [ ] 6.1 `cargo fmt/clippy/test`; `openspec validate --all --strict`; `guard.sh`
- [ ] 6.2 Update `docs/dev/STATE.md` with a `TASK-ALLOWS-DOCS` message
- [ ] 6.3 Archive the change and commit
