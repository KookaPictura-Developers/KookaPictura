# native-depth-composite Specification

## ADDED Requirements

### Requirement: High-depth adjustment layers composite without an 8-bit round-trip

The CPU compositor SHALL apply an adjustment layer at native precision for a
document whose bit depth was recorded on read as 16 or 32, through
`pictura_adjust::apply_native` on an `f32` sample store derived from the canvas,
rather than quantizing the canvas to an 8-bit `PixelBuffer` first. When
`apply_native` reports the adjustment unsupported, the compositor SHALL fall back
to the existing 8-bit application so the result stays defined. An
invalid-parameter error SHALL remain a no-op. For a document with no recorded
source depth the compositor SHALL use the existing 8-bit path exclusively.

#### Scenario: A depth-16 adjustment layer keeps sub-8-bit precision

- **WHEN** a depth-16 document's layers are composited and an adjustment layer is
  applied to a backdrop whose color is not an exact 8-bit code
- **THEN** the native composite (see the next requirement) carries samples that
  are not the 8-bit widening of the `composite_rgba` result

#### Scenario: An 8-bit document is unchanged

- **WHEN** an 8-bit or constructed document with the same adjustment layer is
  composited
- **THEN** `composite_rgba` is byte-identical to before this change

#### Scenario: An unsupported adjustment still applies

- **WHEN** a depth-16 document carries an adjustment layer using `Auto` (outside
  the native set)
- **THEN** the compositor falls back to the 8-bit application and produces the
  same result as before for that layer

### Requirement: A source-depth composite output is exposed

`pictura_render` SHALL expose `composite_native(doc) -> Option<Samples>`
returning the composited canvas at the document's source depth: `Samples::U16`
when the source depth is 16 and `Samples::F32` when it is 32, in planar RGBA
order (straight alpha, clamped to `[0, 1]`), and `None` when the document has no
recorded source depth.

#### Scenario: A depth-16 document yields u16 samples

- **WHEN** `composite_native` is called on a document whose source depth is 16
- **THEN** it returns `Some(Samples::U16(_))` of length `width * height * 4`

#### Scenario: A depth-32 document yields f32 samples

- **WHEN** `composite_native` is called on a document whose source depth is 32
- **THEN** it returns `Some(Samples::F32(_))`

#### Scenario: An 8-bit document yields nothing

- **WHEN** `composite_native` is called on an 8-bit or constructed document
- **THEN** it returns `None`
