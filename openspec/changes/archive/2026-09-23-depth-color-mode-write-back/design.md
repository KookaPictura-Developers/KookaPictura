## Context

`retain_planes` (`read.rs:79-82`) retains native samples for 16/32-bit
Grayscale/RGB, and `source_planes` captures the raw pre-narrow composite; layer
channels already retain native samples generically at 16/32 (`read.rs:628-633`)
regardless of color mode. So the only read change is adding Lab/Cmyk to the 16/32
arm. The 8-bit Lab/CMYK write-back (`color-mode-write-back`, `cmyk-write-back`)
re-emits a retained plane when `lab_to_rgb`/`cmyk_to_rgb` of the retained bytes
still equals the working RGB, else re-encodes 8-bit.

## Goals / Non-Goals

- **Goal:** a 16/32-bit CMYK or Lab document saves back in its source mode **and**
  source depth; unchanged planes are byte-exact.
- **Non-goal:** a true 16-bit sampling model. Editing stays 8-bit, so an edited
  plane is an 8-bit value widened (the same ceiling as Grayscale/RGB).
- **Non-goal:** Multichannel/Duotone (no grounded RGB mapping).

## Decisions

### Narrow the retained plane before the unchanged comparison

`native_plane` (`write.rs:792`) compares `narrow_channel(retained)` against the
plane the builder returns as `current`. For Grayscale/RGB `current` is the
working 8-bit plane, so this is exact. For Lab/CMYK the planes passed around are
**color-encoded** (Lab/CMYK), and the 8-bit retained store is the *narrowed*
encoding: comparing `narrow(native)` to `rgb_to_lab(working)` fails (Lab rounds
to ±1 LSB; `rgb_to_cmyk` forces `K=255` and drops the original K plate). The fix
is to compare in the narrowed-retained domain: pass `composite_retained_8` /
`layer_retained_8` (the retained native plane narrowed to 8-bit) as `current`, so
`narrow(native) == current` holds and `native_plane` borrows the native plane;
an edit still fails the comparison and `native_plane` widens the re-encoded
8-bit plane.

### Two narrowing helpers in `color_mode.rs`

`write.rs` is at 1186/1200, so the helpers live in `color_mode.rs` (494 lines):
`composite_retained_8(doc, depth, index)` and `layer_retained_8(layer, depth,
id)` return the retained plane narrowed to 8-bit (`narrow_channel` over
`row_bytes(w, depth)`), or the plane unchanged at depth 8. The four builder call
sites (`lab_composite_planes`, `cmyk_composite_planes`,
`lab_layer_color_planes`, `cmyk_layer_color_planes`) swap
`composite_retained`/`layer_retained` for these; at depth 8 the helpers are the
identity, so the 8-bit behavior is unchanged.

### Relax the mode guards without breaking 8-bit

`lab_mode`/`cmyk_mode` (`write.rs:982-992`) currently require `depth == 8 &&
source_depth.is_none()`. Change to `source_mode == Some(Lab|Cmyk) &&
composite.channels == 3 && (depth != 8 || source_depth.is_none())`: at depth 8 a
construct with no recorded depth still does not fire, while a 16/32 doc (which
retains a native store, hence `depth != 8`) does. `output_depth` already returns
the source depth once `retains_source_depth` is true.

### App notice

`color_mode_notice` (`impl_core.rs:297-301`) currently treats
`source_depth.is_none()` as "preserves"; a 16/32 Lab/CMYK doc would claim "saved
as RGB". Mirror `depth_notice` and consult `retains_source_depth()` so the notice
reports the source mode when the depth is retained.

## Risks / Trade-offs

- `psd-bit-depth` and `psd-color-modes` currently *mandate* the old 8-bit-RGB
  save; two MODIFIED deltas are required and their existing tests
  (`tests/depth.rs` `depth16/32_cmyk_...`) change to the new behavior.
- The tight `write.rs` budget is why the helpers are in `color_mode.rs`; if it
  still grows, split along the existing `write_indexed.rs`/`write_bitmap.rs`
  seam.
