## ADDED Requirements

### Requirement: Opaque layer blocks and channels are preserved
The codec SHALL retain, and write back byte-for-byte, the layer-and-mask data
the engine does not model: each layer's original 4-byte blend key when it is not
a recognized mode, each layer's layer-blending-ranges bytes, each layer's
additional-layer-info tagged blocks whose keys are not modeled, each layer mask's
bytes beyond the fixed rectangle/default/flags fields, the global layer mask
block, the trailing global additional-layer information, and each layer channel
whose id is not a modeled color/transparency/mask id (for example the `-3` real
user mask), stored with its compression header intact. A recognized blend key
SHALL NOT be stored, so a document constructed in memory still equals one read
from disk after a write/read round trip.

#### Scenario: Unknown tagged blocks survive
- **WHEN** a layer carries additional-layer-info blocks the codec does not model
- **THEN** read→write→read preserves those blocks' keys and payloads exactly

#### Scenario: An unmodeled channel survives
- **WHEN** a layer carries a channel with an id outside the modeled set, such as `-3`
- **THEN** read→write→read preserves that channel's bytes

#### Scenario: An unknown blend key survives
- **WHEN** a layer's blend key is not one of the 27 modes or `pass`
- **THEN** read→write→read preserves the original 4-byte key

#### Scenario: A recognized blend key is not stored
- **WHEN** a layer uses a recognized blend mode and the document is written and read
- **THEN** the reconstructed document equals the original (no spurious preserved key)

#### Scenario: Blending ranges and mask extras survive
- **WHEN** a layer carries non-empty blending ranges or a mask block longer than the fixed fields
- **THEN** read→write→read preserves those bytes
