## ADDED Requirements

### Requirement: Smart Filter layer-menu commands

The Layers menu SHALL expose `Layer > Smart Filter > Clear Smart Filters` with
the stable id `layer.smartFilters.clear`, alongside the Disable Filter Mask and
Delete Filter Mask leaves. Clear Smart Filters SHALL be enabled only when the
current layer carries at least one smart filter, and dispatching it SHALL clear
the current smart object's filter stack through the document operations. The
Disable Filter Mask and Delete Filter Mask leaves SHALL remain disabled until
filter-mask decoding lands.

#### Scenario: Clear is enabled only for a smart object with filters

- **WHEN** the current layer is a smart object with one or more smart filters
- **THEN** Clear Smart Filters is enabled, and it is disabled for a non-smart layer or a smart object with no filters

#### Scenario: Dispatching Clear empties the current stack

- **WHEN** Clear Smart Filters is dispatched on the current smart object
- **THEN** the layer reports no smart filters and the panel's Smart Filters row disappears

#### Scenario: The mask leaves stay disabled

- **WHEN** the current layer is a smart object with filters but no decoded filter mask
- **THEN** Disable Filter Mask and Delete Filter Mask are disabled
