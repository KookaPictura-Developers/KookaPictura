# type-layer-kind Specification

## Purpose
TBD - created by archiving change type-layer-kind. Update Purpose after archive.
## Requirements
### Requirement: Type layers are identified by a preserved TySh block

The system MUST report a layer as kind `type` when its preserved
additional-layer-info blocks include the key `TySh`. Detection MUST be
presence-only: the payload MUST NOT be parsed. `layer_kind`, `layer_row_kind`,
and the shared row-kind projection MUST return the string `type` for such a
layer. Kind order MUST be group, adjustment, background, type, then pixel, so
existing group/adjustment/background behaviour is unchanged. The `TySh` bytes
MUST remain in `Layer.extra_blocks` and MUST re-emit byte-for-byte on
open→save.

#### Scenario: A layer with TySh reports type

- **WHEN** a document is read (or constructed) with a non-group layer whose
  `extra_blocks` contain key `TySh`
- **THEN** `layer_kind` / `layer_row_kind` for that layer is `type`

#### Scenario: A pixel layer without TySh is still pixel

- **WHEN** a non-group, non-adjustment layer has no `TySh` block
- **THEN** its kind is `pixel` (or `background` when flagged background)

#### Scenario: TySh bytes round-trip

- **WHEN** a document whose layer carries a `TySh` block is written and read
  back
- **THEN** the layer still has the same `TySh` key and payload bytes

### Requirement: Type layers carry the CS6 default type locks

When the reader observes a `TySh` block on a layer, it MUST set
`LockFlags::TRANSPARENCY` and `LockFlags::PIXELS` on that layer regardless of
the stored `lspf` bits for those two flags (CS6 defaults: Lock Transparency and
Lock Image on). POSITION and NESTING bits MUST be left as stored. Layers
without `TySh` MUST keep their stored locks unchanged.

#### Scenario: TySh forces transparency and image locks

- **WHEN** a layer with `TySh` is read with `lspf` locks clear
- **THEN** the layer's lock flags include transparency and pixels

#### Scenario: Layers without TySh keep stored locks

- **WHEN** a layer without `TySh` is read with a given `lspf` value
- **THEN** its lock flags equal that stored value (plus the legacy
  transparency-protected bit rule)

