# Tasks

## 1. Theme metrics foundation

- [ ] 1.1 Add `Theme::kChromeBorderWidth = 2`, a `kShadeStep`/`shade(color, ±1)` helper, and the tool-slot base metric constants next to `kPanelBorderWidth`/`kGroupDividerWidth` in `crates/pictura-app/cpp/theme.h`; verify the build and that `qt-test-results` for `tst_smoke` still passes.
- [ ] 1.2 Replace the chrome separator literals with `Theme::kChromeBorderWidth`: menu bar bottom (`theme.cpp:148`), options bar bottom (`:161`), status bar top (`:230`), icon-group grip (`:192`), `QMenu::separator`, and `QToolBar::separator`; draw the widget-column header/bottom rules at 3px from `ui/panel-column`'s shared constant, and leave the Layers filter bar (`:261`), Info grid cross (`:220-222`), and Layers eye-gutter at 1px. Verify with the existing Qt Test chrome checks.
- [ ] 1.3 Add the global idle-less button rule to `styleSheetFor` (`theme.cpp:118+`, scoped to the panel family alongside the existing `QToolButton` rule at `:163-167`): idle has no outline/background, hover is a slightly darker bg with a lighter outline, pressed is one `kShadeStep` darker than hover with the same outline; verify `tst_layers_panel` shows Layers action icons flat at rest.
- [ ] 1.4 Route panel/bar/widget/group backgrounds through `shade(window, +1)` and the workspace through `shade(window, -1)` in the theme tokens, resolving the workspace at the current brightness level and re-applying it on every brightness change, dropping hard-coded shade literals; verify a theme test asserts each surface differs from `window` by one step at every brightness level.
- [ ] 1.5 Add a small Qt Test (in an existing suite) asserting the new constants and the idle/hover/pressed relationship, and run it under `ctest --test-dir build -R '^tst_'`.

## 2. DPI-aware tool slots

- [ ] 2.1 Replace `toolbox.cpp:38` `kSlotButtonSize` and its uses (`:372`, `:427`, `contentWidth :465-474`) with the base 36×28 footprint and 24×20 icon cap scaled by `screen()->logicalDotsPerInch() / 96.0`, deriving the unoutlined idle body as the footprint minus twice the 1px border rather than a separate 34×26 constant; verify one- and two-column widths still hug the grid (`contentWidthForTest`).
- [ ] 2.2 Scale the slot icon size from the same metrics, capped at the scaled 24×20 maximum; verify an icon is never larger than the cap at 96 DPI.
- [ ] 2.3 Re-apply the metrics on `screenChanged` and on float/column-host change; verify a simulated DPI change re-lays out without clipping.
- [ ] 2.4 Add a Qt Test asserting base metrics at 96 DPI and scaled metrics on a high-DPI screen, and that Qt's own `devicePixelRatio` is not applied a second time.

## 3. Toolbox interactions and catalogue

- [ ] 3.1 Keep the Object (3D) and Camera entries in `tool_catalog.cpp` (enum/table unchanged) but stop the toolbox presenting their slots (`tool_catalog.cpp:165-184`); verify the panel presents 21 slots and update the presented-slot count assertion in `selftest_shell_round4.cpp`.
- [ ] 3.2 Change `toolbox.cpp::showSlotMenu` (`:530-552`) to anchor the flyout at the button's right edge, flipping left when it would overflow, and clamping only vertically; verify the flyout never covers the button.
- [ ] 3.3 Make a press on another slot close an open flyout and activate (or open that slot's menu) in the same gesture in `ToolSlotButton::mousePressEvent/ReleaseEvent` (`toolbox.cpp:91-124`); verify one click switches tools while a menu is open.
- [ ] 3.4 Give flyout items a left icon margin and a tighter icon/label gap (flyout `QMenu` construction in `toolbox.cpp`); verify in `tst_*` that the item metrics match.
- [ ] 3.5 Redraw the foreground/background swap control (`ForegroundBackgroundWidget::swapRect :219-223`, paint `:283-293`) as top-left and down-left arrows; verify a paint/geometry test still finds the swap hit target.
- [ ] 3.6 Add the Paint Mask Mode toggle button immediately left of the screen-mode button (`toolbox.cpp:425-439`), toggling only its checked state; verify no Quick Mask session or overlay is created.
- [ ] 3.7 Wire the screen-mode button (`frame_build.cpp:267`) to an InstantPopup menu of the three `ScreenMode` values, checked from the existing `ViewScreenMode*` commands and handled by `frame_menus.cpp:1020-1032`, leaving `F`/`Shift+F` (`:1039-1041`) on the same handler; verify the menu and the key cycle agree.
- [ ] 3.8 Add or extend the toolbox Qt Test covering the side flyout, one-click switch, swap icon, Paint Mask stub, and screen-mode menu.

## 4. Options bar compactness

- [ ] 4.1 Make the brush-settings button idle-less and shrink the options-bar body so it reserves no idle-button vertical space (`options_bar_paint.cpp:87-108`); verify the bar height matches its controls.
- [ ] 4.2 Add a Qt Test asserting the compact options-bar height and the idle-less brush button.

## 5. Panel column chrome

- [ ] 5.1 Remove the icon from normal-mode tabs while keeping it for the iconic strip (`panels/panel_group.cpp` `addTab :161`, insert `:375`; iconic reader `:564`); verify `panel_group_test.cpp:60 titleIconForTest` and the tab text.
- [ ] 5.2 Make the group header background equal the inactive-tab background and remove the lighter horizontal line (`panel_group.cpp` headerCorner `:96-121`, headerBand); verify the header paints one surface behind the corner button.
- [ ] 5.3 Draw the widget column's header line and last bottom line at 3px from a shared constant; verify both measured widths.
- [ ] 5.4 Add the Lucide `panel.menu` SVG under `assets/icons/`, register it in `assets/pictura.qrc`, add its `lucide-map.json` entry and `docs/dev/icon-provenance.md` row, and verify the provenance guard passes.
- [ ] 5.5 Replace the `\u25BE` corner glyph (`panel_group.cpp:112-119`) with `icon("panel.menu")`, vertically centred with a right margin; verify a `tst_*` finds the corner button and its glyph.
- [ ] 5.6 Extend the panel-group Qt Test to cover iconless tabs, header shade/line, 3px rules, and the corner glyph.

## 6. Docking self-anchor fix

- [ ] 6.1 Allow the source column to anchor for panel/group drags at `panels/panel_column_drag.cpp:259` (keep the whole-column `exclude` skip at `frame_columns.cpp:200/214`), and refuse a drop onto the side the item already occupies; verify both left→right and right→left docks land.
- [ ] 6.2 Add a Qt Test (or extend `tst_layers_panel`) that docks a widget to the opposite side of its own column and asserts the resulting layout.

## 7. Layers panel rows

- [ ] 7.1 Paint each row from `shade(listBase, +1)` and split paint/hit-testing into a non-selectable visibility column plus a content column with 2px left padding (`panels/layers_panel_internal.h` `paint :809-835`, `kEyeColumn :663`, `contentLeft :687-695`); verify a click in the gutter toggles visibility without selecting.
- [ ] 7.2 Paint the lighter grey selection directly in the row delegate and remove the scoped `QWidget#layersPanel QTreeView` selection rule, keeping the eye-gutter separator at 1px; verify the selected row colour comes from the delegate and the separator width.
- [ ] 7.3 Extend `tst_layers_panel` to cover the lighter row, gutter non-selection, grey selection, and 2px content padding.

## 8. Info panel menu geometry

- [ ] 8.1 Use `menu->sizeHint()` instead of `menu->size()` in `panels/info_panel.cpp:264`; verify the first open is full-size.
- [ ] 8.2 Extend `tests/tst_info_panel.cpp:238 menuOpensBesideItsButton` to assert the first click opens the menu at its final size.

## 9. Status bar, footer, and hint bar

- [ ] 9.1 Show no readouts or tool hint when no document is open and restore them when one opens (`frame.cpp:1093-1146`, `frame_build.cpp:414-447`); verify the empty and populated states.
- [ ] 9.2 Remove the right-side size-grip corner triangle and add a small left padding to the status bar; verify the grip is absent.
- [ ] 9.3 Make the status magnification and any zoom control show the same value and format; verify they agree across zooms.
- [ ] 9.4 Render the Move tool's nudge hint as four separate up/down/left/right chevrons instead of the single `Arrows` keycap (`tool_catalog.cpp:329-332`, `tool_hint_bar.cpp:51-97`); verify four chevrons and the pressed-key highlight still works.
- [ ] 9.5 Add or extend a Qt Test covering the empty status bar, missing grip, zoom consistency, and the four-chevron arrows hint.

## 10. Numeric field alignment

- [ ] 10.1 Left-align numeric field text (`panels/numeric_field.cpp:61`) and check the dialog fields (`filter_preview_dialog.cpp:421,459`); verify values start at the left edge.
- [ ] 10.2 Add a Qt Test asserting left alignment for a layers field and an options-bar field.

## 11. Integration verification

- [ ] 11.1 Update the hand-rolled self-test assertions for the changed catalogue/chrome (`selftest.cpp:2404`, `selftest_shell_round4.cpp:553-561`); take no retired codes, keep the self-test lower-only, and confirm `scripts/check-selftest-budget.sh` passes.
- [ ] 11.2 Run `bash scripts/verify-fast.sh` and confirm fmt, clippy, the unified report, file-size, guard, and openspec all pass.
- [ ] 11.3 Run `bash scripts/verify-full.sh` (CMake build first) and confirm the app self-test and `ctest -R '^tst_'` suites pass.

## 12. CS6 palette and workspace layout refinements

- [x] 12.1 Replace the derived shade tokens in `theme.cpp`/`theme.h` with the explicit palette: panel `#4d4d4d`, in-body separator and tables `#404040`, headers and file bar `#363636`, outer border and active button `#2e2e2e`, workspace `#1f1f1f`, inputs and text buttons `#3b3b3b`, outlines `#595959`, pressed text button `#303030`; split `base` into table/input/header tokens so menus/headers/tables/inputs are each right; verify `Theme::styleSheet(1)` contains each value and `tst_smoke` passes.
- [x] 12.2 Apply the palette at the default brightness level exactly, and shift every colour by one uniform `Theme::shade` step per level for the other three levels; verify a test walks all four levels.
- [x] 12.3 Reduce the idle-less button/chrome tokens to the palette (`iconHoverBorder`/text outline `#595959`, `iconPressed` `#2e2e2e`, text button `#3b3b3b`/`#303030`); verify `tst_layers_panel`/`tst_command_tree`.
- [x] 12.4 Draw the 3 px central-band frame (2 px panel + 1 px border each side, 1 px inside) below the menu bar and above the status bar without clipping any column, canvas, or options-bar content; verify with a screenshot probe and a Qt Test on the reserved margins.
- [x] 12.5 Remove the 4 px resize seam beside the Tools column while keeping it movable: set the centre-splitter handle width to 0 and give non-tools widget columns a `panelResizeGrip` on the workspace-facing edge that drives `begin/update/endWidgetColumnResize`; keep the widget columns width-draggable and self-test 168/169/193 passing.
- [x] 12.6 In the iconic strip, remove the grip-to-icons line and the 2 px inter-group divider, and add a 1 px group bottom line; verify by screenshot and a `panel_column_test` geometry check.
- [x] 12.7 Make an iconic column resizable from its workspace-facing edge on both left and right docks, persisting the width; add a Qt Test that drags/programs both sides.
- [x] 12.8 Update the tests that assumed shade-derived surfaces (`tst_smoke`, `panel_column_test`, `selftest.cpp` theme checks, `tst_command_tree`) to the explicit palette.
- [x] 12.9 Run `bash scripts/verify-fast.sh` and `bash scripts/verify-full.sh`, and confirm `openspec validate --all --strict` passes.

## 13. Chrome, toolbar, sliders, and layers refinements

- [x] 13.1 Footer: drop the status-bar top border and shorten the bar to ~28 px (content included); remove the status-options `menu-indicator` triangle.
- [x] 13.2 Menus: 1 px separator in `${separator}` (`#404040`) and one-step-brighter disabled item text (new `${menuDisabledText}` token).
- [x] 13.3 Widget panel bodies `#4d4d4d` via `QTabWidget#panelGroupTabs QStackedWidget` (headers unchanged).
- [x] 13.4 Toolbar: screen-mode menu shows a display-only `F` hint on the three items (both the toolbox popup and the View submenu); Paint Mask button sized/iconed like the tool slots in both column modes; foreground/background widget background made transparent.
- [x] 13.5 File tab: append `" @ {zoom}%"` before `(mode/bits)` and refresh on zoom; font 2 px smaller at medium weight; close-button idle-flat with a one-step-lighter hover/pressed background; right padding +2 px.
- [x] 13.6 Widget group header label font 2 px smaller at medium weight.
- [x] 13.7 All sliders jump to the clicked point and track (`JumpSlider` everywhere) with a Qt Test.
- [x] 13.8 Persist window size, position, and maximized/normal state in the session and restore on launch.
- [x] 13.9 Layers: add a `Lock:` label left of the locks; smaller visibility glyph in a wider eye gutter; +2 px content padding; 4 px gap before the layer name; name font 2 px smaller.
- [x] 13.10 Run `bash scripts/verify-full.sh` green (2836 passed) and screenshot-verify the palette, frame, panels, file tab, and layers.

## 14. Menus, columns, footer, tabs, and spacing refinements

- [x] 14.1 Toolbar: the Tools column shows no width-drag cursor/feature — the handle beside it is arrow-cursorred and the column's grip stays hidden.
- [x] 14.2 File bar: medium-weight tab font (font set on the bar so elision metrics match the painted size), text↔close spacing tightened, lighter close icon (light `panel.close.light.png` via QSS `image`, since QtSvg renders `currentColor` black), right padding reduced, no bottom border.
- [x] 14.3 Columns (tools, widget, icon strip): 1 px dark-grey (`${separator}`) side borders and bottom borders, header bottom border dark grey, with the layout inset so the border shows; the frame-side border stays suppressed (`frameEdge`) so the band edge remains 2 px panel + 1 px border; the tools column widened by the border so nothing clips.
- [x] 14.4 Footer: flat options pseudo-button on the footer surface, font −2 px, options button moved between resolution and GPU, three vertical separators (Zoom | Resolution+button | GPU | Hints) in `${separator}`.
- [x] 14.5 Move hints: four direction keycaps as separate buttons in a row, and the Shift hint renders `[Shift] + [←][↑][↓][→]`.
- [x] 14.6 Widget group tabs no longer elide early (the bar font now matches the painted tab font) and render at medium weight; Layers name gets a larger left gap (`kContentPad` 6, `kNameGap` 8).
- [x] 14.7 Menu bar item horizontal padding reduced.
- [x] 14.8 Options bar gets a 1 px top border and 8 px right padding.
- [x] 14.9 Menus: `View ▸ Use GPU Compute` removed and replaced by a real `Performance` page with a `Use GPU Compute` checkbox in Preferences; `View ▸ Options` moved to `Window ▸ Options`; `Window ▸ 3D` removed.
- [x] 14.10 Run `bash scripts/verify-full.sh` green (2838 passed) and screenshot-verify the footer, file tab, column borders, group tabs, and layers.

## 15. Chrome, columns, scrollbars, info, and hints refinements

- [x] 15.1 Widget column: the splitter handles are made inert (arrow cursor, mouse-transparent) and `reapplyColumnStretch` runs at startup, so the workspace-facing edge exposes exactly one width drag — the column's 4 px grip.
- [x] 15.2 Menu bar: bottom border removed (the options bar's top rule is the only line between menu and options).
- [x] 15.3 Options bar: extra left inset so the first control clears the toolbar's drag grip.
- [x] 15.4 Widget and tools column header: bottom rule one `Theme::shade` step darker than the header surface.
- [x] 15.5 Scrollbars: track shifted one step lighter; the workspace canvas's bottom-right corner cell (`QWidget#canvasScrollCorner`) takes the scrollbar track colour instead of the workspace showing through.
- [x] 15.6 Footer hint keycaps: wider direction caps (x-padding), a one-step-lighter outline, and a fill flush with the footer surface.
- [x] 15.7 Info panel: the readout grid no longer flex-grows (trailing stretch), and a new row names the active tool and its keyboard hints.
- [x] 15.8 New `View > Tool Hints` checkable command (default on) hides/restores the footer hint strip and its separator.
- [x] 15.9 Widget group corner menu: `panel.menu` glyph rendered 20% smaller (13 px) at 0.9 opacity, button size unchanged.
- [x] 15.10 File tab close button: centred with a larger outside-right margin and a dimmer light glyph (PNG alpha scaled to 0.8).
- [x] 15.11 Run `bash scripts/verify-full.sh` green and screenshot-verify each change.

