## ADDED Requirements

### Requirement: Type and shape forced locks

The Layers panel lock strip SHALL show Lock Transparency and Lock Image forced on
for a type or shape layer, and SHALL NOT allow either to be deselected. The two
toggle buttons SHALL be disabled while such a layer is the current row so a click
cannot clear them, and the layer's reported lock state SHALL include those two
bits so the strip renders them checked. The engine SHALL enforce the same rule:
a request that would clear the Transparency or Image lock on a type or shape
layer SHALL leave those bits set (clearing any other requested bit such as
Position still succeeds) and SHALL record no history state when nothing changed.

#### Scenario: A shape layer shows the forced locks

- **WHEN** a shape layer becomes the current row
- **THEN** the Lock Transparency and Lock Image toggles are checked and are
  disabled so they cannot be deselected

#### Scenario: A type layer shows the forced locks

- **WHEN** a type layer becomes the current row
- **THEN** the Lock Transparency and Lock Image toggles are checked and are
  disabled so they cannot be deselected

#### Scenario: The bridge refuses to clear a forced bit

- **WHEN** the lock bridge is asked to clear Transparency or Image on a type or
  shape layer
- **THEN** the bit stays set, no history state is recorded, and the layer is
  otherwise unchanged

## MODIFIED Requirements

### Requirement: Panel and row menus

The system SHALL provide a panel menu and a row context menu. The panel menu
SHALL offer `Panel Options…`, New Layer, New Group, Duplicate Layer(s), Delete
Layer(s), Group Layers, Ungroup Layers, Move Layer Up, and Move Layer Down;
commands the system does not yet implement SHALL appear disabled with a
`— not implemented yet` tooltip rather than being hidden.

The row context menu SHALL be assembled **per layer kind**. Each kind (pixel,
background, group, adjustment, type, shape, smart object) SHALL be offered the
commands that apply to it, and SHALL NOT be offered commands that do not apply
(for example, a group or adjustment row SHALL NOT offer `Export As…` or `Quick
Export as PNG`, and a group row SHALL NOT offer `Fill`-related content). Every
applicable command that is not yet implemented SHALL still appear, rendered
disabled, carrying a `— not implemented yet` tooltip, so that implementing a
command enables an existing row rather than adding one. The commands common to
every kind SHALL include Rename and a color-label submenu containing `None`,
`Red`, `Orange`, `Yellow`, `Green`, `Blue`, `Violet`, and `Gray`.

A shape row SHALL, in addition to the common commands, offer **Copy Shape
Attributes**, **Paste Shape Attributes**, and **Rasterize Shape**, each wired to
the shape bridge and acting on that row. A shape row SHALL be resolved by its
shape-layer projection (a fill cut to a vector mask), not by its adjustment
kind, so it is not offered the adjustment-only `Edit Adjustment…` row.

For a pixel row the menu SHALL offer **Add Layer Mask**, **Delete Layer Mask**,
**Enable Layer Mask**, and **Disable Layer Mask**, each wired to the layer-mask
bridge: Add Layer Mask SHALL add a `reveal-selection` mask when a selection
exists and a `reveal-all` mask otherwise; Delete Layer Mask SHALL remove the
mask; and Enable/Disable Layer Mask SHALL set the mask's enabled state. Each
applied action SHALL be one undoable step.

Right-clicking the visibility toggle SHALL offer show/hide this layer only and
show/hide all. The bottom action strip SHALL remain exactly the CS6 seven
buttons, so Move Up/Down SHALL NOT appear in the strip.

#### Scenario: The panel menu contains the wired commands [m39_menus]

- **WHEN** the panel menu is opened
- **THEN** it contains Panel Options, New Layer, New Group, Duplicate, Delete,
  Group, Ungroup, Move Up, and Move Down; commands that are not yet implemented
  are shown disabled with the `— not implemented yet` tooltip

#### Scenario: The row menu sets a color label [m39_menus]

- **WHEN** the user right-clicks a row and picks Red from the color-label submenu
- **THEN** that layer's color label becomes Red

#### Scenario: Move Up/Down reorders from the menu [m39_menus]

- **WHEN** the user picks Move Layer Up for a row that is not the top of its container
- **THEN** the layer swaps with the layer above it within its container in one undo step

#### Scenario: The action strip has no Move buttons [m39_strip]

- **WHEN** the Layers panel is shown
- **THEN** its bottom strip contains exactly the seven CS6 buttons and no Move Up
  or Move Down button

#### Scenario: Pixel and smart-object rows offer export

- **WHEN** the user opens the row menu for a pixel layer or a smart-object layer
- **THEN** it contains Export As… and Quick Export as PNG

#### Scenario: Group and adjustment rows hide export

- **WHEN** the user opens the row menu for a group or an adjustment layer
- **THEN** it does not offer Export As… or Quick Export as PNG

#### Scenario: Type rows hide export

- **WHEN** the user opens the row menu for a type layer
- **THEN** it does not offer Export As… or Quick Export as PNG

#### Scenario: A shape row offers its shape commands

- **WHEN** the user opens the row menu for a shape layer
- **THEN** it contains Copy Shape Attributes, Paste Shape Attributes, and
  Rasterize Shape, enabled and wired to the shape bridge

#### Scenario: A shape row is not offered the adjustment edit

- **WHEN** the user opens the row menu for a shape layer
- **THEN** it does not contain Edit Adjustment…

#### Scenario: A pixel row offers its full command set

- **WHEN** the user opens the row menu for a pixel layer
- **THEN** it contains the common commands, the color-label submenu, `Export
  As…`, `Quick Export as PNG`, and the pixel-applicable mask, style, clipping,
  smart-object, and rasterize commands — each either enabled or disabled with a
  tooltip when not yet implemented

#### Scenario: The row menu adds and deletes a mask [lmk_rowmenu]

- **WHEN** the user picks Add Layer Mask for a pixel row, then Delete Layer Mask
- **THEN** a mask is added and then removed from the active layer, each in one undo step

#### Scenario: The row menu toggles mask enablement [lmk_rowmenu_enable]

- **WHEN** the user picks Enable Layer Mask or Disable Layer Mask for a masked layer
- **THEN** the mask's disabled bit is cleared or set in one undo step

#### Scenario: A group row omits content commands

- **WHEN** the user opens the row menu for a group row
- **THEN** it does not offer `Export As…`, `Quick Export as PNG`, or fill
  commands, and it offers `Ungroup Layers`

#### Scenario: A type row offers rasterize

- **WHEN** the user opens the row menu for a type layer
- **THEN** it offers `Rasterize Type`

#### Scenario: An unimplemented applicable command is disabled, not absent

- **WHEN** the user opens the row menu for a kind and a command applies to that
  kind but is not yet implemented
- **THEN** the command is present, disabled, and shows the `— not implemented
  yet` tooltip
