# Design

## Context

The Tools toolbar is a `QDockWidget` (`Toolbox`) in the main-window dock areas,
while the right-hand panels live in a custom `PanelColumn` of `PanelGroup`s in
the central splitter. The two systems have diverged: the dock floats as an
operating-system top-level window, its placement is resolved by a bespoke
`resolveToolboxDrop`/`dockToolbox`/`floatToolboxAt` path, and its
no-tabification rule is a post-hoc repair (`ensureToolsNotTabified`) rather than
a resolver rule. The widget-panel drag grammar has meanwhile accumulated its own
gaps: a whole column cannot float in-window, a float is not a drop target,
group-on-group drops have no region feedback, the group header background stops
short of the corner button, dimming is target-only, and a floating icon row does
not match the docked strip.

This change makes the Tools panel a `PanelColumn` variant and closes those gaps
in the one grammar.

## Goals / Non-Goals

**Goals:**

- The Tools panel is a tabless, atomic column: column header plus the tool grid
  as a single plain content child, no tab bar, no `PanelGroup`.
- Tools and widget panels never combine, in either direction, with no insertion
  indicator for a forbidden combination.
- The tools header toggle switches the tool-grid width one↔two columns; the
  tools column has no iconic rail mode.
- The tools column floats in-window only, and an outer-band release commits a
  column so the indicator and the placement agree.
- Whole-column in-window float, floats as drop targets, group-on-group tabify
  with a blue outline, a full-width group header background, whole-drag dim, and
  floating icon-row parity.

**Non-Goals:**

- No change to the implemented tool set, the active-tool contract, the flyout
  catalogue, or the options bar.
- No new dependency, and no `docs/` change.
- No rework of the existing per-panel float, group splitter, or session group
  shapes beyond the tools migration.

## Decisions

### D1. The tools column is a `PanelColumn` variant with one plain content child

Reuse `PanelColumn` as the host and give the tools instance a single plain
content widget — the tool grid (`Toolbox`'s content body) — instead of a stack of
`PanelGroup`s. The column already owns the header row, the width toggle, the
column drag, the session shape, and the splitter placement, so the tools panel
inherits all of it rather than shadowing it with `QDockWidget` behaviour. The
tools column reports zero groups and one content child; the resolver and the
session writer treat that as a valid column. This removes `toolsDock_` and its
`QDockWidget` special cases at the frame level.

_Alternative considered:_ keep `QDockWidget` and fix each gap in place. Rejected
— it preserves two drag systems and two float models, and the "atomic target"
rule would stay a repair instead of a resolver property.

### D2. A toggle-mode seam: rail mode vs tool-width mode

The column's existing width toggle is hard-wired to `normal`/`iconic` rail mode.
Introduce one small seam on `PanelColumn` — a toggle kind — so the tools
instance's toggle switches the tool-grid width between one and two columns while
the widget instances keep the rail toggle. The tools instance reports "no iconic
mode", so every rail-specific branch (strip construction, strip width, flyout)
is skipped for it. The tool-width branch reuses the existing content-width
formula that already fits the slot buttons plus the foreground/background
control.

_Alternative considered:_ a separate toggle widget in the frame. Rejected — the
header already carries exactly one toggle, and the seam is one enum, not a new
control.

### D3. Column-level float reuses `PanelFloat`

Extract the whole-column tear-off into a new translation unit,
`panels/panel_column_float.cpp`, and back it with the existing `PanelFloat`
overlay rather than a new window class. `PanelFloat` already provides the
in-window child overlay, the resize grip, the minimum size, and the close
control; the column float supplies the whole-column payload and the redock
target. Both the tools column and a widget column use the same float, so
"floats in-window, never an OS window" holds for every column by construction.

_Alternative considered:_ reuse `QDockWidget::setFloating`. Rejected — that is
precisely the OS-window behaviour being removed.

### D4. An "atomic target" rule in the drop resolver

Replace `ensureToolsNotTabified` with a rule in the resolver: a drop whose
resolved target would combine the tools column with a widget group/panel/column/
float — or the reverse — resolves to nothing. Because the resolver returns no
target, no insertion indicator is drawn for a forbidden combination, which is
the requested behaviour and can no longer disagree with a post-hoc repair. The
tools column remains a valid *sibling* target, so columns still place beside it.

_Alternative considered:_ keep the repair and add indicator suppression at each
call site. Rejected — it leaves the resolver saying one thing and the frame doing
another, which is the bug class this change removes.

### D5. Session migration: drop the old tools dock state

The old tools state lives in the opaque `QMainWindow::saveState()` blob and in
the `toolsColumns`/`useShiftKeyForToolSwitch` keys. On load, discard the legacy
tools dock entry from the opaque layout and seed the tools column from the
stored `toolsColumns` count (the frozen design labels this bump v6→v7; the store
is already at v8, so the concrete bump lands on the next free version and the
unknown-key round-trip is preserved). On save, record the tools column like any
widget column — host side, order, width state, and the one/two tool-column count
— so it round-trips. A store that lacks the tools column loads the default left
tools column.

### D6. The outline indicator widget

Add one outline widget for the group-on-group target: a translucent blue region
outline drawn around the target group's whole rect. It is a sibling overlay to
the existing thin insertion line and is driven from the same resolved target, so
the outline and the tabify commit cannot disagree. It is hidden on drag leave,
cancel, and commit.

### D7. Whole-drag dim and floating icon parity

Dimming moves to the frame's drag lifecycle: dim at drag start, clear at drag
end, regardless of target validity, and clear the drag source on cancel so no
ghost remains. Floating icon rows reuse the docked strip's icon button and
group grip handlers verbatim, so click-to-flyout and grip-drag have no separate
code path to drift.

## Risks / Trade-offs

- **Retiring `QDockWidget` touches the layout restore path.** A stale persisted
  layout that references the old tools dock could restore a widget over the menu
  bar. Mitigation: the existing discarded-layout rule stays, and the tools dock
  entry is stripped on load (D5).
- **The tools column is a `PanelColumn` with no groups.** Code that assumes a
  column is non-empty could drop it. Mitigation: the resolver and the
  empty-column cleanup treat a column with the one tools content child as
  non-empty.
- **The toggle seam is a second toggle kind.** Mitigation: the rail branches are
  skipped by a single predicate, and the tool-width branch reuses the existing
  content-width formula.
- **The outline can overlap the thin insertion line.** Mitigation: one resolved
  target drives both, and the outline is a distinct widget so it can be styled
  independently.

## Migration Plan

1. Land D1–D2 (tools column) and D5 (session migration) behind the existing
   session schema; a missing tools column loads the default left column.
2. Land D3 (column float) and D4 (atomic target), then the widget-panel gaps
   (D6–D7).
3. Rollback: the tools column is additive to the session shape, and the legacy
   `toolsColumns` key is preserved, so reverting the code restores the previous
   dock behaviour from the same store.

## Open Questions

- The frozen design labels the session bump v6→v7 while the store is already at
  v8; the implementation should take the next free version and keep the
  unknown-key round-trip. No behavioural decision depends on the number.
- Whether the tools column's default host side is left (CS6 default) or the
  last docked side. The proposal assumes left.
