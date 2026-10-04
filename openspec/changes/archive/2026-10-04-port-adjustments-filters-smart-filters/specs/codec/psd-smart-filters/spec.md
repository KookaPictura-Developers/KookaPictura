# Spec Delta

## ADDED Requirements

### Requirement: Smart-filter group enable flag round-trips

The smart-object codec SHALL read the `filterFXStyle` group enable flag (`enab`) into `SmartObject::smart_filters_enabled` (defaulting to enabled when absent) and SHALL author it when writing `filterFX`, so a document whose group is disabled re-reads as disabled and a document whose group is enabled re-reads as enabled. The flag SHALL be preserved independently of the per-filter `enab`.

#### Scenario: A disabled group round-trips

- **WHEN** a smart object with a disabled `filterFXStyle` group is read and written
- **THEN** reading the written document reports the group disabled

#### Scenario: A missing flag defaults to enabled

- **WHEN** a `filterFXStyle` without the `enab` key is read
- **THEN** the group is treated as enabled

### Requirement: Generic smart-filter attachment

The codec SHALL expose a generic operation that attaches a single `SmartFilter` to a layer. When the layer has a preserved `SoLd`/`SoLE` descriptor it SHALL insert a new `filterID` entry or merge into the existing one, refreshing the modelled `Fltr` keys while keeping unmodeled keys and `blendOptions`, and rewrite the preserved block consistently; when it is an embedded smart object without a preserved block it SHALL record the filter in `SmartObject::smart_filters`. It SHALL refuse a layer with no smart object or a non-embedded/legacy placed object without mutating it.

#### Scenario: Attaching into a preserved descriptor

- **WHEN** a smart filter is attached to a layer carrying a preserved `SoLd`
- **THEN** the `filterFXList` gains or replaces the matching `filterID` entry and the typed `smart_filters` view matches

#### Scenario: Re-attaching over a modelled filter keeps unmodeled keys

- **WHEN** a filter whose `filterID` already has an entry is attached again over a preserved descriptor
- **THEN** the entry is not duplicated, the new `Fltr` keys win, and unmodeled keys such as `Dhze` and the entry's `blendOptions` survive

#### Scenario: Attaching to an embedded object without a preserved block

- **WHEN** a smart filter is attached to an embedded object with no preserved block
- **THEN** the filter is recorded in the typed `smart_filters` list and authored on save

#### Scenario: Non-smart or legacy layers are refused

- **WHEN** the layer has no smart object, or is a legacy placed-layer object
- **THEN** the operation returns an error and does not mutate the layer
