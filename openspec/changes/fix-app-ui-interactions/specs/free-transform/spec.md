## ADDED Requirements

### Requirement: Free Transform resolves the active layer

The application SHALL begin a Free Transform session on the active layer, where
the active layer is the exactly-one layer resolved by the shared active-layer
resolver, rather than defaulting to the topmost raster layer. When no layer is
active or more than one layer is selected, beginning a session SHALL be refused
and the document SHALL be unchanged. The refusal SHALL be surfaced to the user.
An explicit target path supplied by a Place still takes precedence over the
resolver.

#### Scenario: Free Transform targets the active layer [lft_active_layer]

- **WHEN** Free Transform is begun with exactly one layer active
- **THEN** the session targets that layer and its transform updates that layer

#### Scenario: No single active layer refuses [lft_no_active_layer]

- **WHEN** Free Transform is begun with no layer active or with more than one
  layer selected
- **THEN** no session begins, the document is unchanged, and the refusal is
  reported

#### Scenario: A Place target overrides the resolver [lft_place_target]

- **WHEN** a Place starts a Free Transform session on its newly placed layer
- **THEN** the session targets the placed layer even if the resolver would name a
  different layer
