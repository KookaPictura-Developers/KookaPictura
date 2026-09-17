## Context

M41 (archived) replaced the right-hand `QDockWidget` area with a custom
`PanelColumn` (`crates/pictura-app/cpp/panels/panel_column.{h,cpp}`): a vertical
`QSplitter` of `PanelGroup : QWidget` top-tab groups in a `QScrollArea`, a
`panelColumnToggle`, an iconic icon strip, `Qt::Popup` flyouts, and tab
drag/regroup/insert with a blue drop indicator. M42 (archived) refined it:
bounded normal minimum width, compact-strip transition, a group-styled flyout,
per-widget header menus, and the in-window `PanelFloat` overlay. M43 (archived)
made the central `QSplitter` an ordered multi-column host, generalised
`DropTarget`/`resolveDrop`, scoped the panel tab bar as `panelTabBar`, fixed the
Tools dock width, and moved the session to schema v6.

M44 is the sixth user-requested pass. It is mostly chrome, with one engine bug.
The relevant current state:

- `panel_column.cpp`: the compact strip builds a `panelIconDivider` `QFrame`
  (`:582`) and per-panel `panelIconRow` widgets (`:593`); label visibility is a
  single `iconStrip_->width() >= kIconLabelWidth` (120) gate (`:615–627`);
  `ensureFlyout` hard-codes an inline `#panelIconFlyout` stylesheet
  (`:659–661`) that does not match the docked group; `resolveDrop`/
  `resolveLocalDrop`/`resolveIconicDrop` (`:1054–1249`) resolve within the
  column set but have no cross-presentation group-relative targets and no
  whole-column any-side docking of a widget column; `updateDrag` creates the
  float and the tab's mouse handler then commits on the same press path
  (`:1392–1411`, `:920–940`); `updateColumnToggle` picks
  `panel.columnsTwo`/`panel.columnsOne` (`:538–552`).
- `panel_group.cpp`: `tabs_->tabBar()->setObjectName("panelTabBar")`, elide
  right, no expanding (`:323–325`); `addPanel` has no explicit first-panel
  activation (`:348–362`); the `▾` corner button and close chevron.
- `theme.cpp`: the scoped `#panelTabBar::tab` rules (`:174–176`), the unscoped
  `QTabBar::tab` document rules (`:166–169`), the `${border}` token
  (`window.darker(135)`, `:131`), and `QTabWidget::pane` (`:178`).
- `toolbox.cpp`: `topLevelChanged` resizes to `sizeHint().height()` (`:457–462`)
  but the float height is still drag-resizable; the fixed width is
  `setFixedWidth(content)` (`:528–533`); the M40 allowed areas are left/right.
- `frame.cpp`: `tabs_` (`documentTabs`) and its `documentTabBar` (`:74–76`); the
  default groups (`:1310–1368`).
- `cxxqt_object.rs`: `new_document` (`:812–897`) fills the document white and
  sets `view.image = buffer_to_image(&rendered)` where
  `rendered = current_buffer(&doc, gpu_compute)` and `gpu_compute` defaults
  `true` (`:767`); `buffer_to_image` copies into a fresh `Vec` RGBA and wraps it
  (`:3677–3710`); `composite_gpu`/`composite_gpu_region` clear a canvas and
  dispatch a compute kernel then read it back (`gpu.rs:99–127`, `:863–1102`).
  The M17 self-test already asserts `fresh->image().pixel(1,1)` is white
  (`main.cpp:764–772`), so the failure is not the in-memory buffer: it is what
  the canvas presents on first paint and what a recomposite later corrects.

Constraints: this change is docs/proposal only. No `.rs`, `.cpp`, `.h`,
`CMakeLists.txt`, or asset change lands here; `docs/` stays untouched except the
new brief. No dependency is added. The frozen interfaces live in
`docs/dev/m44-panel-theme-polish.md`.

## Goals / Non-Goals

**Goals:**

- Diagnose and fix the new-document canvas garbage frame at its seam, with a
  named root cause and a runnable regression check.
- Correct the widget-panel chrome: inverted collapse chevrons, default active
  tab, active/inactive tab colours, float-drag continuity, group divider, and
  panel/column borders.
- Make a widget look identical docked, in a compact flyout, and in a float.
- Improve compact drag-and-place (into/between/above/below, group drag handle,
  visual grouping) and elide strip labels as soon as room exists.
- Widen the Tools dock and a whole widget column to any side, reusing the M43
  drop grammar and the one blue indicator.
- Fix the Tools float height and add the document tab-strip borders.
- Ship runnable checks: the `m44_*` self-test steps at exit codes **153–166**,
  headless and layout-forced.

**Non-Goals:**

- Real content for the placeholder panels (Styles, Properties, Gradients,
  Patterns, Libraries) — they stay empty states.
- Pixel-exact CS6 metrics; border thickness, divider weight, drag-handle size,
  and elide behaviour are chosen constants.
- A second drag/drop system; the M41–M43 `DropTarget`/`resolveDrop`/`commitDrop`
  path and its one `#2a7fff` indicator stay the only grammar.
- Persisting a float's position, a per-column compact mode, or multi-monitor /
  OS-window float chrome.
- Changing the tool catalogue, the M40 flyout/shortcut/dock contracts beyond the
  allowed dock sides, the M41 tab menu list, or the M42 per-widget menu
  contents.
- The Layers-panel program (filtering/search, management, styles/effects, smart
  objects) — it shifts again behind this pass (see §7 of the brief).

## Decisions

### 1. New-document render: diagnose empirically, fix at the seam, pin once (E1)

The M17 check reads the in-memory `PictureView::image()` and already passes, so
the garbage is in the first **present** path, not in the document buffer. M44
diagnoses before it edits:

**Root cause (Phase A, measured).** `PictureView::render_gpu` wrote the M0.5
offscreen demo (`render_gradient` — the repeating black/white/green/red bands)
directly into `rust.image` while leaving `rust.doc` as the white document and
`display_dirty` clear. The startup scratch document therefore presented the
gradient until the next recomposite rebuilt `image` from `doc.composite`. The
composite/readback seam is sound: a fresh-process probe composited white
documents (4×3 … 1000×1000) uniformly white on both the first and second GPU
call. The fix is at the writer — `render_gpu` becomes a non-blank probe that
never mutates the document display image.

1. Reproduce under `--headless --self-test` and under `xvfb-run` with a real
   GPU adapter, capturing the first present.
2. Compare `view.image` immediately after `new_document` against
   `buffer_to_image(&current_buffer(...))` recomputed a second time, and
   against `doc.composite`. If the first `composite_active`/`composite_gpu`
   call differs from the second, the root cause is the **GPU first-use path**
   (an uninitialized/stale canvas or an out-of-order clear/submit/readback); if
   the buffer is already white but the canvas paints garbage, the root cause is
   in the **present/Interop path**.
3. Record the root cause in the brief before fixing.

The fix lands at the seam the evidence names — the composite/readback ordering
in `gpu.rs`, `current_buffer`/`store_composite`/`buffer_to_image` in
`cxxqt_object.rs`, or the Interop present. The regression is one Rust test that
composites a fresh white document twice and asserts both frames are uniform
white and equal (skipping the GPU arm when no adapter exists), plus the app
self-test `m44_newdoc` (exit **153**) asserting the initial `image()` is
uniform white with no garbage frame. The M17 pixel check stays as the
lower-layer guard.

*Alternative considered:* patch the present path to re-composite until white.
Rejected: that hides an uninitialized/stale frame and leaves any other
first-use composite wrong.

### 2. Theme is scoped by `objectName`; borders reuse `${border}` (frozen)

Every new rule is scoped to the panel family so the document chrome is not
collateral damage: `PanelColumn` gets a container `objectName`
(`panelColumnContainer`), `PanelGroup` keeps `panelTabBar` and gains a
`panelGroupFrame`, the flyout keeps `panelIconFlyout`, the float keeps
`panelFloat`, and the compact divider keeps `panelIconDivider`. Borders are the
existing darker-grey `${border}` (`window.darker(135)`), used at a fixed
thickness constant; the compact strip's white divider is re-pointed from the
default `QFrame` colour to `${border}`. The document tab strip gets its own
scoped selectors (`QTabWidget#documentTabs` / `QTabBar#documentTabBar`): a
right border and no top border (the options bar already draws a bottom border).

*Alternative considered:* a global `QFrame`/`QWidget` border rule. Rejected: it
would repaint unrelated content widgets and menus.

### 3. Widget presentation parity: one style, three hosts (frozen)

The docked group, the compact flyout, and the float already share
`PanelGroup`; `PanelFlyout` and `PanelFloat` wrap it. M44 removes the flyout's
inline stylesheet and instead gives all three hosts the same scoped
`panelGroupFrame`/`panelTabBar` selectors plus the same margins, so the flyout
presents the same tab bar, background, and borders as the docked panel. A single
`objectName` family drives it; no per-host colour literals remain.

*Alternative considered:* keep a separate flyout stylesheet and sync values by
hand. Rejected: it is the source of the drift this item reports.

### 4. Correct the chrome defaults rather than add states (frozen)

- **Chevrons (W1).** The collapse toggle currently shows `panel.columnsOne` in
  normal mode and `panel.columnsTwo` in iconic mode. The two icons are swapped:
  the button SHALL show the icon for the action it performs (collapse-to-icons
  in normal, expand in iconic). One constant swap in `updateColumnToggle`; the
  `m44_chevrons` check asserts the icon's logical role, not a pixel.
- **Active tab (W2).** M44 first identifies which artefact reads last-active
  (the group's default `currentIndex`, the column's default current panel, or a
  stored page index) and fixes that one so the **first** panel is active on a
  fresh session. The `m44_defaultactive` check asserts the first tab of every
  default group is current and the `PanelColumn`'s current panel is the first
  visible one.
- **Tab colours (W3).** The user reports the active/inactive backgrounds are
  reversed. M44 swaps the two scoped rules so `#panelTabBar::tab:selected` is
  the pane/widget `${base}` and `#panelTabBar::tab` is `${window}`/`${hover}`,
  and the `m44_tabswap` check asserts the active tab equals the pane colour and
  the inactive differs (the M43 `m43_tabcolors` assertion is corrected in
  place).
- **Group divider (W6).** The splitter handle between groups gets a fixed
  thicker width and the darker `${border}` colour. The `m44_divider` check
  asserts the handle width grew and its colour is the darker grey.

*Alternative considered:* new toggle states or per-brightness overrides.
Rejected: the reports are defaults being wrong, not missing states.

### 5. Float drag continues to mouse release (W4) (frozen)

Today the tab-bar press path creates the float on the drag threshold and then
commits on the same press, so the float stops at creation. M44 keeps the drag
alive after `createFloat`: `updateDrag` continues to move the float with the
cursor, and the tab/index is committed only on the group's `dragFinished`
(mouse release) or cancelled on `dragCanceled`. This is the same lifecycle the
group and column drags already use, so it is a routing change in
`PanelGroup::eventFilter`, not a new drag system. `m44_floatdrag` asserts the
float's geometry tracks successive `dragToForTest` moves and that no commit
happens before release.

*Alternative considered:* a separate float-drag loop. Rejected: two drag
lifecycles drift.

### 6. Any-side docking and compact placement extend the M43 resolver (frozen)

`resolveDrop` already resolves new-column-left/right, into-group,
above/below-group and on-strip, with one indicator. M44 adds:

- A **whole widget column** dragged to the left/right/top/bottom of the Tools
  toolbar, of another column/panel, or of the workspace resolves to a
  `new-column-*`/boundary target on that side, reusing `applyNewColumnDrop`/
  `applyGroupDrop`; a widget column is not tabified into a panel group.
- **Compact group-relative placement:** a drop onto a compact group or within a
  small vertical band just above/below it inserts into that group at that place;
  a drop between groups, above the top group, or below the bottom group creates a
  new group at that boundary. The compact strip builds a **drag-handle dots**
  widget above each group's icon row; dragging it drags the whole group. Each
  group's icon row gets a shared background/border so it reads as one unit.

The resolver stays single (`resolveDrop` + `resolveLocalDrop` +
`resolveIconicDrop`), the indicator stays one (`panelDropIndicator` /
`panelStripDropIndicator`), and `commitDrop` keeps remove-then-insert and
empty-group/column cleanup. `m44_dockSides`, `m44_compactdrop`, and
`m44_draghandle` cover the new targets.

*Alternative considered:* a per-presentation resolver. Rejected: it is the
second drag system the M43 design forbids.

### 7. Tools dock: fixed-size float, wider allowed sides (T1/T2) (frozen)

T1: while floating, the dock's height SHALL be its minimum content height and
SHALL NOT be drag-resizable (fixed height, no size grip), keeping the M42
trailing-stretch behaviour. T2: the allowed dock areas widen from left/right to
all four main-window dock areas so the toolbar can sit on any side of the
workspace/panel columns; the fixed content-size rule becomes fixed width for a
left/right dock and fixed height for a top/bottom dock, and the M40
no-tabification fallback stays. The exact grammar of "beside a custom
`PanelColumn`" is a chosen interpretation (main-window dock areas) recorded as
an honest limit. `m44_toolsfloat` asserts the float height does not change under
a simulated resize, and reuses the M43 `m43_tools` width assertion for the
left/right case.

*Alternative considered:* make the Tools panel a splitter pane like a
`PanelColumn`. Rejected: it would break the M40 dock contract and the fixed
content-size rule for a request the dock areas already cover.

### 8. Self-tests: bounded pump, forced layout, codes 153–166 (frozen)

The `m44_*` steps run through `crates/pictura-app/cpp/main.cpp` at codes
**153–166**, pump `QCoreApplication::processEvents()` a bounded number of times
before reading geometry/colour/visibility, and `std::fflush(stderr)`.
`--headless --self-test` asserts the offscreen platform and reserves exit
**152** for a platform mismatch (already landed), so M44 allocates from **153**.
Because offscreen widgets do not get show/resize events on their own, each check
forces a layout (`adjustSize`/`resize` + pump) before reading geometry. The
frozen table is in §4 of the brief and §11 below.

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

*Alternative considered:* reuse the M43 codes. Rejected: the user's headless
self-test already claims 152 and the checks must be independently addressable.

## Risks / Trade-offs

- **The E1 root cause may be GPU-vendor-specific.** → Diagnose on the reported
  setup first; the regression test skips the GPU arm when no adapter exists and
  the app check asserts the invariant regardless of backend; the fix is at the
  seam the evidence names, not a platform patch.
- **W2 is an investigation with three candidate artefacts.** → Fix the one that
  actually reads last-active on a fresh session and assert the first is active;
  if two artefacts are wrong, fix both and keep one check.
- **W5/T2 "any side of the widget panels" is ambiguous.** → The chosen reading
  is the main-window dock areas, recorded as an honest limit; a later change
  can add a column-relative grammar without a second resolver.
- **Forcing layout offscreen can give zero geometry.** → `adjustSize`/`resize`
  plus a bounded pump before every geometry read; checks assert non-empty rects
  and fail loudly rather than passing vacuously.
- **Theme borders can paint document chrome.** → Every new selector is scoped to
  the panel family and `documentTabBar`; `m44_filebar`/`m44_panelborder` assert
  the document tabs are otherwise unchanged.
- **Border/divider constants are unsourced CS6 metrics.** → A chosen constant in
  one place, marked `ponytail:`; a screenshot can retune them.
- **The tab-colour swap may look too close to the pane on some brightness.**
  → It is the user's explicit request; a later theme tweak changes one selector.

## Migration Plan

Additive and app-local. Rollback: restore the M43 collapse/expand icon mapping,
the M43 tab-colour selectors, the single-press float commit, the left/right-only
Tools dock areas and resizable float height, the fixed 120 px elide gate, and the
inline flyout stylesheet, and drop the new scoped selectors; the E1 fix is a
seam-local change. No document, codec, bridge-ABI, session, or on-disk change, so
no data migration is needed. Sequence: freeze the brief → Phase A (E1 diagnose +
fix + regression) → Phase B (theme/borders/parity) → Phase C (drag/dock) →
Phase D (`m44_*` self-tests and close-out prep).

## Open Questions

- **Which artefact reads last-active (W2)?** Fixed by investigation in Phase B;
  the check asserts the first tab/panel is active.
- **Does "dock to any side of the widget panels" mean the four main-window dock
  areas or a column-relative grammar?** Frozen as the dock areas for M44; a
  column-relative grammar is a later change.
- **Should the compact drag handle persist its order across a dock round-trip?**
  M44 reuses the existing session group order; a handle-specific order is not
  persisted separately.
- **Should the document tab strip keep its existing active-tab colour when the
  panel tab colours are swapped?** Yes — the swap is scoped to `panelTabBar`.
