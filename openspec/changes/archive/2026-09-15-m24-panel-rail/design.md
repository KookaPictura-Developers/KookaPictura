## Context

M20 created seven real panels (Layers, History, Navigator, Color, Swatches, Info,
Histogram). M23 tabified six of them into three groups and added the CS6
stylesheet, toolbox grid, and canvas colour. The CS6 reference
(`docs/02-ui-ux/reference/cs6-workspace.png`) shows three groups of four/three/
three tabs plus a narrow right icon rail holding collapsed panels, with every
panel also reachable from `Window > Panels` (the command tree already lists all
of them; the ones we have not built are disabled leaves).

## Goals / Non-Goals

**Goals:**

- The three CS6 tab groups, populated with the panels named in the reference.
- A right icon rail whose buttons toggle the collapsed panels.
- `Window > Panels` commands for every panel in the groups and the rail, sharing
  one toggle path with the rail.
- Panels we do not implement yet exist as honest empty-state placeholders.

**Non-Goals:**

- Real content for the placeholder panels (Gradients/Patterns preset grids,
  Properties binding, Adjustments presets, Libraries, Channels/Paths lists,
  Actions). They are structural for M24.
- Icon-collapse animation/auto-collapse, floating-panel drop zones, workspace
  presets and the workspace switcher, and the panel-title-bar menus.
- Any change to Rust, the document model, or dependencies.

## Decisions

### One `PlaceholderPanel` class, one instance per missing panel

`PlaceholderPanel : QDockWidget` takes a title and an empty-state message
(default "No <title>"), sets its `objectName` from the caller, and shows a
centred label. Frame constructs eight instances.

- *Why:* the panels are structurally identical stubs; eight near-duplicate
  classes would be the abstraction the project avoids. When a panel gains real
  content it becomes its own class, like the M20 panels did.
- *Alternatives considered:* eight skeleton classes (rejected: no behaviour);

### The right icon rail is a vertical `QToolBar` in the right dock area

`addToolBar(Qt::RightToolBarArea, rail)` gives a narrow vertical strip without a
custom widget. Each rail entry is a checkable `QAction` carrying the panel's
`Window > Panels` command id; the frame's existing command handler toggles the
dock, and a `visibilityChanged` connection keeps the button checked state in
sync.

- *Why:* reuses the M20 `Window > Panels` handler/checked-provider pattern and
  the existing command ids; no second toggle mechanism.
- *Alternatives considered:* a custom `QDockWidget` rail (more code, same
  result); an icon-collapse dock (deferred).

### Panel groups by `tabifyDockWidget`

Color+Swatches+Gradients+Patterns, Properties+Adjustments+Libraries, and
Layers+Channels+Paths are tabified on the right; the rail panels stay collapsed
(not tabified into a group). Stable `objectName`s are preserved so the session
layout and `Tab`/`Shift+Tab` keep working.

- *Why:* matches the reference and Qt's dock model.
- *Trade-off:* a restored session layout overrides the defaults; the self-test
  isolates its state dir (M23).

### `Window > Panels` commands are implemented for every grouped/rail panel

Add ids for Gradients, Patterns, Properties, Adjustments, Libraries, Channels,
Paths, Actions; convert their `command_tree` leaves to implemented, checkable
commands; add handlers and checked providers for each. The rail actions dispatch
the same ids.

## Risks / Trade-offs

- **Placeholder panels could read as features** → each shows an explicit empty
  state and the milestone is documented as structural.
- **More docks could crowd the right column** → groups are tabbed and the rail
  is narrow; three groups plus a rail is the CS6 arrangement.
- **Command/handler drift** → both the menu and the rail resolve the same id
  through the registry, so there is one enablement/checked path.
- **Self-test fragility from user layout** → already isolated by the M23
  `XDG_STATE_HOME` temp dir.

## Migration Plan

Additive. Rollback deletes `placeholder_panel.*`, `panel_rail.*`, the new
command ids, and the `buildPanels` grouping changes.

## Open Questions

- Whether the rail should also include Brush, Clone Source, and Animation from
  the reference's full panel list — M24 includes only panels the shell can
  meaningfully toggle (History, Actions, Info, Navigator, Histogram) plus the
  three groups.
- Exact CS6 rail icon order and glyphs — deferred; M24 uses the panel names and
  existing command icons where available.
