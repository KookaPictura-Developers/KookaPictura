## Context

The nineteen investigated defects (`docs/dev/app-bugs-investigation.md`, commit
`7ee4805`) are local to the app shell, with two exceptions that reach the
engine: the opaque-import rule (issue 4) lands in the Rust import post-process,
and lock enforcement (issues 6/9) needs a predicate the renderer can consult.
Everything else is C++ in `crates/pictura-app/cpp/`.

The change is deliberately an **aggregation of small, independently committable
fixes**, not a new subsystem. Several fixes are one line (drag flags, `Ctrl+G`
handler) and several are generalizations of code that already exists and works
for one control (`PercentField`; the private `JumpSlider`). The guiding
constraint is therefore "build each shared primitive once, then route call sites
through it" — the investigation names that as the main drift risk.

Current state, relevant to the decisions below:

- `PercentField` (`crates/pictura-app/cpp/panels/percent_field.{h,cpp}`) already
  implements scrubbing, a slider popup, and an instance-local `JumpSlider` that
  forces `setSliderDown(true)` and tracks. Its popup never focuses the slider and
  handles no key events.
- `ImageView` owns `offset_`/`zoom_` as the only pan state; no writer clamps,
  and `offset_` is in logical widget pixels (100 % maps one image px to one
  logical px). The navigator reads `offset()` as authoritative.
- The layer tree's drag/drop pipeline, drop indicators, refusal guards, and the
  selection-aware `groupSelection`/`group_layers` op are all implemented and
  spec-backed; only the model flags and the menu handler bypass them.
- Session state is `$XDG_STATE_HOME/kooka-pictura/state.json`, schema v6
  (`cpp/session.h:33`). The only unconditional save is in `closeEvent`
  (`cpp/frame.cpp:782`); `File > Exit`/`Ctrl+Q` calls `qApp->quit()`
  (`cpp/frame_menus.cpp:126-133`), which never runs `closeEvent`.
- `Document::from_rgba` always emits one named alpha layer with default locks;
  the compositor already treats a missing `-1` channel as opaque.
- Self-test codes are append-only; the highest in use is **298**
  (`selftest_layers_adjustments.cpp`), and `selftest.cpp` is 6730 LOC, at its
  `scripts/file-size-allowlist.txt` ceiling, so it must not grow.

## Goals / Non-Goals

**Goals:**

- Fix the 17 in-scope issues as five batches, each with its own spec delta,
  tests, and commit boundary.
- Build the shared primitives once: `NumericField`, a hoisted `JumpSlider`, and
  one `offsetRangeFor` range helper.
- Keep `offset_` the single source of truth for pan; scrollbars and the clamp
  are projections/guards over it, never a second state store.
- Keep the self-test exit-code identity append-only and each `selftest*.cpp`
  file inside its allowlist ceiling.
- Migrate the session schema v6 → v7 with defaults-on-missing semantics.

**Non-Goals:**

- Arrow-key nudge (issue 7) and large-canvas paint latency (issue 8) — deferred
  (see below).
- Zoom/offset persistence (D2). The scrollbars are not restored from the store.
- Any `docs/` change; the investigation note is authoritative and untouched.
- New crates, dependencies, or a Layer Style dialog (issue 5 ships a documented
  placeholder).
- Rewriting `PercentField`'s API for external callers beyond making it a thin
  `NumericField` configuration.

## Decisions

### D1. Opaque import is a post-process in `open_image`, not a `from_rgba` change

`from_rgba` keeps its current contract (one named alpha layer), because
`image-import` and the Place path depend on it and the change would have a high
blast radius. The import post-process in `crates/pictura-app/src/cxxqt_object/impl_core.rs`
scans the decoded RGBA for a non-255 alpha byte:

- all-opaque → rename to `Background`, set `background = true`,
  `lock = LockFlags::all()`, drop the `-1` channel, so the composite already
  treats it as opaque; emits a background layer.
- any non-opaque pixel → leave the regular alpha layer, named from the file stem.

Rejected: changing `Document::from_rgba` (higher blast radius, contradicts the
existing `image-import` requirement), and always-Background (loses real
transparency).

### D2. Display name is separate from the save path

`DocEntry` (`cpp/frame.h`) gains a `displayName` set by `openImagePath` /
`openAsSmartObjectPath`, preferred by `documentName`, while `file_path()` stays
empty for an imported raster so Save semantics (Save As on an untitled buffer)
are unchanged. The tab title becomes `base + " (" + modeLabel + "/" + bits + ")"`
plus the dirty marker, using new bridge accessors `document_mode()` /
`document_depth_bits()` and label helpers. `Untitled-N` remains the fallback when
there is no display name. The dirty `*` stays where `updateTabTitle` puts it
today.

### D3. One `NumericField`; `PercentField` is a thin configuration

`NumericField` absorbs issues 13, 14, and 16: a scrubbing label, a spin box, and
a popup hosting the shared `JumpSlider`. It is configured with min/max/step/
decimals/suffix/int-vs-double/optional popup. `PercentField` becomes a thin
configuration (0–100, `%`, integer) so the Layers panel keeps its exact behavior
and the existing `lpc_*`/`lpr_*` checks still pass. The popup gains an explicit
`QEvent::KeyPress` branch mapping Left/Right to a one-step change and
Home/End/PageUp/PageDown to min/max/page, **and** focuses the slider with
`Qt::PopupFocusReason`; the explicit branch is the contract because a `Qt::Popup`
window's focus is not reliable. Scrub sensitivity is 1 unit/px with Shift/Ctrl
scaling. Call sites migrate only where the spec says a control actually moves:
the options bar (brush size/hardness/opacity/flow; selection tolerance/feather),
the New Document / New Layer dialogs, preferences brightness, and the color
panel axes.

Rejected: per-label event filters (duplication), and a `NumericSpinBox` that
adds only the popup (duplicates popup logic and leaves the scrub label).

### D4. The `JumpSlider` is hoisted, not replaced by a global proxy style

The private `JumpSlider` (`percent_field.cpp:21-71`) is moved to a shared header
and used by the navigator and the six color sliders. Rejected: a global
`QProxyStyle` overriding `SH_Slider_AbsoluteSetButtons` to include `LeftButton`.
It is a one-point fix but changes the feel of every slider app-wide and must
coexist with the Fusion theme setup (`cpp/theme.cpp:274`); the hoist is a known-
good, bounded blast radius already covered by `lpr_slider` (code 218).

### D5. Lock enforcement is a renderer predicate at the mutation entry points

`crates/pictura-render` gains a shared predicate (e.g. `layer_move_locked` /
`layer_pixel_locked`) and the mutation entry points call it:

- `POSITION`: the three `translate_layer*` functions and the move preview
  builder (move-tool drag), and content move (selection move).
- `PIXELS`: paint `Stroke::begin`, `apply_filter`, and fills.
- `TRANSPARENCY`: where an edit would change alpha (e.g. Clear/erase fills).

Refusal returns a typed error / `false`, and the app surfaces it (status/refusal
message and a forbidden cursor for pixel-editing tools). It is **not** placed in
`topmost_pixel_layer*`: that selector is shared with `apply_filter`, and a
position lock must not block pixel edits. A new `PaintError::Locked` variant
carries the refusal. Structural panel reordering (Move Up/Down) is deliberately
not blocked by `POSITION`.

### D6. Quit always saves via `QApplication::aboutToQuit`

`File > Exit`/`Ctrl+Q` keeps calling `qApp->quit()`, but the frame connects
`QApplication::aboutToQuit` to the save so every quit path persists. This was
chosen over routing `FileExit` through `close()` because `close()` can be vetoed
by a modal unsaved-changes prompt while `aboutToQuit` runs after the window is
closing and is the single choke point for all quit paths (window close, menu,
`Ctrl+Q`, session logout). Window `closeEvent` keeps its existing save; the
`aboutToQuit` connection is idempotent so a double save is harmless. Column
resize additionally triggers a debounced (single-shot timer) save so a crash does
not lose a width that was dragged.

### D7. Session schema v6 → v7 adds per-column width

Each `panelColumns` entry gains a `width`. On load, a missing `width` (v6 store)
loads the default; the legacy top-level `railWidth` seeds the primary column's
width so an existing store keeps its current width. The saved width is applied
after first layout with a zero-delay timer, and `PanelColumn` seeds
`normalWidthBeforeIconic_` from the restored width so switching out of iconic
restores the real width. Zoom/offset are not added (D2 of the proposal).

### D8. One `offsetRangeFor` serves the clamp and the scrollbars

A single helper computes the allowed `offset_` range for a given image size,
zoom, viewport, and reveal margin. `ImageView::clampOffset()` calls it at every
mutation point (`panBy`, `setZoom`, `centreImage`, `fitOnScreen`, `actualPixels`,
`applyInitialView`/`setImage`, `resizeEvent`, and the navigator proxy). The
margin is `kCanvasRevealMarginPx = 96.0` logical px (one display inch) with a
`ponytail:` note. Clamping is **per axis** (slide the minimum), not recentering,
so the artwork does not jump under the cursor. Read paths (`widgetToImage`,
`paintEvent`) do not clamp; `offset()` stays authoritative for the navigator and
tests. The new `canvas-scrollbars` container wraps `ImageView` and derives its
ranges and values from the same helper, emitting a `panChanged`/`viewChanged`
signal so the bars follow pan/zoom/fit/Navigator and hide (`ScrollBarAsNeeded`)
when the document fits. Rejected: `QAbstractScrollArea` (assumes a fixed content
size and fights the transform canvas and the navigator's use of `size()`).

### D9. Brush outline is painted in `ImageView`, not a cursor pixmap

`ImageView::paintEvent` draws a cosmetic-pen ring in image space under the
existing transform, driven by `setBrushOutline(diameter, imagePos)` and updated
from hover for Brush/Pencil. The ring is in image pixels, so zoom scales it for
free and a cosmetic pen keeps it 1 screen px and DPR-correct. A rebuilt
`QCursor` pixmap is rejected as primary: OS cursor pixmaps cap near 128–256 px,
so a large brush or high zoom cannot be a cursor. `setBrushSize` gains a signal
so the options-bar spin box and `[`/`]` resync each other.

### D10. Test strategy: one new `selftest_*.cpp` per batch, codes from 299

Each batch adds one `crates/pictura-app/cpp/selftest_<batch>.cpp` translation
unit registered in `CMakeLists.txt` and invoked from `runSelfTest()`. New codes
start at **299** (highest used is 298) and are append-only; `selftest.cpp` (6730
LOC, at its allowlist ceiling) does not grow — new checks live in the new files.
Rust-side logic (the lock predicate, the opaque-import post-process) gets unit
tests in the owning crate. Issue 18 changes existing assertions because a pan
past the margin now clamps: `canvas_centre_offset` (exit 64) and
`canvas_middle_pan_delta` (exit 65) in `selftest.cpp`, and the `zoom/pan
transform wrong` check at `selftest.cpp:6702` (the `panBy(10, 5)` delta
assertion) must be rewritten to assert the clamped value. No golden baseline
changes. `scripts/file-size-allowlist.txt` is updated only if a new file
approaches a cap.

## Risks / Trade-offs

- **Two numeric implementations could drift** → D3: one `NumericField`, with
  `PercentField` as a thin configuration and the popup and scrub logic living in
  exactly one place.
- **Two clamp copies could drift** → D5/D8: the pan clamp and the scrollbar
  ranges call the same `offsetRangeFor`; no second range math.
- **`aboutToQuit` runs after some widgets are torn down** → the connection saves
  only scalar session state (layout bytes, widths, mode), captured while widgets
  are alive; it is idempotent with `closeEvent`.
- **Session v7 breaks an older reader** → a missing `width` loads the default and
  the legacy `railWidth` seeds the primary column; an unknown key survives a
  load-then-write cycle, preserving forward compatibility.
- **Suppressing Qt's built-in rename edit is fragile** → D (issue 5): use
  `setEditTriggers(NoEditTriggers)` plus a `MouseButtonDblClick` branch and a
  delegate `nameRect` hit-test; the placeholder `openLayerStyle` is a documented
  no-op until the dialog exists.
- **The lock predicate could block a legitimate pixel edit** → enforcement is not
  in `topmost_pixel_layer*`, which is shared with filters; only `PIXELS` gates
  pixel edits and only `POSITION` gates moves.
- **The brush ring could mis-scale under high DPR** → a cosmetic pen in the
  transformed paint path keeps it 1 screen px; asserted by a paint/size hook.
- **`selftest.cpp` at its ceiling** → no new check is added there; the two pan
  assertions are edited in place and the new checks go to a new file.

## Migration Plan

- **No document format change.** PSD/PSB read/write is untouched.
- **Session store v6 → v7.** Add `width` per column; on load a missing field
  loads the default and the legacy `railWidth` seeds the primary column. Rollback
  is reverting the commit: a v7 store with an extra `width` key loads under the
  old reader because unknown keys survive.
- **No `docs/` change and no new dependency.** The change is reversible commit by
  commit, one batch at a time.

## Open Questions

- **Exact `TRANSPARENCY` enforcement points.** The lock is round-tripped but the
  investigation found no consulted site; this change enforces it where an edit
  changes alpha (Clear paint mode, erase) and leaves any ambiguous site to the
  `layer-locks` spec wording. To be settled during Batch 4 against the CS6
  contract.
- **Layer Style placeholder shape.** Issue 5 lands a documented no-op
  `openLayerStyle(path)` until a dialog exists; whether it should instead be
  fully inert (no signal) is left to implementation and does not affect the
  requirement.
- **Scrollbar visibility threshold.** `ScrollBarAsNeeded` hides a bar when the
  document fits; whether "fits" uses the exact viewport or the viewport minus the
  margin is an implementation detail asserted by the round-trip check.
