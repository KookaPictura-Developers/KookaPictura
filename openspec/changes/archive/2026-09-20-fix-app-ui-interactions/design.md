## Context

The sixteen items are confirmed root causes, not hypotheses; this design records
how to fix each shared piece and how to test the aggregate. Most of the work is
C++ in `crates/pictura-app/cpp/`; four pieces reach Rust:

- the per-column rail-mode persistence lands in `session.{h,cpp}` +
  `frame_session.cpp`;
- the active-layer resolver and the alpha-to-selection bridge op land in
  `crates/pictura-app/src/cxxqt_object/`;
- per-pixel transparency enforcement and the nesting destination guard land in
  `crates/pictura-render/` (paint write path, layer-ops, filter entry);
- the region compositor and incremental dirty land in
  `crates/pictura-render/src/gpu/mod.rs` and `crates/pictura-paint/src/stroke.rs`.

Current state that the decisions build on:

- `SessionState::panelRailMode` is one global string (`cpp/session.h:27`)
  derived from the primary column (`cpp/frame_session.cpp:63-64`); v7
  per-column entries carry only `width` (`:79-88`) and restore stamps the global
  onto every column (`:210-215`).
- `displayName = QFileInfo(path).baseName()` (`cpp/frame.cpp:424,445`) and
  `documentName()` prefers it (`:271-272`).
- `finalize_import` sets `LockFlags::all()` (`impl_core.rs:25`); Photoshop's
  Background locks transparency and position only, pinned to the bottom by the
  `background` flag.
- `canvas_scrollbars.cpp:93-103` hides a bar when the document fits, with a
  two-pass loop that exists only to settle visibility.
- `LayersModel` (`cpp/panels/layers_panel_internal.h:145-360`) implements no
  `mimeTypes()`, `canDropMimeData()`, or `supportedDropActions()`, so
  `QAbstractItemViewPrivate::canDrop()` is false and `dragMoveEvent` never sets
  `dropIndicatorPosition`; `dragEnterEvent` (`:423-430`) never calls
  `setState(DraggingState)`.
- Tool edits ignore the panel's active layer and default to the topmost raster
  (`helpers.rs:593`, `stroke.rs:45-46`, `impl_filters.rs:55`,
  `impl_transform.rs:757`, `impl_selection.rs:143`).
- `TRANSPARENCY` is a blanket refusal at `stroke.rs:52` and `move_content.rs:35`;
  `composite_pixel`/`write_pixel` overwrite alpha (`stroke.rs:196-231,310-326`);
  `filter.rs` never checks it.
- `refreshCursor` still sets a `tool.brush`/`tool.pencil` cursor
  (`cpp/tools.cpp:565-568`); the ring is separate (`cpp/tools_marquee.cpp:126-136`
  → `image_view.cpp:818-828`, painted `:547-558`).
- `paintEvent` clips to the image rect before drawing the ring
  (`image_view.cpp:507` clip, `:547` ring).
- `Stroke::dirty()` never clears (`crates/pictura-paint/src/stroke.rs:139-141`);
  `composite_cpu_region` fully composites then slices
  (`crates/pictura-render/src/gpu/mod.rs:185-205`); every region blit invalidates
  the whole present cache (`image_view.cpp:148`).
- No Space handling exists (`image_view.cpp:782-797`, `frame.cpp:857-895`), and
  `new QShortcut(QKeySequence(Qt::Key_Space, Qt::Key_F), this)` (`frame.cpp:152`)
  makes Qt treat Space as a sequence prefix.
- The status bar (`cpp/frame_build.cpp:326-358`) already has `hintLabel_` fed by
  `updateToolHint` (`frame.cpp:924-935`) with a flat per-tool string.
- Self-test codes are append-only; the highest in use is **323** and
  `selftest.cpp` is 6729 LOC at its 6730 allowlist ceiling.

## Goals / Non-Goals

**Goals:**

- Encode each of the sixteen fixes as an independently testable requirement.
- Build the shared pieces once: per-column rail mode, one active-layer resolver,
  one per-pixel transparency write path, one ring/cursor policy, one status-bar
  hint widget.
- Keep the self-test exit-code identity append-only and every `selftest*.cpp`
  inside its `scripts/file-size-allowlist.txt` ceiling.
- Correct the three prior-spec errors (tab extension, Background locks,
  scrollbar hiding) with explicit BREAKING notes.
- Meet the `docs/dev/canvas-view-spec.md` budget for a 4000² brush: ≤ 16 ms
  input-to-first-pixel per dab and ≥ 60 FPS sustained redraw.

**Non-Goals:**

- No `docs/` change; the canvas-view budget is cited, not edited.
- No new crate, dependency, or Layer Style dialog.
- No artboards or frames: item 8's nesting semantics apply to groups only.
- No re-investigation of the confirmed root causes.

## Decisions

### D1. Per-column rail mode at schema v8; the global is a legacy seed

Each `panelColumns` entry gains a `railMode` string (`"normal"`/`"iconic"`). The
schema advances v7 → v8. Save reads each column's actual mode; restore stamps
each column's own `railMode` and stops applying the primary column's value to
every column. The top-level `panelRailMode` is kept and used only to seed a store
whose per-column entries lack `railMode` (a v7 store), preserving today's
behavior on first load. Rejected: keeping the single global and deriving the
primary column's value on restore — that is exactly the observed loss.

### D2. One active-layer resolver in `pictura-app`

A single resolver in `crates/pictura-app/src/cxxqt_object/` returns the
exactly-one active layer path for tool edits, or a typed refusal. The layers
panel pushes its selection into the view on selection change, and `open_image`
selects the imported layer so a non-PSD raster auto-activates its Background.
Paint, filter, Free Transform, and content move all resolve through it; the
existing "topmost raster" fallback is removed from those entry points. Zero or
multiple selected layers refuse the edit with the same typed error the lock path
uses, surfaced to the user. Rejected: a per-tool heuristic — it is the drift the
item names.

### D3. Per-pixel transparency lock, not a blanket refusal

`TRANSPARENCY` stops being a blanket refusal. The write path carries each
pixel's original alpha: an edit changes RGB where the original alpha is greater
than zero and writes the original alpha back unchanged; a fully transparent
pixel is untouched. `composite_pixel`/`write_pixel` and the region compositor
preserve alpha. `filter.rs` applies the same rule per pixel. A `Clear` paint
mode or an erase, which would lower alpha, remains refused because it is an
alpha mutation, not a color edit. Rejected: per-operation allow-lists — one
per-pixel rule covers paint and filters uniformly.

### D4. One cursor/ring policy: blank cursor plus a drawn ring

For Brush and Pencil the view sets `Qt::BlankCursor` (not an unset cursor) so
only the drawn ring is visible; the ring is the sole size affordance. The ring is
painted with the full-widget clip: `paintEvent` scopes the document clip with
`save()`/`restore()` so the ring draws outside the image rect while the existing
image-space transform still positions it. Rejected: a rebuilt cursor pixmap
(OS caps near 128–256 px) and clipping the ring to the image (it disappears at
the canvas edge).

### D5. Space is a temporary Hand mode, not a shortcut

Holding `Space` sets a temporary pan state: the cursor becomes `Qt::OpenHand`
and drags pan the canvas even while another tool is active; release restores the
previous tool's cursor and pan state. The `Qt::Key_Space, Qt::Key_F` shortcut
that makes Qt treat Space as a sequence prefix is reworked so Space is not
consumed as a modifier prefix (the `F`-key binding is kept without the Space
chord). The Hand tool itself is unchanged; Space is a transient override on top
of it.

### D6. Status-bar hint widget fed by per-tool keycaps

The flat `hintLabel_` is replaced by a small hint widget: each entry is a
bordered keycap plus a description. Per-tool content comes from a table keyed by
the active tool (`tools_->activeTool()`); selection tools show `Shift` = Add to
selection and `Alt` = Subtract, matching Adobe's documented behavior. Command
shortcuts come from `CommandRegistry::action(id)->shortcut()` and tool letters
from the existing `toolShortcutKeys()` (`tools.cpp:225-235`); no second shortcut
source. A key press highlights the matching keycap. Rejected: a rich-text label
(no per-keycap border or highlight) and a second shortcut table.

### D7. Model virtuals plus the dragging state fix the drop

`LayersModel` implements `mimeTypes()`, `canDropMimeData()`, and
`supportedDropActions()` so `QAbstractItemViewPrivate::canDrop()` is true and
`QTreeView::dragMoveEvent` sets a real `dropIndicatorPosition`. `dragEnterEvent`
calls `setState(DraggingState)` so the indicator paints. With the indicator
position set, `dropTargetFor` resolves the above/below (reorder) modes instead of
always mode 2 (into), which `move_path_to_dest` accepts only for groups. The
validator keeps rejecting invalid targets before commit.

### D8. Region compositing and incremental dirty for brush latency

Three changes: `Stroke::dirty()` returns and clears the per-dab dirty rectangle
instead of accumulating the whole stroke; `composite_cpu_region` composites only
the requested region rather than full-compositing and slicing; and the present
cache updates only the blush region instead of invalidating the whole scaled
image. The acceptance budget is the canvas-view spec's ≤ 16 ms per dab and ≥ 60
FPS sustained redraw, measured on a 4000² document.

### D9. Test strategy: one new `selftest_*.cpp` per batch, codes from 324

Each batch adds a `crates/pictura-app/cpp/selftest_<batch>.cpp` translation unit
registered in `CMakeLists.txt` and invoked from `runSelfTest()`. New failure
codes start at **324** (highest in use is 323) and are append-only. `selftest.cpp`
(6729 LOC, ceiling 6730) does not grow; new checks live in the new files. Rust
logic gets unit tests in the owning crate:

- persistence: `selftest_ui_persistence.cpp` (per-column rail mode round-trip and
  the v7 seed);
- layers: `selftest_layers_interactions.cpp` (drag state, Background convert,
  thumbnail selection, nesting refusal);
- gating: Rust tests for the resolver and per-pixel transparency plus
  `selftest_layer_locks.cpp` additions;
- cursor/ring/hints: `selftest_tool_canvas.cpp` (blank cursor, ring outside
  clip, Space pan, hint keycaps);
- performance: `selftest_paint_latency.cpp` (per-dab dirty and region budget)
  plus a Rust region-compositor test.

No golden baseline changes. `scripts/file-size-allowlist.txt` is updated only if
a new file approaches a cap.

## Risks / Trade-offs

- **A v7 store with a global `panelRailMode` but per-column widths** → D1: a
  missing per-column `railMode` seeds from the global, so the first load matches
  today and the next save writes per-column values.
- **The active-layer resolver could refuse a legitimate edit** → D2: the resolver
  only requires exactly one selected layer; the existing topmost-raster fallback
  is removed only from tool entry points, and the refusal is the same typed error
  the lock path already surfaces.
- **Per-pixel transparency could leak alpha through a resample** → D3: the write
  path copies the original alpha for every touched pixel and a bit-identity test
  asserts it; `Clear`/erase stay refused.
- **The ring outside the clip could paint over panels** → D4: it is still painted
  inside the canvas widget; only the document clip is scoped, so it cannot leave
  the widget.
- **Space as a pan key could conflict with shortcuts** → D5: the only conflict is
  the `Space, F` sequence prefix, which is removed; Space is handled as a
  transient key state, not a global shortcut.
- **The hint strip could crowd the status bar** → D6: it is one row of compact
  keycaps and falls back to the existing text hint when a tool has no table row.
- **Region compositing could change pixels at region seams** → D8: the region
  compositor must produce bit-identical results to the full composite; a test
  compares a region composite against the full composite on the same input.
- **`selftest.cpp` at its ceiling** → D9: no check is added there; all new checks
  go to the new files.

## Migration Plan

- **No document format change.** PSD/PSB read/write is untouched.
- **Session store v7 → v8.** Add `railMode` per column entry; a missing field
  seeds from the legacy global `panelRailMode`. Rollback is reverting the commit:
  a v8 store's extra `railMode` key is ignored by the old reader because unknown
  keys survive a load-then-write cycle.
- **No `docs/` change and no new dependency.** The change is reversible commit by
  commit, one batch at a time.

## Open Questions

- **Exact nesting-drop rule.** Photoshop's "prevent auto-nesting" covers artboards
  and frames; no artboards exist yet. This change applies the rule to groups: a
  drop into a nesting-locked group or out of a nesting-locked parent is refused.
  Whether a future artboard inherits the same flag is left to that capability.
- **Hint-bar keycap set per tool.** The required selection-tool hints are
  `Shift`/`Alt`; the full per-tool table (crop, move, paint modifiers) is settled
  during Batch 4 against Adobe's documented shortcuts and does not change the
  requirement.
- **Region-composite seam tolerance.** The test asserts bit-identity; if a seam
  differs only by a documented clamp, the check is relaxed to the compositor
  tolerance rather than changing the requirement.
