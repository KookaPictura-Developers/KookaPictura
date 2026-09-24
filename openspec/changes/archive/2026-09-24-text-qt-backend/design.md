# Design: text-qt-backend

## Context

`render_text_layer` (`pictura-render/src/text_render.rs`) is the only path from
a type layer to pixels; it uses the bundled `fontdue` face. The app calls it
from `PictureView::rasterize_type`. `decode_image_rgba` (C++ in `decode_image.cpp`,
declared in the cxx bridge, used from `helpers.rs`) is the precedent for a
Qt-side helper returning `Vec<u8>`.

## Goals / Non-Goals

**Goals:**

- A Qt system-font text raster for the explicit Rasterize Type command.
- Reuse the app/engine materialization tail.
- A C++ self-test for the helper.
- No change to the bundled default or live compositing.

**Non-Goals:**

- Choosing the backend for live compositing (the compositor stays bundled).
- A user preference/dialog; Qt is tried first for the command, bundled on
  failure.
- Complex layout parity (Qt does its own shaping; the engine layout is not used
  for the Qt path).

## Decisions

### D1. Split the materialization tail

`pub fn materialize_text_rgba(doc: &mut Document, path: &str, rgba: &[u8]) ->
bool`: resolve the layer, require `rgba.len() == rect.w * rect.h * 4`, replace
the `0/1/2/-1` channels, drop `TySh`, clear `type_tool`; false without mutating
otherwise. `render_text_layer` is refactored to call it after building the
buffer (behavior unchanged).

### D2. Qt helper draws with QFont

`render_text_rgba(family: &str, px_size: f64, text: &str, justify: i32, r, g, b,
a: u8, width: i32, height: i32) -> Vec<u8>`: an RGBA8888 `QImage` filled
transparent, a `QFont(family)` at `pixelSize = round(px_size)`, `QPainter`
antialiased, pen from the colour, alignment from `justify` (0 left / 1 right /
2 center) plus top, `text.replace('\r', '\n')`, then copy tightly packed RGBA.
Empty on a non-positive size. The family name is whatever the EngineData named;
Qt substitutes when it is absent, which is exactly the fidelity trade the
command asks for.

### D3. App tries Qt, falls back to bundled

`rasterize_type` reads the layer's text/style/rect, calls the bridge helper,
and on a non-empty result calls `materialize_text_rgba`; on an empty result it
calls the bundled `render_text_layer`. Either success records one `Rasterize
Type` state.

### D4. Self-test the helper

A new check calls `render_text_rgba` for `"Hi"` at a known size and asserts a
non-empty buffer with some non-zero alpha; the engine materializer's positive
path stays covered by the Rust tests.

## Risks / Trade-offs

- [Qt vs bundled divergence] → the Qt path is opt-in per command; live
  compositing stays bundled, so no golden changes.
- [No type-layer fixture] → the app command's positive path is only exercisable
  with a type layer; the helper and the materializer are each tested directly.
- [Font substitution] → recorded as before; this changes which substituted face
  is used for the command, which is the point.
