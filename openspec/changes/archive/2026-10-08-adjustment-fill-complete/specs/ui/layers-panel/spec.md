## ADDED Requirements

### Requirement: Fill / Adjustment creation menu

The Layers panel's New Fill / Adjustment strip button SHALL open a menu whose
Fill group offers Solid Color… and Gradient…, a Pattern… entry that is present
but disabled with the `— not implemented yet` tooltip until pattern authoring
exists, and whose Adjustment group lists all sixteen CS6 adjustment kinds.
Choosing an adjustment kind SHALL create that adjustment layer through the
bridge. The Fill and Adjustment groups SHALL be separated as in CS6.

#### Scenario: The menu lists every adjustment kind

- **WHEN** the New Fill / Adjustment menu is opened
- **THEN** it contains an entry for each of the sixteen adjustment kinds, each enabled, and the choosing creates the matching layer

#### Scenario: Pattern stays disabled until authoring exists

- **WHEN** the New Fill / Adjustment menu is opened before pattern authoring exists
- **THEN** the Pattern… entry is present, disabled, and carries the `— not implemented yet` tooltip

### Requirement: Layer Content Options opens the fill / adjustment editor

`Layer > Layer Content Options…` SHALL be enabled only when the current layer is
a fill or adjustment layer. Invoking it SHALL show the Properties panel and
refresh it so it reflects that layer. For any other current layer the command
SHALL be disabled and invoking it SHALL do nothing.

#### Scenario: The command opens Properties for an adjustment layer

- **WHEN** an adjustment or fill layer is current and `Layer Content Options…` is invoked
- **THEN** the Properties panel is shown and reflects that layer

#### Scenario: The command is disabled for other layers

- **WHEN** the current layer is a pixel, type, group, or Background layer
- **THEN** `Layer Content Options…` is disabled and invoking it does nothing
