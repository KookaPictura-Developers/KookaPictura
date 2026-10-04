# Spec Delta

## ADDED Requirements

### Requirement: Smart-filter chain composites over the embedded source

When a smart-object layer carries a non-empty `filterFX` chain and every enabled filter decodes to a supported operation, the compositor SHALL render the layer from the smart object's embedded source and apply the enabled filters in order, using the group enable flag and each filter's `enab`, rather than compositing the already-baked raster proxy. An unknown enabled filter SHALL leave the layer to the normal proxy path so an imported result is not double-applied. The chain SHALL be deterministic and SHALL preserve alpha.

#### Scenario: A disabled filter is skipped

- **WHEN** a chain's only filter has `enab` false, or the group is disabled
- **THEN** the composite equals the unfiltered embedded source

#### Scenario: The chain is applied to the source, not the proxy

- **WHEN** a layer whose proxy already contains the filtered result carries the same enabled filter
- **THEN** the composite equals the filter applied once to the embedded source, not twice

#### Scenario: An unknown enabled filter falls back to the proxy

- **WHEN** a chain contains an enabled filter whose options cannot be decoded
- **THEN** the layer composites through the existing proxy path unchanged
