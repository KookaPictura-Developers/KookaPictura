# Proposal

## Why

`Window > Workspace` is registered in the command tree with all of the CS6
entries, but every one is an unimplemented stub, so the presets, New/Delete,
Reset, and the active-workspace indicator do nothing. The only layout
persistence is the single implicit session layout; there is no named or
selectable workspace and no factory layout to reset to. Tracked in #219.

## What Changes

- Add a `ui/workspaces` capability: built-in preset workspaces plus user-created
  named workspaces, an active workspace shown as a checked menu item,
  New/Delete/Reset, and auto-remember of a workspace's last arrangement until it
  is explicitly reset.
- Add a re-tab-capable layout apply path. The existing session restore can only
  reorder panels and toggle visibility; it never changes which panels share a
  group, so it cannot express the CS6 preset panel sets.
- Define factory layouts for the presets the panel inventory can express:
  Essentials, Painting, Photography, and Typography. The preset entries with no
  matching panels (3D, Advanced 3D, Motion, New Features) are removed from the
  menu entirely rather than shown disabled.
- Add a workspace switcher at the right end of the Options bar: a button that
  shows the active workspace and opens the `Window > Workspace` menu.
- Group the `Window > Workspace` menu into separated sections: the workspace
  presets (built-in and user), the workspace actions (`New`/`Delete`/`Reset`),
  and additional items (`Keyboard Shortcuts & Menus…`).
- Make `Delete Workspace…` always available and open a chooser listing every
  workspace except the active one; the active workspace cannot be deleted, and
  the pre-created preset workspaces are deletable. Presets are seeded only when
  no store exists, so a deleted preset stays deleted.
- **BREAKING** (UI default): a fresh session opens in the authentic CS6
  Essentials two-column layout — main column `Color | Swatches`,
  `Adjustments | Styles`, `Layers | Channels | Paths`; secondary icon column
  `History`, `Properties` — superseding the current single-column default.
- Persist named workspaces in a new versioned workspace store under
  `$XDG_STATE_HOME/kooka-pictura/workspaces/`, separate from the opaque session
  store so a session/chrome migration can never discard user workspaces.
- Applying a workspace re-docks floating panels; per-workspace floating
  geometry is out of scope (the session store keeps owning float state).

## Capabilities

### New Capabilities

- `ui/workspaces`: the named and built-in workspace model, the active-workspace
  indicator, apply/save/delete/reset, auto-remember, and the workspace store.

### Modified Capabilities

- `ui/panel-column`: the default panel groups requirement changes from the
  current single-column arrangement to the authentic CS6 Essentials two-column
  layout.
- `ui/application-shell`: the default dock grouping requirement changes to match
  the same authentic Essentials layout.
- `ui/panel-rail`: the requirement that the default Essentials set adds `Styles`
  in the former Gradients/Patterns slot and folds `Properties` into `Adjustments`
  is removed, since the authentic Essentials uses a `Styles` tab in the
  `Adjustments` group and a standalone `Properties` panel.

## Impact

- New files: `crates/pictura-app/cpp/workspace_store.{h,cpp}`,
  `crates/pictura-app/cpp/frame_workspaces.cpp`,
  `crates/pictura-app/cpp/frame_workspace_controller.cpp`,
  `crates/pictura-app/cpp/tests/tst_workspaces.cpp`.
- Changed: the command tree and `command_ids`, the frame menu wiring, the frame
  header and constructor, the session serializer (`frame_session.cpp`), the
  Options bar (`options_bar.{h,cpp}`), `CMakeLists.txt`, the Qt test list, and the
  default panel build. Existing Qt tests and self-test checks that assert the old
  default groups are updated (the self-test only shrinks; new coverage goes to
  Qt Test).
- New user state: `$XDG_STATE_HOME/kooka-pictura/workspaces/` (versioned,
  atomic, one file per workspace). The session store's schema is unchanged.
- No engine crates, no Qt-runtime behavior outside the app shell, no new
  dependencies.
