## Why

The M38 toolbox is a single-column list of 23 flyout slots, but it does not
behave like the CS6 Tools panel. The slot's corner marker is
`QToolButton::MenuButtonPopup`'s stock arrow, not the CS6 lower-right triangle;
the flyout menu entries carry no shortcut; there is no one/two-column
double-arrow; the Tools dock can be tabified into a panel group and dropped
anywhere; and the only group-cycling shortcut in the app is the hard-coded
`B`/`Shift+B` Brush/Pencil case in `frame.cpp`. `docs/02-ui-ux/toolbox-and-options-bar.md`
(`UI-004`) and `docs/02-ui-ux/preferences.md` (`UI-010`, `Use Shift Key For Tool
Switch` default on) specify all of this. This change reconciles the canonical
`tool-framework` "Tools panel" requirement — which M38 froze as single-column
"rather than a two-column grid" — with CS6's default-one-column, toggle-to-two
behaviour, and lands the CS6 flyout, column, dock, and cycling contracts. It is
the M40 milestone before the M41 Preferences dialog (the Layers-panel program
shifts by two behind this pair).

## What Changes

- **Flyout indicator.** Replace `QToolButton::MenuButtonPopup` with a custom
  paint on the slot button that draws a small filled triangle at the
  **lower-right** corner whenever the slot's group has two or more members,
  implemented or not; the icon stays centred.
- **Flyout opening.** A ~300 ms press-and-hold opens the group `QMenu` positioned
  below the button; a right-click opens it immediately; a release before the
  timeout activates the slot's current tool and opens nothing. A deterministic
  test hook opens a slot's flyout for the self-test.
- **Flyout shortcut keys.** Every item shows its slot key right-aligned; a
  disabled (unimplemented) item keeps the `— not implemented yet` tooltip. The
  key is displayed with `QAction::setShortcut` +
  `setShortcutVisibleInContextMenu(true)` scoped to the flyout
  (`Qt::WidgetWithChildrenShortcut`); no window-global shortcut is registered, so
  a shared key on a disabled item cannot steal the letter.
- **One/two-column toggle.** A custom title bar (`QDockWidget::setTitleBarWidget`)
  with a `Tools` label and a double-arrow button reflows the 23 slots between one
  and two columns; two new icons `panel.columnsOne` / `panel.columnsTwo`; the
  foreground/background control and the Screen Mode button stay pinned at the
  bottom.
- **Standalone dock.** `setAllowedAreas(Left | Right)` and
  `setFeatures(Movable | Floatable | Closable)`; an event filter refuses a drop
  onto a tab bar and a `dockLocationChanged` fallback re-docks the panel to its
  previous side if it still ends up tabified. (Qt has no clean tabify-veto API;
  the fallback is recorded.)
- **Generic `Shift`+letter cycling.** Replace the hard-coded Brush/Pencil case in
  `frame.cpp` with a per-group handler: a plain letter selects the slot's current
  member; `Shift`+letter cycles to the next implemented member (unimplemented
  members skipped); a group with no implemented member does nothing. Gated by a
  new `useShiftKeyForToolSwitch` session preference (default true). **No
  Preferences dialog in M40** — the preference is session-persisted only; M41
  adds the dialog and gives it a UI.
- **Session schema v4.** `toolsColumns` (1 or 2) and `useShiftKeyForToolSwitch`;
  older/missing stores load the defaults; the existing round-trip must not
  regress.

## Capabilities

### New Capabilities

None. This is Tools-panel view/controller work over the existing tool catalogue
and frame; no file-format, compositor, document, or new persistence capability is
added (the two new session fields extend the existing store).

### Modified Capabilities

- `tool-framework`: the "Tool registry and active tool" requirement becomes a
  generic per-group slot-letter contract gated by the new preference (replacing
  the Brush/Pencil-only modified press); the "Tools panel" requirement is
  reconciled to **one column by default with a two-column toggle** (superseding
  both M23's fixed two-column grid and M38's fixed single column) and gains the
  custom lower-right triangle and hold/right-click flyout; new requirements add
  the column layout, the flyout indicator/opening, the flyout key display, and
  the `Shift`-key preference at session schema v4.
- `application-shell`: a new "Tools panel is a standalone dock" requirement — the
  panel is allowed only in the left/right areas, floats, and cannot be grouped in
  a tab group. The M38 "Toolbox catalogue and flyout groups" requirement stays
  true (hold reveal, corner triangle, `Alt`/slot-shortcut cycling) and is not
  contradicted.

## Impact

- `crates/pictura-app/cpp/toolbox.{h,cpp}` — custom triangle paint, hold/right-click
  flyout opened below the button, key column, custom title bar + grid reflow,
  dock features/allowed areas + tabify-refusal event filter, and the
  `*ForTest` hooks. `QToolButton::MenuButtonPopup` is removed.
- `crates/pictura-app/cpp/tools.{h,cpp}` — a small letter/shift → group lookup
  helper over the existing 71-tool table; no catalogue or implemented-set change.
- `crates/pictura-app/cpp/frame.{h,cpp}` — remove the `B`/`Shift+B`
  `cyclePaintTool` special case for the generic per-letter handler; pass the
  session preference into `buildTools()`; persist the two session fields.
- `crates/pictura-app/cpp/session.{h,cpp}` — `toolsColumns` and
  `useShiftKeyForToolSwitch` at schema v4; the M39 load-before-write save path is
  unchanged.
- `crates/pictura-app/cpp/main.cpp` — the `m40_*` self-test steps with fresh exit
  codes.
- `assets/icons/panel.columnsOne.svg`, `assets/icons/panel.columnsTwo.svg`, and
  the matching `assets/pictura.qrc` entries (implemented in the asset task; this
  change does not touch assets).
- `docs/dev/m40-tools-panel.md` — the brief; `docs/dev/STATE.md` — the program
  status.
- No new dependency, no Rust/bridge/document/compositor change, no PSD format
  change. Every mutation keeps the M34 composite-then-record convention (no
  mutation is added at all).
