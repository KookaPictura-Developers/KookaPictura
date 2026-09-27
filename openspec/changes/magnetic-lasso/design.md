# Design: magnetic-lasso

## Context

CS6's Magnetic Lasso cost function is closed. `docs/03-tools/lasso-selection.md`
recommends a live-wire (Intelligent Scissors) model with Width, Contrast, and
Frequency; photorust implements exactly that as a Sobel-magnitude cost field and
a bounded Dijkstra search, re-traced on every pointer move.

## Goals / Non-Goals

**Goals**

- Edge-following outlines with CS6's options and gestures: click to fasten,
  automatic fastening, close on the first point / double-click / Enter, Delete,
  Escape, `[` / `]`, and refusal on 32-bit documents.

**Non-Goals**

- Alt-drag / Alt-click temporary Lasso / Polygonal Lasso, the Caps Lock width
  ring, Stylus Pressure, fastening-point markers, Adobe cost-function parity.

## Decisions

**Engine.** `EdgeMap::from_buffer(composite, contrast)` takes BT.601 luma (gray
for 1–2 planes), a Sobel magnitude normalised by the image peak, and maps
strength above `contrast / 100` to a cost in `[0, 1]` (border pixels stay 1).
`trace(from, to, width)` runs 8-connected Dijkstra (diagonals cost √2, plus a
0.05 per-step tautness term) inside the segment's bounding box grown by
`width`; off-canvas endpoints, a corridor over 2²⁰ px, or no path return a
Bresenham line. `ponytail:` no gradient-direction / Laplacian terms, so this
approximates Photoshop's snapping.

**Field lifetime.** The field is built from the visible composite once per
gesture (`magnetic_begin`) and cached in `PictureViewRust::edge_map` until
`magnetic_end` (close, cancel, or tool switch).

**Commit path.** The handler accumulates the outline itself and, on close,
feeds it to the existing `begin_lasso`/`lasso_add_point`/`end_lasso`, so the
selection, feather, combine mode, and "one state" history rule are the other
lassos'. Escape calls `cancel_lasso`, leaving the selection untouched.

**Fastening.** A click fastens the wire at the pointer. Frequency `f > 0`
fastens automatically once the live wire reaches `max(8, 108 − f)` px — only on
pointer moves, so Delete can remove an automatic point without it reappearing.
Delete drops the last fastening point and its segment; deleting the first
abandons the trace.

**Bridge placement.** As for the clipboard, `cxxqt_object.rs` is at its
allowlisted ceiling, so a separate `#[cxx_qt::bridge]` exposes `extern "Rust"`
free functions over `PictureView`.

**Selection-move bypass.** A press inside a live selection normally starts a
selection move before any selection handler sees it. `ToolHandler::lassoInProgress`
lets an open magnetic outline receive those presses.

**Options-bar sizing.** Adding three fields made the Magnetic page the widest,
and a `QStackedWidget` reserves its widest page, which raised the window's
minimum width. `OptionsBar::showTool` now gives hidden pages an `Ignored`
horizontal size policy, so only the active page counts.
