# Spec Delta

## ADDED Requirements

### Requirement: Authored EngineData carries the modelled type style

When a type block is built for a layer, the author SHALL write each modelled
character attribute into the `StyleRun` `StyleSheetData` keys — `/Font` (an index
into `/FontSet`), `/FontSize`, `/Leading` with `/AutoLeading`, `/Tracking`,
`/Kerning` with `/AutoKerning`, `/HorizontalScale`, `/VerticalScale`,
`/BaselineShift`, the fill colour `/FillColor`, and the modelled toggle keys —
and each modelled paragraph attribute into the `ParagraphRun` `ParagraphSheet`
`/Properties` keys: `/Justification`, `/FirstLineIndent`, `/StartIndent`,
`/EndIndent`, `/SpaceBefore`, `/SpaceAfter`, `/AutoHyphenate`, `/LeadingType`,
and `/EveryLineComposer`. An unset attribute SHALL be written with its documented
default rather than an arbitrary value. The anti-aliasing method SHALL be
authored into the `TySh` text descriptor's `AntA` enum, and the EngineData
`/AntiAlias` flag SHALL agree with it.

#### Scenario: A modelled character value survives author and re-read

- **WHEN** a type block is authored from a spec whose tracking is 120 and horizontal scale is 85 percent
- **THEN** reading the block back reports tracking 120 and horizontal scale 85

#### Scenario: A modelled paragraph value survives author and re-read

- **WHEN** a type block is authored from a spec whose first-line indent is 24 and space after is 12
- **THEN** reading the block back reports first-line indent 24 and space after 12

#### Scenario: The anti-aliasing method is authored to the descriptor and the flag

- **WHEN** a type block is authored with anti-aliasing None, Sharp, Crisp, Strong, or Smooth
- **THEN** the text descriptor's `AntA` encodes that method's CS6 spelling and the EngineData `/AntiAlias` flag agrees

### Requirement: Re-author preserves unmodeled EngineData keys

When the author rewrites a type block that already exists on a layer, it SHALL
parse the existing EngineData and overwrite only the modelled keys of the text's
single style run and single paragraph sheet, so every other key of that block
survives with its value. When the layer carries no existing EngineData, the
author SHALL emit a complete skeleton with the documented defaults. Text whose
EngineData holds more than one style run is outside the model: its re-author
collapses to the single modelled run, and per-paragraph properties beyond the
first paragraph are likewise collapsed, both as documented ceilings.

#### Scenario: An unmodeled key survives a re-set

- **WHEN** a single-run type layer whose EngineData carries a key the model does not cover is re-set through the author
- **THEN** that key and its value are still present, valid, in the rewritten block

#### Scenario: A layer without EngineData is authored complete

- **WHEN** a new type layer is authored with no existing EngineData
- **THEN** the emitted block contains the full style and paragraph skeleton with documented defaults

#### Scenario: A multi-run block is re-authored to the model's single run

- **WHEN** a type layer whose EngineData holds two style runs is re-set
- **THEN** the result holds one style run (the collapse is a documented ceiling, not a recorded event)
