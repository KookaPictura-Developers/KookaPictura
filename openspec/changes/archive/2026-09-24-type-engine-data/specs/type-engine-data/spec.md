# Specs delta: type-engine-data

## ADDED Requirements

### Requirement: EngineData decodes into a typed tree

`pictura-codec` SHALL parse an EngineData byte stream into a typed tree of
dicts, lists, integers, doubles, booleans, and strings, using bounded token,
depth, and byte caps. A malformed, truncated, or over-cap stream SHALL return a
typed error and MUST NOT panic. The parser SHALL decode parenthesised strings as
UTF-16BE when they begin with the `\xfe\xff` byte-order mark, applying the
`\(`, `\)`, and `\\` escapes, and SHALL decode property names as MacRoman.

#### Scenario: A typed tree round-trips through the parser

- **WHEN** a stream with a dict containing a list, a double, an integer, a boolean, and a string is parsed
- **THEN** the tree exposes each value with its type and key

#### Scenario: A malformed stream errors without panicking

- **WHEN** the parser is given a truncated or depth-overflowing stream
- **THEN** it returns an error and does not panic

### Requirement: TypeTool exposes the font set and first-run style

`read_psd` SHALL fill `TypeTool.fonts` with the embedded source's font-set names
and `TypeTool.style` with the first style run's effective values — font name,
font size, fill colour, tracking, and paragraph justification — resolved against
the paragraph/style defaults when a run omits a field. When the `Txt `
descriptor has no EngineData, or the EngineData cannot be parsed, the style
SHALL be `None`, the font list empty, and the document read SHALL still succeed
with the `Txt ` string and raw descriptor bytes preserved.

#### Scenario: A real text layer exposes its font and size

- **WHEN** the fixture whose first style run sets `FontSize` 150 and defaults to font index 1 is read
- **THEN** `TypeTool.style` reports the font name at index 1, size 150, and the run's fill colour

#### Scenario: A missing EngineData leaves the style unset

- **WHEN** a `TySh` block's `Txt ` descriptor has no `EngineData` value
- **THEN** `TypeTool.style` is `None` and the document read succeeds

#### Scenario: A malformed EngineData does not fail the document

- **WHEN** the `EngineData` value is not parseable
- **THEN** `TypeTool.style` is `None` and the layer still reads with its text and raw `TySh` bytes
