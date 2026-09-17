# M44 — panel/theme polish (sixth pass)

- **Status:** proposed (`openspec/changes/m44-panel-theme-polish`); not
  implemented.
- **Type:** a user-requested refinement pass on the M41–M43 `PanelColumn`
  chrome, plus one engine bug in the new-document render path. It is the **sixth
  panel interruption** of the Layers-panel program; the program shifts behind it
  (see §7). STATE is handled at close-out.
- **Contract:** `openspec/specs/panel-column/`, `application-shell/`,
  `tool-framework/`, and `document-canvas/` are the canonical requirements this
  change reconciles. `docs/02-ui-ux/workspace-and-docks.md` (`UI-003`) governs
  docks. The tab-colour contract is the M43 `panelTabBar` lineage; the
  new-document render is the M17 `application-shell` "initial canvas" lineage.
- **Consumers:** `crates/pictura-app/cpp/panels/panel_column.{h,cpp}`,
  `panels/panel_group.{h,cpp}`, `cpp/toolbox.{h,cpp}`, `cpp/frame.{h,cpp}`,
  `cpp/theme.cpp`, `cpp/main.cpp`, and `crates/pictura-app/src/cxxqt_object.rs`.
- **Non-goals:** placeholder-panel content, pixel-exact CS6 metrics,
  multi-monitor floating, OS-window float chrome, per-column compact modes,
  persisted float positions, a second drag/drop system, and any change to the
  tool catalogue, the M40 flyout/shortcut/dock contracts beyond the allowed
  dock sides, the M41 tab menu list, or the M42 per-widget menu contents. No new
  dependency, no PSD/session/on-disk format change, and no bridge-ABI change.

## 1. The user's requests (all in scope)

**E. New-document canvas render bug (engine)**

1. A newly created document's canvas initially shows a repeating
   black/white/green/red gradient garbage pattern; moving a layer (a
   recomposite) makes it pure white. A new white-background document must render
   **pure white immediately**. Root-cause the initial composite/readback path
   (`new_document` → `current_buffer`/`store_composite`/`buffer_to_image` in
   `cxxqt_object.rs`, or the GPU `composite_active` first-use path), fix it, and
   add a runnable regression check.

**T. Tools toolbar (`Toolbox` dock)**

2. When **floating**, its **height must not be drag-resizable** (it keeps the
   M42 minimum content height).
3. Allow **docking it to any side of the widget panels** as well (currently only
   the workspace left/right dock areas).

**W. Widget panel (docked `PanelColumn`/`PanelGroup`)**

4. The **collapse-panel button's double-chevron icons are inverted** — swap them.
5. The **last item is active by default; the first must be** — investigate
   which artefact (default group tab, default current panel, and/or default
   current page) reads last-active and fix it.
6. The **active/inactive tab background colours are the opposite of what they
   should be** — swap them so the active tab matches the widget and the inactive
   does not.
7. Dragging a **widget tab outward to float** creates the float but **stops
   dragging instantly**; the drag must **continue until mouse release** (the
   float follows the cursor), like a group/column drag.
8. A whole **widget column must be dockable to any side** of the toolbar, of
   another widget panel/column, or of the workspace — currently only the far
   right. Extend the drop/dock targets (consistent with the M43 `resolveDrop`
   model, one indicator).
9. The **divider between widget groups** must be **thicker and darker grey**.

**C. Compact/iconic widget panel**

10. The **popup (flyout) style must match the docked panel widget** exactly
    (same tab bar, background, borders).
11. The **white divider colour** in the compact strip must become **dark grey**.
12. **Greatly improve drag-and-place**: drop onto/among a group or very close
    above/below → insert into that group; drop between groups, above the top, or
    below the bottom → create a new group; the **whole group must be draggable**
    via a small **drag-handle dots** affordance above each group; each group's
    icon buttons must look like they belong together.
13. When **drag-resizing the width**, strip labels must **crop/truncate
    (elide)** as soon as there is *any* room, instead of staying hidden until the
    full label width is available (the M42/M43 `>= 120 px` gate).

**F. File bar (document tab bar)**

14. Add a **dark grey border to its right**.
15. **Remove the extra top border** (the tool/context bar above already has a
    dark grey bottom border).

**S. Global styling**

16. Add a **darker grey border** to all panels/columns (the toolbar, and both
    normal and compact widget panels).
17. A **widget must have the same colour scheme and style whether docked,
    popup, or floating** (unify the three presentations).

## 2. Current state (inventory)

- `crates/pictura-app/cpp/panels/panel_column.{h,cpp}`: compact strip builds a
  `panelIconDivider` `QFrame` (`:582`) and per-panel `panelIconRow` widgets
  (`:593`); label visibility is the single `iconStrip_->width() >= 120` gate
  (`:615–627`); `ensureFlyout` hard-codes an inline `#panelIconFlyout` stylesheet
  (`:659–661`) that does not match the docked group; `updateColumnToggle` picks
  `panel.columnsTwo`/`panel.columnsOne` (`:538–552`); `resolveDrop`/
  `resolveLocalDrop`/`resolveIconicDrop` (`:1054–1249`) resolve within the
  column set but have no whole-column any-side target; `updateDrag`/`commitDrop`
  (`:1392–1411`, `:1413`) create the float and commit on the same press path;
  the blue indicators are `panelDropIndicator`/`panelStripDropIndicator`
  (`:144–156`).
- `crates/pictura-app/cpp/panels/panel_group.{h,cpp}`: `panelTabBar` with elide
  right and no expanding (`:323–325`); `addPanel` with no explicit first-panel
  activation (`:348–362`); the `▾` corner button and close chevron.
- `crates/pictura-app/cpp/theme.cpp`: the scoped `#panelTabBar::tab` rules
  (`:174–176`), the unscoped document `QTabBar::tab` rules (`:166–169`), the
  `${border}` token (`window.darker(135)`, `:131`), `QTabWidget::pane`
  (`:178`), and the `#documentTabs`/`#documentTabBar` names set in
  `frame.cpp:75–76`.
- `crates/pictura-app/cpp/toolbox.cpp`: `topLevelChanged` resizes to
  `sizeHint().height()` (`:457–462`); the fixed width is `setFixedWidth(content)`
  (`:528–533`); the M40 allowed areas are left/right.
- `crates/pictura-app/cpp/frame.cpp`: `documentTabs`/`documentTabBar`
  (`:74–76`), the default groups (`:1310–1368`).
- `crates/pictura-app/cpp/main.cpp`: the M43 checks end at **151**; `--headless`
  asserts the offscreen platform and exits **152** on a mismatch (`:166–171`);
  M44 allocates **153–166**.
- `crates/pictura-app/src/cxxqt_object.rs`: `new_document` (`:812–897`) fills
  the document white and sets `view.image = buffer_to_image(&rendered)` where
  `rendered = current_buffer(&doc, gpu_compute)` and `gpu_compute` defaults
  `true` (`:767`); `buffer_to_image` copies into a fresh `Vec` RGBA and wraps it
  (`:3677–3710`); the M17 self-test already asserts the initial pixel is white
  (`main.cpp:764–772`), so the failure is the first **present**, not the buffer:
  `composite_gpu`/`composite_gpu_region` clear a canvas, dispatch a compute
  kernel, then read it back (`gpu.rs:99–127`, `:863–1102`).
- **No** darker panel border, `documentTabBar` border rules, corrected chevron
  mapping, corrected tab-colour direction, float-drag continuation, any-side
  widget-column dock target, compact drag handle, group-relative compact drop,
  or elide-as-soon-as-room exists today.

## 3. Frozen design

### 3.1 New-document canvas (E1)

**Root cause (Phase A, measured).** The garbage is not in the composite or
readback seam. `PictureView::render_gpu` (`cxxqt_object.rs`) ran the M0.5
offscreen demo (`render_gradient`: `fract(x/97), fract(y/89)` — the reported
repeating black/white/green/red bands) and wrote it straight into
`rust.image`, without touching `rust.doc` and without marking
`display_dirty`. On startup with no PSD, `main.cpp` creates a white scratch
document and calls `render_gpu`, so `frame.refresh()` presented that gradient
while the document composite stayed white; the next recomposite derived the
display from `doc.composite` and corrected it — hence "moving a layer turns it
white". The initial `new_document` buffer itself is correct (M17 already asserts
its pixel is white), and a fresh-process Rust probe composited white documents
up to 1000×1000 uniformly white on the first and second GPU call, so no
stride/readback fault exists. The fix is at the writer: `render_gpu` is now a
pure GPU smoke probe that reports non-blank and never mutates the document
display image.

Diagnose first: reproduce with `--headless --self-test` and an
`xvfb-run`/GPU run; compare `view.image` immediately after `new_document` with a
second `current_buffer`/`buffer_to_image` and with `doc.composite`. If the first
GPU composite differs from the second, the root cause is the GPU first-use path
(uninitialized/stale canvas or an out-of-order clear/submit/readback); if the
buffer is already white, the root cause is the present/Interop path. Record the
named root cause before editing. Fix the seam, not a caller. The regression is
one Rust test that composites a fresh white document twice and asserts both
frames are uniform opaque white and equal (GPU arm skipped without an adapter),
plus `m44_newdoc` (153). The M17 pixel check stays as the lower-layer guard.

### 3.2 Scoped theme by `objectName`

Give `PanelColumn` a container name (`panelColumnContainer`), keep
`panelTabBar`, and add `panelGroupFrame`; keep `panelIconFlyout`,
`panelFloat`, and `panelIconDivider`; scope the document tab strip with
`QTabWidget#documentTabs`/`QTabBar#documentTabBar`. All borders use the existing
darker `${border}` at a fixed constant. No global `QFrame`/`QWidget` rule.

### 3.3 Widget presentation parity

`PanelGroup` is shared by the docked group, the compact flyout, and the float;
M44 drops the flyout's inline stylesheet and gives all three the same scoped
selectors, margins, and borders, so a widget looks the same in every host.

### 3.4 Chrome corrections

- **Chevrons (W1):** swap the two `panel.columns*` icons so each mode shows the
  icon for the action it performs.
- **Active tab (W2):** find which artefact reads last-active and fix it so the
  first panel is active on a fresh session.
- **Tab colours (W3):** swap the scoped rules so `#panelTabBar::tab:selected` is
  the pane `${base}` and `#panelTabBar::tab` is `${window}`/`${hover}`; correct
  the M43 `m43_tabcolors` direction in place.
- **Group divider (W6) and compact divider (C2):** a fixed thicker darker
  splitter handle, and the compact divider re-pointed from white to `${border}`.

### 3.5 Float drag continues to release (W4)

Keep the drag alive after `createFloat`: `updateDrag` keeps moving the float with
the cursor, and the commit/re-dock happens only on `dragFinished` (release) or is
cancelled on `dragCanceled`. Same lifecycle as the group/column drag; not a new
drag system.

### 3.6 Any-side docking and compact placement (W5, T2, C3)

Extend `resolveDrop`/`resolveLocalDrop`/`resolveIconicDrop` and route
`commitDrop` through the existing `DropTarget` kinds and the single indicator. A
whole widget column resolves to a new-column/boundary target on any side of the
Tools toolbar, another widget panel/column, and the workspace; it is never
tabified. In compact mode, onto/just-above/just-below a group inserts into it,
and between/beyond groups creates a new boundary group. A **drag-handle dots**
widget above each group drags the whole group, and each icon group gets one
shared background/border. Remove-then-insert, never double-parent, and
empty-group/column cleanup stay.

### 3.7 Tools float height and dock sides (T1, T2)

While floating, the dock's height is fixed to its content and is not
drag-resizable (no size grip), keeping the M42 trailing-stretch behaviour. The
allowed areas widen from left/right to all four main-window dock areas; the
fixed content-size rule becomes fixed width for a left/right dock and fixed
height for a top/bottom dock; the M40 no-tabification fallback stays. The
interpretation of "beside a custom `PanelColumn`" is the main-window dock areas
(recorded as an honest limit).

### 3.8 Strip label eliding

Labels elide and appear as soon as the row has any room beyond the icon button,
rather than only at the fixed `>= 120 px` threshold; use `QLabel`/`QFontMetrics`
elide to the available row width.

## 4. Frozen self-test exit codes (153–166)

| Code | Check | Covers |
|---|---|---|
| 153 | `m44_newdoc` | E1 |
| 154 | `m44_chevrons` | W1 |
| 155 | `m44_defaultactive` | W2 |
| 156 | `m44_tabswap` | W3 |
| 157 | `m44_floatdrag` | W4 |
| 158 | `m44_docksides` | W5, T2 |
| 159 | `m44_divider` | W6 |
| 160 | `m44_popupstyle` | C1, S2 |
| 161 | `m44_compactdivider` | C2 |
| 162 | `m44_compactdrop` | C3 |
| 163 | `m44_draghandle` | C3 |
| 164 | `m44_elide` | C4 |
| 165 | `m44_filebar` | F1, F2 |
| 166 | `m44_panelborder` | S1, S2 |

Exit **152** is reserved for the headless platform mismatch; M44 allocates from
**153**. Suggested hooks: `collapseIconRoleForTest`,
`defaultActiveTabForTest`, `panelTabColoursForTest`,
`groupDividerMetricsForTest`, `floatDragTrackingForTest`, `dockSidesForTest`,
`toolboxFloatHeightForTest`, `compactDividerColourForTest`,
`compactDropKindForTest`, `dragHandleForTest`, `stripLabelElideForTest`,
`documentTabStripBordersForTest`, `panelBorderForTest`. Each check pumps the
event loop bounded, forces layout so geometry is real offscreen, and
`std::fflush(stderr)`.

## 5. Honest limits

- The E1 root cause may be GPU-vendor-specific; the fix targets the seam the
  evidence names, the Rust regression skips the GPU arm without an adapter, and
  the app check asserts the invariant regardless of backend.
- W2 is an investigation with three candidate artefacts; the check asserts the
  first tab/panel is active on a fresh session, and any remaining stored-page
  oddity is a separate fix.
- "Dock to any side of the widget panels" is frozen as the four main-window dock
  areas; a column-relative grammar is a later change.
- Border thickness, divider weight, drag-handle size, elide behaviour, and the
  compact drop band are chosen constants, not sourced CS6 metrics.
- Forcing layout offscreen can yield zero geometry; every check forces a layout
  and pumps before reading, and fails loudly rather than passing vacuously.
- The panel-tab colour swap is scoped to `panelTabBar`; the document tab strip's
  active colour is deliberately unchanged.
- Untested dock permutations (several columns on one side with a column drag in
  flight) remain best-effort, as in M43.

## 6. Verification

- `openspec validate m44-panel-theme-polish --strict` and
  `openspec validate --all --strict`.
- `git status --porcelain` shows docs + OpenSpec changes only for this proposal.
- Implementation (later) is verified by `cmake --build build`, both app
  self-tests, `cargo fmt`/`clippy`/`nextest`/doc (the E1 Rust regression runs),
  and the `m44_*` steps at codes 153–166.

## 7. Layers-panel program shift (record only)

The M40–M43 interruptions pushed the Layers-panel program back by four; this
sixth pass pushes it once more. The renumbered program is:

- **M45** — layer filtering/search
- **M46** — remaining layer management operations
- **M47** — layer styles / effects
- **M48** — smart objects / vector masks / artboards / layer comps

STATE's program line is updated at close-out; this brief is the record.

## 8. Non-goals

Placeholder-panel content, workspace presets, pixel-exact CS6 metrics,
multi-monitor floating, OS-window float chrome, per-column compact modes,
persisted float positions, a second drag/drop system, implementing any disabled
per-panel menu entry, and any change to the tool catalogue, the M40
flyout/shortcut/dock contracts beyond the allowed dock sides, the M41 tab menu
list, or the M42 per-widget menu contents. No new dependency, no PSD/session/on-disk
format change, and no bridge-ABI change.
