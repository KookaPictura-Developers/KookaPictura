# Workspace and Docks

- **Spec ID:** `UI-003`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — the workspace switcher moved out of the removed Application bar into the panel-dock area, CS6 ships new predefined workspaces (`New in CS6`, `Typography`), and the darker interface plus `Auto-Collapse Icon Panels` shape dock behavior.
- **Depends on:** `UI-001` application-frame, `UI-002` menus, `ARCH-003` qt6-ui-design, `01-architecture/document-model.md`

> All module and widget names below are **design proposals**. No code exists in
> this repository. Facts confirmed by the fetched CS6 Help reference are stated
> plainly; reconstructed/community facts are marked *(inferred)* or
> `(to verify)`.

## CS6 behavior

### Panels and panel groups

- A **panel** is a dockable module (Layers, Color, History, …). Panels can be
  **grouped, stacked, or docked**. A **panel group** is several panels sharing
  one title bar, switched by their tabs.
- Each panel has a **tab** (with its name) and a **panel title bar** — the solid
  empty bar above the tabs. The **panel menu** (☰ / triangle) opens even when the
  panel is minimised.
- Default CS6 Essentials layout: two columns on the right — a wider **main
  column** (Color/Swatches; Adjustments/Styles; Layers/Channels/Paths) and a
  narrower **secondary column** collapsed to icons (History, Properties).
- CS6 added **panel text font size** (`Interface > UI Font Size`) and on-image
  displays; panel colours follow the four-level brightness theme.

### Docking, floating, and moving

- A **dock** is a collection of panels/groups, generally vertical. Create a dock
  by dragging panels to an edge until a **drop zone** appears.
- **Dock a panel:** drag it by its tab to the top, bottom, or between panels.
  **Dock a group:** drag it by its title bar.
- **Undock/float:** drag a panel or group out by its tab/title bar, into another
  dock, or free-floating.
- While moving, **blue highlighted drop zones** appear. The *pointer* position
  activates a drop zone, not the panel position (important UX detail).
- **Prevent docking:** hold `Ctrl` (Windows) / `Cmd` (Mac) while moving a panel.
  **Cancel:** press `Esc` while moving.
- **Stack floating panels:** drag a panel's tab to the drop zone at the bottom of
  another panel; reorder by dragging tabs up/down. Release on the narrow zone
  between panels, not the broad title-bar zone.
- If all panels are removed from a dock, the dock disappears.

### Tabs, minimizing, resizing

- **Minimise/maximise** a panel, group or stack: double-click a tab (or the empty
  tab area).
- **Resize** a panel by dragging any side. Some panels (e.g. Color) cannot be
  resized *(inferred which set)*.
- **Collapse/expand a group:** click its tab; the tab toggles the group.
- **Close a panel:** right-click (`Ctrl`-click on Mac) its tab → `Close`, or
  deselect it in the `Window` menu. **Add a panel:** select it from the `Window`
  menu and dock it.

### Collapsing to icons

- Panels can be **collapsed to icons** to reduce clutter. Collapse/expand all
  icons in a column with the **double arrow** at the top of the dock.
- Expand a single icon by clicking it. Resize the dock narrower until labels
  disappear to show icons only; widen to show labels.
- Collapse an expanded panel back to its icon by clicking its tab, icon, or the
  double arrow in the panel's title bar.
- `Interface > Auto-Collapse Icon Panels` (when offered) collapses an expanded
  icon panel when the user clicks away.
- Add a floating panel/group to an **icon dock** by dragging it in; panels added
  to an icon dock are automatically collapsed to icons.
- Move a panel icon (or icon group) by dragging the icon: up/down within a dock,
  into another dock (it adopts that dock's style), or outside (floating icon).

### Hiding panels

- `Tab` hides/shows **all** panels including the Tools panel and options bar;
  `Shift+Tab` hides/shows all panels **except** the Tools panel and options bar.
- With `Interface > Auto-Show Hidden Panels` selected, hovering the edge of the
  application window (Windows) or monitor (Mac) shows the hidden panels
  temporarily.
- `Window > Options` shows/hides the options bar.

### Workspace presets

A **workspace** is a named panel arrangement. CS6 ships predefined workspaces and
lets users create their own. The workspace switcher lists them; in CS6 it sits in
the dock area (it was in the removed Application bar in CS5).

| Workspace | Status in CS6 | Notes |
|---|---|---|
| Essentials | Confirmed default | General-purpose: Layers, Channels, Adjustments, History, etc. |
| New in CS6 | Confirmed (secondary) | Shows new features; highlights new menu items in blue |
| Painting | Confirmed | Brush Presets, Navigator, Brush, Clone Source, Tool Presets, History |
| Photography | Confirmed | Histogram, Info, Actions, and photo-editing panels |
| Typography | Confirmed (secondary book) | Type-oriented panels |
| 3D | Confirmed (secondary) | Extended-oriented 3D panels |
| Motion | Confirmed (secondary) | Video/animation (Timeline) |
| Advanced 3D | Confirmed (Help PDF, `Window > Workspace > Advanced 3D`) | Extended 3D |
| Design | **Unverified** — listed by the user; CS6 sources name **Typography** instead | See Open questions |

### Save, switch, delete, reset

- **Save:** `Window > Workspace > New Workspace…`; enter a name; optionally
  **Capture** `Keyboard Shortcuts` and `Menus` (both Photoshop). Saved names
  appear in the workspace switcher.
- **Switch:** choose a workspace from the switcher (or `Window > Workspace >`).
  The same document can be shown in any workspace.
- **Delete:** `Window > Workspace > Delete Workspace…` (choose from the switcher)
  and confirm.
- **Reset one workspace:** `Window > Workspace > Reset [Workspace Name]`.
- **Restore all built-ins:** `Interface > Restore Default Workspaces`.
- **Reorder** workspaces by dragging them in the switcher.
- Workspaces remember panel positions as you last left them until reset —
  switching away and back does **not** restore the saved arrangement
  automatically; the explicit `Reset` does.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Panel | Dock widget | n/a | Tab, title bar, panel menu |
| Panel group | Dock widget | n/a | Multiple tabs, one title bar |
| Dock | Dock area | n/a | Vertical; icon or expanded |
| Dock collapse toggle | Double arrow | n/a | Collapse/expand all icons |
| Workspace switcher | Drop-down | n/a | In dock area (CS6) |
| `Window > Workspace` | Menu | n/a | New/Delete/Reset/Keyboard & Menus |
| `Window > [panel]` | Menu toggle | see `UI-002` | Show/hide a panel |
| Hide all panels | Shortcut | `Tab` | Includes tools + options bar |
| Hide all but tools/options | Shortcut | `Shift+Tab` | Auto-show hidden panels option |
| `Window > Options` | Menu toggle | n/a | Show/hide options bar |
| Reset all workspaces | Preference button | n/a | Interface preferences |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Panel width | px (DIP) | theme | e.g. 200–480 | Per panel; persisted |
| Dock tab position | enum | Bottom `(inferred)` | Bottom / Top / Left / Right | Maps to `setTabPosition` |
| Dock nesting | bool | Off | on / off | `AllowNestedDocks` |
| Dock animation | bool | On | on / off | `AnimatedDocks` |
| Icon dock width | px (DIP) | icon size | `(to verify)` | Labels hide below threshold |
| Auto-Collapse Icon Panels | bool | Off `(inferred)` | on / off | Interface preferences |
| Auto-Show Hidden Panels | bool | Off `(inferred)` | on / off | Interface preferences |
| Workspace | enum | Essentials | built-in + custom | Captured per workspace |
| Capture keyboard shortcuts | bool | Off | on / off | New Workspace dialog |
| Capture menus | bool | Off | on / off | New Workspace dialog |
| Panel background | theme token | dark | 4 brightness levels | `UI-001` |

## Algorithms & pipeline

Design proposal.

- **`QDockWidget` is the panel primitive.** Each panel is a `QDockWidget` with a
  custom title bar (`setTitleBarWidget`) so the title bar, tabs and collapse
  arrow match CS6. Grouping is achieved with `QMainWindow::tabifyDockWidget` and
  a shared tab bar; stacking/floating uses `Qt::Floating` `QDockWidget`s whose
  geometry is restored per workspace.
- **Four dock areas + optional nesting.** Default `DockOptions`:
  `AnimatedDocks | AllowTabbedDocks`, and **not** `AllowNestedDocks` (keep drag
  semantics simple, per `ARCH-003`).
- **Drop-zone resolution.** On drag-move, compute nearest dock edge / inter-panel
  gap; honour the modifier that suppresses docking (`Ctrl`/`Cmd`) and `Esc`
  cancel. Use the *cursor* position, matching the documented CS6 behavior.
- **Icon collapse.** A dock can be in `Expanded` or `Icon` mode. Collapse all =
  iterate dock widgets and switch to icon mode (title bar only / vertical tab
  strip). Widen/narrow the dock to show/hide labels. `Auto-Collapse Icon Panels`
  collapses on focus loss.
- **Workspace model.** A workspace is
  `{ name, layout: QByteArray (saveState), panelWidths, iconMode[], activeTabs[],
  capturedShortcutSet?, capturedMenuSet? }`. `saveState()` keys by
  `objectName`, so every dockable panel needs a stable, unique `objectName`.
- **Apply/reset.** `apply(workspace)` calls `restoreState` + panel widths + icon
  modes. `reset(workspace)` restores the *factory* layout stored at build time,
  because CS6 remembers the last-used arrangement between switches.
- **Hide-all.** `Tab`/`Shift+Tab` animate tray hiding of docks; with
  `Auto-Show Hidden Panels`, an edge hover re-shows them.

## Rust module mapping

Proposals.

- `pictura_ui::workspace::Workspace` — name, serialized layout, panel widths,
  icon modes, active tabs, optional captured shortcut/menu sets.
- `pictura_ui::workspace::Registry` — built-in and user workspaces; resolve
  duplicate names; ordering.
- `pictura_ui::workspace::persist` — save/restore against the session/preference
  store; versioned schema so layout persists across releases.
- `pictura_ui::docks::PanelId` — stable ids matching `objectName`s.
- `pictura_ui::docks::IconState` — expanded/icon per dock, animation policy.

Crossing types: `WorkspaceName`, `PanelId`, `QByteArray` layout blob opaque to
Rust (or an equivalent neutral layout description), widths as `f32` DIP. No
document data.

## Qt6 component mapping

| Proposal | Base | Responsibility |
|---|---|---|
| `PicturaMainWindow` | `QMainWindow` | Owns dock areas; `saveState`/`restoreState`; `setDockOptions` |
| `PicturaDock` | `QDockWidget` | Panel host; custom `PanelHeader` title bar; `toggleViewAction` |
| `PanelHeader` | `QWidget` | Title, tab strip, panel-menu button, collapse arrow, drag hints |
| `WorkspaceSwitcher` | `QComboBox`/button+menu | Lists workspaces; New/Delete/Reset entries |
| `WorkspaceController` | `QObject` | Apply/reset/save/delete; persists session layout |
| `IconDockController` | `QObject` | Collapse/expand icon columns, label-width threshold, auto-collapse |

`saveState()`/`restoreState()` require every `QDockWidget` and toolbar to have a
unique `objectName` before the first restore; enforce this at registration.
Nested docks are disabled by default, so `saveState()` output is simpler and more
stable across Qt versions.

## Data-model impact

- Workspace/layout is **UI state**, stored in the preference/session store and
  optionally captured into a workspace; never in PSD/XMP.
- A workspace may capture **keyboard shortcut and menu customisation sets**
  (`UI-002`); those are preference records referenced by name.
- Restoring a workspace must not mutate the document model or add history
  states. Panel visibility toggles likewise produce no history.
- Panel content (Layers/Channels/Paths/History) is a **view over the document
  model**; the panel owns no document data (`ARCH-003`).

## Edge cases

- **Wayland.** Dock/floating geometry is compositor-controlled; restore relative
  layout only, never absolute positions.
- **Multi-monitor float.** A floated panel/group must be clamped to available
  geometry on restore; screen geometry is DIP with scaling gaps.
- **High-DPI change.** Icon assets and dock metrics rebuild when
  `devicePixelRatio` changes; persisted pixel widths should be stored as DIP.
- **Many panels.** Tabification plus deep nesting can exceed screen space; keep
  `AllowNestedDocks` off and provide overflow/scroll.
- **Duplicate `objectName`.** Breaks `restoreState` silently; registration must
  reject duplicates.
- **User workspace name collision** with a built-in (`Essentials`) must be
  prevented or namespaced.
- **Deleted/corrupt layout blob.** Fall back to Essentials and keep the user's
  custom workspaces intact.
- **Panel removed by a future version.** Restoring an old layout must ignore
  unknown panel ids without dropping the rest.
- **Screen mode interacts with docks** — entering Full Screen hides docks; the
  layout must survive the round trip.

## Parity acceptance criteria

- Given the default Essentials workspace, the main and secondary columns contain
  the default panel set, with the secondary column collapsed to icons.
- Given two panels in one dock, dragging a tab above/below another re-docks it
  with a blue drop zone snapped to the pointer; holding `Ctrl`/`Cmd` and dropping
  floats it; pressing `Esc` mid-drag cancels.
- Given a floating panel dragged to the bottom of another floating panel, they
  stack and move together as a unit; dragging one out un-stacks it.
- Given a dock, clicking the top double arrow collapses all its panels to icons
  and clicking again expands them; widening past the label threshold shows names.
- Given `Tab`, all panels including the Tools panel and options bar hide; given
  `Shift+Tab`, the Tools panel and options bar remain; with `Auto-Show Hidden
  Panels` on, hovering the screen edge reveals them.
- Given a rearranged layout, `Window > Workspace > New Workspace…` with Capture
  `Keyboard Shortcuts` saves it; selecting it later restores layout plus
  shortcuts; `Delete Workspace` removes it after confirmation.
- Given a modified built-in workspace, `Reset Essentials` restores the factory
  arrangement; `Restore Default Workspaces` restores every built-in.
- Given `Window > Arrangement`, each panel's `Window`menu entry toggles its
  visibility; right-clicking its tab offers `Close`.
- Given a saved workspace and a restart at the same scale factor, the layout,
  panel widths, icon modes and active tabs restore within one pixel.
- Given `devicePixelRatio = 2`, restored widths and icons are crisp and correct.

## Sources

Fetched for this document:

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official Photoshop CS6 Help reference (downloaded and text-extracted).
  Establishes: panel/group/dock definitions; dock and undock by tab/title bar;
  drop zones and the pointer-activation detail; `Ctrl`/`Cmd` prevent-docking and
  `Esc` cancel; stacking floating panels; double-click to minimise/maximise;
  resizing (and the Color-panel exception); collapse-to-icons and the double
  arrow; `Auto-Collapse Icon Panels`; `Auto-Show Hidden Panels`; `Tab`/`Shift+Tab`
  hide-all behavior; add/remove panels via the `Window` menu and tab context
  menu; UI Font Size; workspace save/switch/delete/reset and `Restore Default
  Workspaces`; `Window > Workspace > New Workspace`, `Delete Workspace`,
  `Keyboard Shortcuts & Menus`, `Advanced 3D`; workspaces auto-remember the last
  arrangement until reset; reorder workspaces by dragging.
- `https://www.photoshopessentials.com/basics/photoshop-cs6-workspaces` —
  secondary CS6 tutorial. Corroborates the Essentials/Painting/Photography
  workspace panel sets, the workspace switcher location (top-right above the main
  panel column), the two-column icon/expanded layout, and the reset behavior.
  Secondary source.
- `Adobe Photoshop CS6 on Demand` (search-result snippet, secondary) — names the
  predefined workspaces `Essentials (Default)`, `New in CS6`, `3D`, `Motion`,
  `Painting`, `Photography`, `Typography`. Used only to flag the preset set;
  marked `(to verify)`.
- `https://itwiki.wpunj.edu/images/e/ee/Photoshop_CS6_Extended_-_Dacier.pdf` —
  secondary student tutorial; corroborates the panel dock / collapsed palette
  bar terminology. Secondary source.
- SearXNG meta-search queries used to locate secondary sources (workspace
  preset list). No facts taken from snippets alone.

Not parsed: `helpx.adobe.com` (HTTP 403 from this environment).

## Open questions

- **`Design` workspace existence in CS6.** The task and some later sources list
  `Design`; the strongest CS6 secondary source names `Typography` instead. Resolve
  with a CS6 workspace-switcher screenshot.
- **Exact built-in workspace list and order.** Essentials is confirmed;
  Painting/Photography are confirmed by a CS6 tutorial; the rest are secondary.
  Resolve with a capture of the switcher menu.
- **Precise dock metrics.** Icon dock width, label-hide threshold, tab height,
  title-bar height are unsourced. Resolve with pixel measurements from a CS6
  screenshot.
- **Nested docks in CS6.** Whether CS6 allowed nesting/splitting docks is not
  verified. Our proposal disables nesting; confirm against CS6.
- **Layout persistence format.** CS6's workspace serialization is proprietary;
  we store our own versioned blob. Cross-version migration policy is open.
- **Panel resize exceptions.** The Help says "some panels, such as the Color
  panel, cannot be resized"; the full exception set is unknown. Resolve with a
  CS6 interaction pass.
- **Auto-Collapse / Auto-Show defaults.** Defaults are inferred. Resolve with a
  first-run CS6 preferences capture.
