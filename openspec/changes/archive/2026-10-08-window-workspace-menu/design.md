# Design

## Context

See proposal.md for motivation. Relevant current state:

- The app already serializes and rebuilds the widget-column layout. Each
  `PanelColumn` → `PanelGroup` stack serializes to a `panelColumns` JSON array
  (`PanelColumn::savePanelState`, `frame_session.cpp::saveSession`), and
  `applyPanelSession` rebuilds columns from it at startup. A group entry is
  `{name, order, visible, minimized, collapsed}`; a column entry is
  `{side, order, width, railMode, tools?, groups:[...]}`.
- Restore can only **reorder columns/groups and toggle panel visibility**. It
  cannot change which panels share a group: group membership is fixed in
  `frame_build.cpp`. `PanelGroup` exposes `takePanel`/`insertPanel` and
  `titleForPanel`/`iconForPanel`; `PanelColumn` exposes `takeGroup`/`adoptGroup`.
  `createPanelColumn` and `clearDynamicColumns` are `PicturaMainWindow` methods.
- `Window > Workspace` is registered in `command_tree.cpp` as `leaf()` stubs; the
  registry already supports `implemented`, `setHandler`,
  `setCheckedProvider`, and `setLabelProvider` (`Reset [Workspace]`). A `leaf()`
  command is registered through `add(id, path, label, shortcut)`, so its
  `CommandSpec.checkable` is `false`; `refresh()` only checks actions whose spec
  is checkable.
- Panels available today: Color, Swatches, Styles, Adjustments, Properties,
  Layers, Channels, Paths, Navigator, Histogram, Info, History, Actions,
  Gradients, Patterns, Notes, Brush, Clone Source, Character, Paragraph, Glyphs,
  Paragraph Styles. There is no 3D, Timeline, Tool Presets, Character Styles, or
  Libraries panel. (Canonical `ui/panel-rail` still lists a `Libraries` panel;
  this change records its absence rather than faking it.)
- The session store (`state.json`) is opaque, schema-versioned, atomic, and
  preserves unknown keys. A whole-column tear-off may be live when a rebuild
  runs.

## Goals / Non-Goals

**Goals:**

- Named and built-in workspaces with an active indicator, apply, save, delete,
  reset, and auto-remember.
- An apply path that can re-group panels (change tabbing), so presets express
  their documented panel sets.
- Factory layouts for Essentials, Painting, Photography, and Typography, with
  Essentials matching the authentic CS6 default.
- A workspace switcher at the right of the Options bar that shows the active
  workspace and opens the `Window > Workspace` menu.
- A workspace store that is independent of the session/chrome lifecycle.

**Non-Goals:**

- The `Window > Workspace` placeholder entries (3D, Advanced 3D, Motion, and New
  in CS6) are not implemented and are removed from the menu; their panels and
  behavior are absent.
- Keyboard-shortcut and menu-set capture (CS6 `Capture` checkboxes) is not
  implemented; `Keyboard Shortcuts & Menus…` stays as it is today.
- The Tools column is not part of a workspace and stays put.
- Per-workspace floating-panel geometry. Applying a workspace re-docks any
  floating panels; float state and geometry remain owned by the session store.

## Decisions

### D1 — A workspace is a named arrangement over the widget columns

A workspace's payload is the same `panelColumns` shape the session already uses,
so it describes side, order, width, rail mode, and per-group panel
order/visibility/minimized/collapsed plus group membership. Factory layouts are
data (`workspacePresets()`), not code paths.

_Alternative:_ store only panel visibility/order. Rejected — it cannot express
the CS6 preset panel sets, and it would make Essentials, Painting, etc.
indistinguishable.

### D2 — A dedicated versioned workspace store, not keys in `state.json`

The store follows the shipped preference-storage contract
(`docs/11-cross-cutting/preference-storage.md:97`: saved workspaces are
`$XDG_STATE_HOME/kooka-pictura/workspaces/*.json`, opaque layout blobs):

```
$XDG_STATE_HOME/kooka-pictura/
  state.json                 # session (unchanged) — also mirrors the active arrangement
  workspaces/
    index.json               # { schemaVersion, activeWorkspace, order: [id], entries: [{id, name, kind}] }
    essentials.json          # built-in remembered arrangement (absent => factory)
    my-edit.json             # { id, name, kind, layout, factory }
```

Each workspace is one atomically written file (`QSaveFile`), so a corrupt
workspace cannot take the others down and a workspace file is portable. Built-in
factories live in code; a built-in file exists only once its arrangement has been
remembered. `index.json` carries the schema version, the active workspace, and
the user-workspace order. (The older `docs/10-workflow-io/workspace-management.md`
proposes a `.work`/`.toml` shape; that proposal is stale and is not followed
here.)

Rationale: workspaces are user-authored content. `state.json`'s schema churns
each UI milestone and its `layout` blob is discarded on a `kLayoutRevision`
change; keeping workspaces in a separate, independently versioned store means a
session/chrome migration cannot destroy them. Unknown keys survive a
load-then-write per file.

_Alternative:_ a single `workspaces.json`. Rejected — it deviates from the
shipped doc and couples every workspace's fate to one file. _Alternative:_ keys
inside `state.json`. Rejected — couples user data to session migrations.

### D3 — Live layout vs stored layouts (auto-remember reconciliation)

The active workspace's arrangement is mirrored in `state.json` (`panelColumns`)
as today, so per-change saving is unchanged, but it is also snapshotted into the
workspace store on save/quit so **every** workspace's arrangement is durable
independent of `state.json`.

- **Switch:** snapshot the outgoing live layout into its store entry, set
  `activeWorkspace`, apply the incoming entry's layout (factory if it has none).
- **Startup:** restore the session layout (existing path), read `activeWorkspace`
  for the checkmark, and write the restored live layout into the active entry so
  the store never drifts.
- **First launch after upgrading** (old session, no workspace store): the
  restored live layout is adopted as Essentials' remembered arrangement rather
  than overwritten with the factory, so an existing user does not lose their
  layout; `activeWorkspace = "Essentials"`.

_Alternative:_ apply the factory layout on every switch. Rejected — not CS6
auto-remember behavior and it discards unsaved rearrangements.

### D4 — A re-group-capable apply path (`regroupAndApply`)

`applyPanelSession` is extended with a rebuild entry point that can re-tab:

1. Guard `restoringPanelSession_` so a mid-rebuild `stateChanged()` does not
   persist a partial layout.
2. Harvest: for every widget column from `panelColumns()` **excluding the tools
   column**, for every docked group and every group hosted in a `PanelFloat`,
   `takePanel` each panel into a pool, recording its title and icon (via
   `titleForPanel`/`iconForPanel` before the group is dissolved). Tear down the
   floats through `PanelFloat::onClose`/the column's close path (the only public
   teardown) and delete the now-empty groups. Panels a preset does not name go
   into a rebuilt hidden **overflow group** so they stay `Window > Panels`
   reachable.
3. `clearDynamicColumns()`; take the primary column's groups into the pool too.
4. Build fresh `PanelGroup`s from the spec, `addPanel(panel, title, icon)` in
   spec order.
5. Place columns in ascending `order`: the first right-hand entry reuses the
   primary `panelColumn_` and each later right-hand entry is anchored after the
   previous. Because the existing model is "lowest `order` = leftmost" (and
   `applyPanelSession` restores the same way), a factory that wants the iconic
   secondary to the **left** of the wide main lists the secondary at `order 0`
   (so it becomes `panelColumn_`) and the main at `order 1` (appended to its
   right). `createPanelColumn(Right, anchor)` inserts *after* the anchor; there
   is no "insert left of the main" step. Left-hand entries are inserted after
   the tools column (or at the splitter head) so the tools column keeps its
   position.
6. Apply per-group visibility/minimized/collapsed, rail mode, and width.
7. Put unnamed panels in the hidden overflow group of the rightmost (main)
   column. If the primary column hid itself during teardown
   (`removeColumnIfEmpty` hides an emptied primary), re-show it. Then
   `refreshSharedFloor`, `reapplyColumnStretch`, and save.

The algorithms in steps 2-3 delete the emptied `PanelGroup`s (`takeGroup` +
`deleteLater`; there is no public "delete empty group") and enumerate floats via
the frame's `findChildren<PanelFloat*>()` rather than a new public API.

The live layout serializer is factored into
`PicturaMainWindow::serializeWorkspaceLayout()` (used by both `saveSession` and
the workspace controller). Preset factories and the engine live in
`frame_workspaces.cpp`; the workspace controller (store, switch/reset/create/
delete, menu wiring, test hooks) lives in `frame_workspace_controller.cpp`.

_Alternative:_ replay drag/drop commits to move panels. Rejected — order-dependent
and fragile; a direct build from the spec is deterministic.

### D5 — Menu wiring, switcher, and delete semantics

- Frozen ids in `command_ids`: `window.workspace.essentials|painting|photography|
  typography|new|delete|reset`. Each preset is registered as a
  `CommandSpec{..., checkable = true}` (not via `leaf()`), because
  `CommandRegistry` has no `setCheckable` and `refresh()` only checks actions
  whose spec is checkable. New/Delete/Reset use their frozen ids so handlers bind.
  The placeholder presets (3D, Advanced 3D, Motion, New Features) are **not
  registered at all** — no disabled leaves.
- Menu grouping: `command_tree.cpp` inserts a separator after the preset commands
  and another after `Reset`, giving `presets | New/Delete/Reset | Keyboard
  Shortcuts & Menus…`.
- Each implemented preset: `setHandler` (switch), `setCheckedProvider`
  (active == name). Reset: `setLabelProvider` → `"Reset " + active` and a handler.
- New/Delete: `QInputDialog` (name / chooser). Name rules: reject empty or
  whitespace-only, reject any collision with an existing workspace, cap the
  length. Delete is always enabled and lists every workspace except the active;
  any non-active workspace — presets included — is deletable.
- **The workspace list is dynamic.** The `Window > Workspace` `QMenu` is refreshed
  on `aboutToShow` (the `refreshRecentMenu` precedent): remove previously-added
  dynamic actions; insert the user workspaces **before the first separator** so
  they join the presets section; and hide any preset's static action whose
  workspace record no longer exists (a deleted preset). Each dynamic action is
  checkable and checks against the active workspace.
- **Options-bar switcher.** `OptionsBar` gains a right-aligned
  `QToolButton` (`objectName workspaceSwitcher`, `InstantPopup`) whose text is the
  active workspace and whose menu is the same `Window > Workspace` `QMenu`.
  `OptionsBar::setWorkspaceMenu(QMenu*)` / `setActiveWorkspace(QString)` are
  called from the controller. `buildMenus()` runs before `buildPanels()`, so the
  switcher is attached in `initWorkspaces()` (which runs after the Options bar
  exists), not from `wireWorkspaceCommands()`.
- **Presets are seeded only when no usable store exists.** `initWorkspaces()`
  seeds the four presets when the store is missing, corrupt, or has no records;
  when a store loads with records, its set is authoritative (deleted presets stay
  deleted). Builtin `factory` is refreshed from `workspacePresets(name)` in code.
  When the session has no panel layout, the active workspace's own saved layout is
  restored rather than overwritten by the fresh default.
- All new wiring lives in `frame_workspaces.cpp` / `frame_workspace_controller.cpp`;
  `frame_menus.cpp` (1113 LOC) only calls a single `wireWorkspaceCommands()` so it
  stays under the file cap.

_Alternative:_ register user workspace commands in the registry at runtime.
Rejected — the registry is append-only and has no removal; a dynamic menu is
simpler and matches an existing pattern.

### D6 — Factory layouts (best-effort over the available panels)

| Preset | Main right column | Secondary right column |
|---|---|---|
| Essentials | `Color\|Swatches`, `Adjustments\|Styles`, `Layers\|Channels\|Paths` | iconic: `History`, `Properties` |
| Painting | `Navigator\|Swatches`, `Brush\|Clone Source`, `Layers\|Channels\|Paths` | iconic: `History` |
| Photography | `Color\|Swatches`, `Adjustments\|Styles`, `Layers\|Channels\|Paths`, `Navigator\|Histogram\|Info` | iconic: `History`, `Actions` |
| Typography | `Character\|Paragraph`, `Paragraph Styles\|Glyphs`, `Layers\|Channels\|Paths` | iconic: `History`, `Properties` |

Painting, Photography, and Typography are approximations of the CS6 panel sets
(`docs/02-ui-ux/workspace-and-docks.md:151-160`) over panels this app has; the
missing CS6 panels (Tool Presets, Character Styles) are noted, not faked. There
is no oracle for panel arrangements, so this is explicitly best-effort.

Essentials is the researched authentic CS6 default (the Photoshop Essentials
"Managing Panels in Photoshop CS6" tutorial: two columns; main
Color/Swatches, Adjustments/Styles, Layers/Channels/Paths; secondary icon
History/Properties).

`workspacePresets(name)` returns these four factories. The factory `width` is the
remembered *normal* width (the live iconic strip width is owned by rail mode), so
an iconic column's factory seeds a small normal-width memory and the actual strip
width is applied by `setRailMode(true)`. Panels absent from a preset are kept
reachable through the hidden overflow group rather than dropped.

### D7 — The fresh-session default changes to Essentials

Because Essentials is both the preset and the fresh-session default, the build
uses the Essentials factory for the initial layout. This supersedes the current
single-column default and is why `ui/panel-column`, `ui/application-shell`, and
`ui/panel-rail` need MODIFIED deltas. `kLayoutRevision` gates only the
`QMainWindow` `layout` blob, not `panelColumns`, and is **not** bumped: the
dock/toolbar shape is unchanged and an existing saved session must keep restoring
its own `panelColumns` (D3). Only fresh sessions get the new default.

## Risks / Trade-offs

- **Default-layout change is BREAKING and touches shipped specs/tests.** →
  `ui/panel-column`, `ui/application-shell`, and `ui/panel-rail` get MODIFIED
  deltas; affected Qt suites and self-test checks are updated in the same change.
  Concrete self-test breakage to plan for: `groups_grouped` (`selftest.cpp:1707`,
  hard `ST_FAIL`), the `panelColumnCountForTest() == 3` assertions
  (`selftest_ui_persistence.cpp:93`, `selftest_session.cpp:130`,
  `selftest.cpp:4470`), and the drag/tear-off checks that pull
  Navigator/Histogram/Info from the primary column. `selftest.cpp` is at its
  file-size allowlist ceiling, so these are adjusted in place (the suite only
  shrinks) and new coverage goes to Qt Test.
- **The default two-column layout collides with startup.** → `applyPanelSession`
  calls `clearDynamicColumns()` first, which deletes any extra column built in
  `buildPanels`; build/adopt the Essentials default *after* the clear (or spare
  the built-in secondary column), and re-show a primary column hidden by
  `removeColumnIfEmpty`.
- **Rebuild touches live, complex code (floats, dynamic columns, primary-column
  identity).** → Keep it in `frame_workspaces.cpp`, use only existing primitives,
  and cover apply/save/restore/re-group with the Qt Test suite before touching
  menu wiring.
- **Dynamic menu actions can duplicate on repeated `aboutToShow`.** → Remove the
  previously-added dynamic actions before inserting.
- **Name collisions / empty names.** → Reject empty/whitespace, duplicate user
  names, and over-length names; never replace a built-in.
- **Store drift between `state.json` and the workspace store.** → Snapshot the
  active entry on save/quit, and reconcile at startup (D3).

## Migration Plan

- No data migration. A missing/corrupt store falls back to built-ins with
  Essentials active; a corrupt single workspace file drops only that workspace.
- An existing `state.json` continues to restore; its layout is adopted as
  Essentials' remembered arrangement on first launch (D3), so no layout is lost.
- Applying a workspace re-docks floating panels (float geometry is not captured);
  the session store continues to own float state.
- Rollback: delete the `workspaces/` directory; the app falls back to built-ins
  and the session layout.

## Open Questions

- The exact Painting/Photography/Typography arrangements are best-effort and can
  be tuned against CS6 screenshots later without changing the specs or the
  architecture.
