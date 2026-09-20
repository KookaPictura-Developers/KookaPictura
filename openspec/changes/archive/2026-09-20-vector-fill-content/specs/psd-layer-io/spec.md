## ADDED Requirements

### Requirement: Vector fill fixtures match independent decoders

The codec SHALL commit a `crates/pictura-codec/tests/fixtures/vector_fill.psd`
fixture carrying a `'vscg'` solid-color vector fill on a shape layer and a
closed-rectangle `'vmsk'` clipping it, authored with `psd-tools` and regenerated
byte-stably by `scripts/generate-fixtures.py`. Existing fixtures SHALL be
unchanged. A test SHALL read the fixture with `psd-tools` and assert the
`'vscg'` block's 4-byte key, version, and decoded `Clr ` colour, and that a
whole-`Document` round trip preserves the block's bytes. A test SHALL read the
fixture with the independent `ag-psd` npm package through `node` and assert the
shape layer's `vectorFill` decodes to the authored solid colour. The ag-psd test
SHALL self-skip with a clear message when `node` or the `ag-psd` package is
unavailable and SHALL NOT fail the suite in that case.

#### Scenario: psd-tools reads the `vscg` fixture block

- **WHEN** `vector_fill.psd` is parsed with `psd-tools`
- **THEN** its shape layer carries a `vscg` block with the authored key and
  version and a `Clr ` object carrying the authored colour

#### Scenario: ag-psd reads the `vscg` fixture block

- **WHEN** `vector_fill.psd` is read by ag-psd
- **THEN** the shape layer's `vectorFill` is a color fill with the authored
  components

#### Scenario: The `vscg` block survives the document round trip

- **WHEN** `vector_fill.psd` is read and written by the codec
- **THEN** the reconstructed document equals the input and the `vscg` block is
  byte-identical

#### Scenario: The ag-psd oracle self-skips without node

- **WHEN** `node` or the `ag-psd` package is not available
- **THEN** the test reports a skip and the suite passes
