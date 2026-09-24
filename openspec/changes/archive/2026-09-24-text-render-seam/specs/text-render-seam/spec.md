# Specs delta: text-render-seam

## ADDED Requirements

### Requirement: Deterministic layout over shaped glyph runs

`pictura-core` SHALL expose a Qt-free `layout_lines` that positions already
shaped glyph runs — advance accumulation scaled from font units to device
pixels by `font_size / units_per_em`, tracking in 1/1000 em, per-line leading,
and alignment — without parsing a font or shaping text. The layout SHALL be a
pure function of its inputs and SHALL NOT depend on the host, the clock, or any
random source.

#### Scenario: Advances accumulate left to right

- **WHEN** a line of glyphs with known advances is laid out
- **THEN** each glyph's x equals the sum of the preceding device advances and the line advance is their total

#### Scenario: Tracking and leading apply

- **WHEN** tracking and leading are non-zero
- **THEN** each device advance includes `tracking * font_size / 1000` and each successive line's baseline is offset by `leading`

#### Scenario: Empty input yields an empty layout

- **WHEN** no lines or empty lines are laid out
- **THEN** the layout has no placed glyphs and zero width

### Requirement: Alignment maps the paragraph justification

The system SHALL provide `TextAlign` with `Left`, `Center`, and `Right` and
SHALL map the EngineData paragraph `Justification` byte 0 to `Left`, 1 to
`Right`, and 2 to `Center`. When a line is aligned inside a `wrap_width`, every
glyph SHALL be shifted by `(wrap_width - line_advance)` times `0.0`, `0.5`, or
`1.0`; without a `wrap_width` no shift SHALL be applied.

#### Scenario: Right alignment shifts by the leftover width

- **WHEN** a line shorter than the wrap width is laid out right-aligned
- **THEN** its last glyph ends at the wrap width

#### Scenario: Unknown justification falls back to left

- **WHEN** the justification byte is outside 0, 1, 2
- **THEN** alignment is `Left`

### Requirement: The glyph rasterizer port is POD-only

The system SHALL define a `Rasterizer` port that takes a `RasterRequest`
(`glyph` id, pixel size, and subpixel position) and returns an optional
`GlyphMask` (`width`, `height`, integer `left`/`top` offsets, and a coverage
buffer), carrying no font bytes and no Qt type. A rasterizer SHALL be usable
without mutable state.

#### Scenario: A backend produces a coverage mask

- **WHEN** a `RasterRequest` for a glyph is passed to a rasterizer backend
- **THEN** it returns a mask whose coverage length equals `width * height`, or `None` when the glyph cannot be rendered

#### Scenario: The request carries no resource handle

- **WHEN** a `RasterRequest` is inspected
- **THEN** it contains only a glyph id, a pixel size, and a subpixel position

### Requirement: Font policy and provenance are recorded

The system SHALL define `FontPolicy` (`BundledOnly`, `HostAllowed`,
`ExactOrRefuse`) and `TextProvenance` recording the requested family, the
resolved family, the resolved font's content hash, and the rasterizer backend
and version, so that a substitution or refusal is an auditable fact.

#### Scenario: Provenance records a substitution

- **WHEN** provenance is constructed for a requested family that resolved to a different face
- **THEN** the record exposes both the requested and the resolved family and the resolved hash

#### Scenario: Policies are distinct

- **WHEN** the three font policies are compared
- **THEN** each is a distinct value that can be persisted and replayed
