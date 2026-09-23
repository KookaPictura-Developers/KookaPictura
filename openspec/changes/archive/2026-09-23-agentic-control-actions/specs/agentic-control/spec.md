## ADDED Requirements

### Requirement: Tool selection

The server SHALL provide a `set_tool` method with a `tool` name. It SHALL accept
either the name `status` reports as `active_tool` or the tool's display label,
set the active tool through the application's own path, and return
`{active_tool}`. An unknown name SHALL be `invalid_param`.

#### Scenario: A tool can be selected by its status name

- **WHEN** a client invokes `set_tool` with the name `status` reported for the active tool or another known tool
- **THEN** the response reports that tool as `active_tool` and a later `status` agrees

#### Scenario: An unknown tool is rejected

- **WHEN** a client invokes `set_tool` with an unknown name
- **THEN** the response has error code `invalid_param`

### Requirement: Selection actions

The server SHALL provide a `selection` method with an `op` of `all`, `deselect`,
`rect`, `ellipse`, `lasso_begin`, `lasso_point`, `lasso_end`, `quick`, or
`wand`, taking the op's coordinates, `mode` (`new`/`add`/`subtract`/`intersect`),
`tolerance`, `contiguous`, and `feather`. `mode` SHALL be one of those four
values and any other value SHALL be `invalid_param` (an unknown mode must not
silently behave as `new`). It SHALL drive the selection model through the
application's paths and return `{has_selection, count, bounds}` where `bounds` is
the selection's `x y w h` when one exists. An op the selection model does not
apply (a rectangle or ellipse with non-positive `w`/`h`, a lasso with fewer than
three points, a wand outside the document) SHALL return `invalid_param`.

#### Scenario: A rectangular selection is applied

- **WHEN** a client invokes `selection` with op `rect` and a rectangle inside the document
- **THEN** the result reports `has_selection` true and a `count` greater than zero

#### Scenario: Deselect clears the selection

- **WHEN** a client invokes `selection` with op `deselect` while a selection exists
- **THEN** the result reports `has_selection` false and `count` zero

#### Scenario: A selection needs a document

- **WHEN** a client invokes `selection` with no document open
- **THEN** the response has error code `no_document`

#### Scenario: A non-applying selection is rejected

- **WHEN** a client invokes `selection` with op `rect` and `w` of zero, or with `mode` `union`
- **THEN** the response has error code `invalid_param`

### Requirement: Filter and adjustment actions

The server SHALL provide a `filter` method (`kind`) that applies one of the
application's filter kinds and returns `{ok, kind}`, and an `adjustment` method
(`kind`) that adds an adjustment layer of that kind and returns `{ok, layers}`.
An unknown kind SHALL be `invalid_param`. A `filter` whose target layer is
unavailable (no editable pixel layer), hidden, or pixel-locked SHALL return
`refused`, so an engine refusal is never misreported as an unknown kind. The
application's fixed seeds make a seeded filter deterministic.

#### Scenario: A seeded filter is deterministic

- **WHEN** a client applies the same seeded `filter` kind twice to the same document
- **THEN** both calls succeed and produce the same pixels

#### Scenario: An unknown kind is rejected

- **WHEN** a client invokes `filter` or `adjustment` with an unknown kind
- **THEN** the response has error code `invalid_param`

#### Scenario: A locked target refuses the filter

- **WHEN** a client pixel-locks the active layer and invokes `filter`
- **THEN** the response has error code `refused` and the kind is not reported as unknown

### Requirement: Layer operations

The server SHALL provide a `layer_op` method with an `op` of `set_name`,
`set_opacity`, `set_visible`, `set_blend`, `set_fill`, `set_lock`, `set_color`,
`move`, `translate`, `delete`, `duplicate`, or `add`, taking `index` and the op's
value. It SHALL apply the operation through the application's layer methods and
return `{ok, layers}`. An out-of-range index or a refused operation SHALL be
reported with the application's error (`invalid_param`/`refused`), not a crash.
`set_visible` SHALL require an explicit boolean, supplied as either `visible` or
`on`, and SHALL return `invalid_param` when neither is a boolean; a missing or
non-boolean value SHALL NOT default to visible. `set_name` SHALL require a
non-empty `name`. `set_lock` SHALL take a `flag` of `transparency`, `pixels`,
`position`, `nesting`, or `all` with an `on` boolean (`list_layers` reports
`lock` as the resulting bitmask). `translate` targets the active layer; when
`index` is present the server SHALL make that layer active before translating so
the requested layer is the one moved, and an out-of-range `index` SHALL be
`invalid_param`.

#### Scenario: A layer property round-trips

- **WHEN** a client sets a layer's opacity with `layer_op` and then reads it with `list_layers`
- **THEN** the reported opacity is the value that was set

#### Scenario: Visibility must be explicit

- **WHEN** a client invokes `layer_op` with op `set_visible` and `on` false
- **THEN** `list_layers` reports the layer as not visible

#### Scenario: A duplicate is created

- **WHEN** a client invokes `layer_op` with op `duplicate`
- **THEN** the result reports one more layer than before

#### Scenario: An out-of-range index is reported

- **WHEN** a client invokes `layer_op` with an out-of-range index
- **THEN** the response has error code `invalid_param` and the application does not crash

#### Scenario: Translate honors its index

- **WHEN** a client invokes `layer_op` with op `translate` and an in-range `index`
- **THEN** that layer is made active and moved; an out-of-range `index` returns `invalid_param`

### Requirement: GPU compute toggle

The server SHALL provide a `set_gpu_compute` method with an `on` boolean that
sets the compute preference through the application and returns
`{gpu_compute, backend}`.

#### Scenario: The toggle is reported

- **WHEN** a client invokes `set_gpu_compute` with `on` false
- **THEN** the result reports `gpu_compute` false and the active backend

### Requirement: Action verification

The self-test SHALL include a block that exercises the action methods over the
control socket using the next append-only check codes: it SHALL select a tool,
apply a rectangular `selection` and assert a non-zero count, apply a seeded
`filter` twice and assert determinism plus an `invalid_param` for an unknown
kind, round-trip a `layer_op` opacity through `list_layers`, and toggle
`set_gpu_compute`.

#### Scenario: The action block proves the methods offscreen

- **WHEN** the self-test runs the action block under `--headless`
- **THEN** every action assertion passes and the block reports pass
